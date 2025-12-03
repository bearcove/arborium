/** @module Interface arborium:host/types@0.1.0 **/
export interface Edit {
  startByte: number,
  oldEndByte: number,
  newEndByte: number,
  startRow: number,
  startCol: number,
  oldEndRow: number,
  oldEndCol: number,
  newEndRow: number,
  newEndCol: number,
}
export interface Span {
  start: number,
  end: number,
  capture: string,
}
export interface Injection {
  start: number,
  end: number,
  language: string,
  includeChildren: boolean,
}
export interface ParseResult {
  spans: Array<Span>,
  injections: Array<Injection>,
}
export interface ParseError {
  message: string,
}
