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
export enum AvailableMoveKind {
  MonMove = 0,
  ManaMove = 1,
  Action = 2,
  Potion = 3,
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
export enum OutputModelKind {
  InvalidInput = 0,
  LocationsToStartFrom = 1,
  NextInputOptions = 2,
  Events = 3,
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
export enum Color {
  White = 0,
  Black = 1,
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
export enum MonKind {
  Demon = 0,
  Drainer = 1,
  Angel = 2,
  Spirit = 3,
  Mystic = 4,
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
export enum Consumable {
  Potion = 0,
  Bomb = 1,
  BombOrPotion = 2,
}
/**
*/
export enum ManaKind {
  Regular = 0,
  Supermana = 1,
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
* @returns {MonsGameModel}
*/
  static new(): MonsGameModel;
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
* @returns {(Location)[]}
*/
  locations(): (Location)[];
/**
* @returns {(NextInputModel)[]}
*/
  next_inputs(): (NextInputModel)[];
/**
* @returns {(EventModel)[]}
*/
  events(): (EventModel)[];
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
