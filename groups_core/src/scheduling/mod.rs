use crate::constants::NUM_HOURS_PER_WEEK;
use crate::student::Student;
use serde::{Deserialize, Serialize};
use time_tz::{timezones, Offset, TimeZone};
use wasm_bindgen::prelude::*;

mod hillclimbing_strategy;
mod min_max_strategy;

/// A group of students, along with suggested meet times.
#[derive(Debug, PartialEq)]
pub struct Group {
    /// Vector of encoded Student.
    pub students: Vec<String>,

    /// Vector of hours in the week when students are all available (in UTC).
    /// 0 = Monday at 12 AM, 1 = Monday at 1 AM, etc.
    pub suggested_meet_times: Vec<usize>,
}

#[derive(Serialize, Deserialize)]
struct DisplayGroup {
    students: Vec<String>,
    suggested_meet_times: Vec<String>,
}

#[wasm_bindgen]
/// Same as `create_groups`, but suitable for calling from WASM because it takes and returns JSValues.
/// `students` is a Javascript array of encoded Student (strings).
/// `output_timezone` is the timezone which will be used when generating the `suggested_meet_times` array in
/// each output group.
/// Returns a Javascript array of JSON objects representing groups.
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

/// Returns the best grouping of students, given the total students in the class and
/// the maximum size of a group.
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
    let now = crate::now();
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
