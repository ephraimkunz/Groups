use crate::constants::NUM_HOURS_PER_WEEK;
use crate::scheduling::Group;
use crate::student::Student;

/// Based on the methodology described in https://www.researchgate.net/publication/258239070_Design_and_validation_of_a_web-based_system_for_assigning_members_to_teams_using_instructor-specified_criteria
// Algo:
// 1. Randomly assign students to teams of size n.
// 2. Calculate question and complicance scores.
// 3. Iteratively change team assignments to maximize the minimum compliance score of the set of teams.
pub fn run(students: &[Student], group_size: usize) -> Vec<Group> {
    for _ in 0..50 {
        // 1. Random assignments to teams of group_size.

        // 2. 
    }
}

/// Equation (3) from paper (page 9).
/// This heuristic returns a value on the interval [0, 1], where the value of zero indicates
/// complete heterogeneity (undesirable: the entire team never is available to meet at the same time) and
/// a value of one indicates adequate homogeneity (desirable: the entire team has at least h hours to meet in common).
fn team_sched_score(team: &[Student]) -> f64 {
    // h is the number of compatible hours beyond which the developers deemed further compatibility unnecessary (h = 40 in Team-Maker Version 1).
    #[allow(non_upper_case_globals)]
    const h: f64 = 40.0;

    // H is the number of blocks of time in a week (H 5 119 in Team-Maker Version 1)
    const H: usize = NUM_HOURS_PER_WEEK;

    let inner_sum: u32 = (0..H)
        .into_iter()
        .map(|i /* i == time block */| {
            let r = team.iter().map(|s| {
                // Invert this because the formula uses knowledge of when students are unavailable and we have knowledge of when they are available.
                if *s.availability_array_in_utc().get(i).unwrap() {
                    0
                } else {
                    1
                }
            });
            let ored = r.fold(0, |accum, elem| accum | elem);
            1 - ored
        })
        .sum();

    f64::min(1.0 / h * inner_sum as f64, 1.0)
}

#[cfg(test)]
mod tests {
    use assert_approx_eq::assert_approx_eq;
    use time_tz::{timezones, TimeZone};

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
        let actual = team_sched_score(&team);
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
        let actual = team_sched_score(&team);
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
        let actual = team_sched_score(&team);
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
        let actual = team_sched_score(&team);

        // This sample team has 34 time blocks with everyone available. In this case the summation in (3) returns a value of 34,
        // and the score s_sch is given by 34/40 = 0.85, a number close to 1, indicating schedule compatibility.
        assert_approx_eq!(actual, 0.85);
    }
}