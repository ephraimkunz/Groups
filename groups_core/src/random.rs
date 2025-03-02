use crate::constants::{NUM_DAYS_PER_WEEK, NUM_HOURS_PER_WEEK};
use crate::{student::Student, timezones::timezones};
use fake::Fake;
use rand::{Rng, RngCore, SeedableRng, rng};
use rand::{prelude::StdRng, prelude::*};

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

/// Generate `count` random student, optionally seeding the RNG with `seed`.
/// Returns vec of students and the seed used.
pub fn random_students(count: usize, seed: Option<u64>) -> (Vec<Student>, u64) {
    let timezones = timezones();
    let seed = match seed {
        Some(s) => s,
        None => rng().next_u64(),
    };

    let mut rng = StdRng::seed_from_u64(seed);

    (
        (0..count)
            .map(|_| {
                use fake::faker::name::en::Name;
                let name: String = Name().fake();
                let timezone = timezones.choose(&mut rng).unwrap();
                let availability = random_week_availability(&mut rng);

                Student::new(&name, timezone, &availability).unwrap()
            })
            .collect(),
        seed,
    )
}
