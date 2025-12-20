use wasm_bindgen::prelude::*;

pub mod constants;
pub mod random;
pub mod scheduling;
pub mod student;
pub mod timezones;

/// Initializes the library for use in WASM. This function should be called before any others in this library in a
/// WASM context. It only needs to be called once.
#[wasm_bindgen]
pub fn groups_core_init_wasm() {
    console_error_panic_hook::set_once();
}
