import { normalizeScoringProfile } from "../scoring/profile-validation.js";
import { validateAutomoveConfig } from "./validation.js";
import type {
  AutomoveBudgetConfig,
  AutomoveConfig,
  AutomoveEvaluationConfig,
  AutomovePlannerConfig,
  AutomovePolicyConfig,
  AutomoveReplyRiskConfig,
  AutomoveTreeSearchConfig,
} from "./types.js";

type AutomoveConfigDefinition = AutomoveConfig;

type SectionPatch<Section> = {
  readonly [Key in keyof Section]?: Section[Key];
};

export type AutomoveConfigPatch = {
  readonly budget?: SectionPatch<AutomoveBudgetConfig>;
  readonly search?: SectionPatch<AutomoveTreeSearchConfig>;
  readonly planner?: SectionPatch<AutomovePlannerConfig>;
  readonly evaluation?: SectionPatch<AutomoveEvaluationConfig>;
  readonly replyRisk?: SectionPatch<AutomoveReplyRiskConfig>;
  readonly policy?: SectionPatch<AutomovePolicyConfig>;
};

function freezeSection<Section extends object>(section: Section): Readonly<Section> {
  return Object.freeze(section);
}

export function defineAutomoveConfig(
  definition: AutomoveConfigDefinition,
): AutomoveConfig {
  const weights = normalizeScoringProfile(definition.evaluation.weights);
  const config: AutomoveConfig = Object.freeze({
    budget: freezeSection({ ...definition.budget }),
    search: freezeSection({ ...definition.search }),
    planner: freezeSection({ ...definition.planner }),
    evaluation: freezeSection({
      ...definition.evaluation,
      weights,
    }),
    replyRisk: freezeSection({ ...definition.replyRisk }),
    policy: freezeSection({ ...definition.policy }),
  });
  validateAutomoveConfig(config);
  return config;
}

export function patchAutomoveConfig(
  config: AutomoveConfig,
  patch: AutomoveConfigPatch,
): AutomoveConfig {
  const evaluation = (() => {
    if (patch.evaluation === undefined) return config.evaluation;
    const weights = normalizeScoringProfile(
      patch.evaluation.weights ?? config.evaluation.weights,
    );
    return freezeSection({
      ...config.evaluation,
      ...patch.evaluation,
      weights,
    });
  })();
  const next: AutomoveConfig = Object.freeze({
    budget:
      patch.budget === undefined
        ? config.budget
        : freezeSection({ ...config.budget, ...patch.budget }),
    search:
      patch.search === undefined
        ? config.search
        : freezeSection({ ...config.search, ...patch.search }),
    planner:
      patch.planner === undefined
        ? config.planner
        : freezeSection({ ...config.planner, ...patch.planner }),
    evaluation,
    replyRisk:
      patch.replyRisk === undefined
        ? config.replyRisk
        : freezeSection({ ...config.replyRisk, ...patch.replyRisk }),
    policy:
      patch.policy === undefined
        ? config.policy
        : freezeSection({ ...config.policy, ...patch.policy }),
  });
  validateAutomoveConfig(next);
  return next;
}
