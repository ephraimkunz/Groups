use super::Group;
use crate::Student;
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::*;

struct AssignmentScore {
    score: u32,
    sorted_students: Vec<Student>,
    sorted_meet_time: Vec<usize>,
}

const NUM_ITERATIONS: usize = 100_000;

pub fn random_strategy(students: &[Student], group_size: usize) -> Vec<Group> {
    if students.is_empty() || group_size == 0 {
        return vec![];
    }

    let initial_students = Vec::from(students);
    let mut scores = Vec::with_capacity(NUM_ITERATIONS);
    for _ in 0..NUM_ITERATIONS {
        scores.push(AssignmentScore {
            score: 0,
            sorted_students: vec![],
            sorted_meet_time: vec![],
        });
    }

    scores.par_iter_mut().for_each(|score| {
        let mut list = initial_students.clone();
        list.shuffle(&mut thread_rng());

        let mut assignment_score = 0;
        let mut assignment_hours = vec![];

        for group in list.chunks(group_size) {
            let mut best_num_students_available_now = 0;
            let mut best_group_time = 0;

            let availabilities: Vec<String> = group
                .iter()
                .filter_map(|s| s.availability_in_timezone("GMT"))
                .collect();

            let iters: Vec<_> = availabilities
                .iter()
                .map(|a| a.chars().map(|c| c.to_digit(10).unwrap()))
                .collect();

            // Find the best hour for this group to meet.
            for (hour, avail) in Multizip(iters).enumerate() {
                let num_students_available_now: u32 = avail.iter().sum();
                if num_students_available_now > best_num_students_available_now {
                    best_num_students_available_now = num_students_available_now;
                    best_group_time = hour;

                    // If all students available now, we won't do better at a different time.
                    if num_students_available_now as usize == avail.len() {
                        break;
                    }
                }
            }

            assignment_score += best_num_students_available_now;
            assignment_hours.push(best_group_time)
        }

        if assignment_score > score.score {
            score.score = assignment_score;
            score.sorted_students = list;
            score.sorted_meet_time = assignment_hours;
        }
    });

    let best_score = scores.iter().max_by_key(|s| s.score).unwrap();

    println!("Best assignment score: {}", best_score.score);

    let mut groups = vec![];
    for (students, &meet_hour) in best_score
        .sorted_students
        .chunks(group_size)
        .zip(best_score.sorted_meet_time.iter())
    {
        let mut encoded_students = students.iter().map(|s| s.encode()).collect_vec();
        encoded_students.sort_unstable(); // To make unit testing easier.

        let group = Group {
            students: encoded_students,
            suggested_meet_time: meet_hour,
        };
        groups.push(group);
    }

    groups.sort_unstable_by_key(|g| g.students[0].to_string()); // To make unit testing easier.
    groups
}

struct Multizip<T>(Vec<T>);

impl<T> Iterator for Multizip<T>
where
    T: Iterator,
{
    type Item = Vec<T::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.iter_mut().map(Iterator::next).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::scheduling::{random_students, NUM_HOURS_PER_DAY};

    use super::*;

    #[test]
    fn test_random() {
        let students: Vec<_> = vec![
            "MXxBZnJpY2EvQWJpZGphbnwxOTIwfDA=",
            "MnxBZnJpY2EvQWJpZGphbnwzMDcyMHww",
            "M3xBZnJpY2EvQWJpZGphbnw0OTE1MjB8MA==",
            "NHxBZnJpY2EvQWJpZGphbnw3ODY0MzIwfDA=",
            // First from above should match with first from here, and so on.
            "NXxBZnJpY2EvQWJpZGphbnwxOTIwfDA=",
            "NnxBZnJpY2EvQWJpZGphbnwzMDcyMHww",
            "N3xBZnJpY2EvQWJpZGphbnw0OTE1MjB8MA==",
            "OHxBZnJpY2EvQWJpZGphbnw3ODY0MzIwfDA=",
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
                        "M3xBZnJpY2EvQWJpZGphbnw0OTE1MjB8MA==".to_string(),
                        "N3xBZnJpY2EvQWJpZGphbnw0OTE1MjB8MA==".to_string(),
                    ],
                    suggested_meet_time: 15,
                },
                Group {
                    students: vec![
                        "MXxBZnJpY2EvQWJpZGphbnwxOTIwfDA=".to_string(),
                        "NXxBZnJpY2EvQWJpZGphbnwxOTIwfDA=".to_string(),
                    ],
                    suggested_meet_time: 7,
                },
                Group {
                    students: vec![
                        "MnxBZnJpY2EvQWJpZGphbnwzMDcyMHww".to_string(),
                        "NnxBZnJpY2EvQWJpZGphbnwzMDcyMHww".to_string(),
                    ],
                    suggested_meet_time: 11,
                },
                Group {
                    students: vec![
                        "NHxBZnJpY2EvQWJpZGphbnw3ODY0MzIwfDA=".to_string(),
                        "OHxBZnJpY2EvQWJpZGphbnw3ODY0MzIwfDA=".to_string(),
                    ],
                    suggested_meet_time: 19,
                }
            ]
        )
    }

    #[test]
    fn test_large_random() {
        let students = random_students(50);
        let best_grouping = random_strategy(&students, 5);

        let times = best_grouping
            .iter()
            .map(|g| {
                let hour = g.suggested_meet_time;
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
                format!("{day_display} at {hour_display}")
            })
            .collect_vec();

        let codes: Vec<_> = best_grouping
            .into_iter()
            .map(|g| g.students)
            .flatten()
            .collect();
        println!("{:#?}\n\n{:?}", times, codes);
    }
}
