use super::types::Num;

pub fn mode_from_counts(counts: &[usize]) -> Num {
    counts.iter().enumerate().max_by_key(|&(_, &count)| count).unwrap().0 as Num + 1
}

pub fn top_two_modes_from_counts(counts: &[usize]) -> (Num, Num) {
    let length = counts.len();
    let mut counts = counts.iter().enumerate().collect::<Vec<_>>();
    counts.sort_by_key(|&(_, &count)| count);

    let (first, second) = (counts[length - 1].0 + 1, counts[length - 2].0 + 1);

    (first, second)
}

pub fn anti_modes(counts: &[Num]) -> Vec<Num> {
    let mode = mode_from_counts(counts);
    let mode_count = counts[mode - 1];
    let mut counts = counts.iter().enumerate().collect::<Vec<_>>();

    counts.sort_by_key(|&(_, &count)| count);

    let modes = counts.iter().filter(|(_, count)| **count == mode_count).collect::<Vec<_>>();
    let nonzeroes = counts.iter().filter(|(_, count)| **count != 0).collect::<Vec<_>>();

    // There is only one nonzero count, so there are no antimodes.
    if nonzeroes.len() <= 1 {
        return vec![];
    }

    // If all of the modes are the nonzeroes, then we have to choose one so it will be re-rolled.
    if modes.len() == nonzeroes.len() {
        return vec![nonzeroes[0].0 + 1];
    }

    let anti_modes = nonzeroes.iter().take_while(|&(_, &count)| count == *nonzeroes[0].1)
        .map(|&(index, _)| *index + 1)
        .collect::<Vec<_>>();

    anti_modes
}

#[cfg(test)]
mod tests {
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
        let expected = (4, 6);

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
}