use super::types::Num;

pub fn mode_from_counts(counts: &[usize]) -> Num {
    counts.iter().enumerate().max_by_key(|&(_, &count)| count).unwrap().0 as Num + 1
}

pub fn top_two_modes_from_counts(counts: &[usize]) -> (Num, Num) {
    let (mut first_index, mut second_index) = (0, 0);
    let (mut first, mut second) = (counts[0], 0);

    for (i, &count) in counts.iter().enumerate().skip(1) {
        if count > first {
            second = first;
            second_index = first_index;
            first = count;
            first_index = i;
        } else if count > second {
            second = count;
            second_index = i;
        }
    }

    (first_index as Num + 1, second_index as Num + 1)
}

pub fn anti_modes(counts: &[Num]) -> Vec<Num> {
    let mode_index = mode_from_counts(counts);
    let mode_count = counts[mode_index - 1];

    // Collect min nonzero count
    let mut min_nonzero = usize::MAX;
    let mut nonzero_count = 0;
    let mut mode_count_occurrences = 0;
    for &val in counts.iter().filter(|v| **v > 0) {
        nonzero_count += 1;
        if val < min_nonzero {
            min_nonzero = val;
        }
        if val == mode_count {
            mode_count_occurrences += 1;
        }
    }

    // If we have only one nonzero, then there are no antimodes.
    if nonzero_count <= 1 {
        return vec![];
    }
    
    // If all nonzeroes are modes, then choose the first one to be an antinode so that the simulation can progress.
    if mode_count_occurrences == nonzero_count {
        let first_nonzero_index = counts.iter().position(|&v| v > 0).unwrap();
        return vec![first_nonzero_index + 1];
    }

    // Gather antimodes with one pass
    let mut result = Vec::new();
    for (k, &val) in counts.iter().enumerate() {
        if val == min_nonzero {
            result.push(k + 1);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;

    use crate::rand::roll;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_mode_from_counts() {
        let counts = vec![1, 2, 3, 4, 2, 3, 1, 1];
        let expected = 4;

        let result = mode_from_counts(&counts);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_top_two_modes_from_counts() {
        let counts = vec![1, 2, 3, 4, 2, 3, 1, 1];
        let expected = (4, 3);

        let result = top_two_modes_from_counts(&counts);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_anti_modes() {
        let counts = vec![3, 1, 1, 0, 2, 2, 1];
        let expected = vec![2, 3, 7];

        let result = anti_modes(&counts);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_anti_modes_empty() {
        let counts = vec![0, 0, 10, 0, 0, 0, 0];
        let expected = vec![];

        let result = anti_modes(&counts);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_anti_modes_tied() {
        let counts = vec![0, 0, 10, 10, 0, 0, 0];
        let expected = vec![3];

        let result = anti_modes(&counts);

        assert_eq!(result, expected);
    }

    #[bench]
    fn bench_mode_from_counts(b: &mut test::Bencher) {
        let  size = 1_000;
        let mut counts = Vec::with_capacity(size);
        for _ in 0..size {
            counts.push(roll(20));
        }

        b.iter(|| black_box(mode_from_counts(&counts)));
    }

    #[bench]
    fn bench_top_two_modes_from_counts(b: &mut test::Bencher) {
        let  size = 1_000;
        let mut counts = Vec::with_capacity(size);
        for _ in 0..size {
            counts.push(roll(20));
        }

        b.iter(|| black_box(top_two_modes_from_counts(&counts)));
    }

    #[bench]
    fn bench_anti_modes(b: &mut test::Bencher) {
        let  size = 1_000;
        let mut counts = Vec::with_capacity(size);
        for _ in 0..size {
            counts.push(roll(20));
        }

        b.iter(|| black_box(anti_modes(&counts)));
    }
}