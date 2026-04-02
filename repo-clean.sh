#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
usage: ./repo-clean.sh [--dry-run] [--local-only]

Remove disposable worktrees, stashes, and branches from this repository.

kept local branches:
  - main
  - master
  - keep/*

kept remote branches:
  - main
  - master
  - assets
  - keep/*

options:
  --dry-run     print actions without executing them
  --local-only  skip remote fetch/prune and remote branch deletion
  -h, --help    show this help
EOF
}

log() {
  printf '[repo-clean] %s\n' "$*"
}

error() {
  printf '[repo-clean] %s\n' "$*" >&2
}

print_dry_run_cmd() {
  local arg
  printf '[repo-clean] would run:'
  for arg in "$@"; do
    printf ' %q' "$arg"
  done
  printf '\n'
}

run_cmd() {
  if [[ "${dry_run}" == true ]]; then
    print_dry_run_cmd "$@"
    return 0
  fi

  "$@"
}

is_local_kept_branch() {
  local branch="$1"
  case "$branch" in
    main | master | keep/*) return 0 ;;
    *) return 1 ;;
  esac
}

is_remote_kept_branch() {
  local branch="$1"
  case "$branch" in
    main | master | assets | keep/*) return 0 ;;
    *) return 1 ;;
  esac
}

find_remote_branch_by_name() {
  local branch_name="$1"
  local remote_ref

  while IFS= read -r remote_ref; do
    case "$remote_ref" in
      */"$branch_name")
        printf '%s\n' "$remote_ref"
        return 0
        ;;
    esac
  done < <(git for-each-ref --format='%(refname:short)' refs/remotes)

  return 1
}

dry_run=false
clean_remote=true

while [[ "$#" -gt 0 ]]; do
  case "$1" in
    --dry-run)
      dry_run=true
      ;;
    --local-only)
      clean_remote=false
      ;;
    -h|--help|help)
      usage
      exit 0
      ;;
    *)
      error "Unknown argument: $1"
      usage >&2
      exit 2
      ;;
  esac
  shift
done

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [[ -z "$repo_root" ]]; then
  error "Not inside a git repository."
  exit 1
fi

cd "$repo_root"

had_errors=0
current_branch="$(git symbolic-ref --quiet --short HEAD || true)"

if [[ -n "$current_branch" ]] && ! is_local_kept_branch "$current_branch"; then
  fallback_branch=""
  fallback_seed_remote=""
  fallback_seed_branch=""

  for candidate in main master; do
    if git show-ref --verify --quiet "refs/heads/$candidate"; then
      fallback_branch="$candidate"
      break
    fi
  done

  if [[ -z "$fallback_branch" ]]; then
    while IFS= read -r branch; do
      if is_local_kept_branch "$branch"; then
        fallback_branch="$branch"
        break
      fi
    done < <(git for-each-ref --format='%(refname:short)' refs/heads)
  fi

  if [[ -z "$fallback_branch" ]]; then
    for candidate in main master; do
      if fallback_seed_remote="$(find_remote_branch_by_name "$candidate")"; then
        fallback_seed_branch="$candidate"
        break
      fi
    done

    if [[ -n "$fallback_seed_remote" ]]; then
      fallback_branch="$fallback_seed_branch"
      log "Creating '$fallback_branch' from '$fallback_seed_remote' before cleanup."
      if run_cmd git switch -c "$fallback_branch" --track "$fallback_seed_remote"; then
        current_branch="$fallback_branch"
      else
        error "Could not create/switch to '$fallback_branch' from '$fallback_seed_remote' before cleanup."
        had_errors=1
      fi
    else
      fallback_branch="main"
      log "Creating '$fallback_branch' from '$current_branch' before cleanup."
      if run_cmd git switch -c "$fallback_branch"; then
        current_branch="$fallback_branch"
      else
        error "Could not create/switch to '$fallback_branch' before cleanup."
        had_errors=1
      fi
    fi
  else
    log "Switching to '$fallback_branch' before branch cleanup."
    if run_cmd git switch "$fallback_branch"; then
      current_branch="$fallback_branch"
    else
      error "Could not switch to '$fallback_branch' before cleanup."
      had_errors=1
    fi
  fi
fi

log "Removing non-primary worktrees..."
while IFS= read -r line; do
  [[ "$line" == worktree\ * ]] || continue

  worktree_path="${line#worktree }"
  if [[ "$worktree_path" == "$repo_root" ]]; then
    continue
  fi

  printf '  - %s\n' "$worktree_path"
  if ! run_cmd git worktree remove --force "$worktree_path"; then
    if ! run_cmd git worktree remove --force --force "$worktree_path"; then
      error "Failed to remove worktree '$worktree_path'."
      had_errors=1
    fi
  fi
done < <(git worktree list --porcelain)

run_cmd git worktree prune --verbose || true

log "Clearing stashes..."
if ! run_cmd git stash clear; then
  error "Failed to clear stashes."
  had_errors=1
fi

log "Deleting local branches..."
while IFS= read -r branch; do
  [[ -n "$branch" ]] || continue

  if is_local_kept_branch "$branch"; then
    continue
  fi

  if [[ -n "$current_branch" && "$branch" == "$current_branch" ]]; then
    error "Skipping current branch '$branch'."
    had_errors=1
    continue
  fi

  printf '  - %s\n' "$branch"
  if ! run_cmd git branch -D "$branch"; then
    had_errors=1
  fi
done < <(git for-each-ref --format='%(refname:short)' refs/heads)

if [[ "${clean_remote}" == true ]]; then
  log "Refreshing remotes..."
  if ! run_cmd git fetch --all --prune; then
    error "Fetch/prune failed; continuing with locally known remote refs."
  fi

  log "Deleting remote branches..."
  while IFS= read -r remote; do
    [[ -n "$remote" ]] || continue

    log "Remote '$remote'"
    if remote_heads="$(git ls-remote --heads --refs "$remote" 2>/dev/null)"; then
      while IFS=$'\t' read -r _sha ref; do
        [[ -n "$ref" ]] || continue

        branch="${ref#refs/heads/}"
        [[ "$branch" != "$ref" ]] || continue

        if is_remote_kept_branch "$branch"; then
          continue
        fi

        printf '  - %s/%s\n' "$remote" "$branch"
        if ! run_cmd git push "$remote" --delete "$branch"; then
          had_errors=1
        fi
      done <<< "$remote_heads"
    else
      error "Could not list heads for '$remote' via ls-remote; using local tracking refs."
      while IFS= read -r remote_ref; do
        [[ -n "$remote_ref" ]] || continue

        branch="$remote_ref"
        if [[ "$branch" == "HEAD" ]]; then
          continue
        fi

        if is_remote_kept_branch "$branch"; then
          continue
        fi

        printf '  - %s/%s\n' "$remote" "$branch"
        if ! run_cmd git push "$remote" --delete "$branch"; then
          had_errors=1
        fi
      done < <(git for-each-ref --format='%(refname:lstrip=3)' "refs/remotes/$remote")
    fi
  done < <(git remote)
else
  log "Skipping remote cleanup (--local-only)."
fi

if [[ "$had_errors" -ne 0 ]]; then
  error "Completed with errors."
  exit 1
fi

log "Done."
