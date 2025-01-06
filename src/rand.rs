use rand::Rng;

use crate::types::Num;

pub fn roll(num_sides: Num) -> Num {
    1 + (get_num() % num_sides)
}

fn get_num() -> Num {
    get_rng().gen::<Num>()
}

#[cfg(not(test))]
fn get_rng() -> rand::rngs::ThreadRng {
    rand::thread_rng()
}

#[cfg(test)]
fn get_rng() -> rand::rngs::StdRng {
    use rand::SeedableRng;

    rand::rngs::StdRng::seed_from_u64(42)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_roll_6() {
        let num_sides = 6;
        let result = roll(num_sides);

        assert!(result >= 1 && result <= num_sides);
    }

    #[test]
    fn test_roll_20() {
        let num_sides = 20;
        let result = roll(num_sides);

        assert!(result >= 1 && result <= num_sides);
    }

    #[test]
    fn test_seed() {
        let num_sides = 1000;
        let expected = 523;

        let result = roll(num_sides);

        assert_eq!(result, expected);
    }
}