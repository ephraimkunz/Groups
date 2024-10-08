/* tslint:disable */
/* eslint-disable */
/**
* Initializes the library for use in WASM. This function should be called before any others in this library in a
* WASM context. It only needs to be called once.
*/
export function groups_core_init_wasm(): void;
/**
* Same as `create_groups`, but suitable for calling from WASM because it takes and returns JSValues.
* `students` is a Javascript array of encoded Student (strings).
* `output_timezone` is the timezone which will be used when generating the `suggested_meet_times` array in
* each output group.
* Returns a Javascript array of JSON objects representing groups.
* @param {any} students
* @param {number} group_size
* @param {string} output_timezone
* @returns {any}
*/
export function create_groups_wasm(students: any, group_size: number, output_timezone: string): any;
/**
* Like `timezones`, but returns a Javascript array of strings for use in WASM.
* @returns {any}
*/
export function timezones_wasm(): any;
/**
* Represents a student and their availability to meet with a group.
*/
export class Student {
  free(): void;
/**
* Create a student with a name, timezone name (one of the values returned by the `timezones()` function),
* and availability string in that timezone (string of length `NUM_HOURS_PER_WEEK` containing 1s and 0s,
* where 1 indicates the student is available that hour, with the first element representing starting Monday at 12 AM, etc).
* @param {string} name
* @param {string} timezone
* @param {string} availability
* @returns {Student | undefined}
*/
  static new(name: string, timezone: string, availability: string): Student | undefined;
/**
* Reconstructs a `Student` from a string produced by `encode()`. Returns None
* if `encoded` doesn't represent a valid student.
* @param {string} encoded
* @returns {Student | undefined}
*/
  static from_encoded(encoded: string): Student | undefined;
/**
* Encode this student into a schedule code. This encapsulates all the information needed to
* reconstitute a Student object later, and is a little bit obfuscated.
* @returns {string}
*/
  encode(): string;
/**
* Returns a string representing the students availability in `timezone`. Returns
* None if the timezone is not one of the timezones returned by `timezones()`.
* The returned string is `NUM_HOURS_PER_WEEK` characters long, where a '1' means the
* student is available and a '0' means the student is not available.
* @param {string} timezone
* @returns {string | undefined}
*/
  availability_in_timezone(timezone: string): string | undefined;
/**
* The student's name.
* @returns {string}
*/
  name(): string;
/**
* The student's timezone.
* @returns {string}
*/
  timezone(): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_student_free: (a: number, b: number) => void;
  readonly student_new: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
  readonly student_from_encoded: (a: number, b: number) => number;
  readonly student_encode: (a: number, b: number) => void;
  readonly student_availability_in_timezone: (a: number, b: number, c: number, d: number) => void;
  readonly student_name: (a: number, b: number) => void;
  readonly student_timezone: (a: number, b: number) => void;
  readonly groups_core_init_wasm: () => void;
  readonly create_groups_wasm: (a: number, b: number, c: number, d: number) => number;
  readonly timezones_wasm: () => number;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
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
