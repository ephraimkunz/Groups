use crate::timezones;

use super::Student;
use rand::{prelude::SmallRng, seq::SliceRandom};
use rand::{thread_rng, Rng, RngCore, SeedableRng};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use self::random_strategy::random_strategy;

mod random_strategy;

pub const NUM_DAYS_PER_WEEK: usize = 7;
pub const NUM_HOURS_PER_DAY: usize = 24;
pub const NUM_HOURS_PER_WEEK: usize = NUM_HOURS_PER_DAY * NUM_DAYS_PER_WEEK;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Group {
    students: Vec<String>,
    suggested_meet_times: Vec<usize>,
}

#[wasm_bindgen]
pub fn create_groups_wasm(students: &JsValue, group_size: usize) -> JsValue {
    let student_strings: Vec<String> = students.into_serde().unwrap();
    let groups = create_groups(&student_strings, group_size);
    JsValue::from_serde(&groups).unwrap()
}

pub fn create_groups(students_encoded: &[String], group_size: usize) -> Vec<Group> {
    let students: Vec<Student> = students_encoded
        .iter()
        .filter_map(|s| Student::from_encoded(s))
        .collect();
    random_strategy(&students, group_size)
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
            "VGVzdHxBZnJpY2EvR2Fib3JvbmV8NTE1Mzk2MDc1NTIwfDA=",
        )];
        assert_eq!(0, create_groups(&students, 0).len())
    }

    #[test]
    fn groups_single_student() {
        assert_eq!(
            1,
            create_groups(
                &[String::from("VGVzdHxBZnJpY2EvS2lnYWxpfDMyMjEyMjU0NzIwfDA=")],
                3
            )
            .len()
        )
    }
}
