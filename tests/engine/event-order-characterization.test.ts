import { describe, expect, it } from "vitest";

import { parseInputArrayFen } from "../../src/engine/fen.js";
import { MonsGame } from "../../src/engine/game.js";

type EventOrderCase = {
  readonly name: string;
  readonly fen: string;
  readonly inputFen: string;
  readonly expectedKinds: readonly string[];
};

const CASES: readonly EventOrderCase[] = [
  {
    name: "spirit movement appends the destination pickup",
    fen: "0 0 w 0 0 5 1 0 3 n03y0xs0xn01a0xe0xn03/n11/n11/n02xxmn01xxmn01xxmn04/n05xxmn01xxmn03/xxQn04d0Un04Y0x/n03xxMn01xxMn01xxMn03/n04xxMD0xxxMn04/n07S0xn03/n11/n03E0xA0xn06",
    inputFen: "l8,7;l6,5;l7,5",
    expectedKinds: ["spirit-target-move", "pickup-mana"],
  },
  {
    name: "demon attacks resolve drops before the additional step",
    fen: "2 3 w 0 0 4 0 0 13 n06a1xe0xn03/n01d0mn01xxmn04Y0xn02/n11/n01E0xn09/n03xxmn02s0xxxmn03/n11/y0xn08xxMn01/n06A0xS0xn03/n02xxMn08/n02xxMn08/n05D0xn05",
    inputFen: "l3,1;l1,1;l0,0",
    expectedKinds: [
      "demon-action",
      "mon-fainted",
      "mana-dropped",
      "demon-additional-step",
    ],
  },
  {
    name: "mystic attacks explode bombs before potion use",
    fen: "4 1 w 1 0 1 1 0 13 n11/n08xxmn02/d0Mn10/n04xxma0xn02Y0xn02/n03xxms0xxxmn05/n10e0B/n02y0xn02xxMn05/n04xxMn03S0xn02/n11/n01D0xn09/n03E0xA0xn06",
    inputFen: "l3,8;l5,10",
    expectedKinds: [
      "mystic-action",
      "mon-fainted",
      "bomb-explosion",
      "use-potion",
    ],
  },
  {
    name: "bomb attacks faint and drop carried mana in order",
    fen: "0 3 w 0 0 0 0 0 9 n07e0xn03/n02y0xn01s0xn03xxmn02/n03xxmn02a0xn04/n07d0mn03/n05xxmn05/xxQn03E0xn05Y0B/n05xxMS0xxxMn03/n06xxMn04/n03xxMn03D0xn03/xxMn10/n04A0xn06",
    inputFen: "l5,10;l3,7",
    expectedKinds: ["bomb-attack", "mon-fainted", "mana-dropped"],
  },
  {
    name: "turn advancement precedes waking mons",
    fen: "2 3 b 1 0 5 0 0 12 n06a1xe0xn03/n01d0mn02xxmn03Y0xn02/n11/n11/n03xxmn02s0xxxmn03/n11/y0xn08xxMn01/n02E0xn03A0xS0xn03/n02xxMn08/n02xxMn08/n05D1xn05",
    inputFen: "l1,4;l1,3",
    expectedKinds: ["mana-move", "next-turn", "mon-awake"],
  },
  {
    name: "the scoring move precedes game over",
    fen: "4 4 b 0 0 2 0 0 12 n01d0mn01y0xn03e0xn03/n04a0xn06/n05s0xn01xxmn03/n11/n07xxmn03/xxQn09Y0x/n11/n11/n11/n03S0xn02xxMn04/n03E0xA0xn05D0x",
    inputFen: "l0,1;l0,0",
    expectedKinds: ["mon-move", "mana-scored", "game-over"],
  },
  {
    name: "supermana returns before the action potion is consumed",
    fen: "0 0 b 1 0 5 1 1 6 n11/n04d0xa0xn05/n04s0xn02xxmn03/n02xxmn01xxmn01xxmn04/n07xxmn03/E0xy0xn08e0x/n03xxMn01xxMn05/n03D0Un02xxMn01xxMn02/n03xxMn07/n06S0xn01Y0xn02/n04A2xn06",
    inputFen: "l5,1;l7,3",
    expectedKinds: [
      "mystic-action",
      "mon-fainted",
      "supermana-back-to-base",
      "use-potion",
    ],
  },
  {
    name: "consumable selection follows the movement event",
    fen: "0 0 w 0 0 4 0 0 3 n03y0xs0xn01a0xe0xn03/n11/n11/n02xxmn01xxmn01xxmn04/n05xxmn01xxmn03/xxQn04d0Un04xxQ/n03xxMn01xxMn01xxMn02Y0x/n04xxMD0xxxMn04/n07S0xn03/n11/n03E0xA0xn06",
    inputFen: "l6,10;l5,10;mp",
    expectedKinds: ["mon-move", "pickup-potion"],
  },
];

describe("valid-game event ordering", () => {
  for (const testCase of CASES) {
    it(testCase.name, () => {
      const game = MonsGame.fromFen(testCase.fen, false);
      const inputs = parseInputArrayFen(testCase.inputFen);
      expect(game).toBeDefined();
      expect(inputs).toBeDefined();
      if (game === undefined || inputs === undefined) return;

      const output = game.processInput(inputs, false, false);
      expect(output.kind).toBe("events");
      if (output.kind !== "events") return;
      expect(output.events.map(({ kind }) => kind)).toEqual(
        testCase.expectedKinds,
      );
    });
  }
});
