use crate::scheduling::{NUM_DAYS_PER_WEEK, NUM_HOURS_PER_WEEK};
use crate::{timezones, Student};
use fake::Fake;
use rand::{prelude::SmallRng, seq::SliceRandom};
use rand::{thread_rng, Rng, RngCore, SeedableRng};

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

pub fn random_students(count: usize, seed: Option<u64>) -> (Vec<Student>, u64) {
    let timezones = timezones();
    let seed = match seed {
        Some(s) => s,
        None => thread_rng().next_u64(),
    };

    let mut rng = SmallRng::seed_from_u64(seed);

    (
        (0..count)
            .into_iter()
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
