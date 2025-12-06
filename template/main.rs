use anyhow::Result;
use util::Solution;

struct Puzzle {}

impl Puzzle {
    fn new(example: bool) -> Result<Self> {
        Ok(Self {})
    }
}

impl Solution for Puzzle {
    const DAY: u8 = 000000;

    fn part1(&self) -> String {
        "Part 1 not implemented".to_string()
    }

    fn part2(&self) -> String {
        "Part 2 not implemented".to_string()
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
        assert_eq!(puzzle.part1(), "Part 1 not implemented");
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        let puzzle = Puzzle::new(true)?;
        assert_eq!(puzzle.part2(), "Part 2 not implemented");
        Ok(())
    }
}
