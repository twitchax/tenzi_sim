use std::{simd::{cmp::SimdPartialEq, Simd}, sync::OnceLock};

use super::types::Num;

const LANES: usize = 64;
static CANDIDATES: OnceLock<Vec<Simd<usize, LANES>>> = OnceLock::new();

pub fn get_mode(dice: &mut [Num], num_sides: Num) -> Num {
    #[cfg(not(feature = "simd"))]
    let mode = mutated_serial(dice);
    #[cfg(feature = "simd")]
    let mode = simd(dice, num_sides);

    mode
}

fn serial(dice: &[Num]) -> Num {
    *dice.iter().max_by_key(|&x| dice.iter().filter(|&y| y == x).count()).unwrap()
}

fn mutated_serial(dice: &mut [Num]) -> Num {
    // Sort.
    dice.sort_unstable();

    // Find the mode.
    let mut mode = dice[0];
    let mut mode_count = 1;
    let mut current = dice[0];
    let mut current_count = 1;

    for die in dice.iter().skip(1) {
        if *die == current {
            current_count += 1;
        } else {
            if current_count > mode_count {
                mode = current;
                mode_count = current_count;
            }
            current = *die;
            current_count = 1;
        }
    }

    if current_count > mode_count {
        mode = current;
    }

    mode
}

fn simd(dice: &[Num], num_sides: Num) -> Num {
    let candidates = CANDIDATES.get_or_init(|| candidates(num_sides));

    let target = std::simd::usizex64::load_or_default(dice);

    let counts = candidates.iter().map(|candidate| {
        let mask = target.simd_eq(*candidate);
        mask.to_bitmask().count_ones()
    }).collect::<Vec<_>>();

    let mode = counts.iter().enumerate().max_by_key(|&(_, &count)| count).unwrap().0 + 1;

    mode
}

fn candidates(num_sides: Num) -> Vec<Simd<usize, 64>> {
    (1..num_sides + 1).map(|k| {
        std::simd::usizex64::splat(k)
    }).collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;

    use super::*;
    use pretty_assertions::assert_eq;
    use test::Bencher;

    #[test]
    fn test_serial() {
        let dice = vec![1, 1, 1, 1, 2, 3, 3, 3, 3, 3, 3, 4, 5, 6, 6, 6, 6, 6];
        let expected = 3;

        let result = serial(&dice);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_mutated_serial() {
        let mut dice = vec![1, 1, 1, 1, 2, 3, 3, 3, 3, 3, 3, 4, 5, 6, 6, 6, 6, 6];
        let expected = 3;

        let result = mutated_serial(&mut dice);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_simd() {
        let dice = vec![1, 1, 1, 1, 2, 3, 3, 3, 3, 3, 3, 4, 5, 6, 6, 6, 6, 6];
        let num_sides = 6;
        let expected = 3;

        let result = simd(&dice, num_sides);

        assert_eq!(result, expected);
    }

    #[bench]
    fn bench_serial(b: &mut Bencher) {
        let dice = vec![1, 2, 3, 3, 3, 3, 3, 3, 4, 5, 6, 1, 1, 1,  6, 6, 6, 6];

        b.iter(|| black_box(mutated_serial(dice.clone().as_mut_slice())));
    }

    #[bench]
    fn bench_mutated_serial(b: &mut Bencher) {
        let dice = vec![1, 2, 3, 3, 3, 3, 3, 3, 4, 5, 6, 1, 1, 1,  6, 6, 6, 6];

        b.iter(|| black_box(mutated_serial(dice.clone().as_mut_slice())));
    }

    #[bench]
    fn bench_simd(b: &mut Bencher) {
        let dice = vec![1, 2, 3, 3, 3, 3, 3, 3, 4, 5, 6, 1, 1, 1,  6, 6, 6, 6];
        let num_sides = 6;

        b.iter(|| black_box(simd(&dice, num_sides)));
    }
}