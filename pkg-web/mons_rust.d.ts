/* tslint:disable */
/* eslint-disable */
/**
* @returns {string}
*/
export function hello(): string;
/**
* @param {string} fen_w
* @param {string} fen_b
* @param {string} flat_moves_string_w
* @param {string} flat_moves_string_b
* @returns {string}
*/
export function winner(fen_w: string, fen_b: string, flat_moves_string_w: string, flat_moves_string_b: string): string;
/**
*/
export enum Modifier {
  SelectPotion = 0,
  SelectBomb = 1,
  Cancel = 2,
}
/**
*/
export enum Color {
  White = 0,
  Black = 1,
}
/**
*/
export class Location {
  free(): void;
/**
*/
  i: number;
/**
*/
  j: number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_location_free: (a: number) => void;
  readonly __wbg_get_location_i: (a: number) => number;
  readonly __wbg_set_location_i: (a: number, b: number) => void;
  readonly __wbg_get_location_j: (a: number) => number;
  readonly __wbg_set_location_j: (a: number, b: number) => void;
  readonly hello: (a: number) => void;
  readonly winner: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
