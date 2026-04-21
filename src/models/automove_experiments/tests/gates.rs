use super::*;

#[test]
fn duel_timing_stats_merge_and_average_track_profile_a_and_profile_b_turns() {
    let mut first = DuelTimingStats::default();
    first.record_profile_a_turn(120.0);
    first.record_profile_a_turn(180.0);
    first.record_profile_b_turn(80.0);

    let mut second = DuelTimingStats::default();
    second.record_profile_a_turn(60.0);
    second.record_profile_b_turn(20.0);
    second.record_profile_b_turn(40.0);

    first.merge(second);

    assert_eq!(first.profile_a_turns, 3);
    assert_eq!(first.profile_b_turns, 3);
    assert!((first.profile_a_total_ms - 360.0).abs() < 0.001);
    assert!((first.profile_b_total_ms - 140.0).abs() < 0.001);
    assert!((first.profile_a_avg_ms() - 120.0).abs() < 0.001);
    assert!((first.profile_b_avg_ms() - 46.666_666_7).abs() < 0.001);
}

#[test]
fn pro_reliability_gate_passes_only_when_all_matchups_clear_win_confidence_and_move_time() {
    let passing = ProReliabilityGateMetrics {
        win_rate: 0.90,
        confidence: 0.99,
        frontier_avg_ms: 700.0,
    };
    assert!(pro_reliability_gate_passes(passing, passing, passing));
    assert!(!pro_reliability_gate_passes(
        ProReliabilityGateMetrics {
            win_rate: 0.89,
            ..passing
        },
        passing,
        passing
    ));
    assert!(!pro_reliability_gate_passes(
        passing,
        ProReliabilityGateMetrics {
            confidence: 0.98,
            ..passing
        },
        passing
    ));
    assert!(!pro_reliability_gate_passes(
        passing,
        passing,
        ProReliabilityGateMetrics {
            frontier_avg_ms: 700.01,
            ..passing
        }
    ));
}

#[test]
fn runtime_preflight_checks_run_when_not_skipped() {
    let stage1_calls = std::cell::Cell::new(0);
    let exact_calls = std::cell::Cell::new(0);

    maybe_run_runtime_preflight_checks(
        false,
        || stage1_calls.set(stage1_calls.get() + 1),
        || exact_calls.set(exact_calls.get() + 1),
    );

    assert_eq!(stage1_calls.get(), 1);
    assert_eq!(exact_calls.get(), 1);
}

#[test]
fn runtime_preflight_checks_are_skipped_when_requested() {
    let stage1_calls = std::cell::Cell::new(0);
    let exact_calls = std::cell::Cell::new(0);

    maybe_run_runtime_preflight_checks(
        true,
        || stage1_calls.set(stage1_calls.get() + 1),
        || exact_calls.set(exact_calls.get() + 1),
    );

    assert_eq!(stage1_calls.get(), 0);
    assert_eq!(exact_calls.get(), 0);
}

#[test]
fn stage1_cpu_is_advisory_by_default_for_frontier_pro_profiles() {
    with_env_override("SMART_STAGE1_CPU_ADVISORY", "", || {
        assert!(stage1_cpu_is_advisory("frontier_pro_v2_guarded"));
        assert!(!stage1_cpu_is_advisory("shipping_pro_search"));
    });
}

#[test]
fn stage1_cpu_advisory_can_be_forced_off_for_frontier_pro_profiles() {
    with_env_override("SMART_STAGE1_CPU_ADVISORY", "false", || {
        assert!(!stage1_cpu_is_advisory("frontier_pro_v2_guarded"));
    });
}

#[test]
fn pro_signal_triage_accepts_target_change_with_bounded_off_target_churn() {
    assert!(pro_signal_triage_passes(
        "frontier_pro_v2_guarded",
        "shipping_pro_search",
        2,
        1
    ));
    assert!(pro_signal_triage_passes(
        "frontier_pro_v2_guarded",
        "shipping_pro_search",
        1,
        0
    ));
    assert!(pro_signal_triage_passes(
        "frontier_pro_v2_guarded",
        "shipping_pro_search",
        0,
        0
    ));
    assert!(!pro_signal_triage_passes(
        "frontier_pro_v2_guarded",
        "shipping_pro_search",
        1,
        2
    ));
}

#[test]
fn smart_automove_pool_profile_registry_resolves_retained_profiles() {
    for profile_id in retained_profile_ids() {
        assert!(
            profile_selector_from_name(profile_id).is_some(),
            "retained profile '{}' should resolve",
            profile_id
        );
    }
}

#[test]
fn smart_automove_pool_retained_profile_ids_match_active_registry() {
    assert_eq!(
        retained_profile_ids(),
        vec!["shipping_pro_search", "frontier_pro_v2_guarded"]
    );
}

#[test]
fn smart_automove_pool_archived_profiles_do_not_resolve() {
    for profile_id in [
        "base",
        "runtime_release_safe_pre_exact",
        "runtime_eff_exact_lite_v1",
        "runtime_pre_fast_root_quality_v1_normal_conversion_v3",
        "swift_2024_eval_reference",
        "swift_2024_style_reference",
        "runtime_normal_from_fast_reference_v1",
        "runtime_pro_turn_engine_v1",
    ] {
        assert!(
            profile_selector_from_name(profile_id).is_none(),
            "archived profile '{}' should not resolve",
            profile_id
        );
    }
}

#[test]
fn selected_profile_env_aliases_accept_legacy_candidate_names_and_ids() {
    with_env_override("SMART_SELECTED_PROFILE", "", || {
        with_env_override("SMART_FRONTIER_PROFILE", "", || {
            with_env_override("SMART_CANDIDATE_PROFILE", "", || {
                with_env_override(
                    "SMART_PRO_CANDIDATE_PROFILE",
                    "runtime_pro_turn_engine_v30",
                    || {
                        assert_eq!(selected_profile_id_from_env(), "frontier_pro_v2_guarded");
                    },
                );
            });
            with_env_override("SMART_PRO_CANDIDATE_PROFILE", "", || {
                with_env_override("SMART_CANDIDATE_PROFILE", "runtime_current", || {
                    assert_eq!(selected_profile_id_from_env(), "shipping_pro_search");
                });
            });
        });
    });
}

#[test]
fn frontier_and_shipping_profile_helpers_accept_legacy_candidate_and_baseline_envs() {
    with_env_override("SMART_FRONTIER_PROFILE", "", || {
        with_env_override("SMART_SHIPPING_PROFILE", "", || {
            with_env_override(
                "SMART_PRO_CANDIDATE_PROFILE",
                "runtime_pro_turn_engine_v30",
                || {
                    assert_eq!(frontier_profile_id(), "frontier_pro_v2_guarded");
                },
            );
            with_env_override("SMART_PRO_BASELINE_PROFILE", "runtime_current", || {
                assert_eq!(shipping_profile_id(), "shipping_pro_search");
            });
        });
    });
}

#[test]
fn reliability_and_probe_profile_helpers_accept_legacy_ids() {
    with_env_override(
        "SMART_PRO_RELIABILITY_FRONTIER_PROFILE",
        "runtime_pro_turn_engine_v30",
        || {
            assert_eq!(reliability_frontier_profile_id(), "frontier_pro_v2_guarded");
        },
    );
    with_env_override(
        "SMART_PRO_RELIABILITY_SHIPPING_PROFILE",
        "runtime_current",
        || {
            assert_eq!(reliability_shipping_profile_id(), "shipping_pro_search");
        },
    );
    with_env_override(
        "SMART_PROBE_FRONTIER_PROFILE",
        "runtime_pro_turn_engine_v30",
        || {
            assert_eq!(probe_frontier_profile_id(), "frontier_pro_v2_guarded");
        },
    );
    with_env_override("SMART_PROBE_SHIPPING_PROFILE", "runtime_current", || {
        assert_eq!(probe_shipping_profile_id(), "shipping_pro_search");
    });
}

#[test]
fn raw_env_string_preserves_legacy_profile_ids_for_seed_tags() {
    with_env_override("SMART_PRO_RELIABILITY_SEED_TAG", "runtime_current", || {
        assert_eq!(
            env_string_value("SMART_PRO_RELIABILITY_SEED_TAG"),
            Some("runtime_current".to_string())
        );
    });
    with_env_override(
        "SMART_PRO_RELIABILITY_SEED_TAG",
        "runtime_pro_turn_engine_v30",
        || {
            assert_eq!(
                env_string_value("SMART_PRO_RELIABILITY_SEED_TAG"),
                Some("runtime_pro_turn_engine_v30".to_string())
            );
        },
    );
}

#[test]
#[ignore = "tactical guardrail suite for the selected profile"]
fn smart_automove_tactical_selected_profile() {
    let profile_id = selected_profile_id().as_str().to_string();
    assert_tactical_guardrails(SELECTED_PROFILE_MODEL.select_inputs, profile_id.as_str());
    assert_interview_policy_regressions(SELECTED_PROFILE_MODEL.select_inputs, profile_id.as_str());
}

#[test]
#[ignore = "stage-1 cpu gate against shipping_pro_search; advisory-only for Pro frontiers when enabled"]
fn smart_automove_pool_stage1_cpu_non_regression_gate() {
    let frontier_profile_name = selected_profile_id().as_str().to_string();
    assert_stage1_cpu_non_regression(
        frontier_profile_name.as_str(),
        SELECTED_PROFILE_MODEL.select_inputs,
    );
}

#[test]
#[ignore = "exact-lite diagnostics gate for per-move budgets and cache efficiency"]
fn smart_automove_pool_exact_lite_diagnostics_gate() {
    let frontier_profile_name = selected_profile_id().as_str().to_string();
    assert_exact_lite_diagnostics_gate_if_enabled(
        frontier_profile_name.as_str(),
        SELECTED_PROFILE_MODEL.select_inputs,
    );
}

#[test]
#[ignore = "deterministic fixture-first triage for pro opening_reply and primary_pro surfaces"]
fn smart_automove_pool_pro_signal_triage() {
    let surface = triage_surface_from_env();
    let frontier_profile_name = frontier_profile_id();
    let shipping_profile_name = shipping_profile_id();
    let frontier_selector = profile_selector_from_name(frontier_profile_name.as_str())
        .unwrap_or_else(|| panic!("frontier '{}' not found", frontier_profile_name));
    let shipping_selector = profile_selector_from_name(shipping_profile_name.as_str())
        .unwrap_or_else(|| panic!("shipping '{}' not found", shipping_profile_name));

    assert_tactical_guardrails(frontier_selector, frontier_profile_name.as_str());
    assert_interview_policy_regressions(frontier_selector, frontier_profile_name.as_str());

    let opening_changed = compare_pro_triage_fixture_pack(
        TriageSurface::OpeningReply,
        frontier_profile_name.as_str(),
        frontier_selector,
        shipping_profile_name.as_str(),
        shipping_selector,
        opening_reply_triage_fixtures().as_slice(),
    );
    let primary_changed = compare_pro_triage_fixture_pack(
        TriageSurface::PrimaryPro,
        frontier_profile_name.as_str(),
        frontier_selector,
        shipping_profile_name.as_str(),
        shipping_selector,
        primary_pro_triage_fixtures().as_slice(),
    );

    let (target_changed, off_target_changed) = match surface {
        TriageSurface::OpeningReply => (opening_changed, primary_changed),
        TriageSurface::PrimaryPro => (primary_changed, opening_changed),
    };

    println!(
        "pro triage surface={} target_changed={} off_target_changed={}",
        surface.as_str(),
        target_changed,
        off_target_changed
    );
    assert!(
        pro_signal_triage_passes(
            frontier_profile_name.as_str(),
            shipping_profile_name.as_str(),
            target_changed,
            off_target_changed
        ),
        "pro triage failed for surface='{}': frontier='{}' shipping='{}' target_changed={} off_target_changed={} (expected target movement with <=1 off-target change, or a stable 0/0 result for the promoted frontier_pro_v2_guarded vs shipping_pro_search pair)",
        surface.as_str(),
        frontier_profile_name,
        shipping_profile_name,
        target_changed,
        off_target_changed
    );
}

#[test]
#[ignore = "reliability gate: retained pro profile vs shipping_pro_search pro, normal, and fast at pro budget with move-time cap"]
fn smart_automove_pool_pro_reliability_gate() {
    let frontier_profile = reliability_frontier_profile_id();
    let shipping_profile = reliability_shipping_profile_id();
    let frontier_selector = profile_selector_from_name(frontier_profile.as_str())
        .unwrap_or_else(|| panic!("frontier '{}' not found", frontier_profile));
    let shipping_selector = profile_selector_from_name(shipping_profile.as_str())
        .unwrap_or_else(|| panic!("shipping '{}' not found", shipping_profile));

    let skip_guardrails = env_bool("SMART_PRO_RELIABILITY_SKIP_GUARDRAILS").unwrap_or(false);
    if skip_guardrails {
        println!(
            "pro reliability gate: guardrails skipped by SMART_PRO_RELIABILITY_SKIP_GUARDRAILS=true"
        );
    } else {
        assert_runtime_preflight_if_required(frontier_profile.as_str(), frontier_selector);
        assert_tactical_guardrails(frontier_selector, frontier_profile.as_str());
        assert_tactical_guardrails(shipping_selector, shipping_profile.as_str());
    }

    let repeats = env_usize("SMART_PRO_RELIABILITY_REPEATS")
        .unwrap_or(3)
        .max(1);
    let games = env_usize("SMART_PRO_RELIABILITY_GAMES").unwrap_or(2).max(1);
    let max_plies_floor = if skip_guardrails { 8 } else { 56 };
    let max_plies = env_usize("SMART_PRO_RELIABILITY_MAX_PLIES")
        .unwrap_or(96)
        .max(max_plies_floor);
    let seed_tag = env_string_value("SMART_PRO_RELIABILITY_SEED_TAG")
        .unwrap_or_else(|| "pro_turn_planner_reliability_v1".to_string());
    let normal_seed_tag = format!("{}_vs_normal", seed_tag);
    let fast_seed_tag = format!("{}_vs_fast", seed_tag);

    let pro_stats = run_cross_budget_duel_with_timing(CrossBudgetDuelConfig {
        profile_a: frontier_profile.as_str(),
        mode_a: SmartAutomovePreference::Pro,
        profile_b: shipping_profile.as_str(),
        mode_b: SmartAutomovePreference::Pro,
        seed_tag: seed_tag.as_str(),
        repeats,
        games_per_repeat: games,
        max_plies,
        use_white_opening_book: false,
    });
    let normal_stats = run_cross_budget_duel_with_timing(CrossBudgetDuelConfig {
        profile_a: frontier_profile.as_str(),
        mode_a: SmartAutomovePreference::Pro,
        profile_b: shipping_profile.as_str(),
        mode_b: SmartAutomovePreference::Normal,
        seed_tag: normal_seed_tag.as_str(),
        repeats,
        games_per_repeat: games,
        max_plies,
        use_white_opening_book: false,
    });
    let fast_stats = run_cross_budget_duel_with_timing(CrossBudgetDuelConfig {
        profile_a: frontier_profile.as_str(),
        mode_a: SmartAutomovePreference::Pro,
        profile_b: shipping_profile.as_str(),
        mode_b: SmartAutomovePreference::Fast,
        seed_tag: fast_seed_tag.as_str(),
        repeats,
        games_per_repeat: games,
        max_plies,
        use_white_opening_book: false,
    });

    let pro_total_games = pro_stats.matchup.total_games();
    let pro_metrics = ProReliabilityGateMetrics {
        win_rate: pro_stats.matchup.win_rate_points(),
        confidence: pro_stats.matchup.confidence_better_than_even(),
        frontier_avg_ms: pro_stats.timing.profile_a_avg_ms(),
    };
    println!(
        "pro reliability gate vs shipping pro: frontier={} shipping={} total_games={} win_rate={:.4} confidence={:.4} frontier_avg_ms={:.2} shipping_avg_ms={:.2} frontier_turns={} shipping_turns={}",
        frontier_profile,
        shipping_profile,
        pro_total_games,
        pro_metrics.win_rate,
        pro_metrics.confidence,
        pro_metrics.frontier_avg_ms,
        pro_stats.timing.profile_b_avg_ms(),
        pro_stats.timing.profile_a_turns,
        pro_stats.timing.profile_b_turns
    );

    let normal_total_games = normal_stats.matchup.total_games();
    let normal_metrics = ProReliabilityGateMetrics {
        win_rate: normal_stats.matchup.win_rate_points(),
        confidence: normal_stats.matchup.confidence_better_than_even(),
        frontier_avg_ms: normal_stats.timing.profile_a_avg_ms(),
    };
    println!(
        "pro reliability gate vs shipping normal: frontier={} shipping={} total_games={} win_rate={:.4} confidence={:.4} frontier_avg_ms={:.2} shipping_avg_ms={:.2} frontier_turns={} shipping_turns={}",
        frontier_profile,
        shipping_profile,
        normal_total_games,
        normal_metrics.win_rate,
        normal_metrics.confidence,
        normal_metrics.frontier_avg_ms,
        normal_stats.timing.profile_b_avg_ms(),
        normal_stats.timing.profile_a_turns,
        normal_stats.timing.profile_b_turns
    );

    let fast_total_games = fast_stats.matchup.total_games();
    let fast_metrics = ProReliabilityGateMetrics {
        win_rate: fast_stats.matchup.win_rate_points(),
        confidence: fast_stats.matchup.confidence_better_than_even(),
        frontier_avg_ms: fast_stats.timing.profile_a_avg_ms(),
    };
    println!(
        "pro reliability gate vs shipping fast: frontier={} shipping={} total_games={} win_rate={:.4} confidence={:.4} frontier_avg_ms={:.2} shipping_avg_ms={:.2} frontier_turns={} shipping_turns={}",
        frontier_profile,
        shipping_profile,
        fast_total_games,
        fast_metrics.win_rate,
        fast_metrics.confidence,
        fast_metrics.frontier_avg_ms,
        fast_stats.timing.profile_b_avg_ms(),
        fast_stats.timing.profile_a_turns,
        fast_stats.timing.profile_b_turns
    );

    let expected_games = repeats.saturating_mul(games).saturating_mul(2);
    assert_eq!(
        pro_total_games, expected_games,
        "pro reliability gate vs shipping pro expected {} mirrored games but ran {}",
        expected_games, pro_total_games
    );
    assert_eq!(
        normal_total_games, expected_games,
        "pro reliability gate vs shipping normal expected {} mirrored games but ran {}",
        expected_games, normal_total_games
    );
    assert_eq!(
        fast_total_games, expected_games,
        "pro reliability gate vs shipping fast expected {} mirrored games but ran {}",
        expected_games, fast_total_games
    );
    assert!(
        pro_reliability_gate_passes(pro_metrics, normal_metrics, fast_metrics),
        "pro reliability gate failed overall: vs_shipping_pro [win_rate {:.4} confidence {:.4} frontier_avg_ms {:.2}ms] vs_shipping_normal [win_rate {:.4} confidence {:.4} frontier_avg_ms {:.2}ms] vs_shipping_fast [win_rate {:.4} confidence {:.4} frontier_avg_ms {:.2}ms] (required each duel to satisfy win_rate >= {:.2}, confidence >= {:.2}, frontier_avg_ms <= {:.2}ms)",
        pro_metrics.win_rate,
        pro_metrics.confidence,
        pro_metrics.frontier_avg_ms,
        normal_metrics.win_rate,
        normal_metrics.confidence,
        normal_metrics.frontier_avg_ms,
        fast_metrics.win_rate,
        fast_metrics.confidence,
        fast_metrics.frontier_avg_ms,
        SMART_PRO_RELIABILITY_WIN_RATE_MIN,
        SMART_PRO_RELIABILITY_CONFIDENCE_MIN,
        SMART_PRO_RELIABILITY_MOVE_AVG_MS_MAX
    );
    assert_pro_reliability_duel_passes("pro reliability gate vs shipping pro", pro_metrics);
    assert_pro_reliability_duel_passes("pro reliability gate vs shipping normal", normal_metrics);
    assert_pro_reliability_duel_passes("pro reliability gate vs shipping fast", fast_metrics);
}
