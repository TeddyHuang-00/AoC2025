use anyhow::Result;
use ndarray::{parallel::prelude::*, prelude::*};
use rayon::prelude::*;
use util::{
    Solution,
    reader::{parse_char_grid, read_file},
};

struct Puzzle {
    banks: Array2<u32>,
}

impl Puzzle {
    fn new(example: bool) -> Result<Self> {
        let banks = parse_char_grid(read_file(Self::DAY, example)?, |c| {
            c.to_digit(10)
                .ok_or_else(|| anyhow::anyhow!("Failed to parse {c} as digit"))
        })?;
        Ok(Self { banks })
    }
}

impl Solution for Puzzle {
    const DAY: u8 = 3;

    /// For each bank, find the largest digit in the bank[:-1] so that there is
    /// at least one digit after it, then find the largest digit after it.
    /// Put them together and sum the results for all banks.
    fn part1(&self) -> String {
        self.banks
            .outer_iter()
            .par_bridge()
            .map(|bank| {
                // Find the largest digit in the bank[:-1]
                let (idx, &first_digit) = bank
                    .slice(s![..bank.len() - 1])
                    .iter()
                    .enumerate()
                    .reduce(|(l_idx, l_val), (idx, val)| {
                        if val > l_val {
                            (idx, val)
                        } else {
                            (l_idx, l_val)
                        }
                    })
                    .unwrap_or_else(|| unreachable!("Bank should have at least two digits"));
                // Find the largest digit in the bank after idx
                let &second_digit = bank.slice(s![idx + 1..]).iter().max().unwrap_or_else(|| {
                    unreachable!("At least one digit should be after the largest digit")
                });
                first_digit * 10 + second_digit
            })
            .sum::<u32>()
            .to_string()
    }

    /// For each bank, use dynamic programming to find the largest 12-digit
    /// number that can be formed by the digits in the bank while maintaining
    /// their order.
    fn part2(&self) -> String {
        self.banks
            .outer_iter()
            .par_bridge()
            .map(|bank| {
                bank.iter().fold(vec![0u64; 13], |mut dp, &digit| {
                    for len in (1..=12).rev() {
                        // At each position, either take the digit or not
                        dp[len] = dp[len].max(dp[len - 1] * 10 + u64::from(digit));
                    }
                    dp
                })[12]
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
        assert_eq!(puzzle.part1(), "357");
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        let puzzle = Puzzle::new(true)?;
        assert_eq!(puzzle.part2(), "3121910778619");
        Ok(())
    }
}
