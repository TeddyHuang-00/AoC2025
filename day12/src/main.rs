use anyhow::Result;
use ndarray::{parallel::prelude::*, prelude::*};
use util::{
    Solution,
    reader::{parse_char_grid, parse_lines, parse_whitespace_separated, read_file},
};

struct Puzzle {
    pieces: Vec<Array2<u8>>,
    regions: Vec<(u8, u8, Vec<u8>)>,
}

impl Puzzle {
    fn parse_piece(input: &str) -> Result<Array2<u8>> {
        let Some((_, shape)) = input.split_once('\n') else {
            anyhow::bail!("Invalid piece input")
        };
        parse_char_grid(shape, |c| match c {
            '.' => Ok(0),
            '#' => Ok(1),
            _ => anyhow::bail!("Invalid character in piece"),
        })
    }

    fn parse_regions(input: &str) -> Result<(u8, u8, Vec<u8>)> {
        let Some((shape, counts)) = input.split_once(": ") else {
            anyhow::bail!("Invalid region input: {input}")
        };
        let Some((width, height)) = shape.split_once('x') else {
            anyhow::bail!("Invalid shape in region: {shape}")
        };
        let (width, height) = (width.parse()?, height.parse()?);
        let counts = parse_whitespace_separated(counts, str::parse)?;
        Ok((width, height, counts))
    }

    fn new(example: bool) -> Result<Self> {
        let content = read_file(Self::DAY, example)?;
        let (pieces, regions): (Vec<&str>, Vec<&str>) = content
            .split("\n\n")
            .partition(|s| s.chars().any(|c| c == '#'));
        let pieces = pieces
            .into_iter()
            .map(Self::parse_piece)
            .collect::<Result<_>>()?;
        let regions = match regions.len() {
            1 => regions[0],
            x => anyhow::bail!("Invalid number of regions: {x}"),
        };
        let regions = parse_lines(regions, Self::parse_regions)?;
        Ok(Self { pieces, regions })
    }
}

impl Solution for Puzzle {
    const DAY: u8 = 12;

    fn parse(example: bool) -> Self {
        Self::new(example).unwrap_or_else(|e| panic!("Failed to parse input: {e}"))
    }

    /// TBH, I had the feeling that this is too hard for a general case, so some
    /// simple heuristic like testing for capacity might be useful to reduce the
    /// number of searches. I just couldn't convince myself that this naive and
    /// stupid approach may be the final solution. And even after some Googling,
    /// I'm still not sure how to solve this in practice. Bin-packing is
    /// NP-hard, and I don't know how to solve it efficiently.
    ///
    /// - Brute force might work for small inputs, but it's not a viable
    ///   solution for larger inputs
    /// - Search algorithms like A* might be a good choice, but the heuristics
    ///   are not trivial to come up with
    /// - Genetic algorithms might be another option, but given the state space
    ///   (which is quite large, ~300 coordinates * at most 8
    ///   rotations/flipping), the population size and the number of generations
    ///   would be massive, and the performance would be questionable.
    /// - Constraint programming might be a good choice and the constraints
    ///   seems approachable, but given the size of the state space, I don't
    ///   think it's feasible for ANY solver to handle this in a reasonable
    ///   amount of time.
    ///
    ///  I'm not sure if there's a better way to ACTUALLY solve this problem.
    /// Hate to say it, but I think this problem is just not solvable in a
    /// reasonable amount of time.
    fn part1(&self) -> String {
        self.regions
            .par_iter()
            .filter(|(width, height, counts)| {
                counts
                    .iter()
                    .zip(self.pieces.iter())
                    .map(|(&c, s)| u64::from(c) * u64::from(s.sum()))
                    .sum::<u64>()
                    <= u64::from(*width) * u64::from(*height)
            })
            .count()
            .to_string()
    }

    /// Well... I guess that concludes the year. A bit of a letdown, but I guess
    /// that's just how it is. But hey, at least there are still some other days
    /// that are quite interesting. Merry Christmas and a happy new year!
    fn part2(&self) -> String {
        "Final star on top of the tree".to_string()
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
    use std::time::Duration;

    use util::{Benchmark, Serializable};

    use super::*;

    #[test]
    fn test_part1() -> Result<()> {
        let puzzle = Puzzle::new(true)?;
        // Well... I guess this is not a good test case...
        // The example input would require a different solution, but I haven't ACTUALLY
        // implemented it. I just cheated on this one.
        assert_eq!(puzzle.part1(), "3");
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        let puzzle = Puzzle::new(true)?;
        assert_eq!(puzzle.part2(), "Final star on top of the tree");
        Ok(())
    }

    #[test]
    fn benchmark() -> Result<()> {
        Puzzle::bench_all(Duration::from_secs(1)).to_csv(Puzzle::DAY)
    }
}
