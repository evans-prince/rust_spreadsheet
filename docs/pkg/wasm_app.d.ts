/* tslint:disable */
/* eslint-disable */

export class SpreadsheetApp {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Run one command, e.g. "A1=23", "B2=A1+10", "C1=SUM(A1:A10)", "U" (undo), "R" (redo).
     * Returns false only for the quit command ("Q"); true otherwise.
     */
    execute(input: string): boolean;
    /**
     * Displayed value of a cell (0-indexed), formatted as a string.
     * Empty/unset cells return "".
     */
    get_cell(row: number, col: number): string;
    /**
     * The raw formula/input that produced a cell's value (for editing it again).
     */
    get_formula(row: number, col: number): string;
    /**
     * How long the last command took to execute, in milliseconds.
     */
    last_time_ms(): number;
    /**
     * Create a fresh, empty spreadsheet.
     */
    constructor();
    redo(): void;
    /**
     * Status of the last executed command (e.g. "ok", "circular_reference", "DIV0").
     */
    status(): string;
    undo(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_spreadsheetapp_free: (a: number, b: number) => void;
    readonly spreadsheetapp_execute: (a: number, b: number, c: number) => number;
    readonly spreadsheetapp_get_cell: (a: number, b: number, c: number) => [number, number];
    readonly spreadsheetapp_get_formula: (a: number, b: number, c: number) => [number, number];
    readonly spreadsheetapp_last_time_ms: (a: number) => number;
    readonly spreadsheetapp_new: () => number;
    readonly spreadsheetapp_redo: (a: number) => void;
    readonly spreadsheetapp_status: (a: number) => [number, number];
    readonly spreadsheetapp_undo: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
