import {
  type Event,
  type NextInput,
  NextInputKind,
  type Output,
} from "../domain.js";
import { compareAscii, locationFen } from "./common.js";
import { colorFen, itemFen, manaFen, monFen } from "./domain-item.js";
import { inputFen } from "./input.js";

export function nextInputKindFen(kind: NextInputKind): string {
  switch (kind) {
    case NextInputKind.MonMove:
      return "mm";
    case NextInputKind.ManaMove:
      return "mma";
    case NextInputKind.MysticAction:
      return "ma";
    case NextInputKind.DemonAction:
      return "da";
    case NextInputKind.DemonAdditionalStep:
      return "das";
    case NextInputKind.SpiritTargetCapture:
      return "stc";
    case NextInputKind.SpiritTargetMove:
      return "stm";
    case NextInputKind.SelectConsumable:
      return "sc";
    case NextInputKind.BombAttack:
      return "ba";
  }
}

export function nextInputFen(nextInput: NextInput): string {
  return `${inputFen(nextInput.input)} ${nextInputKindFen(nextInput.kind)} ${
    nextInput.actorMonItem === undefined ? "o" : itemFen(nextInput.actorMonItem)
  }`;
}

export function eventFen(event: Event): string {
  switch (event.kind) {
    case "mon-move":
      return `mm ${itemFen(event.item)} ${locationFen(event.from)} ${locationFen(event.to)}`;
    case "mana-move":
      return `mma ${manaFen(event.mana)} ${locationFen(event.from)} ${locationFen(event.to)}`;
    case "mana-scored":
      return `ms ${manaFen(event.mana)} ${locationFen(event.at)}`;
    case "mystic-action":
      return `ma ${monFen(event.mystic)} ${locationFen(event.from)} ${locationFen(event.to)}`;
    case "demon-action":
      return `da ${monFen(event.demon)} ${locationFen(event.from)} ${locationFen(event.to)}`;
    case "demon-additional-step":
      return `das ${monFen(event.demon)} ${locationFen(event.from)} ${locationFen(event.to)}`;
    case "spirit-target-move":
      return `stm ${itemFen(event.item)} ${locationFen(event.from)} ${locationFen(event.to)} ${locationFen(event.by)}`;
    case "pickup-bomb":
      return `pb ${monFen(event.by)} ${locationFen(event.at)}`;
    case "pickup-potion":
      return `pp ${itemFen(event.by)} ${locationFen(event.at)}`;
    case "use-potion":
      return `up ${locationFen(event.from)} ${locationFen(event.to)}`;
    case "pickup-mana":
      return `pm ${manaFen(event.mana)} ${monFen(event.by)} ${locationFen(event.at)}`;
    case "mon-fainted":
      return `mf ${monFen(event.mon)} ${locationFen(event.from)} ${locationFen(event.to)}`;
    case "mana-dropped":
      return `md ${manaFen(event.mana)} ${locationFen(event.at)}`;
    case "supermana-back-to-base":
      return `sb ${locationFen(event.from)} ${locationFen(event.to)}`;
    case "bomb-attack":
      return `ba ${monFen(event.by)} ${locationFen(event.from)} ${locationFen(event.to)}`;
    case "mon-awake":
      return `maw ${monFen(event.mon)} ${locationFen(event.at)}`;
    case "bomb-explosion":
      return `be ${locationFen(event.at)}`;
    case "next-turn":
      return `nt ${colorFen(event.color)}`;
    case "game-over":
      return `go ${colorFen(event.winner)}`;
    case "takeback":
      return "z";
  }
}

/** Tracking entities retain event application order rather than sorting it. */
export function eventArrayFen(events: readonly Event[]): string {
  return events.map(eventFen).join(" ");
}

export function outputFen(output: Output): string {
  switch (output.kind) {
    case "invalid-input":
      return "i";
    case "locations-to-start-from":
      return `l${output.locations
        .map(locationFen)
        .sort(compareAscii)
        .join("/")}`;
    case "next-input-options":
      return `n${output.nextInputs
        .map(nextInputFen)
        .sort(compareAscii)
        .join("/")}`;
    case "events":
      return `e${output.events.map(eventFen).sort(compareAscii).join("/")}`;
  }
}
