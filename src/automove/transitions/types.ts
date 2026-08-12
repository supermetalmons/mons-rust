import type { Event, Input } from "../../engine/model/domain.js";
import type { MonsGame } from "../../engine/game/mons-game.js";

export type LegalInputTransition = {
  readonly inputs: readonly Input[];
  readonly game: MonsGame;
  readonly events: readonly Event[];
};
