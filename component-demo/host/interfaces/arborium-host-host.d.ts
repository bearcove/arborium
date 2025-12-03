/** @module Interface arborium:host/host@0.1.0 **/
export function createDocument(language: string): Document | undefined;
export function freeDocument(doc: Document): void;
export function setText(doc: Document, text: string): void;
export function applyEdit(doc: Document, text: string, edit: Edit): void;
export function highlight(doc: Document, maxDepth: number): HighlightResult;
export function cancel(doc: Document): void;
export function getRequiredLanguages(doc: Document): Array<string>;
export type Edit = import('./arborium-host-types.js').Edit;
export type Document = number;
export interface HighlightSpan {
  start: number,
  end: number,
  capture: string,
  language: string,
}
export interface HighlightResult {
  spans: Array<HighlightSpan>,
}
