use crate::student::Student;
/// This strategy randomly assigns a number of starting assignments, then uses hill climbing to find local maxima of
/// each starting assignment by swapping students between groups in that assignment. The assignment
/// with the highest score is chosen. Scoring is based on the number of consecutive overlapping hours shared
/// by students in a group.
use crate::{constants::NUM_HOURS_PER_WEEK, scheduling::Group};
use itertools::Itertools;
use num::Integer;
use plotters::prelude::*;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

use super::SchedulingStrategy;

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

    /// For plotting the convergence over time (makes it easier to tune parameters)
    score_history: Vec<usize>,
}

impl Assignment {
    fn new(student_size: usize, group_size: usize) -> Self {
        Assignment {
            score: 0,
            group_size,
            students: (0..student_size).collect(),
            meet_hours: vec![],
            score_history: vec![],
        }
    }

    fn score_assignment_and_get_meet_hours(
        groups: &[usize],
        group_size: usize,
        students: &[Student],
    ) -> (usize, Vec<Vec<usize>>) {
        let mut score = 0;
        let mut meet_hours = Vec::with_capacity(Integer::div_ceil(&groups.len(), &group_size));

        for group in groups.chunks(group_size) {
            let availabilities: Vec<_> = group
                .iter()
                .map(|&i| students[i].availability_array_in_utc())
                .collect();

            let mut num_students_avail_at_hour = vec![0; NUM_HOURS_PER_WEEK];
            for (i, num_avail_at_hour_slot) in num_students_avail_at_hour.iter_mut().enumerate() {
                let mut count = 0;
                for a in &availabilities {
                    count += if *a.get(i).unwrap() { 1 } else { 0 };
                }
                *num_avail_at_hour_slot = count;
            }

            // The group score is either max number of students that can meet at one time if not all can meet at the same
            // time, or if they can meet at the same time the num of consecutive hours they are all availalble * num students.
            // Ths punishes groups where not all students can meet at the same time, and rewards those with multiple consecutive time slots.

            let max_num_students_simultaneously_available =
                *num_students_avail_at_hour.iter().max().unwrap();
            if max_num_students_simultaneously_available < group.len() as u8 {
                // No time slot includes all students. Find all the ones that include the max number of students and use
                // those as the suggested times.

                score += max_num_students_simultaneously_available as usize;
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

                meet_hours.push(hours_with_this_many_students);
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
                const MAX_REWARDED_CONSECUTIVE_SLOTS: usize = 4;

                let mut consecutive_slots = *length_of_consecutive_avail_slot.iter().max().unwrap();
                consecutive_slots = consecutive_slots.min(MAX_REWARDED_CONSECUTIVE_SLOTS);
                if consecutive_slots < MAX_REWARDED_CONSECUTIVE_SLOTS {
                    consecutive_slots = 1;
                }

                score += consecutive_slots * max_num_students_simultaneously_available as usize;

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

                meet_hours.push(hours_with_this_many_students);
            }
        }

        (score, meet_hours)
    }

    fn find_best_grouping(&mut self, students: &[Student]) {
        // Start with a randomly chosen group assignment.
        self.students.shuffle(&mut thread_rng());
        (self.score, self.meet_hours) =
            Self::score_assignment_and_get_meet_hours(&self.students, self.group_size, students);
        self.score_history.push(self.score);

        // Then hillclimb. Try a maximum of this number of neighbor solutions for any given assignment before
        // giving up if we can't find a better solutions.
        const NUM_TRIES_FOR_BETTER_NEIGHBOR: usize = 1000;

        let mut iter = 0;
        while iter < NUM_TRIES_FOR_BETTER_NEIGHBOR {
            // Generate a neighbor by randomly swapping 2 elements.
            let mut groups = self.students.clone();
            let a = thread_rng().gen_range(0..groups.len());
            let b = thread_rng().gen_range(0..groups.len());
            groups.swap(a, b);

            // See if it scores better. If so, keep it. Otherwise, generate another neighbor.
            let (score, meet_hours) =
                Self::score_assignment_and_get_meet_hours(&groups, self.group_size, students);
            if score > self.score {
                self.students.swap(a, b);
                self.score = score;
                self.score_history.push(score);
                self.meet_hours = meet_hours;
                iter = 0;
            } else {
                iter += 1;
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

pub struct HillClimbingStrategy;

impl SchedulingStrategy for HillClimbingStrategy {
    fn run(students: &[Student], group_size: usize) -> Vec<Group> {
        if students.is_empty() || group_size == 0 {
            return vec![];
        }

        let students = Vec::from(students);

        // When hillclimbing, we want multiple starting points to try to avoid getting stuck in a local minima.
        const NUM_STARTING_POINTS: usize = 100;
        let mut assignments = Vec::with_capacity(NUM_STARTING_POINTS);
        for _ in 0..NUM_STARTING_POINTS {
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

        // Plotting
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        plot_convergence(&assignments);

        let best_assignment = assignments.iter().max_by_key(|s| s.score).unwrap();
        best_assignment.groups(&students)
    }
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
fn plot_convergence(assignments: &[Assignment]) {
    const NUM_LINES_TO_PLOT: usize = 10;

    let max_iterations = assignments
        .iter()
        .take(NUM_LINES_TO_PLOT)
        .max_by_key(|i| i.score_history.len())
        .unwrap()
        .score_history
        .len();

    let max_score = assignments
        .iter()
        .take(NUM_LINES_TO_PLOT)
        .max_by_key(|s| s.score)
        .unwrap()
        .score;

    let root = BitMapBackend::new("out.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Group assignment hill-climbing convergence",
            ("sans-serif", (5).percent_height()),
        )
        .set_label_area_size(LabelAreaPosition::Left, (8).percent())
        .set_label_area_size(LabelAreaPosition::Bottom, (4).percent())
        .margin((1).percent())
        .build_cartesian_2d(0..max_iterations, 0..(max_score + 10))
        .unwrap();

    chart
        .configure_mesh()
        .x_desc("Iteration")
        .y_desc("Score")
        .draw()
        .unwrap();

    for (idx, assignment) in assignments.iter().take(NUM_LINES_TO_PLOT).enumerate() {
        let color = Palette99::pick(idx).mix(0.9);
        chart
            .draw_series(LineSeries::new(
                assignment
                    .score_history
                    .iter()
                    .enumerate()
                    .map(|(i, &score)| (i, score)),
                color.stroke_width(3),
            ))
            .unwrap();
    }
    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::NUM_HOURS_PER_DAY;
    use crate::random::random_students;

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

        let best_grouping = HillClimbingStrategy::run(&students, 2);
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
        let best_grouping = HillClimbingStrategy::run(&students, 5);

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
