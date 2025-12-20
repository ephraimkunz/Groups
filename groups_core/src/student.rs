use crate::constants::NUM_HOURS_PER_WEEK;
use base64::{Engine as _, engine::general_purpose};
use bitvec::prelude::*;
use time::OffsetDateTime;
use time_tz::{Offset, TimeZone, Tz, timezones};
use wasm_bindgen::prelude::*;

type AvailabilityBits = BitArr!(for NUM_HOURS_PER_WEEK, in u32, Lsb0);

/// Represents a student and their availability to meet with a group.
#[wasm_bindgen]
#[derive(Debug, PartialEq, Clone, Eq)]
pub struct Student {
    /// Student name
    name: String,

    /// Timezone student has expressed their availability in.
    timezone: &'static Tz,

    /// Set bit at an hour if the student is available then, else false.
    /// Starts on Monday at 12:00 AM.
    /// We want to store NUM_HOURS_PER_WEEK bits, so use these two integer types.
    /// We store it like this so the encoded version is very compact when base64 encoded.
    availability_bits: AvailabilityBits,
}

#[wasm_bindgen]
impl Student {
    /// Create a student with a name, timezone name (one of the values returned by the `timezones()` function),
    /// and availability string in that timezone (string of length `NUM_HOURS_PER_WEEK` containing 1s and 0s,
    /// where 1 indicates the student is available that hour, with the first element representing starting Monday at 12 AM, etc).
    pub fn new(name: &str, timezone: &str, availability: &str) -> Option<Student> {
        if availability.len() != NUM_HOURS_PER_WEEK {
            return None;
        }

        let tz = timezones::get_by_name(timezone)?;

        let mut availability_bits = bitarr!(u32, Lsb0; 0; NUM_HOURS_PER_WEEK);

        for (i, c) in availability.chars().enumerate() {
            availability_bits.set(i, c == '1');
        }

        Some(Student {
            name: name.to_string(),
            timezone: tz,
            availability_bits,
        })
    }

    /// Reconstructs a `Student` from a string produced by `encode()`. Returns None
    /// if `encoded` doesn't represent a valid student.
    pub fn from_encoded(encoded: &str) -> Option<Student> {
        // Decode from base64(<name>|<timezone name>|<u64 as a base 10 string>|<u64 as a base 10 string>|...).
        let bytes = general_purpose::STANDARD.decode(encoded).ok()?;
        let s = std::str::from_utf8(&bytes).ok()?;
        let pieces: Vec<_> = s.split('|').collect();

        if pieces.len() != 8 {
            None
        } else {
            let mut data = [0; 6];
            for (i, piece) in pieces.iter().skip(2).enumerate() {
                data[i] = piece.parse().ok()?;
            }

            let availability_bits = BitArray::new(data);

            Some(Self {
                name: pieces[0].to_string(),
                timezone: timezones::get_by_name(pieces[1])?,
                availability_bits,
            })
        }
    }

    fn availability_iter(&self, offset: i8) -> AvailabilityIter {
        let mut availability_bits = self.availability_bits;
        // We need to make sure we wrap the total bit array in a bitslice that once cares about
        // the bits we care about, since otherwise we could rotate things "off screen".
        // This is because bit array acts on the underlying storage, not the number of bits we told it.
        let slice = availability_bits.split_at_mut(NUM_HOURS_PER_WEEK).0;
        if offset > 0 {
            slice.rotate_left(offset as usize)
        } else {
            slice.rotate_right(offset.unsigned_abs() as usize)
        };

        AvailabilityIter {
            current: 0,
            availability_bits,
        }
    }

    /// Encode this student into a schedule code. This encapsulates all the information needed to
    /// reconstitute a Student object later, and is a little bit obfuscated.
    pub fn encode(&self) -> String {
        // Encode into base64(<name>|<timezone name>|<u32 as a base 10 string>|<32 as a base 10 string>|...).

        let mut s = format!("{}|{}", self.name, self.timezone.name(),);

        let availability = self.availability_bits.as_raw_slice();
        for i in availability {
            let segment = format!("|{}", i);
            s.push_str(&segment);
        }
        general_purpose::STANDARD.encode(s)
    }

    fn availability_offset_for_output_timezone(&self, timezone: &Tz) -> i8 {
        let old_tz = self.timezone;
        let new_tz = timezone;

        let now = OffsetDateTime::now_utc();

        // We're going to assume here that all the timezones we care about have hour granularity offsets,
        // which isn't true for all timezones but simplifies things a lot.

        let old_offset = old_tz.get_offset_utc(&now);
        let new_offset = new_tz.get_offset_utc(&now);
        old_offset.to_utc().whole_hours() - new_offset.to_utc().whole_hours()
    }

    /// Returns a string representing the students availability in `timezone`. Returns
    /// None if the timezone is not one of the timezones returned by `timezones()`.
    /// The returned string is `NUM_HOURS_PER_WEEK` characters long, where a '1' means the
    /// student is available and a '0' means the student is not available.
    pub fn availability_in_timezone(&self, timezone: &str) -> Option<String> {
        let new_tz = timezones::get_by_name(timezone)?;
        let difference = self.availability_offset_for_output_timezone(new_tz);

        let result: String = self
            .availability_iter(difference)
            .map(|a| if a { '1' } else { '0' })
            .collect();
        Some(result)
    }

    pub(crate) fn availability_array_in_utc(&self) -> AvailabilityBits {
        let utc = timezones::db::UTC;
        let difference = self.availability_offset_for_output_timezone(utc);
        self.availability_iter(difference).inner_availability()
    }

    /// The student's name.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// The student's timezone.
    pub fn timezone(&self) -> String {
        self.timezone.name().to_string()
    }
}

struct AvailabilityIter {
    availability_bits: AvailabilityBits,
    current: usize,
}

impl AvailabilityIter {
    fn inner_availability(&self) -> AvailabilityBits {
        self.availability_bits
    }
}

impl Iterator for AvailabilityIter {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == NUM_HOURS_PER_WEEK {
            None
        } else {
            let value = *self
                .availability_bits
                .get(self.current)
                .expect("Invalid access in availability iterator");
            self.current += 1;
            Some(value)
        }
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
        let encoded = general_purpose::STANDARD.encode("hi|111");
        let decoded = Student::from_encoded(&encoded);
        assert_eq!(decoded, None)
    }

    #[test]
    fn too_short_availability_decode() {
        let encoded = general_purpose::STANDARD.encode("hi|yo|111");
        let decoded = Student::from_encoded(&encoded);
        assert_eq!(decoded, None)
    }

    #[test]
    fn too_long_availability_decode() {
        let availability: String = (0..=NUM_HOURS_PER_WEEK).map(|_| "1").collect();
        let encoded = general_purpose::STANDARD.encode(format!("hi|yo|{}", availability));
        let decoded = Student::from_encoded(&encoded);
        assert_eq!(decoded, None)
    }

    #[test]
    fn invalid_timezone_decode() {
        let availability: String = (0..NUM_HOURS_PER_WEEK).map(|_| "1").collect();
        let encoded = general_purpose::STANDARD.encode(format!("hi|yo|{}", availability));
        let decoded = Student::from_encoded(&encoded);
        assert_eq!(decoded, None)
    }

    #[test]
    fn invalid_availability_char_decode() {
        let availability: String = (0..NUM_HOURS_PER_WEEK).map(|_| "x").collect();
        let encoded =
            general_purpose::STANDARD.encode(format!("hi|America/Los_Angeles|{}", availability));
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
    fn test_avail_offset_retarded_wraparound() {
        let avail: String = std::iter::once('1')
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
    fn test_avail_offset_retarded() {
        let avail: String = std::iter::once('0')
            .chain(std::iter::once('1'))
            .chain(std::iter::repeat('0').take(NUM_HOURS_PER_WEEK - 2))
            .collect();

        let student = Student::new("test", "America/Los_Angeles", &avail).unwrap();

        let avail_result = student
            .availability_in_timezone("America/Anchorage")
            .unwrap();

        let expected: String = std::iter::once('1')
            .chain(std::iter::repeat('0').take(NUM_HOURS_PER_WEEK - 1))
            .collect();

        assert_eq!(avail_result, expected)
    }

    #[test]
    fn test_avail_offset_advanced_wraparound() {
        let avail: String = std::iter::repeat('0')
            .take(NUM_HOURS_PER_WEEK - 1)
            .chain(std::iter::once('1'))
            .collect();

        let student = Student::new("test", "America/Los_Angeles", &avail).unwrap();
        let avail_result = student.availability_in_timezone("America/Boise").unwrap();

        let expected: String = std::iter::once('1')
            .chain(std::iter::repeat('0').take(NUM_HOURS_PER_WEEK - 1))
            .collect();

        assert_eq!(avail_result, expected)
    }

    #[test]
    fn test_avail_offset_advanced() {
        let avail: String = std::iter::repeat('0')
            .take(NUM_HOURS_PER_WEEK - 2)
            .chain(std::iter::once('1'))
            .chain(std::iter::once('0'))
            .collect();

        let student = Student::new("test", "America/Los_Angeles", &avail).unwrap();
        let avail_result = student.availability_in_timezone("America/Boise").unwrap();

        let expected: String = std::iter::repeat('0')
            .take(NUM_HOURS_PER_WEEK - 1)
            .chain(std::iter::once('1'))
            .collect();

        assert_eq!(avail_result, expected)
    }
}
