use itertools::Itertools;
use num::Integer;
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::scheduling::Group;
use crate::student::Student;

use super::SchedulingStrategy;

pub struct MinMaxStrategy;

impl SchedulingStrategy for MinMaxStrategy {
    /// Based on the methodology described in https://www.researchgate.net/publication/258239070_Design_and_validation_of_a_web-based_system_for_assigning_members_to_teams_using_instructor-specified_criteria
    // Algo:
    // 1. Randomly assign students to teams of size n.
    // 2. Calculate question and complicance scores.
    // 3. Iteratively change team assignments to maximize the minimum compliance score of the set of teams.
    fn run(students: &[Student], group_size: usize) -> Vec<Group> {
        if students.is_empty() || group_size < 1 {
            return vec![];
        }

        const RANDOM_STARTS: usize = 50;
        const TEAM_SWAP_MAX_PASSES: usize = 20;

        let num_teams = Integer::div_ceil(&students.len(), &group_size);

        // Start out with the given array. Hopefully we'll generate something better.
        let mut best_assignment = (0..students.len()).collect_vec();
        let mut best_assignment_min_score = best_assignment
            .chunks(group_size)
            .map(|t| team_sched_score(t, students))
            .fold(f64::INFINITY, |a, b| a.min(b));

        for _ in 0..RANDOM_STARTS {
            let mut teams = (0..students.len()).collect_vec();
            teams.shuffle(&mut thread_rng());

            let mut min_score: f64 = 0.0;

            for _ in 0..TEAM_SWAP_MAX_PASSES {
                let mut swap_happened = false;

                for team_a_index in 0..(num_teams - 1) {
                    let team_a_start_index = team_a_index * group_size;
                    let team_a_size = if (team_a_start_index + group_size) <= students.len() {
                        group_size
                    } else {
                        students.len() - (team_a_start_index + group_size)
                    };
                    for team_b_index in (team_a_index + 1)..num_teams {
                        let team_b_start_index = team_b_index * group_size;
                        let team_b_size = if (team_b_start_index + group_size) <= students.len() {
                            group_size
                        } else {
                            students.len() - (team_b_start_index + group_size)
                        };
                        for student_a_index in
                            team_a_start_index..(team_a_start_index + team_a_size)
                        {
                            for student_b_index in
                                team_b_start_index..(team_b_start_index + team_b_size)
                            {
                                let old_team_a_score = team_sched_score(
                                    &teams[team_a_start_index..(team_a_start_index + team_a_size)],
                                    students,
                                );
                                let old_team_b_score = team_sched_score(
                                    &teams[team_b_start_index..(team_b_start_index + team_b_size)],
                                    students,
                                );
                                let old = old_team_a_score.min(old_team_b_score);

                                teams.swap(student_a_index, student_b_index);

                                let new_team_a_score = team_sched_score(
                                    &teams[team_a_start_index..(team_a_start_index + team_a_size)],
                                    students,
                                );
                                let new_team_b_score = team_sched_score(
                                    &teams[team_b_start_index..(team_b_start_index + team_b_size)],
                                    students,
                                );
                                let new = new_team_a_score.min(new_team_b_score);

                                if new > old {
                                    swap_happened = true;
                                    min_score = min_score.max(new);
                                } else {
                                    // If the new teams arrangement is no better than the old, revert swap by swapping again.
                                    teams.swap(student_a_index, student_b_index);
                                }
                            }
                        }
                    }
                }

                if !swap_happened {
                    break;
                }
            }

            if min_score > best_assignment_min_score {
                best_assignment_min_score = min_score;
                best_assignment = teams;
            }
        }

        // Convert best_assignment to Vec<Group>.
        let mut result = Vec::with_capacity(num_teams);
        for team in best_assignment.chunks(group_size) {
            let mut student_ids = team.iter().map(|&i| students[i].encode()).collect_vec();
            student_ids.sort_unstable(); // To make unit testing easier.

            let meet_times = team
                .iter()
                .map(|&i| students[i].availability_array_in_utc())
                .reduce(|accum, item| accum & item)
                .unwrap();
            let suggested_meet_times = meet_times.iter_ones().collect();
            result.push(Group {
                students: student_ids,
                suggested_meet_times,
            });
        }

        result.sort_unstable_by_key(|g| g.students[0].to_string()); // To make unit testing easier.

        result
    }
}

/// Equation (3) from paper (page 9).
/// This heuristic returns a value on the interval [0, 1], where the value of zero indicates
/// complete heterogeneity (undesirable: the entire team never is available to meet at the same time) and
/// a value of one indicates adequate homogeneity (desirable: the entire team has at least h hours to meet in common).
fn team_sched_score(team: &[usize], students: &[Student]) -> f64 {
    // h is the number of compatible hours beyond which the developers deemed further compatibility unnecessary (h = 40 in Team-Maker Version 1).
    #[allow(non_upper_case_globals)]
    const h: f64 = 40.0;

    let anded = team
        .iter()
        .map(|&s| students[s].availability_array_in_utc())
        .reduce(|accum, item| accum & item)
        .unwrap();

    let sum = anded.count_ones();

    f64::min(1.0 / h * sum as f64, 1.0)
}

#[cfg(test)]
mod tests {
    use assert_approx_eq::assert_approx_eq;
    use time_tz::{timezones, TimeZone};

    use crate::constants::{NUM_HOURS_PER_DAY, NUM_HOURS_PER_WEEK};
    use crate::random::random_students;

    use super::*;

    #[test]
    fn test_team_sched_score_complete_incompatibility() {
        let avail: String = (0..NUM_HOURS_PER_WEEK).map(|_| "0").collect();
        let tz = timezones::db::america::LOS_ANGELES.name();

        let team = vec![
            Student::new("1", tz, &avail).unwrap(),
            Student::new("2", tz, &avail).unwrap(),
            Student::new("3", tz, &avail).unwrap(),
        ];
        let actual = team_sched_score(&[0, 1, 2], &team);
        assert_eq!(actual, 0.0);
    }

    #[test]
    fn test_team_sched_score_complete_compatibility() {
        let avail: String = (0..NUM_HOURS_PER_WEEK).map(|_| "1").collect();
        let tz = timezones::db::america::LOS_ANGELES.name();

        let team = vec![
            Student::new("1", tz, &avail).unwrap(),
            Student::new("2", tz, &avail).unwrap(),
            Student::new("3", tz, &avail).unwrap(),
        ];
        let actual = team_sched_score(&[0, 1, 2], &team);
        assert_eq!(actual, 1.0);
    }

    #[test]
    fn test_team_sched_score_further_compatibility_unnecessary() {
        let h = 40;
        let avail: String = (0..h)
            .map(|_| "1")
            .chain((0..(NUM_HOURS_PER_WEEK - h)).map(|_| "0"))
            .collect();
        let tz = timezones::db::america::LOS_ANGELES.name();

        let team = vec![
            Student::new("1", tz, &avail).unwrap(),
            Student::new("2", tz, &avail).unwrap(),
            Student::new("3", tz, &avail).unwrap(),
        ];
        let actual = team_sched_score(&[0, 1, 2], &team);
        assert_eq!(actual, 1.0);
    }

    #[test]
    fn test_team_sched_score_34_blocks_compatible() {
        const BLOCKS_WITH_ALL_AVAIL: usize = 34;
        let tz = timezones::db::america::LOS_ANGELES.name();

        let team = vec![
            Student::new(
                "always avail",
                tz,
                &(0..NUM_HOURS_PER_WEEK).map(|_| "1").collect::<String>(),
            )
            .unwrap(),
            Student::new(
                "minimal avail",
                tz,
                &(0..BLOCKS_WITH_ALL_AVAIL)
                    .map(|_| "1")
                    .chain((0..(NUM_HOURS_PER_WEEK - BLOCKS_WITH_ALL_AVAIL)).map(|_| "0"))
                    .collect::<String>(),
            )
            .unwrap(),
            Student::new(
                "medium avail",
                tz,
                &(0..(BLOCKS_WITH_ALL_AVAIL + 10))
                    .map(|_| "1")
                    .chain((0..(NUM_HOURS_PER_WEEK - BLOCKS_WITH_ALL_AVAIL - 10)).map(|_| "0"))
                    .collect::<String>(),
            )
            .unwrap(),
            Student::new(
                "medium avail",
                tz,
                &(0..(BLOCKS_WITH_ALL_AVAIL + 20))
                    .map(|_| "1")
                    .chain((0..(NUM_HOURS_PER_WEEK - BLOCKS_WITH_ALL_AVAIL - 20)).map(|_| "0"))
                    .collect::<String>(),
            )
            .unwrap(),
        ];
        let actual = team_sched_score(&[0, 1, 2, 3], &team);

        // This sample team has 34 time blocks with everyone available. In this case the summation in (3) returns a value of 34,
        // and the score s_sch is given by 34/40 = 0.85, a number close to 1, indicating schedule compatibility.
        assert_approx_eq!(actual, 0.85);
    }

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

        let best_grouping = MinMaxStrategy::run(&students, 2);
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
        let best_grouping = MinMaxStrategy::run(&students, 5);

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
