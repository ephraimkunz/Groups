use crate::{now, timezones};

use super::Student;
use rand::{prelude::SmallRng, seq::SliceRandom};
use rand::{thread_rng, Rng, RngCore, SeedableRng};
use serde::{Deserialize, Serialize};
use time_tz::{Offset, TimeZone};
use wasm_bindgen::prelude::*;

mod hillclimbing_strategy;

pub const NUM_DAYS_PER_WEEK: usize = 7;
pub const NUM_HOURS_PER_DAY: usize = 24;
pub const NUM_HOURS_PER_WEEK: usize = NUM_HOURS_PER_DAY * NUM_DAYS_PER_WEEK;

/// Internal representation of a group, where students is a vector of encoded Student and
/// suggested_meet_times is a vector of hours in the week when students are all available in UTC.
#[derive(Debug, PartialEq)]
pub struct Group {
    students: Vec<String>,
    suggested_meet_times: Vec<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct DisplayGroup {
    students: Vec<String>,
    suggested_meet_times: Vec<String>,
}

#[wasm_bindgen]
pub fn create_groups_wasm(
    students: &JsValue,
    group_size: usize,
    output_timezone: String,
) -> JsValue {
    let student_strings: Vec<String> = students.into_serde().unwrap();
    let groups = create_groups(&student_strings, group_size);
    let display = display_groups(&groups, &output_timezone);
    JsValue::from_serde(&display).unwrap()
}

pub fn create_groups(students_encoded: &[String], group_size: usize) -> Vec<Group> {
    let students: Vec<Student> = students_encoded
        .iter()
        .filter_map(|s| Student::from_encoded(s))
        .collect();
    hillclimbing_strategy::run(&students, group_size)
}

fn display_groups(groups: &[Group], timezone: &str) -> Vec<DisplayGroup> {
    groups
        .iter()
        .map(|g| DisplayGroup {
            students: g.students.clone(),
            suggested_meet_times: pretty_hours(&g.suggested_meet_times, timezone),
        })
        .collect()
}

fn pretty_hours(hours_in_utc: &[usize], output_timezone: &str) -> Vec<String> {
    let tz = timezones::get_by_name(output_timezone).unwrap();
    let now = now();
    let offset = tz.get_offset_utc(&now);
    let rotate = offset.to_utc().whole_hours() as i16;

    let hours = hours_in_utc.iter().map(|h| {
        let adjusted = (*h as i16 + rotate).rem_euclid(NUM_HOURS_PER_WEEK as i16);
        adjusted as usize
    });

    let mut result = Vec::with_capacity(hours_in_utc.len());
    for hour in hours {
        let day = hour / 24;
        let hour_in_day = hour % 24;

        let hour_display = if hour_in_day > 12 {
            let modded = hour_in_day - 12;
            format!("{modded} PM")
        } else if hour_in_day == 0 {
            "12 AM".to_string()
        } else if hour_in_day == 12 {
            "12 PM".to_string()
        } else {
            format!("{hour_in_day} AM")
        };

        let day_names = [
            "Monday", "Tuesday", "Wedesday", "Thursday", "Friday", "Saturday", "Sunday",
        ];
        let day_display = day_names[day];
        result.push(format!("{day_display} at {hour_display}"))
    }

    result
}

fn add_random_day_availability<R: Rng>(buffer: &mut String, rng: &mut R) {
    buffer.push_str("0000000");

    const BLOCKS: [&str; 2] = ["1111", "0000"];
    for _ in 0..4 {
        let block = BLOCKS.choose(rng).unwrap();
        buffer.push_str(block);
    }

    buffer.push('0')
}

fn random_week_availability<R: Rng>(rng: &mut R) -> String {
    let mut week_availability = String::with_capacity(NUM_HOURS_PER_WEEK);
    for _ in 0..NUM_DAYS_PER_WEEK {
        add_random_day_availability(&mut week_availability, rng);
    }

    week_availability
}

fn random_students(count: usize, seed: Option<u64>) -> (Vec<Student>, u64) {
    let timezones = timezones();
    let seed = match seed {
        Some(s) => s,
        None => thread_rng().next_u64(),
    };

    let mut rng = SmallRng::seed_from_u64(seed);

    (
        (0..count)
            .into_iter()
            .map(|i| {
                let name = format!("Student {i}");
                let timezone = timezones.choose(&mut rng).unwrap();
                let availability = random_week_availability(&mut rng);

                Student::new(&name, timezone, &availability).unwrap()
            })
            .collect(),
        seed,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn groups_no_students() {
        assert_eq!(0, create_groups(&[], 3).len())
    }

    #[test]
    fn groups_no_size() {
        let students = [String::from(
            "ZGZzZGZzfEFmcmljYS9BbGdpZXJzfDE5MjB8MjE0NzQ4Mzc2OHw3fDB8MHww=",
        )];
        assert_eq!(0, create_groups(&students, 0).len())
    }

    #[test]
    fn groups_single_student() {
        assert_eq!(
            1,
            create_groups(
                &[String::from(
                    "ZGZzZGZzfEFmcmljYS9BbGdpZXJzfDE5MjB8MjE0NzQ4Mzc2OHw3fDB8MHww"
                )],
                3
            )
            .len()
        )
    }

    #[test]
    fn pretty_hours_negative_offset() {
        let hours = [5, 6, 7, 8];
        let tz = "America/Los_Angeles";

        let result = pretty_hours(&hours, tz);
        let expected = [
            "Sunday at 10 PM".to_string(),
            "Sunday at 11 PM".to_string(),
            "Monday at 12 AM".to_string(),
            "Monday at 1 AM".to_string(),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn pretty_hours_positive_offset() {
        let hours = [5, 6, 7, 8];
        let tz = "Asia/Hovd";

        let result = pretty_hours(&hours, tz);
        let expected = [
            "Monday at 12 PM".to_string(),
            "Monday at 1 PM".to_string(),
            "Monday at 2 PM".to_string(),
            "Monday at 3 PM".to_string(),
        ];
        assert_eq!(result, expected);
    }
}
