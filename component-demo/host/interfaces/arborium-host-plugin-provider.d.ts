/** @module Interface arborium:host/plugin-provider@0.1.0 **/
export function loadPlugin(language: string): PluginHandle | undefined;
export function getInjectionLanguages(plugin: PluginHandle): Array<string>;
export function createPluginSession(plugin: PluginHandle): PluginSession;
export function freePluginSession(plugin: PluginHandle, session: PluginSession): void;
export function pluginSetText(plugin: PluginHandle, session: PluginSession, text: string): void;
export function pluginApplyEdit(plugin: PluginHandle, session: PluginSession, text: string, edit: Edit): void;
export function pluginParse(plugin: PluginHandle, session: PluginSession): ParseResult;
export function pluginCancel(plugin: PluginHandle, session: PluginSession): void;
export type PluginHandle = number;
export type PluginSession = number;
export type Edit = import('./arborium-host-types.js').Edit;
export type ParseResult = import('./arborium-host-types.js').ParseResult;
export type ParseError = import('./arborium-host-types.js').ParseError;
