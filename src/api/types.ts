export const Color = Object.freeze({
  White: "white",
  Black: "black",
} as const);
export type Color = (typeof Color)[keyof typeof Color];

export const GameVariant = Object.freeze({
  Classic: "Classic",
  SwappedManaRows: "SwappedManaRows",
  OffsetArcManaRows: "OffsetArcManaRows",
  CenterSpokeManaRows: "CenterSpokeManaRows",
  AlternatingManaRows: "AlternatingManaRows",
  InnerWedgeManaRows: "InnerWedgeManaRows",
  OuterWedgeManaRows: "OuterWedgeManaRows",
  BentCenterManaRows: "BentCenterManaRows",
  OuterEdgeManaRows: "OuterEdgeManaRows",
  SplitFlankManaRows: "SplitFlankManaRows",
  ForwardBridgeManaRows: "ForwardBridgeManaRows",
  CornerChainManaRows: "CornerChainManaRows",
} as const);
export type GameVariant = (typeof GameVariant)[keyof typeof GameVariant];

export const MonKind = Object.freeze({
  Demon: "demon",
  Drainer: "drainer",
  Angel: "angel",
  Spirit: "spirit",
  Mystic: "mystic",
} as const);
export type MonKind = (typeof MonKind)[keyof typeof MonKind];

export const Consumable = Object.freeze({
  Potion: "potion",
  Bomb: "bomb",
  BombOrPotion: "bomb-or-potion",
} as const);
export type Consumable = (typeof Consumable)[keyof typeof Consumable];

export const Modifier = Object.freeze({
  SelectPotion: "select-potion",
  SelectBomb: "select-bomb",
} as const);
export type Modifier = (typeof Modifier)[keyof typeof Modifier];

export type Position = {
  readonly row: number;
  readonly column: number;
};

export type Mon = {
  readonly kind: MonKind;
  readonly color: Color;
  readonly cooldown: number;
};

export type Mana =
  { readonly kind: "regular"; readonly color: Color } | { readonly kind: "supermana" };

type Carryable =
  | { readonly kind: "mana"; readonly mana: Mana }
  | { readonly kind: "consumable"; readonly consumable: Consumable };

export type BoardItem =
  | {
      readonly kind: "mon";
      readonly mon: Mon;
      readonly carrying?: Carryable;
    }
  | Carryable;

export type Square =
  | { readonly kind: "regular" }
  | { readonly kind: "consumable-base" }
  | { readonly kind: "supermana-base" }
  | { readonly kind: "mana-base"; readonly color: Color }
  | { readonly kind: "mana-pool"; readonly color: Color }
  | {
      readonly kind: "mon-base";
      readonly monKind: MonKind;
      readonly color: Color;
    };

export type Input =
  | { readonly kind: "takeback" }
  | { readonly kind: "position"; readonly position: Position }
  | { readonly kind: "modifier"; readonly modifier: Modifier };

export type InputAction =
  | "mon-move"
  | "mana-move"
  | "mystic-action"
  | "demon-action"
  | "demon-additional-step"
  | "spirit-target-capture"
  | "spirit-target-move"
  | "select-consumable"
  | "bomb-attack";

type PositionInput = Extract<Input, { readonly kind: "position" }>;
type ModifierInput = Extract<Input, { readonly kind: "modifier" }>;

type PositionInputOption<Action extends Exclude<InputAction, "select-consumable">> = {
  readonly action: Action;
  readonly input: PositionInput;
  readonly actor?: BoardItem;
};

export type InputOption =
  | PositionInputOption<"mon-move">
  | PositionInputOption<"mana-move">
  | PositionInputOption<"mystic-action">
  | PositionInputOption<"demon-action">
  | PositionInputOption<"demon-additional-step">
  | PositionInputOption<"spirit-target-capture">
  | PositionInputOption<"spirit-target-move">
  | PositionInputOption<"bomb-attack">
  | {
      readonly action: "select-consumable";
      readonly input: ModifierInput;
      readonly actor?: BoardItem;
    };

export type GameEvent =
  | {
      readonly kind: "mon-move";
      readonly item: BoardItem;
      readonly from: Position;
      readonly to: Position;
    }
  | {
      readonly kind: "mana-move";
      readonly mana: Mana;
      readonly from: Position;
      readonly to: Position;
    }
  | {
      readonly kind: "mana-scored";
      readonly mana: Mana;
      readonly at: Position;
    }
  | {
      readonly kind: "mystic-action";
      readonly mystic: Mon;
      readonly from: Position;
      readonly to: Position;
    }
  | {
      readonly kind: "demon-action";
      readonly demon: Mon;
      readonly from: Position;
      readonly to: Position;
    }
  | {
      readonly kind: "demon-additional-step";
      readonly demon: Mon;
      readonly from: Position;
      readonly to: Position;
    }
  | {
      readonly kind: "spirit-target-move";
      readonly item: BoardItem;
      readonly from: Position;
      readonly to: Position;
      readonly by: Position;
    }
  | {
      readonly kind: "pickup-bomb";
      readonly by: Mon;
      readonly at: Position;
    }
  | {
      readonly kind: "pickup-potion";
      readonly by: BoardItem;
      readonly at: Position;
    }
  | {
      readonly kind: "use-potion";
      readonly from: Position;
      readonly to: Position;
    }
  | {
      readonly kind: "pickup-mana";
      readonly mana: Mana;
      readonly by: Mon;
      readonly at: Position;
    }
  | {
      readonly kind: "mon-fainted";
      readonly mon: Mon;
      readonly from: Position;
      readonly to: Position;
    }
  | {
      readonly kind: "mana-dropped";
      readonly mana: Mana;
      readonly at: Position;
    }
  | {
      readonly kind: "supermana-back-to-base";
      readonly from: Position;
      readonly to: Position;
    }
  | {
      readonly kind: "bomb-attack";
      readonly by: Mon;
      readonly from: Position;
      readonly to: Position;
    }
  | { readonly kind: "mon-awake"; readonly mon: Mon; readonly at: Position }
  | { readonly kind: "bomb-explosion"; readonly at: Position }
  | { readonly kind: "next-turn"; readonly color: Color }
  | { readonly kind: "game-over"; readonly winner: Color }
  | { readonly kind: "takeback" };

type InvalidInputResolution = {
  readonly kind: "invalid";
  readonly inputFen: string;
};

type CompleteInputResolution = {
  readonly kind: "complete";
  readonly inputFen: string;
  readonly events: readonly GameEvent[];
};

export type InputResolution =
  | InvalidInputResolution
  | {
      readonly kind: "awaiting-start";
      readonly inputFen: string;
      readonly positions: readonly Position[];
    }
  | {
      readonly kind: "awaiting-input";
      readonly inputFen: string;
      readonly options: readonly InputOption[];
    }
  | CompleteInputResolution;

export type PlayResult = InvalidInputResolution | CompleteInputResolution;

export type AvailableMoveCounts = {
  readonly monMoves: number;
  readonly manaMoves: number;
  readonly actions: number;
  readonly potions: number;
};

export type TrackingEntry = {
  readonly fen: string;
  readonly color: Color;
  readonly events: readonly GameEvent[];
  readonly eventsFen: string;
};

export type MoveSuggestion = {
  readonly inputs: readonly Input[];
  readonly inputFen: string;
  readonly events: readonly GameEvent[];
};

type PlayerSubmission = {
  readonly fen: string;
  readonly moves: readonly string[];
};

export type MatchSubmission = {
  readonly white: PlayerSubmission;
  readonly black: PlayerSubmission;
};

export type MatchResolution =
  | { readonly kind: "ongoing" }
  | { readonly kind: "winner"; readonly winner: Color }
  | { readonly kind: "invalid" };
