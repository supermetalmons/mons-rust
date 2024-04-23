/* tslint:disable */
/* eslint-disable */
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
export enum Consumable {
  Potion = 0,
  Bomb = 1,
  BombOrPotion = 2,
}
/**
*/
export enum AvailableMoveKind {
  MonMove = 0,
  ManaMove = 1,
  Action = 2,
  Potion = 3,
}
/**
*/
export enum EventModelKind {
  MonMove = 0,
  ManaMove = 1,
  ManaScored = 2,
  MysticAction = 3,
  DemonAction = 4,
  DemonAdditionalStep = 5,
  SpiritTargetMove = 6,
  PickupBomb = 7,
  PickupPotion = 8,
  PickupMana = 9,
  MonFainted = 10,
  ManaDropped = 11,
  SupermanaBackToBase = 12,
  BombAttack = 13,
  MonAwake = 14,
  BombExplosion = 15,
  NextTurn = 16,
  GameOver = 17,
}
/**
*/
export enum ManaKind {
  Regular = 0,
  Supermana = 1,
}
/**
*/
export enum MonKind {
  Demon = 0,
  Drainer = 1,
  Angel = 2,
  Spirit = 3,
  Mystic = 4,
}
/**
*/
export enum Modifier {
  SelectPotion = 0,
  SelectBomb = 1,
  Cancel = 2,
}
/**
*/
export enum OutputModelKind {
  InvalidInput = 0,
  LocationsToStartFrom = 1,
  NextInputOptions = 2,
  Events = 3,
}
/**
*/
export enum ItemModelKind {
  Mon = 0,
  Mana = 1,
  MonWithMana = 2,
  MonWithConsumable = 3,
  Consumable = 4,
}
/**
*/
export enum SquareModelKind {
  Regular = 0,
  ConsumableBase = 1,
  SupermanaBase = 2,
  ManaBase = 3,
  ManaPool = 4,
  MonBase = 5,
}
/**
*/
export enum Color {
  White = 0,
  Black = 1,
}
/**
*/
export enum NextInputKind {
  MonMove = 0,
  ManaMove = 1,
  MysticAction = 2,
  DemonAction = 3,
  DemonAdditionalStep = 4,
  SpiritTargetCapture = 5,
  SpiritTargetMove = 6,
  SelectConsumable = 7,
  BombAttack = 8,
}
/**
*/
export class EventModel {
  free(): void;
}
/**
*/
export class ItemModel {
  free(): void;
}
/**
*/
export class Location {
  free(): void;
/**
* @param {number} i
* @param {number} j
*/
  constructor(i: number, j: number);
/**
*/
  i: number;
/**
*/
  j: number;
}
/**
*/
export class ManaModel {
  free(): void;
/**
*/
  color: Color;
/**
*/
  kind: ManaKind;
}
/**
*/
export class Mon {
  free(): void;
/**
* @param {MonKind} kind
* @param {Color} color
* @param {number} cooldown
* @returns {Mon}
*/
  static new(kind: MonKind, color: Color, cooldown: number): Mon;
/**
* @returns {boolean}
*/
  is_fainted(): boolean;
/**
*/
  faint(): void;
/**
*/
  decrease_cooldown(): void;
/**
*/
  color: Color;
/**
*/
  cooldown: number;
/**
*/
  kind: MonKind;
}
/**
*/
export class MonsGameModel {
  free(): void;
/**
* @param {string} fen
* @returns {MonsGameModel | undefined}
*/
  static from_fen(fen: string): MonsGameModel | undefined;
/**
* @returns {string}
*/
  fen(): string;
/**
* @param {(Location)[]} locations
* @param {Modifier | undefined} [modifier]
* @returns {OutputModel}
*/
  process_input(locations: (Location)[], modifier?: Modifier): OutputModel;
/**
* @param {string} input_fen
* @returns {OutputModel}
*/
  process_input_fen(input_fen: string): OutputModel;
/**
* @param {Location} at
* @returns {ItemModel | undefined}
*/
  item(at: Location): ItemModel | undefined;
/**
* @param {Location} at
* @returns {SquareModel}
*/
  square(at: Location): SquareModel;
/**
* @param {string} other_fen
* @returns {boolean}
*/
  is_later_than(other_fen: string): boolean;
/**
* @returns {Color}
*/
  active_color(): Color;
/**
* @returns {Color | undefined}
*/
  winner_color(): Color | undefined;
/**
* @returns {number}
*/
  black_score(): number;
/**
* @returns {number}
*/
  white_score(): number;
/**
* @returns {Int32Array}
*/
  available_move_kinds(): Int32Array;
/**
* @returns {(Location)[]}
*/
  locations_with_content(): (Location)[];
}
/**
*/
export class NextInputModel {
  free(): void;
/**
*/
  actor_mon_item?: ItemModel;
/**
*/
  kind: NextInputKind;
/**
*/
  location?: Location;
/**
*/
  modifier?: Modifier;
}
/**
*/
export class OutputModel {
  free(): void;
/**
* @returns {Array<any>}
*/
  locations(): Array<any>;
/**
* @returns {Array<any>}
*/
  next_inputs(): Array<any>;
/**
* @returns {Array<any>}
*/
  events(): Array<any>;
/**
* @returns {string}
*/
  input_fen(): string;
/**
*/
  kind: OutputModelKind;
}
/**
*/
export class SquareModel {
  free(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_monsgamemodel_free: (a: number) => void;
  readonly monsgamemodel_from_fen: (a: number, b: number) => number;
  readonly monsgamemodel_fen: (a: number, b: number) => void;
  readonly monsgamemodel_process_input: (a: number, b: number, c: number, d: number) => number;
  readonly monsgamemodel_process_input_fen: (a: number, b: number, c: number) => number;
  readonly monsgamemodel_item: (a: number, b: number) => number;
  readonly monsgamemodel_square: (a: number, b: number) => number;
  readonly monsgamemodel_is_later_than: (a: number, b: number, c: number) => number;
  readonly monsgamemodel_active_color: (a: number) => number;
  readonly monsgamemodel_winner_color: (a: number) => number;
  readonly monsgamemodel_black_score: (a: number) => number;
  readonly monsgamemodel_white_score: (a: number) => number;
  readonly monsgamemodel_available_move_kinds: (a: number, b: number) => void;
  readonly monsgamemodel_locations_with_content: (a: number, b: number) => void;
  readonly __wbg_outputmodel_free: (a: number) => void;
  readonly __wbg_get_outputmodel_kind: (a: number) => number;
  readonly __wbg_set_outputmodel_kind: (a: number, b: number) => void;
  readonly outputmodel_locations: (a: number) => number;
  readonly outputmodel_next_inputs: (a: number) => number;
  readonly outputmodel_events: (a: number) => number;
  readonly outputmodel_input_fen: (a: number, b: number) => void;
  readonly __wbg_itemmodel_free: (a: number) => void;
  readonly __wbg_manamodel_free: (a: number) => void;
  readonly __wbg_get_manamodel_kind: (a: number) => number;
  readonly __wbg_set_manamodel_kind: (a: number, b: number) => void;
  readonly __wbg_get_manamodel_color: (a: number) => number;
  readonly __wbg_set_manamodel_color: (a: number, b: number) => void;
  readonly __wbg_nextinputmodel_free: (a: number) => void;
  readonly __wbg_get_nextinputmodel_location: (a: number) => number;
  readonly __wbg_set_nextinputmodel_location: (a: number, b: number) => void;
  readonly __wbg_get_nextinputmodel_modifier: (a: number) => number;
  readonly __wbg_set_nextinputmodel_modifier: (a: number, b: number) => void;
  readonly __wbg_get_nextinputmodel_kind: (a: number) => number;
  readonly __wbg_set_nextinputmodel_kind: (a: number, b: number) => void;
  readonly __wbg_get_nextinputmodel_actor_mon_item: (a: number) => number;
  readonly __wbg_set_nextinputmodel_actor_mon_item: (a: number, b: number) => void;
  readonly __wbg_eventmodel_free: (a: number) => void;
  readonly __wbg_squaremodel_free: (a: number) => void;
  readonly __wbg_mon_free: (a: number) => void;
  readonly __wbg_get_mon_kind: (a: number) => number;
  readonly __wbg_set_mon_kind: (a: number, b: number) => void;
  readonly __wbg_get_mon_color: (a: number) => number;
  readonly __wbg_set_mon_color: (a: number, b: number) => void;
  readonly __wbg_get_mon_cooldown: (a: number) => number;
  readonly __wbg_set_mon_cooldown: (a: number, b: number) => void;
  readonly mon_new: (a: number, b: number, c: number) => number;
  readonly mon_is_fainted: (a: number) => number;
  readonly mon_faint: (a: number) => void;
  readonly mon_decrease_cooldown: (a: number) => void;
  readonly __wbg_location_free: (a: number) => void;
  readonly __wbg_get_location_i: (a: number) => number;
  readonly __wbg_set_location_i: (a: number, b: number) => void;
  readonly __wbg_get_location_j: (a: number) => number;
  readonly __wbg_set_location_j: (a: number, b: number) => void;
  readonly location_new: (a: number, b: number) => number;
  readonly winner: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
