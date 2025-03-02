use time_tz::{TimeZone, timezones};
use wasm_bindgen::prelude::*;

/// Like `timezones`, but returns a Javascript array of strings for use in WASM.
#[wasm_bindgen]
pub fn timezones_wasm() -> JsValue {
    let names = timezones();
    serde_wasm_bindgen::to_value(&names).unwrap()
}

/// Get a list of all supported timezone names.
pub fn timezones() -> Vec<String> {
    let mut names: Vec<String> = timezones::iter().map(|tz| tz.name().to_string()).collect();
    names.sort_unstable();
    names
}
