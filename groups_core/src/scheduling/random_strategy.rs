/// This strategy randomly assigns groups, then uses hieristics to score the assignment. The assignment
/// with the highest score is chosen. Heuristics include consecutive number of overlapping hours shared
/// by students in a group.
use super::{Group, NUM_HOURS_PER_WEEK};
use crate::Student;
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Default)]
struct Assignment {
    /// Calculated score indicating goodness of group. Higher is better.
    score: usize,
    /// Size of each group.
    group_size: usize,
    /// Indices representing students in group (first n are the first group, 2nd n are the second group, etc where n is group_size. Last group may be smaller.)
    students: Vec<usize>,
    /// For each group, list of available hours shared by the most group members (1) or all members (multiple). In UTC.
    meet_hours: Vec<Vec<usize>>,
}

impl Assignment {
    fn new(student_size: usize, group_size: usize) -> Self {
        Assignment {
            score: 0,
            group_size,
            students: (0..student_size).collect(),
            meet_hours: vec![],
        }
    }

    fn find_best_grouping(&mut self, students: &[Student]) {
        self.students.shuffle(&mut thread_rng());

        for group in self.students.chunks(self.group_size) {
            let availabilities: Vec<_> = group
                .iter()
                .map(|&i| students[i].availability_array_in_utc())
                .collect();

            let mut num_students_avail_at_hour = vec![0; NUM_HOURS_PER_WEEK];
            for i in 0..NUM_HOURS_PER_WEEK {
                let mut count = 0;
                for a in &availabilities {
                    count += if *a.get(i).unwrap() { 1 } else { 0 };
                }
                num_students_avail_at_hour[i] = count;
            }

            // The group score is either max number of students that can meet at one time if not all can meet at the same
            // time, or if they can meet at the same time the num of consecutive hours they are all availalble * num students.
            // Ths punishes groups where not all students can meet at the same time, and rewards those with multiple consecutive time slots.

            let max_num_students_simultaneously_available =
                *num_students_avail_at_hour.iter().max().unwrap();
            if max_num_students_simultaneously_available < group.len() as u8 {
                // No time slot includes all students. Find all the ones that include the max number of students and use
                // those as the suggested times.

                self.score += max_num_students_simultaneously_available as usize;
                let hours_with_this_many_students: Vec<_> = num_students_avail_at_hour
                    .iter()
                    .enumerate()
                    .filter_map(|(hour, &student_count)| {
                        if student_count == max_num_students_simultaneously_available {
                            Some(hour)
                        } else {
                            None
                        }
                    })
                    .collect();

                self.meet_hours.push(hours_with_this_many_students);
            } else {
                // At least one time slot includes all students. Find the width of the max consecutive time slot that includes all students.
                // The width * height (num students in group) is the score.
                // Use dynamic programming to find the widest time slot.
                let mut length_of_consecutive_avail_slot =
                    vec![0; num_students_avail_at_hour.len() + 1];
                for i in 1..length_of_consecutive_avail_slot.len() {
                    length_of_consecutive_avail_slot[i] = if num_students_avail_at_hour[i - 1]
                        == max_num_students_simultaneously_available
                    {
                        1 + length_of_consecutive_avail_slot[i - 1]
                    } else {
                        1
                    };
                }

                // Cap the max number of consecutive slots for scoring purposes.
                // This helps make it so we don't inflate our score by just forcing more consecutive slots
                // in this group while other groups may have not enough.
                // Also penalize consecutive slots less than this by treating as a single entry slots, to
                // encourage these to get more slots.

                let mut consecutive_slots = *length_of_consecutive_avail_slot.iter().max().unwrap();
                consecutive_slots = consecutive_slots.min(4);
                if consecutive_slots < 4 {
                    consecutive_slots = 1;
                }

                self.score +=
                    consecutive_slots * max_num_students_simultaneously_available as usize;

                let hours_with_this_many_students: Vec<_> = num_students_avail_at_hour
                    .iter()
                    .enumerate()
                    .filter_map(|(hour, &student_count)| {
                        if student_count == max_num_students_simultaneously_available {
                            Some(hour)
                        } else {
                            None
                        }
                    })
                    .collect();

                self.meet_hours.push(hours_with_this_many_students);
            }
        }
    }

    fn groups(&self, students: &[Student]) -> Vec<Group> {
        let mut groups = vec![];
        for (indices, meet_times) in self
            .students
            .chunks(self.group_size)
            .zip(self.meet_hours.iter())
        {
            let mut encoded_students = indices.iter().map(|&i| students[i].encode()).collect_vec();
            encoded_students.sort_unstable(); // To make unit testing easier.

            let group = Group {
                students: encoded_students,
                suggested_meet_times: meet_times.clone(),
            };
            groups.push(group);
        }

        groups.sort_unstable_by_key(|g| g.students[0].to_string()); // To make unit testing easier.
        groups
    }
}

const NUM_ITERATIONS: usize = 100_000;

pub fn random_strategy(students: &[Student], group_size: usize) -> Vec<Group> {
    if students.is_empty() || group_size == 0 {
        return vec![];
    }

    let students = Vec::from(students);

    let mut assignments = Vec::with_capacity(NUM_ITERATIONS);
    for _ in 0..NUM_ITERATIONS {
        assignments.push(Assignment::new(students.len(), group_size))
    }

    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    {
        use rayon::prelude::*;
        assignments.par_iter_mut().for_each(|assignment| {
            assignment.find_best_grouping(&students);
        });
    }

    // Rayon isn't well supported on WASM so do it sequentially there.
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    {
        assignments.iter_mut().for_each(|assignment| {
            assignment.find_best_grouping(&students);
        });
    }

    let best_assignment = assignments.iter().max_by_key(|s| s.score).unwrap();
    best_assignment.groups(&students)
}

#[cfg(test)]
mod tests {
    use crate::scheduling::{random_students, NUM_HOURS_PER_DAY};

    use super::*;

    #[test]
    fn test_random() {
        let students: Vec<_> = vec![
            "VGVzdDF8QWZyaWNhL0FiaWRqYW58MTkyMHwwfDB8MHwwfDA=",
            "VGVzdDN8QWZyaWNhL0FiaWRqYW58MzA3MjB8MHwwfDB8MHww",
            "VGVzdDV8QWZyaWNhL0FiaWRqYW58NDkxNTIwfDB8MHwwfDB8MA==",
            "VGVzdDd8QWZyaWNhL0FiaWRqYW58Nzg2NDMyMHwwfDB8MHwwfDA=",
            // First from above should match with first from here, and so on.
            "VGVzdDJ8QWZyaWNhL0FiaWRqYW58MTkyMHwwfDB8MHwwfDA=",
            "VGVzdDR8QWZyaWNhL0FiaWRqYW58MzA3MjB8MHwwfDB8MHww",
            "VGVzdDZ8QWZyaWNhL0FiaWRqYW58NDkxNTIwfDB8MHwwfDB8MA==",
            "VGVzdDh8QWZyaWNhL0FiaWRqYW58Nzg2NDMyMHwwfDB8MHwwfDA=",
        ]
        .iter()
        .map(|s| Student::from_encoded(s).unwrap())
        .collect();

        let best_grouping = random_strategy(&students, 2);
        assert_eq!(best_grouping.len(), 4); // 4 groups of 2.
        assert_eq!(
            best_grouping,
            vec![
                Group {
                    students: vec![
                        "VGVzdDF8QWZyaWNhL0FiaWRqYW58MTkyMHwwfDB8MHwwfDA=".to_string(),
                        "VGVzdDJ8QWZyaWNhL0FiaWRqYW58MTkyMHwwfDB8MHwwfDA=".to_string()
                    ],
                    suggested_meet_times: vec![7, 8, 9, 10]
                },
                Group {
                    students: vec![
                        "VGVzdDN8QWZyaWNhL0FiaWRqYW58MzA3MjB8MHwwfDB8MHww".to_string(),
                        "VGVzdDR8QWZyaWNhL0FiaWRqYW58MzA3MjB8MHwwfDB8MHww".to_string()
                    ],
                    suggested_meet_times: vec![11, 12, 13, 14]
                },
                Group {
                    students: vec![
                        "VGVzdDV8QWZyaWNhL0FiaWRqYW58NDkxNTIwfDB8MHwwfDB8MA==".to_string(),
                        "VGVzdDZ8QWZyaWNhL0FiaWRqYW58NDkxNTIwfDB8MHwwfDB8MA==".to_string()
                    ],
                    suggested_meet_times: vec![15, 16, 17, 18]
                },
                Group {
                    students: vec![
                        "VGVzdDd8QWZyaWNhL0FiaWRqYW58Nzg2NDMyMHwwfDB8MHwwfDA=".to_string(),
                        "VGVzdDh8QWZyaWNhL0FiaWRqYW58Nzg2NDMyMHwwfDB8MHwwfDA=".to_string()
                    ],
                    suggested_meet_times: vec![19, 20, 21, 22]
                }
            ]
        )
    }

    #[test]
    fn test_large_random() {
        let (students, seed) = random_students(50, None);
        let best_grouping = random_strategy(&students, 5);

        let times = best_grouping
            .iter()
            .map(|g| {
                let mut strings = vec![];
                for hour in &g.suggested_meet_times {
                    let day = hour / NUM_HOURS_PER_DAY;
                    let hour_in_day = hour % NUM_HOURS_PER_DAY;

                    let hour_display = if hour_in_day > 12 {
                        let modded = hour_in_day - 12;
                        format!("{modded} PM")
                    } else {
                        let modded = if hour_in_day == 0 { 12 } else { hour_in_day };
                        format!("{modded} AM")
                    };

                    let day_display = match day {
                        0 => "Monday",
                        1 => "Tuesday",
                        2 => "Wednesday",
                        3 => "Thursday",
                        4 => "Friday",
                        5 => "Saturday",
                        6 => "Sunday",
                        d => unreachable!("Invalid day number: {d}"),
                    };
                    strings.push(format!("{day_display} at {hour_display}"));
                }
                strings
            })
            .collect_vec();

        let codes: Vec<_> = best_grouping
            .into_iter()
            .map(|g| g.students)
            .flatten()
            .collect();

        println!("Seed: {seed}");
        println!("{:#?}\n\n{:?}", times, codes);
    }
}
