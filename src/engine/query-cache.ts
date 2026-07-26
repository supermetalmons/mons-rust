import {
  copyEvent,
  copyNextInput,
  copyOutput,
  type Event,
  type NextInput,
  type Output,
} from "./domain.js";

export type InputStageResult = [Event[], NextInput[]] | undefined;

const CACHE_CAPACITY = Object.freeze({
  startSuggestions: 8,
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
  readonly #startSuggestions = new Map<string, Output>();
  readonly #completionViability = new Map<string, boolean>();
  readonly #secondInputOptions = new Map<string, readonly NextInput[]>();
  readonly #secondStage = new Map<string, InputStageResult>();
  readonly #thirdStage = new Map<string, InputStageResult>();

  public invalidate(): void {
    this.#startSuggestions.clear();
    this.#completionViability.clear();
    this.#secondInputOptions.clear();
    this.#secondStage.clear();
    this.#thirdStage.clear();
  }

  public getStartSuggestion(key: string): Output | undefined {
    const output = this.#startSuggestions.get(key);
    return output === undefined ? undefined : copyOutput(output);
  }

  public setStartSuggestion(key: string, output: Output): void {
    setBounded(
      this.#startSuggestions,
      key,
      copyOutput(output),
      CACHE_CAPACITY.startSuggestions,
    );
  }

  public getCompletionViability(key: string): boolean | undefined {
    return this.#completionViability.get(key);
  }

  public setCompletionViability(key: string, applicable: boolean): void {
    setBounded(
      this.#completionViability,
      key,
      applicable,
      CACHE_CAPACITY.completionViability,
    );
  }

  public getSecondInputOptions(key: string): NextInput[] | undefined {
    const options = this.#secondInputOptions.get(key);
    return options === undefined ? undefined : options.map(copyNextInput);
  }

  public setSecondInputOptions(
    key: string,
    options: readonly NextInput[],
  ): void {
    setBounded(
      this.#secondInputOptions,
      key,
      options.map(copyNextInput),
      CACHE_CAPACITY.secondInputOptions,
    );
  }

  public hasSecondStage(key: string): boolean {
    return this.#secondStage.has(key);
  }

  public getSecondStage(key: string): InputStageResult {
    return copyStageResult(this.#secondStage.get(key));
  }

  public setSecondStage(key: string, result: InputStageResult): void {
    setBounded(
      this.#secondStage,
      key,
      copyStageResult(result),
      CACHE_CAPACITY.secondStage,
    );
  }

  public hasThirdStage(key: string): boolean {
    return this.#thirdStage.has(key);
  }

  public getThirdStage(key: string): InputStageResult {
    return copyStageResult(this.#thirdStage.get(key));
  }

  public setThirdStage(key: string, result: InputStageResult): void {
    setBounded(
      this.#thirdStage,
      key,
      copyStageResult(result),
      CACHE_CAPACITY.thirdStage,
    );
  }
}
