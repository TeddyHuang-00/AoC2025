use std::collections::BTreeSet;

use anyhow::Result;
use util::{Solution, reader::parse_comma_separated_from_file};

type Range = (u64, u64);

struct Puzzle {
    ranges: Vec<Range>,
}

impl Puzzle {
    fn parse_range(input: &str) -> Result<Range> {
        let Some((start, end)) = input.split_once('-') else {
            anyhow::bail!("Invalid range format: {input}");
        };
        let start = start.parse()?;
        let end = end.parse()?;
        Ok((start, end))
    }

    fn new(example: bool) -> Result<Self> {
        let mut ranges = parse_comma_separated_from_file(Self::DAY, example, Self::parse_range)?;
        // Merge overlapping or contiguous ranges
        ranges.sort_unstable();
        let ranges = ranges
            .into_iter()
            .fold(vec![], |mut acc: Vec<Range>, curr: Range| {
                if let Some(last) = acc.last_mut()
                    && curr.0 <= last.1 + 1
                {
                    last.1 = last.1.max(curr.1);
                    return acc;
                }
                acc.push(curr);
                acc
            });
        Ok(Self { ranges })
    }

    /// Find prime factors of a number
    ///
    /// This is a helper function that will be useful for part 2,
    /// where we need to find all repeat patterns for a given length n.
    fn prime_factors(mut n: u32) -> Vec<u32> {
        let mut factors = BTreeSet::new();
        while n.is_multiple_of(2) {
            factors.insert(2);
            n /= 2;
        }
        let mut divisor = 3;
        while divisor * divisor <= n {
            while n.is_multiple_of(divisor) {
                factors.insert(divisor);
                n /= divisor;
            }
            divisor += 2;
        }
        if n > 1 {
            factors.insert(n);
        }

        factors.into_iter().collect()
    }

    /// Calculate the sum of invalid IDs in the given range for IDs using n digits
    /// with a certain repeat pattern.
    ///
    /// For example, for n=6 and repeat=3, the invalid IDs are of the form:
    /// ababab where a,b are digits from 0-9 (with a != 0)
    fn get_sum_invalid_ids(range: Range, n: u32, repeat: u32) -> u64 {
        // The pattern repeats every k = n / repeat digits
        let k = n / repeat;
        // Calculate the lower and upper bounds for n-digit numbers with the given pattern
        let lower = ((k - 1)..n)
            .step_by(k as usize)
            .map(|k| 10u64.pow(k))
            .sum::<u64>();
        let base = lower / 10u64.pow(k - 1);
        let upper = base * (10u64.pow(k) - 1);
        // Get the overlap between the given range and (lower, upper)
        let (start, end) = range;
        let start = start.max(lower);
        let end = end.min(upper);
        // Convert back to the base range
        let start = start.div_ceil(base);
        let end = end / base;
        if start > end {
            return 0;
        }
        // Generate all invalid IDs in the range
        (end - start + 1) * (start + end) / 2 * base
    }
}

impl Solution for Puzzle {
    const DAY: u8 = 2;

    /// For invalid IDs, we can see that they must be in the form of
    /// 11, 22, ..., 99 (base 11)
    /// 1010, 2020, ..., 9999 (base 101)
    /// 100100, 200200, ..., 999999 (base 1001)
    /// and so on.
    ///
    /// We can generalize this to say that for any odd length n,
    /// the invalid IDs are of the form:
    /// k * (10^ceil(n/2) + 1) for k in [10^(n//2), 10^ceil(n/2) - 1]
    ///
    /// We then find the overlap of these ranges with the given ranges
    /// and sum the invalid IDs.
    fn part1(&self) -> String {
        self.ranges
            .iter()
            .map(|&(start, end)| {
                // Determine the min and max number of digits in the range
                let min_n = start.ilog10() + 1;
                let max_n = end.ilog10() + 1;

                (min_n..=max_n)
                    .filter(|n| n % 2 == 0)
                    .map(|n| Self::get_sum_invalid_ids((start, end), n, 2))
                    .sum::<u64>()
            })
            .sum::<u64>()
            .to_string()
    }

    /// For part 2, we need to consider all repeating patterns.
    ///
    /// This can be seen as a direct extension of part 1, where instead of just
    /// considering the pattern where the patterns repeat twice (e.g., 1212 for n=4),
    /// we consider all patterns where the digits repeat k times, for all k that
    /// are factors of n.
    ///
    /// We can further find that only prime factors need to be considered, since
    /// any composite factor can be formed by combining smaller prime factors,
    /// and their contributions have already been counted. For example, for n=8,
    /// the pattern that repeats 4 times (e.g., abcdabcd) can be formed by combining
    /// the patterns that repeat 2 times (e.g., abababab).
    ///
    /// However, we still need to consider the case where all digits are the same
    /// (e.g., 1111, 2222, ..., 9999 for n=4), which is covered by all prime factors.
    /// Therefore, we handle this case separately, by subtracting it out after adding
    /// the contributions from prime factors.
    fn part2(&self) -> String {
        self.ranges
            .iter()
            .map(|&(start, end)| {
                // Determine the min and max number of digits in the range
                let min_n = start.ilog10() + 1;
                let max_n = end.ilog10() + 1;

                (min_n..=max_n)
                    .filter(|&n| n > 1)
                    .map(|n| {
                        // Sum of all repeating digits (e.g., 1111, 2222, ..., 9999 for n=4)
                        let all_same = Self::get_sum_invalid_ids((start, end), n, n);
                        // Get all patterns with smaller, prime repeat factors
                        Self::prime_factors(n).into_iter().filter(|&k| k < n).fold(
                            all_same,
                            |mut sum, k| {
                                sum += Self::get_sum_invalid_ids((start, end), n, k);
                                sum -= all_same;
                                sum
                            },
                        )
                    })
                    .sum::<u64>()
            })
            .sum::<u64>()
            .to_string()
    }
}

fn main() -> Result<()> {
    let puzzle = Puzzle::new(false)?;
    println!("Day {} Part 1: {}", Puzzle::DAY, puzzle.part1());
    println!("Day {} Part 2: {}", Puzzle::DAY, puzzle.part2());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() -> Result<()> {
        let puzzle = Puzzle::new(true)?;
        assert_eq!(puzzle.part1(), "1227775554");
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        let puzzle = Puzzle::new(true)?;
        assert_eq!(puzzle.part2(), "4174379265");
        Ok(())
    }
}
