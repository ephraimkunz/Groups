/* tslint:disable */
/* eslint-disable */
/**
 * Same as `create_groups`, but suitable for calling from WASM because it takes and returns JSValues.
 * `students` is a Javascript array of encoded Student (strings).
 * `output_timezone` is the timezone which will be used when generating the `suggested_meet_times` array in
 * each output group.
 * Returns a Javascript array of JSON objects representing groups.
 */
export function create_groups_wasm(students: any, group_size: number, output_timezone: string): any;
/**
 * Like `timezones`, but returns a Javascript array of strings for use in WASM.
 */
export function timezones_wasm(): any;
/**
 * Initializes the library for use in WASM. This function should be called before any others in this library in a
 * WASM context. It only needs to be called once.
 */
export function groups_core_init_wasm(): void;
/**
 * Represents a student and their availability to meet with a group.
 */
export class Student {
  private constructor();
  free(): void;
  /**
   * Create a student with a name, timezone name (one of the values returned by the `timezones()` function),
   * and availability string in that timezone (string of length `NUM_HOURS_PER_WEEK` containing 1s and 0s,
   * where 1 indicates the student is available that hour, with the first element representing starting Monday at 12 AM, etc).
   */
  static new(name: string, timezone: string, availability: string): Student | undefined;
  /**
   * Reconstructs a `Student` from a string produced by `encode()`. Returns None
   * if `encoded` doesn't represent a valid student.
   */
  static from_encoded(encoded: string): Student | undefined;
  /**
   * Encode this student into a schedule code. This encapsulates all the information needed to
   * reconstitute a Student object later, and is a little bit obfuscated.
   */
  encode(): string;
  /**
   * Returns a string representing the students availability in `timezone`. Returns
   * None if the timezone is not one of the timezones returned by `timezones()`.
   * The returned string is `NUM_HOURS_PER_WEEK` characters long, where a '1' means the
   * student is available and a '0' means the student is not available.
   */
  availability_in_timezone(timezone: string): string | undefined;
  /**
   * The student's name.
   */
  name(): string;
  /**
   * The student's timezone.
   */
  timezone(): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_student_free: (a: number, b: number) => void;
  readonly student_new: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
  readonly student_from_encoded: (a: number, b: number) => number;
  readonly student_encode: (a: number) => [number, number];
  readonly student_availability_in_timezone: (a: number, b: number, c: number) => [number, number];
  readonly student_name: (a: number) => [number, number];
  readonly student_timezone: (a: number) => [number, number];
  readonly create_groups_wasm: (a: any, b: number, c: number, d: number) => any;
  readonly timezones_wasm: () => any;
  readonly groups_core_init_wasm: () => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
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
