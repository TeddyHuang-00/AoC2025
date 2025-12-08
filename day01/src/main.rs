use anyhow::Result;
use util::{Solution, reader::parse_lines_from_file};

type Operation = i32;

struct Puzzle {
    operations: Vec<Operation>,
}

impl Puzzle {
    fn parse_operation(input: &str) -> Result<Operation> {
        if input.is_empty() {
            anyhow::bail!("Empty input");
        }
        let (op, num) = input.split_at(1);
        match op {
            "L" => Ok(-num.parse()?),
            "R" => Ok(num.parse()?),
            _ => anyhow::bail!("Invalid operation: {input}"),
        }
    }

    fn new(example: bool) -> Result<Self> {
        let operations = parse_lines_from_file(Self::DAY, example, Self::parse_operation)?;
        Ok(Self { operations })
    }
}

impl Solution for Puzzle {
    const DAY: u8 = 1;

    /// Simulate the operations and count the number of times we pass position 0
    fn part1(&self) -> String {
        let (_, cnt) = self.operations.iter().fold((50, 0), |(pos, cnt), op| {
            let new_pos = (pos + op).rem_euclid(100);
            (new_pos, cnt + u32::from(new_pos == 0))
        });
        format!("{cnt}")
    }

    /// Simulate the operations, breaking down large moves into full circles and
    /// remainders and handle passing position 0 correctly for remainders
    fn part2(&self) -> String {
        let (_, cnt) = self.operations.iter().fold((50, 0), |(pos, cnt), op| {
            let full_circle = (op.abs() / 100).unsigned_abs();
            let new_pos = pos + (op % 100);
            let rem_zero = u32::from(pos > 0 && new_pos <= 0 || pos < 100 && new_pos >= 100);
            (new_pos.rem_euclid(100), cnt + rem_zero + full_circle)
        });
        format!("{cnt}")
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
        assert_eq!(puzzle.part1(), "3");
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        let puzzle = Puzzle::new(true)?;
        assert_eq!(puzzle.part2(), "6");
        Ok(())
    }
}
