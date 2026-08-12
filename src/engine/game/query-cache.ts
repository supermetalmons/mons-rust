import {
  copyEvent,
  copyNextInput,
  copyOutput,
  type Event,
  type NextInput,
  type Output,
} from "../model/domain.js";

export type InputStageResult = [Event[], NextInput[]] | undefined;
export const CACHE_MISS = Symbol("cache-miss");
const CACHED_UNDEFINED = Symbol("cached-undefined");
type StoredInputStageResult =
  Exclude<InputStageResult, undefined> | typeof CACHED_UNDEFINED;

const CACHE_CAPACITY = Object.freeze({
  completionViability: 16_384,
  secondInputOptions: 4_096,
  secondStage: 8_192,
  thirdStage: 8_192,
});

function copyStageResult(result: InputStageResult): InputStageResult {
  return result === undefined
    ? undefined
    : [result[0].map(copyEvent), result[1].map(copyNextInput)];
}

function storeStageResult(result: InputStageResult): StoredInputStageResult {
  return result === undefined
    ? CACHED_UNDEFINED
    : [result[0].map(copyEvent), result[1].map(copyNextInput)];
}

function copyStoredStageResult(result: StoredInputStageResult): InputStageResult {
  return result === CACHED_UNDEFINED ? undefined : copyStageResult(result);
}

function setBounded<T>(
  cache: Map<string, T>,
  key: string,
  value: T,
  capacity: number,
): void {
  if (cache.size >= capacity && !cache.has(key)) {
    cache.clear();
  }
  cache.set(key, value);
}

/**
 * Per-game cache for pure input-resolution queries.
 *
 * Mutations clear every cache, and copied or simulation games start with an
 * independent cold cache.
 */
export class RulesQueryCache {
  #startSuggestions: (Output | undefined)[] | undefined;
  #completionViability: Map<string, boolean> | undefined;
  #secondInputOptions: Map<string, readonly NextInput[]> | undefined;
  #secondStage: Map<string, StoredInputStageResult> | undefined;
  #thirdStage: Map<string, StoredInputStageResult> | undefined;

  public invalidate(): void {
    this.#startSuggestions = undefined;
    this.#completionViability = undefined;
    this.#secondInputOptions = undefined;
    this.#secondStage = undefined;
    this.#thirdStage = undefined;
  }

  public getStartSuggestion(key: number): Output | undefined {
    const output = this.#startSuggestions?.[key];
    return output === undefined ? undefined : copyOutput(output);
  }

  public setStartSuggestion(key: number, output: Output): void {
    (this.#startSuggestions ??= new Array<Output | undefined>(4))[key] =
      copyOutput(output);
  }

  public getCompletionViability(key: string): boolean | undefined {
    return this.#completionViability?.get(key);
  }

  public setCompletionViability(key: string, applicable: boolean): void {
    setBounded(
      (this.#completionViability ??= new Map<string, boolean>()),
      key,
      applicable,
      CACHE_CAPACITY.completionViability,
    );
  }

  public getSecondInputOptions(key: string): NextInput[] | undefined {
    const options = this.#secondInputOptions?.get(key);
    return options === undefined ? undefined : options.map(copyNextInput);
  }

  public setSecondInputOptions(key: string, options: readonly NextInput[]): void {
    setBounded(
      (this.#secondInputOptions ??= new Map<string, readonly NextInput[]>()),
      key,
      options.map(copyNextInput),
      CACHE_CAPACITY.secondInputOptions,
    );
  }

  public lookupSecondStage(key: string): InputStageResult | typeof CACHE_MISS {
    const result = this.#secondStage?.get(key);
    return result === undefined ? CACHE_MISS : copyStoredStageResult(result);
  }

  public setSecondStage(key: string, result: InputStageResult): void {
    setBounded(
      (this.#secondStage ??= new Map<string, StoredInputStageResult>()),
      key,
      storeStageResult(result),
      CACHE_CAPACITY.secondStage,
    );
  }

  public lookupThirdStage(key: string): InputStageResult | typeof CACHE_MISS {
    const result = this.#thirdStage?.get(key);
    return result === undefined ? CACHE_MISS : copyStoredStageResult(result);
  }

  public setThirdStage(key: string, result: InputStageResult): void {
    setBounded(
      (this.#thirdStage ??= new Map<string, StoredInputStageResult>()),
      key,
      storeStageResult(result),
      CACHE_CAPACITY.thirdStage,
    );
  }
}
