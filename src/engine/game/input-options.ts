import type { Board } from "../board/storage.js";
import {
  bombReachableLocations,
  demonReachableLocations,
  locationBetween,
  mysticReachableLocations,
  nearbyLocations,
  spiritReachableLocations,
  type Location,
} from "../board/geometry.js";
import {
  Consumable,
  MonKind,
  NextInputKind,
  isMonFainted,
  isSpiritTargetAllowed,
  itemMon,
  otherColor,
  type Color,
  type Item,
  type NextInput,
} from "../model/domain.js";
import { isLocationGuardedByAngelLocation } from "./input-support.js";
import {
  regularManaMoveDestinationAllowed,
  regularSquareForMovement,
} from "../rules/legality.js";

type InputOptionGame = {
  readonly board: Board;
  readonly activeColor: Color;
  playerCanMoveMon(): boolean;
  playerCanMoveMana(): boolean;
  playerCanUseAction(): boolean;
  nextInputsFromLocations(
    locations: readonly Location[],
    kind: NextInputKind,
    specific: Location | undefined,
    filter: (location: Location) => boolean,
  ): NextInput[];
};

export function generateSecondInputOptions(
  game: InputOptionGame,
  startLocation: Location,
  startItem: Item,
  specificLocation: Location | undefined,
): NextInput[] {
  const startSquare = game.board.squareAt(startLocation);
  const options: NextInput[] = [];

  switch (startItem.kind) {
    case "mon": {
      const mon = startItem.mon;
      if (mon.color !== game.activeColor || isMonFainted(mon)) {
        break;
      }
      if (game.playerCanMoveMon()) {
        options.push(
          ...game.nextInputsFromLocations(
            nearbyLocations(startLocation),
            NextInputKind.MonMove,
            specificLocation,
            (at) => {
              const item = game.board.get(at);
              const square = game.board.squareAt(at);
              let itemAllows: boolean;
              switch (item?.kind) {
                case "mon":
                case "mon-with-mana":
                case "mon-with-consumable":
                  itemAllows = false;
                  break;
                case "mana":
                  itemAllows = mon.kind === MonKind.Drainer;
                  break;
                case "consumable":
                case undefined:
                  itemAllows = true;
                  break;
              }
              if (!itemAllows) {
                return false;
              }
              switch (square.kind) {
                case "regular":
                case "consumable-base":
                case "mana-base":
                case "mana-pool":
                  return true;
                case "supermana-base":
                  return (
                    mon.kind === MonKind.Drainer &&
                    (item === undefined ||
                      (item.kind === "mana" && item.mana.kind === "supermana"))
                  );
                case "mon-base":
                  return square.monKind === mon.kind && square.color === mon.color;
              }
            },
          ),
        );
      }

      if (startSquare.kind !== "mon-base" && game.playerCanUseAction()) {
        switch (mon.kind) {
          case MonKind.Angel:
          case MonKind.Drainer:
            break;
          case MonKind.Mystic: {
            const opponentsAngelLocation = game.board.findAwakeAngel(
              otherColor(game.activeColor),
            );
            options.push(
              ...game.nextInputsFromLocations(
                mysticReachableLocations(startLocation),
                NextInputKind.MysticAction,
                specificLocation,
                (at) => {
                  const item = game.board.get(at);
                  if (
                    item === undefined ||
                    isLocationGuardedByAngelLocation(opponentsAngelLocation, at)
                  ) {
                    return false;
                  }
                  const targetMon = itemMon(item);
                  return (
                    targetMon !== undefined &&
                    mon.color !== targetMon.color &&
                    !isMonFainted(targetMon)
                  );
                },
              ),
            );
            break;
          }
          case MonKind.Demon: {
            const opponentsAngelLocation = game.board.findAwakeAngel(
              otherColor(game.activeColor),
            );
            options.push(
              ...game.nextInputsFromLocations(
                demonReachableLocations(startLocation),
                NextInputKind.DemonAction,
                specificLocation,
                (at) => {
                  const item = game.board.get(at);
                  const between = locationBetween(startLocation, at);
                  const betweenSquare = game.board.squareAt(between);
                  if (
                    item === undefined ||
                    isLocationGuardedByAngelLocation(opponentsAngelLocation, at) ||
                    game.board.get(between) !== undefined ||
                    betweenSquare.kind === "supermana-base" ||
                    betweenSquare.kind === "mon-base"
                  ) {
                    return false;
                  }
                  const targetMon = itemMon(item);
                  return (
                    targetMon !== undefined &&
                    mon.color !== targetMon.color &&
                    !isMonFainted(targetMon)
                  );
                },
              ),
            );
            break;
          }
          case MonKind.Spirit:
            options.push(
              ...game.nextInputsFromLocations(
                spiritReachableLocations(startLocation),
                NextInputKind.SpiritTargetCapture,
                specificLocation,
                (at) => {
                  const item = game.board.get(at);
                  if (item === undefined) {
                    return false;
                  }
                  return isSpiritTargetAllowed(item);
                },
              ),
            );
            break;
        }
      }
      break;
    }
    case "mana":
      if (
        startItem.mana.kind === "regular" &&
        startItem.mana.color === game.activeColor &&
        game.playerCanMoveMana()
      ) {
        options.push(
          ...game.nextInputsFromLocations(
            nearbyLocations(startLocation),
            NextInputKind.ManaMove,
            specificLocation,
            (at) => regularManaMoveDestinationAllowed(game.board, at),
          ),
        );
      }
      break;
    case "mon-with-mana":
      if (startItem.mon.color === game.activeColor && game.playerCanMoveMon()) {
        options.push(
          ...game.nextInputsFromLocations(
            nearbyLocations(startLocation),
            NextInputKind.MonMove,
            specificLocation,
            (at) => {
              const item = game.board.get(at);
              const square = game.board.squareAt(at);
              switch (item?.kind) {
                case "mon":
                case "mon-with-mana":
                case "mon-with-consumable":
                  return false;
                case "mana":
                case "consumable":
                  return true;
                case undefined:
                  if (regularSquareForMovement(square)) {
                    return true;
                  }
                  return (
                    square.kind === "supermana-base" &&
                    startItem.mana.kind === "supermana"
                  );
              }
            },
          ),
        );
      }
      break;
    case "mon-with-consumable": {
      const mon = startItem.mon;
      if (mon.color !== game.activeColor) {
        break;
      }
      if (game.playerCanMoveMon()) {
        options.push(
          ...game.nextInputsFromLocations(
            nearbyLocations(startLocation),
            NextInputKind.MonMove,
            specificLocation,
            (at) => {
              const item = game.board.get(at);
              const square = game.board.squareAt(at);
              switch (item?.kind) {
                case "mon":
                case "mana":
                case "mon-with-mana":
                case "mon-with-consumable":
                  return false;
                case "consumable":
                  return true;
                case undefined:
                  return regularSquareForMovement(square);
              }
            },
          ),
        );
      }
      if (startItem.consumable === Consumable.Bomb) {
        options.push(
          ...game.nextInputsFromLocations(
            bombReachableLocations(startLocation),
            NextInputKind.BombAttack,
            specificLocation,
            (at) => {
              const item = game.board.get(at);
              const targetMon = item === undefined ? undefined : itemMon(item);
              return (
                targetMon !== undefined &&
                mon.color !== targetMon.color &&
                !isMonFainted(targetMon)
              );
            },
          ),
        );
      }
      break;
    }
    case "consumable":
      break;
  }

  return options;
}
