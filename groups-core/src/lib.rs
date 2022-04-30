use scheduling::NUM_HOURS_PER_WEEK;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time_tz::{timezones, Offset, TimeZone, Tz};
use wasm_bindgen::prelude::*;

pub mod scheduling;

const NUM_USED_BITS_IN_AVAILABILITY1: usize = NUM_HOURS_PER_WEEK - 128;

// Use https://rustwasm.github.io/docs/wasm-bindgen/reference/arbitrary-data-with-serde.html
// since wasm_bindgen doesn't yet support returning an array of strings.
#[derive(Serialize, Deserialize)]
pub struct Timezones {
    names: Vec<String>,
}

#[wasm_bindgen]
pub fn tz_groups_init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn timezones_wasm() -> JsValue {
    let names = timezones();
    let tzs = Timezones { names };
    JsValue::from_serde(&tzs).unwrap()
}

pub fn timezones() -> Vec<String> {
    let mut names: Vec<String> = timezones::iter().map(|tz| tz.name().to_string()).collect();
    names.sort_unstable();
    names
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Clone)]
pub struct Student {
    /// Student name
    name: String,

    /// Timezone student has expressed their availability in.
    timezone: &'static Tz,

    /// Set bit at an hour if the student is available then, else false.
    /// Starts on Monday at 12:00 AM.
    /// We want to store NUM_HOURS_PER_WEEK bits, so use these two integer types.
    /// We store it like this so the encoded version is very compact when base64 encoded.
    availability0: u128,
    availability1: u64,
}

#[wasm_bindgen]
impl Student {
    /// Create a student with a name, timezone name (one of the values returned by the timezones() function),
    /// and availability string in that timezone
    /// (string of NUM_HOURS_PER_WEEK 1s and 0s, where 1 indicated available that hour, starting Monday at 12 AM).
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str, timezone: &str, availability: &str) -> Option<Student> {
        if availability.len() != NUM_HOURS_PER_WEEK {
            return None;
        }

        let tz = timezones::get_by_name(timezone)?;

        let mut availability0 = 0u128;
        for (i, c) in availability.chars().take(128).enumerate() {
            if c == '1' {
                availability0 |= 1 << i;
            }
        }

        let mut availability1 = 0u64;
        for (i, c) in availability
            .chars()
            .skip(128)
            .take(NUM_USED_BITS_IN_AVAILABILITY1)
            .enumerate()
        {
            if c == '1' {
                availability1 |= 1 << i;
            }
        }

        Some(Student {
            name: name.to_string(),
            timezone: tz,
            availability0,
            availability1,
        })
    }

    pub fn from_encoded(encoded: &str) -> Option<Student> {
        // Decode from base64(<name>|<timezone name>|<u128 as a base 10 string>|<u64 as a base 10 string>).
        let bytes = base64::decode(encoded).ok()?;
        let s = std::str::from_utf8(&bytes).ok()?;
        let pieces: Vec<_> = s.split('|').collect();

        if pieces.len() != 4 {
            None
        } else {
            let availability0 = pieces[2].parse().ok()?;
            let availability1 = pieces[3].parse().ok()?;

            Some(Self {
                name: pieces[0].to_string(),
                timezone: timezones::get_by_name(pieces[1])?,
                availability0,
                availability1,
            })
        }
    }

    /// Convert internal availability representation of packed bits to a string where 1 represents available, 0 represents unavailable.
    fn availability_string(&self) -> String {
        let mut availability_string = String::with_capacity(NUM_HOURS_PER_WEEK);

        for i in 0..128 {
            let c = if self.availability0 & (1 << i) == 0 {
                '0'
            } else {
                '1'
            };
            availability_string.push(c);
        }

        for i in 0..(NUM_USED_BITS_IN_AVAILABILITY1) {
            let c = if self.availability1 & (1 << i) == 0 {
                '0'
            } else {
                '1'
            };
            availability_string.push(c);
        }

        availability_string
    }

    pub fn encode(&self) -> String {
        // Encode into base64(<name>|<timezone name>|<u128 as a base 10 string>|<u64 as a base 10 string>).

        let s = format!(
            "{}|{}|{}|{}",
            self.name,
            self.timezone.name(),
            self.availability0,
            self.availability1
        );
        base64::encode(s)
    }

    fn now(&self) -> OffsetDateTime {
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

    pub fn availability_in_timezone(&self, timezone: &str) -> Option<String> {
        let mut char_array: Vec<_> = self.availability_string().chars().collect();

        let now = self.now();
        let old_tz = self.timezone;
        let new_tz = timezones::get_by_name(timezone)?;

        // We're going to assume here that all the timezones we care about have hour granularity offsets,
        // which isn't true for all timezones but simplifies things a lot.

        let old_offset = old_tz.get_offset_utc(&now);
        let new_offset = new_tz.get_offset_utc(&now);
        let difference = old_offset.to_utc().whole_hours() - new_offset.to_utc().whole_hours();
        // Rotating here is how we shift timezones, since it causes everything to wrap around the week properly.
        if difference > 0 {
            char_array.rotate_left(difference as usize)
        } else {
            char_array.rotate_right(difference.abs() as usize)
        }

        Some(char_array.iter().collect())
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn timezone(&self) -> String {
        self.timezone.name().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_simple_encoding() {
        let avail: String = (0..NUM_HOURS_PER_WEEK).map(|_| "0").collect();
        let student = Student::new(
            "Test Student",
            timezones::db::america::LOS_ANGELES.name(),
            &avail,
        )
        .unwrap();

        let encoded = student.encode();
        let decoded = Student::from_encoded(&encoded);

        assert_eq!(Some(student), decoded)
    }

    #[test]
    fn round_trip_complex_encoding() {
        let avail = "100000000000000000000000000000000001111000000000000000000000000111100000000000110000001000000000000000000000111111100000011000001100000000000000000000111111000000000000";
        let student = Student::new(
            "Test Student",
            timezones::db::america::LOS_ANGELES.name(),
            avail,
        )
        .unwrap();

        let encoded = student.encode();

        println!("{}", encoded);
        let decoded = Student::from_encoded(&encoded);

        assert_eq!(Some(student), decoded)
    }

    #[test]
    fn empty_decode() {
        let encoded = "";
        let decoded = Student::from_encoded(encoded);

        assert_eq!(decoded, None)
    }

    #[test]
    fn missing_section_decode() {
        let encoded = base64::encode("hi|111");
        let decoded = Student::from_encoded(&encoded);
        assert_eq!(decoded, None)
    }

    #[test]
    fn too_short_availability_decode() {
        let encoded = base64::encode("hi|yo|111");
        let decoded = Student::from_encoded(&encoded);
        assert_eq!(decoded, None)
    }

    #[test]
    fn too_long_availability_decode() {
        let availability: String = (0..=NUM_HOURS_PER_WEEK).map(|_| "1").collect();
        let encoded = base64::encode(format!("hi|yo|{}", availability));
        let decoded = Student::from_encoded(&encoded);
        assert_eq!(decoded, None)
    }

    #[test]
    fn invalid_timezone_decode() {
        let availability: String = (0..NUM_HOURS_PER_WEEK).map(|_| "1").collect();
        let encoded = base64::encode(format!("hi|yo|{}", availability));
        let decoded = Student::from_encoded(&encoded);
        assert_eq!(decoded, None)
    }

    #[test]
    fn invalid_availability_char_decode() {
        let availability: String = (0..NUM_HOURS_PER_WEEK).map(|_| "x").collect();
        let encoded = base64::encode(format!("hi|America/Los_Angeles|{}", availability));
        let decoded = Student::from_encoded(&encoded);
        assert_eq!(decoded, None)
    }

    #[test]
    fn test_avail_same_tz() {
        let avail: String = "1"
            .chars()
            .chain(std::iter::repeat('0').take(NUM_HOURS_PER_WEEK - 1))
            .collect();

        let student = Student::new("test", "America/Los_Angeles", &avail).unwrap();
        let avail_result = student
            .availability_in_timezone("America/Los_Angeles")
            .unwrap();

        assert_eq!(avail, avail_result)
    }

    #[test]
    fn test_avail_offset_retarded() {
        // Midnight hour on Monday in LA, 11 PM Sunday in Anchorage
        let avail: String = "1"
            .chars()
            .chain(std::iter::repeat('0').take(NUM_HOURS_PER_WEEK - 1))
            .collect();

        let student = Student::new("test", "America/Los_Angeles", &avail).unwrap();
        let avail_result = student
            .availability_in_timezone("America/Anchorage")
            .unwrap();

        let expected: String = std::iter::repeat('0')
            .take(NUM_HOURS_PER_WEEK - 1)
            .chain("1".chars())
            .collect();

        assert_eq!(avail_result, expected)
    }

    #[test]
    fn test_avail_offset_advanced() {
        // Midnight hour on Monday in LA, 1 AM Monday in Boise
        let avail: String = "1"
            .chars()
            .chain(std::iter::repeat('0').take(NUM_HOURS_PER_WEEK - 1))
            .collect();

        let student = Student::new("test", "America/Los_Angeles", &avail).unwrap();
        let avail_result = student.availability_in_timezone("America/Boise").unwrap();

        let expected: String = "0"
            .chars()
            .chain("1".chars())
            .chain(std::iter::repeat('0').take(NUM_HOURS_PER_WEEK - 2))
            .collect();

        assert_eq!(avail_result, expected)
    }
}
