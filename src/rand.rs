use rand::Rng;

use crate::types::Num;

pub fn roll(num_sides: Num) -> Num {
    1 + (get_num() % num_sides)
}

#[cfg(not(test))]
fn get_num() -> Num {
    rand::thread_rng().gen::<Num>()
}

#[cfg(test)]
fn get_num() -> Num {
    TEST_RNG.with_borrow_mut(|r| r.gen::<Num>())
}

#[cfg(test)]
thread_local! {
    static TEST_RNG: std::cell::RefCell<rand::rngs::StdRng> = std::cell::RefCell::new(rand::SeedableRng::seed_from_u64(42));
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
        
        assert_eq!(roll(num_sides), 523);
        assert_eq!(roll(num_sides), 190);
    }
}