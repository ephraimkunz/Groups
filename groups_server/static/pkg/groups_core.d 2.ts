/* tslint:disable */
/* eslint-disable */
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
* Initializes the library for use in WASM. This function should be called before any others in this library in a 
* WASM context. It only needs to be called once.
*/
export function groups_core_init(): void;
/**
* Like `timezones`, but returns a Javascript array of strings for use in WASM.
* @returns {any}
*/
export function timezones_wasm(): any;
/**
*/
export class Student {
  free(): void;
/**
* Create a student with a name, timezone name (one of the values returned by the timezones() function),
* and availability string in that timezone
* (string of NUM_HOURS_PER_WEEK 1s and 0s, where 1 indicated available that hour, starting Monday at 12 AM).
* @param {string} name
* @param {string} timezone
* @param {string} availability
*/
  constructor(name: string, timezone: string, availability: string);
/**
* @param {string} encoded
* @returns {Student | undefined}
*/
  static from_encoded(encoded: string): Student | undefined;
/**
* @returns {string}
*/
  encode(): string;
/**
* @param {string} timezone
* @returns {string | undefined}
*/
  availability_in_timezone(timezone: string): string | undefined;
/**
* @returns {string}
*/
  name(): string;
/**
* @returns {string}
*/
  timezone(): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly create_groups_wasm: (a: number, b: number, c: number, d: number) => number;
  readonly groups_core_init: () => void;
  readonly __wbg_student_free: (a: number) => void;
  readonly student_new: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
  readonly student_from_encoded: (a: number, b: number) => number;
  readonly student_encode: (a: number, b: number) => void;
  readonly student_availability_in_timezone: (a: number, b: number, c: number, d: number) => void;
  readonly student_name: (a: number, b: number) => void;
  readonly student_timezone: (a: number, b: number) => void;
  readonly timezones_wasm: () => number;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
