use super::*;

#[test]
fn duel_timing_stats_merge_and_average_track_candidate_and_baseline_turns() {
    let mut first = DuelTimingStats::default();
    first.record_candidate_turn(120.0);
    first.record_candidate_turn(180.0);
    first.record_baseline_turn(80.0);

    let mut second = DuelTimingStats::default();
    second.record_candidate_turn(60.0);
    second.record_baseline_turn(20.0);
    second.record_baseline_turn(40.0);

    first.merge(second);

    assert_eq!(first.candidate_turns, 3);
    assert_eq!(first.baseline_turns, 3);
    assert!((first.candidate_total_ms - 360.0).abs() < 0.001);
    assert!((first.baseline_total_ms - 140.0).abs() < 0.001);
    assert!((first.candidate_avg_ms() - 120.0).abs() < 0.001);
    assert!((first.baseline_avg_ms() - 46.666_666_7).abs() < 0.001);
}

#[test]
fn pro_reliability_gate_passes_only_when_all_matchups_clear_win_confidence_and_move_time() {
    let passing = ProReliabilityGateMetrics {
        win_rate: 0.90,
        confidence: 0.99,
        candidate_avg_ms: 700.0,
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
            candidate_avg_ms: 700.01,
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
fn pro_signal_triage_accepts_target_change_with_bounded_off_target_churn() {
    assert!(pro_signal_triage_passes(
        "runtime_pro_turn_engine_v30",
        "runtime_current",
        2,
        1
    ));
    assert!(pro_signal_triage_passes(
        "runtime_pro_turn_engine_v30",
        "runtime_current",
        1,
        0
    ));
    assert!(pro_signal_triage_passes(
        "runtime_pro_turn_engine_v30",
        "runtime_current",
        0,
        0
    ));
    assert!(!pro_signal_triage_passes(
        "runtime_eff_exact_lite_v1",
        "runtime_release_safe_pre_exact",
        0,
        0
    ));
    assert!(!pro_signal_triage_passes(
        "runtime_pro_turn_engine_v30",
        "runtime_current",
        1,
        2
    ));
}

#[test]
fn triage_calibration_probe_detects_reply_risk_profile_delta() {
    let candidate =
        reply_risk_calibration_probe("runtime_pre_fast_root_quality_v1_normal_conversion_v3");
    let baseline = reply_risk_calibration_probe("runtime_release_safe_pre_exact");
    assert!(candidate > baseline);
}

#[test]
fn triage_calibration_probe_detects_opponent_mana_profile_delta() {
    let candidate =
        opponent_mana_calibration_probe("runtime_pre_fast_root_quality_v1_normal_conversion_v3");
    let baseline = opponent_mana_calibration_probe("runtime_release_safe_pre_exact");
    assert_ne!(candidate, baseline);
}

#[test]
fn triage_calibration_probe_detects_supermana_profile_delta() {
    let candidate = supermana_calibration_probe("runtime_eff_exact_lite_v1");
    let baseline = supermana_calibration_probe("runtime_release_safe_pre_exact");
    assert!(candidate);
    assert!(!baseline);
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
        vec![
            "base",
            "runtime_current",
            "runtime_release_safe_pre_exact",
            "runtime_eff_exact_lite_v1",
            "swift_2024_eval_reference",
            "swift_2024_style_reference",
            "runtime_pre_fast_root_quality_v1_normal_conversion_v3",
            "runtime_normal_from_fast_reference_v1",
            "runtime_pro_turn_engine_v30",
        ]
    );
}

#[test]
fn smart_automove_pool_archived_runtime_pro_turn_engine_v1_does_not_resolve() {
    assert!(profile_selector_from_name("runtime_pro_turn_engine_v1").is_none());
}

#[test]
fn smart_automove_pool_curated_pool_profiles_are_unique_and_resolvable() {
    let pool = selected_pool_models();
    assert_eq!(pool.len(), CURATED_POOL_SIZE);

    for model in &pool {
        assert!(
            retained_profile_ids().contains(&model.id),
            "curated pool model '{}' should come from retained registry",
            model.id
        );
    }

    for (index, left) in pool.iter().enumerate() {
        for right in pool.iter().skip(index + 1) {
            assert_ne!(left.id, right.id, "curated pool ids must be unique");
            assert!(
                !std::ptr::fn_addr_eq(left.select_inputs, right.select_inputs),
                "curated pool selectors must be unique: {} and {}",
                left.id,
                right.id
            );
        }
    }
}

#[test]
fn smart_automove_pool_smoke_runs() {
    let probe_model = AutomoveModel {
        id: "smoke_probe_candidate",
        select_inputs: model_first_legal_automove,
    };
    let quick_budgets = [SearchBudget {
        label: "smoke_probe",
        depth: 1,
        max_nodes: 1,
    }];
    let pool = vec![AutomoveModel {
        id: "smoke_probe_pool",
        select_inputs: model_first_legal_automove,
    }];

    let evaluation =
        evaluate_candidate_against_pool_with_max_plies(probe_model, &pool, 1, &quick_budgets, 2);
    assert_eq!(evaluation.opponents.len(), pool.len());
    assert_eq!(evaluation.games_per_matchup, 1);
}

#[test]
#[ignore = "tactical guardrail suite for selected candidate profile"]
fn smart_automove_tactical_candidate_profile() {
    let profile_name = candidate_profile().as_str().to_string();
    assert_tactical_guardrails(CANDIDATE_MODEL.select_inputs, profile_name.as_str());
    assert_interview_policy_regressions(CANDIDATE_MODEL.select_inputs, profile_name.as_str());
}

#[test]
#[ignore = "stage-1 cpu gate against runtime_current; advisory-only for Pro candidates when enabled"]
fn smart_automove_pool_stage1_cpu_non_regression_gate() {
    let candidate_profile_name = candidate_profile().as_str().to_string();
    assert_stage1_cpu_non_regression(
        candidate_profile_name.as_str(),
        CANDIDATE_MODEL.select_inputs,
    );
}

#[test]
#[ignore = "exact-lite diagnostics gate for per-move budgets and cache efficiency"]
fn smart_automove_pool_exact_lite_diagnostics_gate() {
    let candidate_profile_name = candidate_profile().as_str().to_string();
    assert_exact_lite_diagnostics_gate_if_enabled(
        candidate_profile_name.as_str(),
        CANDIDATE_MODEL.select_inputs,
    );
}

#[test]
#[ignore = "retained-profile calibration for triage surfaces"]
fn smart_automove_pool_surface_calibration() {
    let surface = triage_surface_from_env();
    let candidate_profile_name = candidate_profile().as_str().to_string();
    let baseline_profile_name = gate_baseline_profile_name();

    match surface {
        TriageSurface::ReplyRisk => {
            let candidate_probe = reply_risk_calibration_probe(candidate_profile_name.as_str());
            let baseline_probe = reply_risk_calibration_probe(baseline_profile_name.as_str());
            println!(
                "triage-calibrate surface=reply_risk candidate={} probe={} baseline={} probe={} delta={}",
                candidate_profile_name,
                candidate_probe,
                baseline_profile_name,
                baseline_probe,
                candidate_probe - baseline_probe
            );
            assert!(
                candidate_probe.abs_diff(baseline_probe) >= 20,
                "reply_risk calibration found no meaningful profile delta: candidate={} baseline={}",
                candidate_probe,
                baseline_probe
            );
        }
        TriageSurface::OpponentMana => {
            let candidate_pick = opponent_mana_calibration_probe(candidate_profile_name.as_str());
            let baseline_pick = opponent_mana_calibration_probe(baseline_profile_name.as_str());
            println!(
                "triage-calibrate surface=opponent_mana candidate={} pick={} baseline={} pick={}",
                candidate_profile_name, candidate_pick, baseline_profile_name, baseline_pick
            );
            assert_ne!(
                candidate_pick, baseline_pick,
                "opponent_mana calibration found no profile delta: candidate_pick={} baseline_pick={}",
                candidate_pick, baseline_pick
            );
        }
        TriageSurface::Supermana => {
            let candidate_probe = supermana_calibration_probe(candidate_profile_name.as_str());
            let baseline_probe = supermana_calibration_probe(baseline_profile_name.as_str());
            println!(
                "triage-calibrate surface=supermana candidate={} exact_lite={} baseline={} exact_lite={}",
                candidate_profile_name,
                candidate_probe,
                baseline_profile_name,
                baseline_probe
            );
            assert_ne!(
                candidate_probe, baseline_probe,
                "supermana calibration found no profile delta: candidate_exact_lite={} baseline_exact_lite={}",
                candidate_probe, baseline_probe
            );
        }
        _ => {
            panic!(
                "triage-calibrate only supports SMART_TRIAGE_SURFACE=reply_risk|opponent_mana|supermana; got '{}'",
                surface.as_str()
            );
        }
    }
}

#[test]
#[ignore = "deterministic fixture-first triage for pro opening_reply and primary_pro surfaces"]
fn smart_automove_pool_pro_signal_triage() {
    let surface = triage_surface_from_env();
    let candidate_profile_name = pro_candidate_profile_name();
    let baseline_profile_name = pro_baseline_profile_name();
    let candidate_selector = profile_selector_from_name(candidate_profile_name.as_str())
        .unwrap_or_else(|| panic!("candidate '{}' not found", candidate_profile_name));
    let baseline_selector = profile_selector_from_name(baseline_profile_name.as_str())
        .unwrap_or_else(|| panic!("baseline '{}' not found", baseline_profile_name));

    assert_tactical_guardrails(candidate_selector, candidate_profile_name.as_str());
    assert_interview_policy_regressions(candidate_selector, candidate_profile_name.as_str());

    let opening_changed = compare_pro_triage_fixture_pack(
        TriageSurface::OpeningReply,
        candidate_profile_name.as_str(),
        candidate_selector,
        baseline_profile_name.as_str(),
        baseline_selector,
        opening_reply_triage_fixtures().as_slice(),
    );
    let primary_changed = compare_pro_triage_fixture_pack(
        TriageSurface::PrimaryPro,
        candidate_profile_name.as_str(),
        candidate_selector,
        baseline_profile_name.as_str(),
        baseline_selector,
        primary_pro_triage_fixtures().as_slice(),
    );

    let (target_changed, off_target_changed) = match surface {
        TriageSurface::OpeningReply => (opening_changed, primary_changed),
        TriageSurface::PrimaryPro => (primary_changed, opening_changed),
        _ => {
            panic!(
                "pro-triage only supports SMART_TRIAGE_SURFACE=opening_reply or primary_pro; got '{}'",
                surface.as_str()
            );
        }
    };

    println!(
        "pro triage surface={} target_changed={} off_target_changed={}",
        surface.as_str(),
        target_changed,
        off_target_changed
    );
    assert!(
        pro_signal_triage_passes(
            candidate_profile_name.as_str(),
            baseline_profile_name.as_str(),
            target_changed,
            off_target_changed
        ),
        "pro triage failed for surface='{}': candidate='{}' baseline='{}' target_changed={} off_target_changed={} (expected target movement with <=1 off-target change, or a stable 0/0 result for the promoted runtime_pro_turn_engine_v30 vs runtime_current pair)",
        surface.as_str(),
        candidate_profile_name,
        baseline_profile_name,
        target_changed,
        off_target_changed
    );
}

#[test]
#[ignore = "reliability gate: retained pro profile vs runtime_current pro, normal, and fast at pro budget with move-time cap"]
fn smart_automove_pool_pro_reliability_gate() {
    let candidate_profile = env_profile_name("SMART_PRO_RELIABILITY_CANDIDATE_PROFILE")
        .unwrap_or_else(|| "runtime_pro_turn_engine_v30".to_string());
    let baseline_profile = env_profile_name("SMART_PRO_RELIABILITY_BASELINE_PROFILE")
        .unwrap_or_else(|| "runtime_current".to_string());
    let candidate_selector = profile_selector_from_name(candidate_profile.as_str())
        .unwrap_or_else(|| panic!("candidate '{}' not found", candidate_profile));
    let baseline_selector = profile_selector_from_name(baseline_profile.as_str())
        .unwrap_or_else(|| panic!("baseline '{}' not found", baseline_profile));

    let skip_guardrails = env_bool("SMART_PRO_RELIABILITY_SKIP_GUARDRAILS").unwrap_or(false);
    if skip_guardrails {
        println!(
            "pro reliability gate: guardrails skipped by SMART_PRO_RELIABILITY_SKIP_GUARDRAILS=true"
        );
    } else {
        assert_runtime_preflight_if_required(candidate_profile.as_str(), candidate_selector);
        assert_tactical_guardrails(candidate_selector, candidate_profile.as_str());
        assert_tactical_guardrails(baseline_selector, baseline_profile.as_str());
    }

    let repeats = env_usize("SMART_PRO_RELIABILITY_REPEATS")
        .unwrap_or(3)
        .max(1);
    let games = env_usize("SMART_PRO_RELIABILITY_GAMES").unwrap_or(2).max(1);
    let max_plies_floor = if skip_guardrails { 8 } else { 56 };
    let max_plies = env_usize("SMART_PRO_RELIABILITY_MAX_PLIES")
        .unwrap_or(96)
        .max(max_plies_floor);
    let seed_tag = env_profile_name("SMART_PRO_RELIABILITY_SEED_TAG")
        .unwrap_or_else(|| "pro_turn_planner_reliability_v1".to_string());
    let normal_seed_tag = format!("{}_vs_normal", seed_tag);
    let fast_seed_tag = format!("{}_vs_fast", seed_tag);

    let pro_stats = run_cross_budget_duel_with_timing(CrossBudgetDuelConfig {
        profile_a: candidate_profile.as_str(),
        mode_a: SmartAutomovePreference::Pro,
        profile_b: baseline_profile.as_str(),
        mode_b: SmartAutomovePreference::Pro,
        seed_tag: seed_tag.as_str(),
        repeats,
        games_per_repeat: games,
        max_plies,
        use_white_opening_book: false,
    });
    let normal_stats = run_cross_budget_duel_with_timing(CrossBudgetDuelConfig {
        profile_a: candidate_profile.as_str(),
        mode_a: SmartAutomovePreference::Pro,
        profile_b: baseline_profile.as_str(),
        mode_b: SmartAutomovePreference::Normal,
        seed_tag: normal_seed_tag.as_str(),
        repeats,
        games_per_repeat: games,
        max_plies,
        use_white_opening_book: false,
    });
    let fast_stats = run_cross_budget_duel_with_timing(CrossBudgetDuelConfig {
        profile_a: candidate_profile.as_str(),
        mode_a: SmartAutomovePreference::Pro,
        profile_b: baseline_profile.as_str(),
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
        candidate_avg_ms: pro_stats.timing.candidate_avg_ms(),
    };
    println!(
        "pro reliability gate vs current pro: candidate={} baseline={} total_games={} win_rate={:.4} confidence={:.4} candidate_avg_ms={:.2} baseline_avg_ms={:.2} candidate_turns={} baseline_turns={}",
        candidate_profile,
        baseline_profile,
        pro_total_games,
        pro_metrics.win_rate,
        pro_metrics.confidence,
        pro_metrics.candidate_avg_ms,
        pro_stats.timing.baseline_avg_ms(),
        pro_stats.timing.candidate_turns,
        pro_stats.timing.baseline_turns
    );

    let normal_total_games = normal_stats.matchup.total_games();
    let normal_metrics = ProReliabilityGateMetrics {
        win_rate: normal_stats.matchup.win_rate_points(),
        confidence: normal_stats.matchup.confidence_better_than_even(),
        candidate_avg_ms: normal_stats.timing.candidate_avg_ms(),
    };
    println!(
        "pro reliability gate vs current normal: candidate={} baseline={} total_games={} win_rate={:.4} confidence={:.4} candidate_avg_ms={:.2} baseline_avg_ms={:.2} candidate_turns={} baseline_turns={}",
        candidate_profile,
        baseline_profile,
        normal_total_games,
        normal_metrics.win_rate,
        normal_metrics.confidence,
        normal_metrics.candidate_avg_ms,
        normal_stats.timing.baseline_avg_ms(),
        normal_stats.timing.candidate_turns,
        normal_stats.timing.baseline_turns
    );

    let fast_total_games = fast_stats.matchup.total_games();
    let fast_metrics = ProReliabilityGateMetrics {
        win_rate: fast_stats.matchup.win_rate_points(),
        confidence: fast_stats.matchup.confidence_better_than_even(),
        candidate_avg_ms: fast_stats.timing.candidate_avg_ms(),
    };
    println!(
        "pro reliability gate vs current fast: candidate={} baseline={} total_games={} win_rate={:.4} confidence={:.4} candidate_avg_ms={:.2} baseline_avg_ms={:.2} candidate_turns={} baseline_turns={}",
        candidate_profile,
        baseline_profile,
        fast_total_games,
        fast_metrics.win_rate,
        fast_metrics.confidence,
        fast_metrics.candidate_avg_ms,
        fast_stats.timing.baseline_avg_ms(),
        fast_stats.timing.candidate_turns,
        fast_stats.timing.baseline_turns
    );

    let expected_games = repeats.saturating_mul(games).saturating_mul(2);
    assert_eq!(
        pro_total_games, expected_games,
        "pro reliability gate vs current pro expected {} mirrored games but ran {}",
        expected_games, pro_total_games
    );
    assert_eq!(
        normal_total_games, expected_games,
        "pro reliability gate vs current normal expected {} mirrored games but ran {}",
        expected_games, normal_total_games
    );
    assert_eq!(
        fast_total_games, expected_games,
        "pro reliability gate vs current fast expected {} mirrored games but ran {}",
        expected_games, fast_total_games
    );
    assert!(
        pro_reliability_gate_passes(pro_metrics, normal_metrics, fast_metrics),
        "pro reliability gate failed overall: vs_current_pro [win_rate {:.4} confidence {:.4} candidate_avg_ms {:.2}ms] vs_current_normal [win_rate {:.4} confidence {:.4} candidate_avg_ms {:.2}ms] vs_current_fast [win_rate {:.4} confidence {:.4} candidate_avg_ms {:.2}ms] (required each duel to satisfy win_rate >= {:.2}, confidence >= {:.2}, candidate_avg_ms <= {:.2}ms)",
        pro_metrics.win_rate,
        pro_metrics.confidence,
        pro_metrics.candidate_avg_ms,
        normal_metrics.win_rate,
        normal_metrics.confidence,
        normal_metrics.candidate_avg_ms,
        fast_metrics.win_rate,
        fast_metrics.confidence,
        fast_metrics.candidate_avg_ms,
        SMART_PRO_RELIABILITY_WIN_RATE_MIN,
        SMART_PRO_RELIABILITY_CONFIDENCE_MIN,
        SMART_PRO_RELIABILITY_MOVE_AVG_MS_MAX
    );
    assert_pro_reliability_duel_passes("pro reliability gate vs current pro", pro_metrics);
    assert_pro_reliability_duel_passes("pro reliability gate vs current normal", normal_metrics);
    assert_pro_reliability_duel_passes("pro reliability gate vs current fast", fast_metrics);
}
