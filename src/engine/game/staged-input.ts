import type { Board } from "../board/storage.js";
import { type Location } from "../board/geometry.js";
import {
  inputEquals,
  type Input,
  type Item,
  type NextInput,
  type NextInputKind,
  type Output,
} from "../model/domain.js";
import { compileFourthInput } from "./event-compilation.js";
import { firstOptionFromEachKindGroup } from "./input-support.js";
import type { SuggestedStartInputOptions } from "./input-support.js";
import type { InputStageResult } from "./query-cache.js";

type StagedInputContext = {
  board(): Board;
  requiresCounterCapacityFiltering(): boolean;
  secondInputOptions(
    startLocation: Location,
    startItem: Item,
    cacheComputed: boolean,
    specificLocation: Location | undefined,
  ): NextInput[];
  processSecondInput(
    kind: NextInputKind,
    startItem: Item,
    startLocation: Location,
    targetLocation: Location,
    cacheComputed: boolean,
  ): InputStageResult;
  processThirdInput(
    thirdInput: NextInput,
    startItem: Item,
    startLocation: Location,
    targetLocation: Location,
    cacheComputed: boolean,
  ): InputStageResult;
  applicableInputOptions(
    prefix: readonly Input[],
    options: NextInput[],
    suggestedStartOptions: SuggestedStartInputOptions,
    required: boolean,
    oneOptionEnough: boolean,
  ): NextInput[];
  resolveEvents(events: readonly Event[], doNotApplyEvents: boolean): Output;
};

import type { Event } from "../model/domain.js";

export function dispatchStagedInput(
  game: StagedInputContext,
  input: readonly Input[],
  doNotApplyEvents: boolean,
  oneOptionEnough: boolean,
  suggestedStartOptions: SuggestedStartInputOptions,
  filterCounterCapacityOptions: boolean,
): Output {
  const firstInput = input[0];
  if (firstInput?.kind !== "location") {
    return { kind: "invalid-input" };
  }
  const startLocation = firstInput.location;
  const startItem = game.board().get(startLocation);
  if (startItem === undefined) {
    return { kind: "invalid-input" };
  }
  const specificSecondInput = input[1];
  if (specificSecondInput !== undefined && specificSecondInput.kind !== "location") {
    return { kind: "invalid-input" };
  }
  const filterCounterCapacity =
    filterCounterCapacityOptions && game.requiresCounterCapacityFiltering();
  const secondInputOptions = game.secondInputOptions(
    startLocation,
    startItem,
    doNotApplyEvents,
    doNotApplyEvents ? undefined : specificSecondInput?.location,
  );

  if (specificSecondInput === undefined) {
    const presentedOptions =
      oneOptionEnough && !filterCounterCapacity
        ? firstOptionFromEachKindGroup(secondInputOptions)
        : secondInputOptions;
    const applicableOptions = game.applicableInputOptions(
      input,
      presentedOptions,
      suggestedStartOptions,
      filterCounterCapacity,
      oneOptionEnough,
    );
    return applicableOptions.length === 0
      ? { kind: "invalid-input" }
      : { kind: "next-input-options", nextInputs: applicableOptions };
  }
  const targetLocation = specificSecondInput.location;
  const secondOption = secondInputOptions.find((option) =>
    inputEquals(option.input, specificSecondInput),
  );
  if (secondOption === undefined) {
    return { kind: "invalid-input" };
  }

  const specificThirdInput = input[2];
  const secondResult = game.processSecondInput(
    secondOption.kind,
    startItem,
    startLocation,
    targetLocation,
    doNotApplyEvents,
  );
  const events = secondResult?.[0] ?? [];
  const thirdInputOptions = secondResult?.[1] ?? [];

  if (specificThirdInput === undefined) {
    if (thirdInputOptions.length !== 0) {
      const applicableOptions = game.applicableInputOptions(
        input,
        thirdInputOptions,
        suggestedStartOptions,
        filterCounterCapacity,
        oneOptionEnough,
      );
      return applicableOptions.length === 0
        ? { kind: "invalid-input" }
        : { kind: "next-input-options", nextInputs: applicableOptions };
    }
    return events.length === 0
      ? { kind: "invalid-input" }
      : game.resolveEvents(events, doNotApplyEvents);
  }

  const thirdInput = thirdInputOptions.find((option) =>
    inputEquals(option.input, specificThirdInput),
  );
  if (thirdInput === undefined) {
    return { kind: "invalid-input" };
  }
  const specificFourthInput = input[3];
  const thirdResult = game.processThirdInput(
    thirdInput,
    startItem,
    startLocation,
    targetLocation,
    doNotApplyEvents,
  );
  events.push(...(thirdResult?.[0] ?? []));
  const fourthInputOptions = thirdResult?.[1] ?? [];

  if (specificFourthInput === undefined) {
    if (fourthInputOptions.length !== 0) {
      const applicableOptions = game.applicableInputOptions(
        input,
        fourthInputOptions,
        suggestedStartOptions,
        filterCounterCapacity,
        oneOptionEnough,
      );
      return applicableOptions.length === 0
        ? { kind: "invalid-input" }
        : { kind: "next-input-options", nextInputs: applicableOptions };
    }
    return events.length === 0
      ? { kind: "invalid-input" }
      : game.resolveEvents(events, doNotApplyEvents);
  }

  const fourthEvent = compileFourthInput(
    specificFourthInput,
    thirdInput,
    fourthInputOptions,
  );
  if (fourthEvent === undefined) {
    return { kind: "invalid-input" };
  }
  events.push(fourthEvent);
  return game.resolveEvents(events, doNotApplyEvents);
}
