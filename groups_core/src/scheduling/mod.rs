use crate::constants::NUM_HOURS_PER_WEEK;
use crate::student::Student;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time_tz::{timezones, Offset, TimeZone};
use wasm_bindgen::prelude::*;

use self::hillclimbing_strategy::HillClimbingStrategy;

mod hillclimbing_strategy;
mod min_max_strategy;

type DefaultStrategy = HillClimbingStrategy;

/// A trait representing a specific scheduler for groups.
pub trait SchedulingStrategy {
    fn run(students: &[Student], group_size: usize) -> Vec<Group>;
}

/// A group of students, along with suggested meet times.
#[derive(Debug, PartialEq, Eq)]
pub struct Group {
    /// Vector of encoded Student.
    pub students: Vec<String>,

    /// Vector of hours in the week when the most number of students in the group are all available (in UTC).
    /// Not guaranteed that all students in this group are available at these times.
    /// 0 = Monday at 12 AM, 1 = Monday at 1 AM, etc.
    pub suggested_meet_times: Vec<usize>,
}

impl Group {
    fn percent_at_suggested_times(&self) -> f64 {
        let students: Vec<Student> = self
            .students
            .iter()
            .filter_map(|s| Student::from_encoded(s))
            .collect();

        let group: Vec<usize> = (0..self.students.len()).collect();

        let avail = num_students_available_at_hour(&group, &students);
        avail[self.suggested_meet_times[0]] as f64 / self.students.len() as f64
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DisplayGroup {
    students: Vec<String>,
    suggested_meet_times: Vec<String>,
    percent_at_suggested_times: f64,
}

#[wasm_bindgen]
/// Same as `create_groups`, but suitable for calling from WASM because it takes and returns JSValues.
/// `students` is a Javascript array of encoded Student (strings).
/// `output_timezone` is the timezone which will be used when generating the `suggested_meet_times` array in
/// each output group.
/// Returns a Javascript array of JSON objects representing groups.
pub fn create_groups_wasm(
    students: JsValue,
    group_size: usize,
    output_timezone: String,
) -> JsValue {
    let student_strings: Vec<String> = serde_wasm_bindgen::from_value(students).unwrap();

    //  For WASM, the default strategy is the MinMax strategy.
    let groups = create_groups_default_strategy(&student_strings, group_size);
    let display = display_groups(&groups, &output_timezone);
    serde_wasm_bindgen::to_value(&display).unwrap()
}

/// Returns the best grouping of students, given the total students in the class and
/// the maximum size of a group.
pub fn create_groups<S: SchedulingStrategy>(
    students_encoded: &[String],
    group_size: usize,
) -> Vec<Group> {
    let students: Vec<Student> = students_encoded
        .iter()
        .filter_map(|s| Student::from_encoded(s))
        .collect();
    S::run(&students, group_size)
}

fn create_groups_default_strategy(students_encoded: &[String], group_size: usize) -> Vec<Group> {
    create_groups::<DefaultStrategy>(students_encoded, group_size)
}

fn display_groups(groups: &[Group], timezone: &str) -> Vec<DisplayGroup> {
    groups
        .iter()
        .map(|g| DisplayGroup {
            students: g.students.clone(),
            suggested_meet_times: pretty_hours(&g.suggested_meet_times, timezone),
            percent_at_suggested_times: g.percent_at_suggested_times(),
        })
        .collect()
}

/// Returns a count for each hour in UTC in a week, where the count is the number of students available at that hour.
fn num_students_available_at_hour(
    group: &[usize],
    students: &[Student],
) -> [u32; NUM_HOURS_PER_WEEK] {
    let availabilities: Vec<_> = group
        .iter()
        .map(|&i| students[i].availability_array_in_utc())
        .collect();

    let mut result = [0; NUM_HOURS_PER_WEEK];
    for (i, num_avail_at_hour_slot) in result.iter_mut().enumerate() {
        let mut count = 0;
        for a in &availabilities {
            count += if *a.get(i).unwrap() { 1 } else { 0 };
        }
        *num_avail_at_hour_slot = count;
    }

    result
}

fn hours_with_n_or_more_available_students(
    n: u32,
    availabilities: [u32; NUM_HOURS_PER_WEEK],
) -> Vec<usize> {
    availabilities
        .iter()
        .enumerate()
        .filter_map(
            |(hour, &student_count)| {
                if student_count >= n {
                    Some(hour)
                } else {
                    None
                }
            },
        )
        .collect()
}

fn pretty_hours(hours_in_utc: &[usize], output_timezone: &str) -> Vec<String> {
    let tz = timezones::get_by_name(output_timezone).unwrap();
    let now = OffsetDateTime::now_utc();
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
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
            "Sunday",
        ];
        let day_display = day_names[day];
        result.push(format!("{day_display} at {hour_display}"))
    }

    result
}

#[cfg(test)]
mod tests {
    use super::{min_max_strategy::MinMaxStrategy, *};

    #[test]
    fn groups_no_students() {
        assert_eq!(0, create_groups_default_strategy(&[], 3).len())
    }

    #[test]
    fn groups_no_size() {
        let students = [String::from(
            "ZGZzZGZzfEFmcmljYS9BbGdpZXJzfDE5MjB8MjE0NzQ4Mzc2OHw3fDB8MHww=",
        )];
        assert_eq!(0, create_groups_default_strategy(&students, 0).len())
    }

    #[test]
    fn groups_single_student() {
        assert_eq!(
            1,
            create_groups_default_strategy(
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

    #[test]
    fn no_crash_if_unfull_team() {
        let students: Vec<String> = ["TG91aXMgQ2hpbHVtYmF8QWZyaWNhL0pvaGFubmVzYnVyZ3wwfDB8MjAxMzI2NjA0MHwwfDB8MA==",
        "SXbDoW4gTWF4aW1pbGlhbm8gTW9udGUgfEFtZXJpY2EvQnVlbm9zX0FpcmVzfDc4NjQzMjB8MzA3MjB8MjAxMzI2NjA0MHwyMTU1MzQ3OTY4fDd8MA==",
        "VmxhZGlzbG92YXMgS2FyYWxpdXN8RXVyb3BlL1ZpbG5pdXN8Nzg2NDMyMHwzMDcyMHwyMDEzMjY2MDQwfDc4NjQzMjB8MzI3NjB8MA==",
        "QW1hbmRhIENvbGV8QW1lcmljYS9EZW52ZXJ8MHwxMjU4NTk4NDB8MjAxMzc1NzU2MHwwfDB8MA==",
        "THkgRGFuZ3xBbWVyaWNhL0RlbnZlcnw0OTE1MjB8MjE0NzQ4NTU2OHw3fDB8MHww",
        "VmlvbGEgRm9uZ3xBbWVyaWNhL0xvc19BbmdlbGVzfDIxNDgwMDU4ODh8MjI3MzMxNDY5NXwxMjd8ODM4ODQ4MHwwfDA=",
        "RW1tYW51ZWwgREsgRG9sb3xBZnJpY2EvQWNjcmF8Nzg2NDMyMHwzMDcyMHwyMDEzMjY2MDQwfDc4NjQzMjB8MzA3MjB8MA==", 
        "TW9uaXF1ZSBSb2JlcnRzfEFtZXJpY2EvRGVudmVyfDc4NjQzMjB8MzA3MjB8MTI1ODI5MTIwfDB8MHww",
       "U3RldmVuIEZvc3RlcnxBbWVyaWNhL0RlbnZlcnwwfDMwNzIwfDIwMTMyNjYwNDB8MHwwfDA="].into_iter().map(String::from).collect();

        let groups = create_groups::<MinMaxStrategy>(&students, 5);
        assert_eq!(2, groups.len())
    }
}
