import type { Board } from "../board/storage.js";
import { nearbyLocations, type Location } from "../board/geometry.js";
import {
  Consumable,
  Modifier,
  MonKind,
  NextInputKind,
  inputEquals,
  itemConsumable,
  itemMana,
  itemMon,
  type Event,
  type Input,
  type Item,
  type NextInput,
} from "../model/domain.js";
import { spiritDestinationItemAllowed } from "../rules/legality.js";
import { nextInput } from "./input-support.js";
import { regularSquareForMovement } from "../rules/legality.js";
import type { InputStageResult } from "./query-cache.js";

type EventCompilationContext = {
  readonly board: Board;
  nextInputsFromLocations(
    locations: readonly Location[],
    kind: NextInputKind,
    specific: Location | undefined,
    filter: (location: Location) => boolean,
  ): NextInput[];
};

export function compileSecondInput(
  game: EventCompilationContext,
  kind: NextInputKind,
  startItem: Item,
  startLocation: Location,
  targetLocation: Location,
): InputStageResult {
  const thirdInputOptions: NextInput[] = [];
  const events: Event[] = [];
  const targetSquare = game.board.squareAt(targetLocation);
  const targetItem = game.board.get(targetLocation);

  switch (kind) {
    case NextInputKind.MonMove: {
      const movingMon = itemMon(startItem);
      if (movingMon === undefined) {
        return undefined;
      }
      events.push({
        kind: "mon-move",
        item: startItem,
        from: startLocation,
        to: targetLocation,
      });

      if (targetItem !== undefined) {
        switch (targetItem.kind) {
          case "mon":
          case "mon-with-mana":
          case "mon-with-consumable":
            return undefined;
          case "mana": {
            const startMana = itemMana(startItem);
            if (startMana !== undefined) {
              events.push({
                kind: "mana-dropped",
                mana: startMana,
                at: startLocation,
              });
            }
            events.push({
              kind: "pickup-mana",
              mana: targetItem.mana,
              by: movingMon,
              at: targetLocation,
            });
            break;
          }
          case "consumable":
            switch (targetItem.consumable) {
              case Consumable.Bomb:
              case Consumable.Potion:
                return undefined;
              case Consumable.BombOrPotion:
                if (
                  itemConsumable(startItem) !== undefined ||
                  itemMana(startItem) !== undefined
                ) {
                  events.push({
                    kind: "pickup-potion",
                    by: startItem,
                    at: targetLocation,
                  });
                } else {
                  thirdInputOptions.push(
                    nextInput(
                      { kind: "modifier", modifier: Modifier.SelectBomb },
                      NextInputKind.SelectConsumable,
                      startItem,
                    ),
                  );
                  thirdInputOptions.push(
                    nextInput(
                      { kind: "modifier", modifier: Modifier.SelectPotion },
                      NextInputKind.SelectConsumable,
                      startItem,
                    ),
                  );
                }
                break;
            }
            break;
        }
      }

      if (targetSquare.kind === "mana-pool") {
        const manaInHand = itemMana(startItem);
        if (manaInHand !== undefined) {
          events.push({
            kind: "mana-scored",
            mana: manaInHand,
            at: targetLocation,
          });
        }
      }
      break;
    }
    case NextInputKind.ManaMove: {
      if (startItem.kind !== "mana") {
        return undefined;
      }
      const mana = startItem.mana;
      events.push({
        kind: "mana-move",
        mana,
        from: startLocation,
        to: targetLocation,
      });
      if (targetItem !== undefined) {
        if (targetItem.kind !== "mon") {
          return undefined;
        }
        events.push({
          kind: "pickup-mana",
          mana,
          by: targetItem.mon,
          at: targetLocation,
        });
      }
      switch (targetSquare.kind) {
        case "regular":
        case "mana-base":
        case "consumable-base":
          break;
        case "mana-pool":
          events.push({
            kind: "mana-scored",
            mana,
            at: targetLocation,
          });
          break;
        case "mon-base":
        case "supermana-base":
          return undefined;
      }
      break;
    }
    case NextInputKind.MysticAction: {
      if (startItem.kind !== "mon") {
        return undefined;
      }
      events.push({
        kind: "mystic-action",
        mystic: startItem.mon,
        from: startLocation,
        to: targetLocation,
      });
      if (targetItem !== undefined) {
        const targetMon = itemMon(targetItem);
        if (targetMon === undefined) {
          return undefined;
        }
        events.push({
          kind: "mon-fainted",
          mon: targetMon,
          from: targetLocation,
          to: game.board.base(targetMon),
        });
        if (targetItem.kind === "mon-with-mana") {
          if (targetItem.mana.kind === "regular") {
            events.push({
              kind: "mana-dropped",
              mana: targetItem.mana,
              at: targetLocation,
            });
          } else {
            events.push({
              kind: "supermana-back-to-base",
              from: targetLocation,
              to: game.board.supermanaBase(),
            });
          }
        }
        if (targetItem.kind === "mon-with-consumable") {
          switch (targetItem.consumable) {
            case Consumable.Bomb:
              events.push({ kind: "bomb-explosion", at: targetLocation });
              break;
            case Consumable.Potion:
            case Consumable.BombOrPotion:
              return undefined;
          }
        }
      }
      break;
    }
    case NextInputKind.DemonAction: {
      if (startItem.kind !== "mon") {
        return undefined;
      }
      const startMon = startItem.mon;
      events.push({
        kind: "demon-action",
        demon: startMon,
        from: startLocation,
        to: targetLocation,
      });
      let requiresAdditionalStep = false;
      let attackerFainted = false;
      if (targetItem !== undefined) {
        const targetMon = itemMon(targetItem);
        if (targetMon === undefined) {
          return undefined;
        }
        events.push({
          kind: "mon-fainted",
          mon: targetMon,
          from: targetLocation,
          to: game.board.base(targetMon),
        });
        if (targetItem.kind === "mon-with-mana") {
          if (targetItem.mana.kind === "regular") {
            requiresAdditionalStep = true;
            events.push({
              kind: "mana-dropped",
              mana: targetItem.mana,
              at: targetLocation,
            });
          } else {
            events.push({
              kind: "supermana-back-to-base",
              from: targetLocation,
              to: game.board.supermanaBase(),
            });
          }
        }
        if (targetItem.kind === "mon-with-consumable") {
          switch (targetItem.consumable) {
            case Consumable.Bomb:
              attackerFainted = true;
              events.push({ kind: "bomb-explosion", at: targetLocation });
              events.push({
                kind: "mon-fainted",
                mon: startMon,
                from: targetLocation,
                to: game.board.base(startMon),
              });
              break;
            case Consumable.Potion:
            case Consumable.BombOrPotion:
              return undefined;
          }
        }
      }
      if (
        !attackerFainted &&
        (targetSquare.kind === "supermana-base" || targetSquare.kind === "mon-base")
      ) {
        requiresAdditionalStep = true;
      }
      if (requiresAdditionalStep) {
        for (const at of nearbyLocations(targetLocation)) {
          const item = game.board.get(at);
          const square = game.board.squareAt(at);
          if (item !== undefined && item.kind !== "consumable") {
            continue;
          }
          if (regularSquareForMovement(square)) {
            thirdInputOptions.push(
              nextInput(
                { kind: "location", location: at },
                NextInputKind.DemonAdditionalStep,
              ),
            );
          } else if (
            square.kind === "mon-base" &&
            square.monKind === startMon.kind &&
            square.color === startMon.color
          ) {
            thirdInputOptions.push(
              nextInput(
                { kind: "location", location: at },
                NextInputKind.DemonAdditionalStep,
              ),
            );
          }
        }
      }
      break;
    }
    case NextInputKind.SpiritTargetCapture: {
      if (targetItem === undefined) {
        return undefined;
      }
      const targetMon = itemMon(targetItem);
      const targetMana = itemMana(targetItem);
      thirdInputOptions.push(
        ...game.nextInputsFromLocations(
          nearbyLocations(targetLocation),
          NextInputKind.SpiritTargetMove,
          undefined,
          (at) => {
            const destinationItem = game.board.get(at);
            const destinationSquare = game.board.squareAt(at);
            if (!spiritDestinationItemAllowed(targetItem, destinationItem)) {
              return false;
            }
            switch (destinationSquare.kind) {
              case "regular":
              case "consumable-base":
              case "mana-base":
              case "mana-pool":
                return true;
              case "supermana-base": {
                const destinationHasSupermana =
                  destinationItem?.kind === "mana" &&
                  destinationItem.mana.kind === "supermana";
                return (
                  targetMana?.kind === "supermana" ||
                  (targetMana === undefined &&
                    targetMon?.kind === MonKind.Drainer &&
                    (destinationItem === undefined || destinationHasSupermana)) ||
                  (targetMon?.kind === MonKind.Drainer && destinationHasSupermana)
                );
              }
              case "mon-base":
                return (
                  targetMon?.kind === destinationSquare.monKind &&
                  targetMon.color === destinationSquare.color &&
                  targetMana === undefined &&
                  itemConsumable(targetItem) === undefined
                );
            }
          },
        ),
      );
      break;
    }
    case NextInputKind.BombAttack: {
      const startMon = itemMon(startItem);
      if (startMon === undefined) {
        return undefined;
      }
      events.push({
        kind: "bomb-attack",
        by: startMon,
        from: startLocation,
        to: targetLocation,
      });
      if (targetItem !== undefined) {
        const targetMon = itemMon(targetItem);
        if (targetMon === undefined) {
          return undefined;
        }
        events.push({
          kind: "mon-fainted",
          mon: targetMon,
          from: targetLocation,
          to: game.board.base(targetMon),
        });
        if (targetItem.kind === "mon-with-mana") {
          if (targetItem.mana.kind === "regular") {
            events.push({
              kind: "mana-dropped",
              mana: targetItem.mana,
              at: targetLocation,
            });
          } else {
            events.push({
              kind: "supermana-back-to-base",
              from: targetLocation,
              to: game.board.supermanaBase(),
            });
          }
        }
        if (targetItem.kind === "mon-with-consumable") {
          switch (targetItem.consumable) {
            case Consumable.Bomb:
              events.push({ kind: "bomb-explosion", at: targetLocation });
              break;
            case Consumable.Potion:
            case Consumable.BombOrPotion:
              return undefined;
          }
        }
      }
      break;
    }
    case NextInputKind.DemonAdditionalStep:
    case NextInputKind.SpiritTargetMove:
    case NextInputKind.SelectConsumable:
      break;
  }

  return [events, thirdInputOptions];
}

export function compileThirdInput(
  board: Board,
  thirdInput: NextInput,
  startItem: Item,
  startLocation: Location,
  targetLocation: Location,
): InputStageResult {
  const targetItem = board.get(targetLocation);
  const fourthInputOptions: NextInput[] = [];
  const events: Event[] = [];

  switch (thirdInput.kind) {
    case NextInputKind.SpiritTargetMove: {
      if (thirdInput.input.kind !== "location" || targetItem === undefined) {
        return undefined;
      }
      const destinationLocation = thirdInput.input.location;
      const destinationItem = board.get(destinationLocation);
      const destinationSquare = board.squareAt(destinationLocation);
      events.push({
        kind: "spirit-target-move",
        item: targetItem,
        from: targetLocation,
        to: destinationLocation,
        by: startLocation,
      });

      if (destinationItem !== undefined) {
        switch (targetItem.kind) {
          case "mon":
            switch (destinationItem.kind) {
              case "mon":
              case "mon-with-mana":
              case "mon-with-consumable":
                return undefined;
              case "mana":
                events.push({
                  kind: "pickup-mana",
                  mana: destinationItem.mana,
                  by: targetItem.mon,
                  at: destinationLocation,
                });
                break;
              case "consumable":
                switch (destinationItem.consumable) {
                  case Consumable.Potion:
                  case Consumable.Bomb:
                    return undefined;
                  case Consumable.BombOrPotion:
                    fourthInputOptions.push(
                      nextInput(
                        { kind: "modifier", modifier: Modifier.SelectBomb },
                        NextInputKind.SelectConsumable,
                        targetItem,
                      ),
                    );
                    fourthInputOptions.push(
                      nextInput(
                        { kind: "modifier", modifier: Modifier.SelectPotion },
                        NextInputKind.SelectConsumable,
                        targetItem,
                      ),
                    );
                    break;
                }
                break;
            }
            break;
          case "mana":
            if (destinationItem.kind !== "mon") {
              return undefined;
            }
            events.push({
              kind: "pickup-mana",
              mana: targetItem.mana,
              by: destinationItem.mon,
              at: destinationLocation,
            });
            break;
          case "mon-with-mana":
            switch (destinationItem.kind) {
              case "mon":
              case "mon-with-mana":
              case "mon-with-consumable":
                return undefined;
              case "mana":
                events.push({
                  kind: "mana-dropped",
                  mana: targetItem.mana,
                  at: targetLocation,
                });
                events.push({
                  kind: "pickup-mana",
                  mana: destinationItem.mana,
                  by: targetItem.mon,
                  at: destinationLocation,
                });
                break;
              case "consumable":
                switch (destinationItem.consumable) {
                  case Consumable.Potion:
                  case Consumable.Bomb:
                    return undefined;
                  case Consumable.BombOrPotion:
                    events.push({
                      kind: "pickup-potion",
                      by: targetItem,
                      at: destinationLocation,
                    });
                    break;
                }
                break;
            }
            break;
          case "mon-with-consumable":
            if (
              destinationItem.kind !== "consumable" ||
              destinationItem.consumable !== Consumable.BombOrPotion
            ) {
              return undefined;
            }
            events.push({
              kind: "pickup-potion",
              by: targetItem,
              at: destinationLocation,
            });
            break;
          case "consumable":
            switch (destinationItem.kind) {
              case "mana":
              case "consumable":
                return undefined;
              case "mon":
                fourthInputOptions.push(
                  nextInput(
                    { kind: "modifier", modifier: Modifier.SelectBomb },
                    NextInputKind.SelectConsumable,
                    destinationItem,
                  ),
                );
                fourthInputOptions.push(
                  nextInput(
                    { kind: "modifier", modifier: Modifier.SelectPotion },
                    NextInputKind.SelectConsumable,
                    destinationItem,
                  ),
                );
                break;
              case "mon-with-mana":
              case "mon-with-consumable":
                switch (targetItem.consumable) {
                  case Consumable.Potion:
                  case Consumable.Bomb:
                    return undefined;
                  case Consumable.BombOrPotion:
                    events.push({
                      kind: "pickup-potion",
                      by: destinationItem,
                      at: destinationLocation,
                    });
                    break;
                }
                break;
            }
            break;
        }
      }

      if (destinationSquare.kind === "mana-pool") {
        const mana = itemMana(targetItem);
        if (mana !== undefined) {
          events.push({
            kind: "mana-scored",
            mana,
            at: destinationLocation,
          });
        }
      }
      break;
    }
    case NextInputKind.DemonAdditionalStep: {
      if (thirdInput.input.kind !== "location") {
        return undefined;
      }
      const demon = itemMon(startItem);
      if (demon === undefined) {
        return undefined;
      }
      const destinationLocation = thirdInput.input.location;
      events.push({
        kind: "demon-additional-step",
        demon,
        from: targetLocation,
        to: destinationLocation,
      });
      const destinationItem = board.get(destinationLocation);
      if (destinationItem?.kind === "consumable") {
        switch (destinationItem.consumable) {
          case Consumable.Potion:
          case Consumable.Bomb:
            return undefined;
          case Consumable.BombOrPotion:
            fourthInputOptions.push(
              nextInput(
                { kind: "modifier", modifier: Modifier.SelectBomb },
                NextInputKind.SelectConsumable,
                startItem,
              ),
            );
            fourthInputOptions.push(
              nextInput(
                { kind: "modifier", modifier: Modifier.SelectPotion },
                NextInputKind.SelectConsumable,
                startItem,
              ),
            );
            break;
        }
      }
      break;
    }
    case NextInputKind.SelectConsumable: {
      if (thirdInput.input.kind !== "modifier") {
        return undefined;
      }
      const mon = itemMon(startItem);
      if (mon === undefined) {
        return undefined;
      }
      switch (thirdInput.input.modifier) {
        case Modifier.SelectBomb:
          events.push({
            kind: "pickup-bomb",
            by: mon,
            at: targetLocation,
          });
          break;
        case Modifier.SelectPotion:
          events.push({
            kind: "pickup-potion",
            by: startItem,
            at: targetLocation,
          });
          break;
      }
      break;
    }
    case NextInputKind.MonMove:
    case NextInputKind.ManaMove:
    case NextInputKind.MysticAction:
    case NextInputKind.DemonAction:
    case NextInputKind.SpiritTargetCapture:
    case NextInputKind.BombAttack:
      return undefined;
  }

  return [events, fourthInputOptions];
}

export function compileFourthInput(
  input: Input,
  thirdInput: NextInput,
  options: readonly NextInput[],
): Event | undefined {
  if (input.kind !== "modifier" || thirdInput.input.kind !== "location") {
    return undefined;
  }
  const fourthInput = options.find((option) => inputEquals(option.input, input));
  const actorMonItem = fourthInput?.actorMonItem;
  const actorMon = actorMonItem === undefined ? undefined : itemMon(actorMonItem);
  if (actorMonItem === undefined || actorMon === undefined) {
    return undefined;
  }
  switch (input.modifier) {
    case Modifier.SelectBomb:
      return {
        kind: "pickup-bomb",
        by: actorMon,
        at: thirdInput.input.location,
      };
    case Modifier.SelectPotion:
      return {
        kind: "pickup-potion",
        by: actorMonItem,
        at: thirdInput.input.location,
      };
  }
}
