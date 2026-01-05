use std::collections::BTreeSet;

use anyhow::Result;
use ndarray::parallel::prelude::*;
use rayon::prelude::*;
use util::{
    Solution,
    reader::{parse_char_grid, read_file},
};

type Position = (usize, usize);

struct Puzzle {
    start: Position,
    /// Positions of splitters in row-major order
    splitters: BTreeSet<Position>,
    /// Width of the grid, used to initialize beam states
    width: usize,
}

impl Puzzle {
    fn new(example: bool) -> Result<Self> {
        let grid = parse_char_grid(read_file(Self::DAY, example)?, |c| match c {
            '.' | 'S' | '^' => Ok(c),
            _ => anyhow::bail!("Invalid character in grid: {c}"),
        })?;
        let start = grid
            .indexed_iter()
            .par_bridge()
            .find_map_any(|((r, c), &v)| (matches!(v, 'S')).then_some((r, c)))
            .ok_or_else(|| anyhow::anyhow!("No start position found in grid"))?;
        let splitters = grid
            .indexed_iter()
            .par_bridge()
            .filter_map(|((r, c), &v)| (matches!(v, '^')).then_some((r, c)))
            .collect();
        let (_, width) = grid.dim();
        Ok(Self {
            start,
            splitters,
            width,
        })
    }
}

impl Solution for Puzzle {
    const DAY: u8 = 7;

    fn parse(example: bool) -> Self {
        Self::new(example).unwrap_or_else(|e| panic!("Failed to parse input: {e}"))
    }

    /// To find all splitters along the path, we can just iterate over the
    /// splitters by row order. If there is an active beam at the column of the
    /// splitter, it is activated, and we update the state of the beams
    /// accordingly.
    fn part1(&self) -> String {
        let mut beams = vec![false; self.width];
        beams[self.start.1] = true;
        let (_, count) =
            self.splitters
                .iter()
                .fold((beams, 0), |(mut beams, mut count), &(_, c)| {
                    if beams[c] {
                        // Found a splitter along an active beam
                        count += 1;
                        beams[c] = false;
                        if let Some(v) = beams.get_mut(c.wrapping_add_signed(-1)) {
                            *v = true;
                        }
                        if let Some(v) = beams.get_mut(c + 1) {
                            *v = true;
                        }
                    }
                    (beams, count)
                });
        count.to_string()
    }

    /// Similar to part 1, but we instead keep track of the number of beams to
    /// reach each column. When we reach the bottom row, those are counts of
    /// unique paths reaching the bottom through that position. We sum those
    /// counts to get the total number of unique paths to the bottom.
    fn part2(&self) -> String {
        let mut beams = vec![0; self.width];
        beams[self.start.1] = 1;
        let beams = self.splitters.iter().fold(beams, |mut beams, &(_, c)| {
            match beams[c] {
                0 => beams,
                cnt => {
                    // Found a splitter along an active beam
                    if let Some(v) = beams.get_mut(c.wrapping_add_signed(-1)) {
                        *v += cnt;
                    }
                    if let Some(v) = beams.get_mut(c + 1) {
                        *v += cnt;
                    }
                    beams[c] = 0;
                    beams
                }
            }
        });
        beams.into_iter().sum::<u64>().to_string()
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
        assert_eq!(puzzle.part1(), "21");
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        let puzzle = Puzzle::new(true)?;
        assert_eq!(puzzle.part2(), "40");
        Ok(())
    }

    #[test]
    fn benchmark() -> Result<()> {
        Puzzle::bench_all(Duration::from_secs(1)).to_csv(Puzzle::DAY)
    }
}
