use anyhow::Result;
use ndarray::{Zip, parallel::prelude::*, prelude::*};
use util::{
    Solution,
    reader::{parse_fixed_width_grid, parse_whitespace_separated, read_file},
};

#[derive(Clone, Copy)]
enum Operator {
    Add,
    Multiply,
}

#[derive(Clone, Copy)]
enum AlignedValue {
    Left(u64),
    Right(u64),
}

struct Puzzle {
    numbers: Array2<AlignedValue>,
    operators: Array1<Operator>,
}

impl Puzzle {
    /// Parse the input into a grid of aligned numbers and a list of operators.
    ///
    /// This does the heavy lifting of parsing fixed-width columns where each
    /// column may have numbers aligned either to the left or right. This
    /// alignment affects how we interpret the digits in part 2.
    fn new(example: bool) -> Result<Self> {
        let content = read_file(Self::DAY, example)?;
        let num_lines = content.lines().count();
        // Only the last line contains operators, the rest are numbers
        let operator_line = content
            .lines()
            .last()
            .ok_or_else(|| anyhow::anyhow!("No lines in input"))?;
        let column_widths = operator_line
            .trim()
            .trim_matches(['+', '*']) // Remove leading/trailing operators
            .split(['+', '*'])
            .map(|s| s.len() + 1) // +1 for the whole column
            .collect::<Vec<usize>>();
        let operators = parse_whitespace_separated(operator_line, |s| match s {
            "+" => Ok(Operator::Add),
            "*" => Ok(Operator::Multiply),
            _ => anyhow::bail!("Unknown operator: {s}"),
        })?
        .into();
        let numbers = parse_fixed_width_grid(
            content
                .lines()
                .take(num_lines - 1)
                .collect::<Vec<&str>>()
                .join("\n"),
            column_widths,
            |s| match s {
                v if v.starts_with(' ') => {
                    let num: u64 = v.trim().parse()?;
                    Ok(AlignedValue::Right(num))
                }
                v => {
                    let num: u64 = v.trim().parse()?;
                    Ok(AlignedValue::Left(num))
                }
            },
        )?;
        Ok(Self { numbers, operators })
    }

    fn row_compute(numbers: &ArrayView1<AlignedValue>, op: Operator) -> u64 {
        match op {
            Operator::Add => numbers
                .iter()
                .map(|&v| match v {
                    AlignedValue::Left(n) | AlignedValue::Right(n) => n,
                })
                .sum::<u64>(),
            Operator::Multiply => numbers
                .iter()
                .map(|&v| match v {
                    AlignedValue::Left(n) | AlignedValue::Right(n) => n,
                })
                .product(),
        }
    }

    fn column_compute(numbers: &ArrayView1<AlignedValue>, op: Operator) -> u64 {
        // First, get the maximum number of digits in any number as full column width
        let max_digit = numbers
            .iter()
            .map(|&v| match v {
                AlignedValue::Left(n) | AlignedValue::Right(n) => n,
            })
            .max()
            .unwrap_or(1)
            .ilog10()
            + 1;
        let values = numbers
            .iter()
            // For each aligned number, extract its digits, then pad with None to align to
            // `max_digit`. For example, for `max_digit`=4:
            // - `AlignedValue::Left(23)` -> `[None, None, Some(2), Some(3)]`
            // - `AlignedValue::Right(45)` -> `[Some(4), Some(5), None, None]`
            // We cannot simply use 0 as padding because that would affect the value.
            .map(|&v| match v {
                AlignedValue::Left(n) => {
                    let digits = n.ilog10() + 1;
                    std::iter::repeat_n(None, (max_digit - digits) as usize)
                        .chain((0..digits).map(|d| Some(n / 10u64.pow(d) % 10)))
                        .rev()
                        .collect::<Vec<_>>()
                }
                AlignedValue::Right(n) => {
                    let digits = n.ilog10() + 1;
                    std::iter::repeat_n(None, (max_digit - digits) as usize)
                        .chain((0..digits).map(|d| Some(n / 10u64.pow(d) % 10)).rev())
                        .collect::<Vec<_>>()
                }
            })
            // Then we can interpret the columns as numbers by folding the digits.
            // `None` are ignored.
            .fold(
                std::iter::repeat_n(0, max_digit as usize).collect::<Vec<_>>(),
                |acc, digits| {
                    acc.into_iter()
                        .zip(digits)
                        .map(|(n, digit)| digit.map_or(n, |d| n * 10 + d))
                        .collect()
                },
            );

        match op {
            Operator::Add => values.into_iter().sum::<u64>(),
            Operator::Multiply => values.into_iter().product(),
        }
    }
}

impl Solution for Puzzle {
    const DAY: u8 = 6;

    /// Evaluate the expressions in parallel, summing the results.
    ///
    /// Pretty straightforward.
    fn part1(&self) -> String {
        Zip::from(self.numbers.lanes(Axis(0)))
            .and(&self.operators)
            .into_par_iter()
            .map(|(lane, &op)| Self::row_compute(&lane, op))
            .sum::<u64>()
            .to_string()
    }

    /// For each expression, we first interpret the aligned numbers as they are
    /// written in columns. Then we evaluate the expression.
    ///
    /// This is so f**king tedious but straightforward.
    fn part2(&self) -> String {
        Zip::from(self.numbers.lanes(Axis(0)))
            .and(&self.operators)
            .into_par_iter()
            .map(|(lane, &op)| Self::column_compute(&lane, op))
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
        assert_eq!(puzzle.part1(), "4277556");
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        let puzzle = Puzzle::new(true)?;
        assert_eq!(puzzle.part2(), "3263827");
        Ok(())
    }
}
