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
