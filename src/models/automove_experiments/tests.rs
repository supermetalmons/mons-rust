use super::*;
use super::harness::*;
use super::profiles::*;

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
#[ignore = "profile speed probe on fixed opening positions"]
fn smart_automove_pool_profile_speed_probe() {
    let positions = env_usize("SMART_SPEED_POSITIONS").unwrap_or(20).max(1);
    let openings = generate_opening_fens(seed_for_pairing("speed", "probe"), positions);
    let profile = candidate_profile().as_str().to_string();
    let selector = CANDIDATE_MODEL.select_inputs;
    let client_budgets = client_budgets();

    println!(
        "speed probe: profile={} positions={} modes={:?}",
        profile,
        positions,
        client_budgets
            .iter()
            .map(|budget| budget.key().to_string())
            .collect::<Vec<_>>()
    );

    for stat in profile_speed_by_mode_ms(selector, openings.as_slice(), &client_budgets) {
        println!(
            "speed probe mode {}: avg_ms_per_position={:.2}",
            stat.budget.key(),
            stat.avg_ms
        );
    }
}

#[test]
#[ignore = "diagnostic: compare candidate vs baseline pool deltas per mode/opponent"]
fn smart_automove_pool_pool_regression_diagnostic() {
    let candidate_profile_name = candidate_profile().as_str().to_string();
    let baseline_profile_name = gate_baseline_profile_name();
    let candidate = AutomoveModel {
        id: "candidate",
        select_inputs: CANDIDATE_MODEL.select_inputs,
    };
    let baseline = AutomoveModel {
        id: "baseline",
        select_inputs: profile_selector_from_name(baseline_profile_name.as_str())
            .unwrap_or_else(|| panic!("baseline '{}' not found", baseline_profile_name)),
    };
    let budgets = client_budgets().to_vec();
    let pool_games = env_usize("SMART_GATE_POOL_GAMES").unwrap_or(3).max(1);
    let (candidate_eval, baseline_eval, candidate_wr, baseline_wr) =
        run_pool_non_regression_check(candidate, baseline, budgets.as_slice(), pool_games);

    println!(
        "pool regression diagnostic: candidate={} baseline={} games={} candidate_wr={:.3} baseline_wr={:.3} delta={:+.3}",
        candidate_profile_name,
        baseline_profile_name,
        pool_games,
        candidate_wr,
        baseline_wr,
        candidate_wr - baseline_wr
    );
    println!(
        "candidate beaten={}/{} | baseline beaten={}/{}",
        candidate_eval.beaten_opponents,
        candidate_eval.opponents.len(),
        baseline_eval.beaten_opponents,
        baseline_eval.opponents.len()
    );

    for budget in budgets {
        let Some(candidate_mode) = candidate_eval
            .mode_results
            .iter()
            .find(|mode| mode.budget.key() == budget.key())
        else {
            continue;
        };
        let Some(baseline_mode) = baseline_eval
            .mode_results
            .iter()
            .find(|mode| mode.budget.key() == budget.key())
        else {
            continue;
        };
        println!(
            "mode {}: candidate_wr={:.3} baseline_wr={:.3} delta={:+.3}",
            budget.key(),
            candidate_mode.aggregate_stats.win_rate_points(),
            baseline_mode.aggregate_stats.win_rate_points(),
            candidate_mode.aggregate_stats.win_rate_points()
                - baseline_mode.aggregate_stats.win_rate_points(),
        );
    }
}

#[test]
#[ignore = "tactical guardrail suite for runtime candidate quality"]
fn smart_automove_tactical_suite() {
    let runtime_selector = profile_selector_from_name("runtime_current")
        .expect("runtime_current selector should exist");
    assert_tactical_guardrails(runtime_selector, "runtime_current");
}

#[test]
#[ignore = "tactical guardrail suite for selected candidate profile"]
fn smart_automove_tactical_candidate_profile() {
    let profile_name = candidate_profile().as_str().to_string();
    assert_tactical_guardrails(CANDIDATE_MODEL.select_inputs, profile_name.as_str());
    assert_interview_policy_regressions(CANDIDATE_MODEL.select_inputs, profile_name.as_str());
}

#[test]
#[ignore = "quick progressive screen: ~10-20 seconds, 2 tiers"]
fn smart_automove_pool_fast_screen() {
    let candidate_profile_name = candidate_profile().as_str().to_string();
    let baseline_profile_name = gate_baseline_profile_name();
    let allow_self_baseline = env_bool("SMART_GATE_ALLOW_SELF_BASELINE").unwrap_or(false);
    if !allow_self_baseline {
        assert!(
            candidate_profile_name != baseline_profile_name,
            "candidate and baseline must differ (set SMART_GATE_ALLOW_SELF_BASELINE=true to override)"
        );
    }

    let candidate = AutomoveModel {
        id: "candidate",
        select_inputs: CANDIDATE_MODEL.select_inputs,
    };
    let baseline = AutomoveModel {
        id: "baseline",
        select_inputs: profile_selector_from_name(baseline_profile_name.as_str())
            .unwrap_or_else(|| panic!("baseline '{}' not found", baseline_profile_name)),
    };
    let budgets = client_budgets().to_vec();
    let config = ProgressiveDuelConfig::from_env_with_defaults("screen");
    let artifact_path = env::var("SMART_LADDER_ARTIFACT_PATH")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default_progressive_artifact_path(&candidate_profile_name));
    let result = run_progressive_duel(
        candidate,
        baseline,
        budgets.as_slice(),
        &ProgressiveDuelConfig {
            initial_games: config.initial_games.max(2),
            max_games_per_seed: config.max_games_per_seed.max(4).min(8),
            seed_tags: vec!["neutral_v1"],
            max_plies: 72,
            early_exit_delta_floor: -0.10,
            ..config
        },
        Some(artifact_path.as_str()),
    );

    println!(
        "fast screen: {} vs {} | games={} delta={:.4} confidence={:.3} stop={:?}",
        candidate_profile_name,
        baseline_profile_name,
        result.total_games,
        result.final_delta,
        result.final_confidence,
        result.stop_reason
    );

    match result.stop_reason {
        ProgressiveStopReason::EarlyReject | ProgressiveStopReason::MathematicalReject => {
            panic!("fast screen rejected candidate");
        }
        ProgressiveStopReason::EarlyPromote => {}
        ProgressiveStopReason::MaxGamesReached => {
            assert!(
                result.final_delta >= 0.0,
                "fast screen: negative delta at max games"
            );
        }
    }
}

#[test]
#[ignore = "progressive evaluation: geometric doubling, 2→4→8→16→32 games"]
fn smart_automove_pool_progressive_duel() {
    let candidate_profile_name = candidate_profile().as_str().to_string();
    let baseline_profile_name = gate_baseline_profile_name();
    let allow_self_baseline = env_bool("SMART_GATE_ALLOW_SELF_BASELINE").unwrap_or(false);
    if !allow_self_baseline {
        assert!(
            candidate_profile_name != baseline_profile_name,
            "candidate and baseline must differ"
        );
    }

    let candidate = AutomoveModel {
        id: "candidate",
        select_inputs: CANDIDATE_MODEL.select_inputs,
    };
    let baseline = AutomoveModel {
        id: "baseline",
        select_inputs: profile_selector_from_name(baseline_profile_name.as_str())
            .unwrap_or_else(|| panic!("baseline '{}' not found", baseline_profile_name)),
    };
    let budgets = client_budgets().to_vec();
    let config = if env_bool("SMART_PROGRESSIVE_PRIMARY").unwrap_or(false) {
        ProgressiveDuelConfig::from_env_with_defaults("primary")
    } else {
        ProgressiveDuelConfig::from_env_with_defaults("duel")
    };
    let artifact_path = env::var("SMART_LADDER_ARTIFACT_PATH")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default_progressive_artifact_path(&candidate_profile_name));
    let result = run_progressive_duel(
        candidate,
        baseline,
        budgets.as_slice(),
        &config,
        Some(artifact_path.as_str()),
    );

    println!(
        "progressive duel: {} vs {} | total_games={} delta={:.4} confidence={:.3} stop={:?}",
        candidate_profile_name,
        baseline_profile_name,
        result.total_games,
        result.final_delta,
        result.final_confidence,
        result.stop_reason
    );

    match result.stop_reason {
        ProgressiveStopReason::EarlyReject | ProgressiveStopReason::MathematicalReject => {
            panic!(
                "progressive duel rejected: {:?} at {} games, δ={:.4}",
                result.stop_reason,
                result.total_games,
                result.final_delta
            );
        }
        ProgressiveStopReason::EarlyPromote | ProgressiveStopReason::MaxGamesReached => {
            assert!(
                result.final_delta >= 0.0,
                "progressive duel failed aggregate non-regression: delta {:.4} < 0.0",
                result.final_delta
            );
        }
    }
}

#[test]
#[ignore = "staged promotion ladder with early-stop pruning and artifact output"]
fn smart_automove_pool_promotion_ladder() {
    let candidate_profile_name = candidate_profile().as_str().to_string();
    let baseline_profile_name = gate_baseline_profile_name();
    let allow_self_baseline = env_bool("SMART_GATE_ALLOW_SELF_BASELINE").unwrap_or(false);
    if !allow_self_baseline {
        assert!(
            candidate_profile_name != baseline_profile_name,
            "candidate profile and baseline profile must differ for ladder gate"
        );
    }

    let candidate = AutomoveModel {
        id: "candidate",
        select_inputs: CANDIDATE_MODEL.select_inputs,
    };
    let baseline = AutomoveModel {
        id: "baseline",
        select_inputs: profile_selector_from_name(baseline_profile_name.as_str()).unwrap_or_else(
            || panic!("baseline selector '{}' should exist", baseline_profile_name),
        ),
    };
    let budgets = client_budgets().to_vec();
    let mut artifacts = Vec::<String>::new();

    assert_tactical_guardrails(candidate.select_inputs, candidate_profile_name.as_str());
    assert_interview_policy_regressions(candidate.select_inputs, candidate_profile_name.as_str());
    assert_tactical_guardrails(baseline.select_inputs, baseline_profile_name.as_str());
    assert_interview_policy_regressions(baseline.select_inputs, baseline_profile_name.as_str());
    artifacts.push(format!(
        r#"{{"stage":"A_tactical","profile":"{}","status":"pass"}}"#,
        candidate_profile_name
    ));

    let speed_positions = env_usize("SMART_GATE_SPEED_POSITIONS").unwrap_or(12).max(4);
    let speed_seed = seed_for_pairing("promotion_ladder", "speed");
    let speed_openings = generate_opening_fens_cached(speed_seed, speed_positions);
    let baseline_speed =
        profile_speed_by_mode_ms(baseline.select_inputs, speed_openings.as_slice(), budgets.as_slice());
    let candidate_speed =
        profile_speed_by_mode_ms(candidate.select_inputs, speed_openings.as_slice(), budgets.as_slice());
    let baseline_map = baseline_speed
        .iter()
        .map(|stat| (stat.budget.key(), stat.avg_ms))
        .collect::<std::collections::HashMap<_, _>>();
    let mut speed_ratios = std::collections::HashMap::new();
    for stat in &candidate_speed {
        let baseline_ms = baseline_map.get(stat.budget.key()).copied().unwrap_or(1.0);
        speed_ratios.insert(
            stat.budget.key(),
            if baseline_ms > 0.0 {
                stat.avg_ms / baseline_ms
            } else {
                1.0
            },
        );
    }
    let fast_ratio = speed_ratios.get("fast").copied().unwrap_or(1.0);
    let normal_ratio = speed_ratios.get("normal").copied().unwrap_or(1.0);
    artifacts.push(format!(
        r#"{{"stage":"A_speed","fast_ratio":{:.5},"normal_ratio":{:.5}}}"#,
        fast_ratio, normal_ratio
    ));
    assert!(fast_ratio <= 1.15, "fast cpu gate failed: ratio={:.3}", fast_ratio);
    assert!(
        normal_ratio <= 1.15,
        "normal cpu gate failed: ratio={:.3}",
        normal_ratio
    );

    let budget_duel_games = env_usize("SMART_GATE_BUDGET_DUEL_GAMES")
        .unwrap_or(3)
        .max(1);
    let budget_duel_repeats = env_usize("SMART_GATE_BUDGET_DUEL_REPEATS")
        .unwrap_or(4)
        .max(1);
    let budget_duel_max_plies = env_usize("SMART_GATE_BUDGET_DUEL_MAX_PLIES")
        .or_else(|| env_usize("SMART_POOL_MAX_PLIES"))
        .unwrap_or(56)
        .max(32);
    let budget_duel_seed_tag = env_profile_name("SMART_GATE_BUDGET_DUEL_SEED_TAG")
        .unwrap_or_else(|| "fast_normal_v1".to_string());
    let baseline_budget_conversion = run_budget_conversion_diagnostic(
        baseline_profile_name.as_str(),
        baseline.select_inputs,
        budget_duel_games,
        budget_duel_repeats,
        budget_duel_max_plies,
        budget_duel_seed_tag.as_str(),
    );
    let candidate_budget_conversion = run_budget_conversion_diagnostic(
        candidate_profile_name.as_str(),
        candidate.select_inputs,
        budget_duel_games,
        budget_duel_repeats,
        budget_duel_max_plies,
        budget_duel_seed_tag.as_str(),
    );
    let conversion_delta =
        candidate_budget_conversion.normal_edge - baseline_budget_conversion.normal_edge;
    artifacts.push(format!(
        r#"{{"stage":"A_budget_conversion","baseline_fast_wr":{:.5},"baseline_normal_edge":{:.5},"candidate_fast_wr":{:.5},"candidate_normal_edge":{:.5},"delta":{:.5}}}"#,
        baseline_budget_conversion.fast_win_rate,
        baseline_budget_conversion.normal_edge,
        candidate_budget_conversion.fast_win_rate,
        candidate_budget_conversion.normal_edge,
        conversion_delta
    ));
    if candidate_budget_conversion.normal_edge + SMART_BUDGET_CONVERSION_REGRESSION_TOLERANCE
        < baseline_budget_conversion.normal_edge
    {
        println!(
            "promotion gate budget conversion NOTE: candidate normal_edge {:.3} < baseline {:.3}",
            candidate_budget_conversion.normal_edge,
            baseline_budget_conversion.normal_edge
        );
    }

    let progressive_config = ProgressiveDuelConfig::from_env_with_defaults("ladder");
    let progressive_artifact = default_progressive_artifact_path(candidate_profile_name.as_str());
    let progressive_result = run_progressive_duel(
        candidate,
        baseline,
        budgets.as_slice(),
        &progressive_config,
        Some(progressive_artifact.as_str()),
    );
    match progressive_result.stop_reason {
        ProgressiveStopReason::EarlyReject => {
            artifacts.push(format!(
                r#"{{"stage":"B_progressive","status":"early_reject","total_games":{},"delta":{:.5},"confidence":{:.5}}}"#,
                progressive_result.total_games,
                progressive_result.final_delta,
                progressive_result.final_confidence
            ));
            persist_ladder_artifacts(artifacts.as_slice());
            panic!(
                "progressive duel early reject: delta={:.3} after {} games",
                progressive_result.final_delta,
                progressive_result.total_games
            );
        }
        ProgressiveStopReason::MathematicalReject => {
            artifacts.push(format!(
                r#"{{"stage":"B_progressive","status":"math_reject","total_games":{},"delta":{:.5},"confidence":{:.5}}}"#,
                progressive_result.total_games,
                progressive_result.final_delta,
                progressive_result.final_confidence
            ));
            persist_ladder_artifacts(artifacts.as_slice());
            panic!(
                "progressive duel mathematical reject: no mode can reach improvement threshold after {} games",
                progressive_result.total_games
            );
        }
        ProgressiveStopReason::EarlyPromote | ProgressiveStopReason::MaxGamesReached => {
            let mut any_mode_improved = false;
            for budget in &budgets {
                let stats = progressive_result
                    .final_mode_stats
                    .get(budget.key())
                    .copied()
                    .unwrap_or_default();
                let mode_delta = stats.win_rate_points() - 0.5;
                let non_regression_floor = progressive_config
                    .mode_non_regression_delta
                    .get(budget.key())
                    .copied()
                    .unwrap_or(-0.03);
                assert!(
                    mode_delta >= non_regression_floor,
                    "progressive mode {} non-regression failed: delta {:.3} < {:.3}",
                    budget.key(),
                    mode_delta,
                    non_regression_floor
                );
                let improvement_delta = progressive_config
                    .mode_improvement_delta
                    .get(budget.key())
                    .copied()
                    .unwrap_or(0.02);
                let improvement_confidence = progressive_config
                    .mode_improvement_confidence
                    .get(budget.key())
                    .copied()
                    .unwrap_or(0.60);
                let mode_confidence = stats.confidence_better_than_even();
                if mode_delta >= improvement_delta && mode_confidence >= improvement_confidence {
                    any_mode_improved = true;
                }
            }
            assert!(
                any_mode_improved,
                "progressive duel: no mode showed sufficient improvement after {} games",
                progressive_result.total_games
            );
            assert!(
                progressive_result.final_delta >= 0.0,
                "progressive aggregate non-regression failed: delta {:.3} < 0.0",
                progressive_result.final_delta
            );
            artifacts.push(format!(
                r#"{{"stage":"B_progressive","status":"ok","total_games":{},"tiers":{},"delta":{:.5},"confidence":{:.5}}}"#,
                progressive_result.total_games,
                progressive_result.tiers.len(),
                progressive_result.final_delta,
                progressive_result.final_confidence
            ));
        }
    }

    let confirm_games = env_usize("SMART_GATE_CONFIRM_GAMES").unwrap_or(4).max(2);
    let confirm_repeats = env_usize("SMART_GATE_CONFIRM_REPEATS").unwrap_or(6).max(2);
    let confirm_max_plies = env_usize("SMART_GATE_CONFIRM_MAX_PLIES")
        .unwrap_or(80)
        .max(56);
    let confirm_results = run_mirrored_duel_for_seed_tag(
        candidate,
        baseline,
        budgets.as_slice(),
        "prod_open_v1",
        confirm_repeats,
        confirm_games,
        confirm_max_plies,
        true,
    );
    let mut confirm_aggregate = MatchupStats::default();
    for (_, stats) in &confirm_results {
        confirm_aggregate.merge(*stats);
    }
    let confirm_delta = confirm_aggregate.win_rate_points() - 0.5;
    let confirm_confidence = confirm_aggregate.confidence_better_than_even();
    artifacts.push(format!(
        r#"{{"stage":"D_confirm","wins":{},"losses":{},"draws":{},"delta":{:.5},"confidence":{:.5}}}"#,
        confirm_aggregate.wins,
        confirm_aggregate.losses,
        confirm_aggregate.draws,
        confirm_delta,
        confirm_confidence
    ));

    let pool_games = env_usize("SMART_GATE_POOL_GAMES").unwrap_or(3).max(1);
    let (candidate_pool_eval, baseline_pool_eval, candidate_pool_wr, baseline_pool_wr) =
        run_pool_non_regression_check(candidate, baseline, budgets.as_slice(), pool_games);
    artifacts.push(format!(
        r#"{{"stage":"D_pool","candidate_beaten":{},"candidate_total":{},"baseline_beaten":{},"baseline_total":{},"candidate_wr":{:.5},"baseline_wr":{:.5}}}"#,
        candidate_pool_eval.beaten_opponents,
        candidate_pool_eval.opponents.len(),
        baseline_pool_eval.beaten_opponents,
        baseline_pool_eval.opponents.len(),
        candidate_pool_wr,
        baseline_pool_wr
    ));
    assert!(
        candidate_pool_eval.beaten_opponents >= baseline_pool_eval.beaten_opponents,
        "pool non-regression failed beaten opponents: candidate {} < baseline {}",
        candidate_pool_eval.beaten_opponents,
        baseline_pool_eval.beaten_opponents
    );
    assert!(
        candidate_pool_wr + 0.01 >= baseline_pool_wr,
        "pool non-regression failed aggregate win-rate: candidate {:.3} baseline {:.3}",
        candidate_pool_wr,
        baseline_pool_wr
    );

    persist_ladder_artifacts(artifacts.as_slice());
}

#[test]
#[ignore = "pro fast screen against runtime normal baseline"]
fn smart_automove_pool_pro_fast_screen_vs_normal() {
    let candidate_profile = pro_candidate_profile_name();
    let baseline_profile = pro_baseline_profile_name();
    let seed_tag = env_profile_name("SMART_PRO_FAST_SCREEN_SEED_TAG")
        .unwrap_or_else(|| "pro_fast_screen_vs_normal_v1".to_string());
    let repeats = env_usize("SMART_PRO_FAST_SCREEN_REPEATS").unwrap_or(2).max(1);
    let games = env_usize("SMART_PRO_FAST_SCREEN_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_PRO_FAST_SCREEN_MAX_PLIES")
        .unwrap_or(72)
        .max(56);

    let stats = run_cross_budget_duel(
        candidate_profile.as_str(),
        SmartAutomovePreference::Pro,
        baseline_profile.as_str(),
        SmartAutomovePreference::Normal,
        seed_tag.as_str(),
        repeats,
        games,
        max_plies,
        false,
    );
    let (delta, confidence) = stats_delta_confidence(stats);
    println!(
        "pro fast screen vs normal: profile={} baseline={} delta={:.4} confidence={:.3}",
        candidate_profile, baseline_profile, delta, confidence
    );
    assert!(
        delta >= SMART_PRO_FAST_SCREEN_DELTA_MIN,
        "pro fast screen vs normal failed: delta {:.4} < {:.4}",
        delta,
        SMART_PRO_FAST_SCREEN_DELTA_MIN
    );
}

#[test]
#[ignore = "pro fast screen against runtime fast baseline"]
fn smart_automove_pool_pro_fast_screen_vs_fast() {
    let candidate_profile = pro_candidate_profile_name();
    let baseline_profile = pro_baseline_profile_name();
    let seed_tag = env_profile_name("SMART_PRO_FAST_SCREEN_SEED_TAG")
        .unwrap_or_else(|| "pro_fast_screen_vs_fast_v1".to_string());
    let repeats = env_usize("SMART_PRO_FAST_SCREEN_REPEATS").unwrap_or(2).max(1);
    let games = env_usize("SMART_PRO_FAST_SCREEN_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_PRO_FAST_SCREEN_MAX_PLIES")
        .unwrap_or(72)
        .max(56);

    let stats = run_cross_budget_duel(
        candidate_profile.as_str(),
        SmartAutomovePreference::Pro,
        baseline_profile.as_str(),
        SmartAutomovePreference::Fast,
        seed_tag.as_str(),
        repeats,
        games,
        max_plies,
        false,
    );
    let (delta, confidence) = stats_delta_confidence(stats);
    println!(
        "pro fast screen vs fast: profile={} baseline={} delta={:.4} confidence={:.3}",
        candidate_profile, baseline_profile, delta, confidence
    );
    assert!(
        delta >= SMART_PRO_FAST_SCREEN_DELTA_MIN,
        "pro fast screen vs fast failed: delta {:.4} < {:.4}",
        delta,
        SMART_PRO_FAST_SCREEN_DELTA_MIN
    );
}

#[test]
#[ignore = "pro progressive duel against runtime normal baseline"]
fn smart_automove_pool_pro_progressive_vs_normal() {
    let candidate_profile = pro_candidate_profile_name();
    let baseline_profile = pro_baseline_profile_name();
    let (stats, _) = run_pro_progressive_matchup(
        candidate_profile.as_str(),
        baseline_profile.as_str(),
        SmartAutomovePreference::Normal,
        "pro_progressive_vs_normal",
    );
    let (delta, confidence) = stats_delta_confidence(stats);
    println!(
        "pro progressive vs normal: profile={} baseline={} delta={:.4} confidence={:.3}",
        candidate_profile, baseline_profile, delta, confidence
    );
    assert!(delta >= 0.0, "pro progressive vs normal failed: delta {:.4} < 0.0", delta);
}

#[test]
#[ignore = "pro progressive duel against runtime fast baseline"]
fn smart_automove_pool_pro_progressive_vs_fast() {
    let candidate_profile = pro_candidate_profile_name();
    let baseline_profile = pro_baseline_profile_name();
    let (stats, _) = run_pro_progressive_matchup(
        candidate_profile.as_str(),
        baseline_profile.as_str(),
        SmartAutomovePreference::Fast,
        "pro_progressive_vs_fast",
    );
    let (delta, confidence) = stats_delta_confidence(stats);
    println!(
        "pro progressive vs fast: profile={} baseline={} delta={:.4} confidence={:.3}",
        candidate_profile, baseline_profile, delta, confidence
    );
    assert!(delta >= 0.0, "pro progressive vs fast failed: delta {:.4} < 0.0", delta);
}

#[test]
#[ignore = "strict pro promotion ladder against fast and normal baselines"]
fn smart_automove_pool_pro_promotion_ladder() {
    let candidate_profile = pro_candidate_profile_name();
    let baseline_profile = pro_baseline_profile_name();
    let candidate_selector = profile_selector_from_name(candidate_profile.as_str())
        .unwrap_or_else(|| panic!("candidate selector '{}' should exist", candidate_profile));
    let baseline_selector = profile_selector_from_name(baseline_profile.as_str())
        .unwrap_or_else(|| panic!("baseline selector '{}' should exist", baseline_profile));

    assert_tactical_guardrails(candidate_selector, candidate_profile.as_str());
    assert_interview_policy_regressions(candidate_selector, candidate_profile.as_str());
    assert_tactical_guardrails(baseline_selector, baseline_profile.as_str());
    assert_interview_policy_regressions(baseline_selector, baseline_profile.as_str());

    let speed_positions = env_usize("SMART_PRO_GATE_SPEED_POSITIONS")
        .unwrap_or(12)
        .max(4);
    let speed_seed = seed_for_pairing("pro_promotion_ladder", "speed");
    let speed_openings = generate_opening_fens_cached(speed_seed, speed_positions);
    let pro_ms = profile_speed_by_mode_ms(candidate_selector, speed_openings.as_slice(), &[pro_budget()])
        .first()
        .map(|stat| stat.avg_ms)
        .unwrap_or(0.0);
    let normal_ms = profile_speed_by_mode_ms(
        baseline_selector,
        speed_openings.as_slice(),
        &[SearchBudget::from_preference(SmartAutomovePreference::Normal)],
    )
    .first()
    .map(|stat| stat.avg_ms)
    .unwrap_or(1.0)
    .max(0.001);
    let speed_ratio = pro_ms / normal_ms;
    assert!(
        speed_ratio >= SMART_PRO_CPU_RATIO_TARGET_MIN,
        "pro cpu gate below target: ratio {:.3} < {:.3}",
        speed_ratio,
        SMART_PRO_CPU_RATIO_TARGET_MIN
    );
    assert!(
        speed_ratio <= SMART_PRO_CPU_RATIO_TARGET_MAX,
        "pro cpu gate above hard cap: ratio {:.3} > {:.3}",
        speed_ratio,
        SMART_PRO_CPU_RATIO_TARGET_MAX
    );

    let primary_games = env_usize("SMART_PRO_GATE_PRIMARY_GAMES").unwrap_or(6).max(2);
    let primary_repeats = env_usize("SMART_PRO_GATE_PRIMARY_REPEATS").unwrap_or(6).max(2);
    let primary_max_plies = env_usize("SMART_PRO_GATE_PRIMARY_MAX_PLIES")
        .unwrap_or(96)
        .max(56);
    let primary_seed_tags = ["neutral_v1", "neutral_v2", "neutral_v3"];

    let vs_normal_stats = run_pro_matchup_across_seeds(
        candidate_profile.as_str(),
        baseline_profile.as_str(),
        SmartAutomovePreference::Normal,
        "pro_primary_vs_normal",
        &primary_seed_tags,
        primary_repeats,
        primary_games,
        primary_max_plies,
        false,
    );
    let vs_fast_stats = run_pro_matchup_across_seeds(
        candidate_profile.as_str(),
        baseline_profile.as_str(),
        SmartAutomovePreference::Fast,
        "pro_primary_vs_fast",
        &primary_seed_tags,
        primary_repeats,
        primary_games,
        primary_max_plies,
        false,
    );
    let (vs_normal_delta, vs_normal_confidence) = stats_delta_confidence(vs_normal_stats);
    let (vs_fast_delta, vs_fast_confidence) = stats_delta_confidence(vs_fast_stats);
    assert!(
        vs_normal_delta >= SMART_PRO_PRIMARY_IMPROVEMENT_DELTA_MIN_VS_NORMAL,
        "pro primary vs normal failed: delta {:.4} < {:.4}",
        vs_normal_delta,
        SMART_PRO_PRIMARY_IMPROVEMENT_DELTA_MIN_VS_NORMAL
    );
    assert!(
        vs_normal_confidence >= SMART_PRO_PRIMARY_IMPROVEMENT_CONFIDENCE_MIN,
        "pro primary vs normal confidence failed: {:.3} < {:.3}",
        vs_normal_confidence,
        SMART_PRO_PRIMARY_IMPROVEMENT_CONFIDENCE_MIN
    );
    assert!(
        vs_fast_delta >= SMART_PRO_PRIMARY_IMPROVEMENT_DELTA_MIN_VS_FAST,
        "pro primary vs fast failed: delta {:.4} < {:.4}",
        vs_fast_delta,
        SMART_PRO_PRIMARY_IMPROVEMENT_DELTA_MIN_VS_FAST
    );
    assert!(
        vs_fast_confidence >= SMART_PRO_PRIMARY_IMPROVEMENT_CONFIDENCE_MIN,
        "pro primary vs fast confidence failed: {:.3} < {:.3}",
        vs_fast_confidence,
        SMART_PRO_PRIMARY_IMPROVEMENT_CONFIDENCE_MIN
    );

    let confirm_games = env_usize("SMART_PRO_GATE_CONFIRM_GAMES").unwrap_or(4).max(2);
    let confirm_repeats = env_usize("SMART_PRO_GATE_CONFIRM_REPEATS").unwrap_or(4).max(2);
    let confirm_max_plies = env_usize("SMART_PRO_GATE_CONFIRM_MAX_PLIES")
        .unwrap_or(96)
        .max(56);
    let confirm_vs_normal = run_cross_budget_duel(
        candidate_profile.as_str(),
        SmartAutomovePreference::Pro,
        baseline_profile.as_str(),
        SmartAutomovePreference::Normal,
        "pro_confirm_vs_normal_v1",
        confirm_repeats,
        confirm_games,
        confirm_max_plies,
        true,
    );
    let confirm_vs_fast = run_cross_budget_duel(
        candidate_profile.as_str(),
        SmartAutomovePreference::Pro,
        baseline_profile.as_str(),
        SmartAutomovePreference::Fast,
        "pro_confirm_vs_fast_v1",
        confirm_repeats,
        confirm_games,
        confirm_max_plies,
        true,
    );
    assert!(
        stats_delta_confidence(confirm_vs_normal).0 >= 0.0,
        "pro confirmation vs normal failed non-regression"
    );
    assert!(
        stats_delta_confidence(confirm_vs_fast).0 >= 0.0,
        "pro confirmation vs fast failed non-regression"
    );

    let pool_games = env_usize("SMART_PRO_GATE_POOL_GAMES").unwrap_or(1).max(1);
    let pool_max_plies = env_usize("SMART_PRO_GATE_POOL_MAX_PLIES")
        .unwrap_or(80)
        .max(56);
    let candidate_pool_vs_normal = run_profile_vs_pool_cross_budget(
        candidate_profile.as_str(),
        SmartAutomovePreference::Pro,
        SmartAutomovePreference::Normal,
        pool_games,
        pool_max_plies,
        "pro_pool_vs_normal",
    );
    let baseline_pool_vs_normal = run_profile_vs_pool_cross_budget(
        baseline_profile.as_str(),
        SmartAutomovePreference::Normal,
        SmartAutomovePreference::Normal,
        pool_games,
        pool_max_plies,
        "baseline_pool_vs_normal",
    );
    let candidate_pool_vs_fast = run_profile_vs_pool_cross_budget(
        candidate_profile.as_str(),
        SmartAutomovePreference::Pro,
        SmartAutomovePreference::Fast,
        pool_games,
        pool_max_plies,
        "pro_pool_vs_fast",
    );
    let baseline_pool_vs_fast = run_profile_vs_pool_cross_budget(
        baseline_profile.as_str(),
        SmartAutomovePreference::Fast,
        SmartAutomovePreference::Fast,
        pool_games,
        pool_max_plies,
        "baseline_pool_vs_fast",
    );
    assert!(
        stats_delta_confidence(candidate_pool_vs_normal).0 + 0.01
            >= stats_delta_confidence(baseline_pool_vs_normal).0,
        "pro pool non-regression vs normal-opponents failed"
    );
    assert!(
        stats_delta_confidence(candidate_pool_vs_fast).0 + 0.01
            >= stats_delta_confidence(baseline_pool_vs_fast).0,
        "pro pool non-regression vs fast-opponents failed"
    );
}

#[test]
#[ignore = "ultra fast screen against runtime pro baseline"]
fn smart_automove_pool_ultra_fast_screen_vs_pro() {
    let candidate_profile = ultra_candidate_profile_name();
    let baseline_profile = ultra_baseline_profile_name();
    let seed_tag = env_profile_name("SMART_ULTRA_FAST_SCREEN_SEED_TAG")
        .unwrap_or_else(|| "ultra_fast_screen_vs_pro_v1".to_string());
    let repeats = env_usize("SMART_ULTRA_FAST_SCREEN_REPEATS").unwrap_or(2).max(1);
    let games = env_usize("SMART_ULTRA_FAST_SCREEN_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_ULTRA_FAST_SCREEN_MAX_PLIES")
        .unwrap_or(72)
        .max(56);
    let stats = run_cross_budget_duel(
        candidate_profile.as_str(),
        SmartAutomovePreference::Ultra,
        baseline_profile.as_str(),
        SmartAutomovePreference::Pro,
        seed_tag.as_str(),
        repeats,
        games,
        max_plies,
        false,
    );
    assert!(
        stats_delta_confidence(stats).0 >= SMART_ULTRA_FAST_SCREEN_DELTA_MIN,
        "ultra fast screen vs pro failed"
    );
}

#[test]
#[ignore = "ultra fast screen against runtime normal baseline"]
fn smart_automove_pool_ultra_fast_screen_vs_normal() {
    let candidate_profile = ultra_candidate_profile_name();
    let baseline_profile = ultra_baseline_profile_name();
    let seed_tag = env_profile_name("SMART_ULTRA_FAST_SCREEN_SEED_TAG")
        .unwrap_or_else(|| "ultra_fast_screen_vs_normal_v1".to_string());
    let repeats = env_usize("SMART_ULTRA_FAST_SCREEN_REPEATS").unwrap_or(2).max(1);
    let games = env_usize("SMART_ULTRA_FAST_SCREEN_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_ULTRA_FAST_SCREEN_MAX_PLIES")
        .unwrap_or(72)
        .max(56);
    let stats = run_cross_budget_duel(
        candidate_profile.as_str(),
        SmartAutomovePreference::Ultra,
        baseline_profile.as_str(),
        SmartAutomovePreference::Normal,
        seed_tag.as_str(),
        repeats,
        games,
        max_plies,
        false,
    );
    assert!(
        stats_delta_confidence(stats).0 >= SMART_ULTRA_FAST_SCREEN_DELTA_MIN,
        "ultra fast screen vs normal failed"
    );
}

#[test]
#[ignore = "ultra fast screen against runtime fast baseline"]
fn smart_automove_pool_ultra_fast_screen_vs_fast() {
    let candidate_profile = ultra_candidate_profile_name();
    let baseline_profile = ultra_baseline_profile_name();
    let seed_tag = env_profile_name("SMART_ULTRA_FAST_SCREEN_SEED_TAG")
        .unwrap_or_else(|| "ultra_fast_screen_vs_fast_v1".to_string());
    let repeats = env_usize("SMART_ULTRA_FAST_SCREEN_REPEATS").unwrap_or(2).max(1);
    let games = env_usize("SMART_ULTRA_FAST_SCREEN_GAMES").unwrap_or(2).max(1);
    let max_plies = env_usize("SMART_ULTRA_FAST_SCREEN_MAX_PLIES")
        .unwrap_or(72)
        .max(56);
    let stats = run_cross_budget_duel(
        candidate_profile.as_str(),
        SmartAutomovePreference::Ultra,
        baseline_profile.as_str(),
        SmartAutomovePreference::Fast,
        seed_tag.as_str(),
        repeats,
        games,
        max_plies,
        false,
    );
    assert!(
        stats_delta_confidence(stats).0 >= SMART_ULTRA_FAST_SCREEN_DELTA_MIN,
        "ultra fast screen vs fast failed"
    );
}

#[test]
#[ignore = "ultra progressive duel against runtime pro baseline"]
fn smart_automove_pool_ultra_progressive_vs_pro() {
    let candidate_profile = ultra_candidate_profile_name();
    let baseline_profile = ultra_baseline_profile_name();
    let (stats, _) = run_ultra_progressive_matchup(
        candidate_profile.as_str(),
        baseline_profile.as_str(),
        SmartAutomovePreference::Pro,
        "ultra_progressive_vs_pro",
    );
    assert!(
        stats_delta_confidence(stats).0 >= 0.0,
        "ultra progressive vs pro failed non-regression"
    );
}

#[test]
#[ignore = "ultra progressive duel against runtime normal baseline"]
fn smart_automove_pool_ultra_progressive_vs_normal() {
    let candidate_profile = ultra_candidate_profile_name();
    let baseline_profile = ultra_baseline_profile_name();
    let (stats, _) = run_ultra_progressive_matchup(
        candidate_profile.as_str(),
        baseline_profile.as_str(),
        SmartAutomovePreference::Normal,
        "ultra_progressive_vs_normal",
    );
    assert!(
        stats_delta_confidence(stats).0 >= 0.0,
        "ultra progressive vs normal failed non-regression"
    );
}

#[test]
#[ignore = "ultra progressive duel against runtime fast baseline"]
fn smart_automove_pool_ultra_progressive_vs_fast() {
    let candidate_profile = ultra_candidate_profile_name();
    let baseline_profile = ultra_baseline_profile_name();
    let (stats, _) = run_ultra_progressive_matchup(
        candidate_profile.as_str(),
        baseline_profile.as_str(),
        SmartAutomovePreference::Fast,
        "ultra_progressive_vs_fast",
    );
    assert!(
        stats_delta_confidence(stats).0 >= 0.0,
        "ultra progressive vs fast failed non-regression"
    );
}

#[test]
#[ignore = "strict ultra promotion ladder against pro, fast, and normal baselines"]
fn smart_automove_pool_ultra_promotion_ladder() {
    let candidate_profile = ultra_candidate_profile_name();
    let baseline_profile = ultra_baseline_profile_name();
    let candidate_selector = profile_selector_from_name(candidate_profile.as_str())
        .unwrap_or_else(|| panic!("candidate selector '{}' should exist", candidate_profile));
    let baseline_selector = profile_selector_from_name(baseline_profile.as_str())
        .unwrap_or_else(|| panic!("baseline selector '{}' should exist", baseline_profile));

    assert_tactical_guardrails(candidate_selector, candidate_profile.as_str());
    assert_interview_policy_regressions(candidate_selector, candidate_profile.as_str());
    assert_tactical_guardrails(baseline_selector, baseline_profile.as_str());
    assert_interview_policy_regressions(baseline_selector, baseline_profile.as_str());

    let speed_positions = env_usize("SMART_ULTRA_GATE_SPEED_POSITIONS")
        .unwrap_or(12)
        .max(4);
    let speed_seed = seed_for_pairing("ultra_promotion_ladder", "speed");
    let speed_openings = generate_opening_fens_cached(speed_seed, speed_positions);
    let ultra_ms =
        profile_speed_by_mode_ms(candidate_selector, speed_openings.as_slice(), &[ultra_budget()])
            .first()
            .map(|stat| stat.avg_ms)
            .unwrap_or(0.0);
    let pro_ms = profile_speed_by_mode_ms(baseline_selector, speed_openings.as_slice(), &[pro_budget()])
        .first()
        .map(|stat| stat.avg_ms)
        .unwrap_or(1.0)
        .max(0.001);
    let speed_ratio = ultra_ms / pro_ms;
    assert!(
        speed_ratio >= SMART_ULTRA_CPU_RATIO_TARGET_MIN_VS_PRO,
        "ultra cpu gate below target: ratio {:.3} < {:.3}",
        speed_ratio,
        SMART_ULTRA_CPU_RATIO_TARGET_MIN_VS_PRO
    );
    assert!(
        speed_ratio <= SMART_ULTRA_CPU_RATIO_TARGET_MAX_VS_PRO,
        "ultra cpu gate above hard cap: ratio {:.3} > {:.3}",
        speed_ratio,
        SMART_ULTRA_CPU_RATIO_TARGET_MAX_VS_PRO
    );

    let primary_games = env_usize("SMART_ULTRA_GATE_PRIMARY_GAMES")
        .unwrap_or(6)
        .max(2);
    let primary_repeats = env_usize("SMART_ULTRA_GATE_PRIMARY_REPEATS")
        .unwrap_or(6)
        .max(2);
    let primary_max_plies = env_usize("SMART_ULTRA_GATE_PRIMARY_MAX_PLIES")
        .unwrap_or(96)
        .max(56);
    let primary_seed_tags = ["neutral_v1", "neutral_v2", "neutral_v3"];
    let vs_pro_stats = run_ultra_matchup_across_seeds(
        candidate_profile.as_str(),
        baseline_profile.as_str(),
        SmartAutomovePreference::Pro,
        "ultra_primary_vs_pro",
        &primary_seed_tags,
        primary_repeats,
        primary_games,
        primary_max_plies,
        false,
    );
    let vs_normal_stats = run_ultra_matchup_across_seeds(
        candidate_profile.as_str(),
        baseline_profile.as_str(),
        SmartAutomovePreference::Normal,
        "ultra_primary_vs_normal",
        &primary_seed_tags,
        primary_repeats,
        primary_games,
        primary_max_plies,
        false,
    );
    let vs_fast_stats = run_ultra_matchup_across_seeds(
        candidate_profile.as_str(),
        baseline_profile.as_str(),
        SmartAutomovePreference::Fast,
        "ultra_primary_vs_fast",
        &primary_seed_tags,
        primary_repeats,
        primary_games,
        primary_max_plies,
        false,
    );
    let (vs_pro_delta, vs_pro_confidence) = stats_delta_confidence(vs_pro_stats);
    assert!(
        vs_pro_delta >= SMART_ULTRA_PRIMARY_IMPROVEMENT_DELTA_MIN_VS_PRO,
        "ultra primary vs pro failed: delta {:.4} < {:.4}",
        vs_pro_delta,
        SMART_ULTRA_PRIMARY_IMPROVEMENT_DELTA_MIN_VS_PRO
    );
    assert!(
        vs_pro_confidence >= SMART_ULTRA_PRIMARY_IMPROVEMENT_CONFIDENCE_MIN,
        "ultra primary vs pro confidence failed: {:.3} < {:.3}",
        vs_pro_confidence,
        SMART_ULTRA_PRIMARY_IMPROVEMENT_CONFIDENCE_MIN
    );
    assert!(
        stats_delta_confidence(vs_normal_stats).0 >= 0.0,
        "ultra primary vs normal failed non-regression"
    );
    assert!(
        stats_delta_confidence(vs_fast_stats).0 >= 0.0,
        "ultra primary vs fast failed non-regression"
    );

    let confirm_games = env_usize("SMART_ULTRA_GATE_CONFIRM_GAMES").unwrap_or(4).max(2);
    let confirm_repeats = env_usize("SMART_ULTRA_GATE_CONFIRM_REPEATS").unwrap_or(4).max(2);
    let confirm_max_plies = env_usize("SMART_ULTRA_GATE_CONFIRM_MAX_PLIES")
        .unwrap_or(96)
        .max(56);
    assert!(
        stats_delta_confidence(run_cross_budget_duel(
            candidate_profile.as_str(),
            SmartAutomovePreference::Ultra,
            baseline_profile.as_str(),
            SmartAutomovePreference::Pro,
            "ultra_confirm_vs_pro_v1",
            confirm_repeats,
            confirm_games,
            confirm_max_plies,
            true,
        ))
        .0 >= 0.0,
        "ultra confirmation vs pro failed non-regression"
    );
    assert!(
        stats_delta_confidence(run_cross_budget_duel(
            candidate_profile.as_str(),
            SmartAutomovePreference::Ultra,
            baseline_profile.as_str(),
            SmartAutomovePreference::Normal,
            "ultra_confirm_vs_normal_v1",
            confirm_repeats,
            confirm_games,
            confirm_max_plies,
            true,
        ))
        .0 >= 0.0,
        "ultra confirmation vs normal failed non-regression"
    );
    assert!(
        stats_delta_confidence(run_cross_budget_duel(
            candidate_profile.as_str(),
            SmartAutomovePreference::Ultra,
            baseline_profile.as_str(),
            SmartAutomovePreference::Fast,
            "ultra_confirm_vs_fast_v1",
            confirm_repeats,
            confirm_games,
            confirm_max_plies,
            true,
        ))
        .0 >= 0.0,
        "ultra confirmation vs fast failed non-regression"
    );

    let pool_games = env_usize("SMART_ULTRA_GATE_POOL_GAMES").unwrap_or(1).max(1);
    let pool_max_plies = env_usize("SMART_ULTRA_GATE_POOL_MAX_PLIES")
        .unwrap_or(80)
        .max(56);
    assert!(
        stats_delta_confidence(run_profile_vs_pool_cross_budget(
            candidate_profile.as_str(),
            SmartAutomovePreference::Ultra,
            SmartAutomovePreference::Pro,
            pool_games,
            pool_max_plies,
            "ultra_pool_vs_pro",
        ))
        .0 + 0.01
            >= stats_delta_confidence(run_profile_vs_pool_cross_budget(
                baseline_profile.as_str(),
                SmartAutomovePreference::Pro,
                SmartAutomovePreference::Pro,
                pool_games,
                pool_max_plies,
                "baseline_pool_vs_pro",
            ))
            .0,
        "ultra pool non-regression vs pro-opponents failed"
    );
    assert!(
        stats_delta_confidence(run_profile_vs_pool_cross_budget(
            candidate_profile.as_str(),
            SmartAutomovePreference::Ultra,
            SmartAutomovePreference::Normal,
            pool_games,
            pool_max_plies,
            "ultra_pool_vs_normal",
        ))
        .0 + 0.01
            >= stats_delta_confidence(run_profile_vs_pool_cross_budget(
                baseline_profile.as_str(),
                SmartAutomovePreference::Normal,
                SmartAutomovePreference::Normal,
                pool_games,
                pool_max_plies,
                "baseline_pool_vs_normal",
            ))
            .0,
        "ultra pool non-regression vs normal-opponents failed"
    );
    assert!(
        stats_delta_confidence(run_profile_vs_pool_cross_budget(
            candidate_profile.as_str(),
            SmartAutomovePreference::Ultra,
            SmartAutomovePreference::Fast,
            pool_games,
            pool_max_plies,
            "ultra_pool_vs_fast",
        ))
        .0 + 0.01
            >= stats_delta_confidence(run_profile_vs_pool_cross_budget(
                baseline_profile.as_str(),
                SmartAutomovePreference::Fast,
                SmartAutomovePreference::Fast,
                pool_games,
                pool_max_plies,
                "baseline_pool_vs_fast",
            ))
            .0,
        "ultra pool non-regression vs fast-opponents failed"
    );
}
