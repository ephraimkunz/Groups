use time::OffsetDateTime;
use wasm_bindgen::prelude::*;

pub mod constants;
pub mod random;
pub mod scheduling;
pub mod student;
pub mod timezones;

fn now() -> OffsetDateTime {
    // Shim getting the now UTC date since the time crate doesn't support WASM and will panic otherwise.
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    {
        time::OffsetDateTime::now_utc()
    }
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    {
        OffsetDateTime::UNIX_EPOCH
            + time::Duration::milliseconds(js_sys::Date::new_0().get_time() as i64)
    }
}

/// Initializes the library for use in WASM. This function should be called before any others in this library in a
/// WASM context. It only needs to be called once.
#[wasm_bindgen]
pub fn groups_core_init_wasm() {
    console_error_panic_hook::set_once();
}
