use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use ndarray::{Zip, parallel::prelude::*, prelude::*};
use rayon::iter::ParallelBridge;
use util::{
    Solution,
    reader::{parse_char_grid, read_file},
};

#[derive(Clone, Copy)]
enum Grid {
    Empty,
    Start,
    Splitter,
}

struct Puzzle {
    start: (usize, usize),
    /// Step distance to next splitter in downward direction.
    ///
    /// This is intended to speed up traversal, as we can skip over empty cells
    /// in one step.
    ///
    /// We may also transform this into a graph for the whole grid, but this is
    /// simpler at the cost of some more runtime memory.
    shortcut: Array2<usize>,
}

impl Puzzle {
    fn new(example: bool) -> Result<Self> {
        let grid = parse_char_grid(read_file(Self::DAY, example)?, |c| match c {
            '.' => Ok(Grid::Empty),
            'S' => Ok(Grid::Start),
            '^' => Ok(Grid::Splitter),
            _ => anyhow::bail!("Invalid character in grid: {c}"),
        })?;
        let start = grid
            .indexed_iter()
            .par_bridge()
            .find_map_any(|((r, c), &v)| (matches!(v, Grid::Start)).then_some((r, c)))
            .ok_or_else(|| anyhow::anyhow!("No start position found in grid"))?;
        let mut shortcut = Array2::zeros((grid.nrows(), grid.ncols()));
        Zip::from(shortcut.lanes_mut(Axis(0)))
            .and(grid.lanes(Axis(0)))
            .par_for_each(|mut shortpass, lane| {
                let mut next_splitter = 0;
                for (s, c) in shortpass.iter_mut().zip(lane.iter()).rev() {
                    match c {
                        // Reset counter at splitter
                        Grid::Splitter => next_splitter = 0,
                        // Increase distance at empty or start
                        _ => next_splitter += 1,
                    }
                    *s = next_splitter;
                }
            });

        Ok(Self { start, shortcut })
    }
}

impl Solution for Puzzle {
    const DAY: u8 = 7;

    /// To find all splitters along the path, we can do a depth-first search
    /// from the start position, keeping track of all visited positions
    /// (splitters), and let frontiers be the start for the next beam.
    ///
    /// BFS would also work, they just differ in the order of visiting nodes.
    fn part1(&self) -> String {
        let width = self.shortcut.ncols();
        let height = self.shortcut.nrows();
        let mut visited = BTreeSet::new();
        let mut frontier = vec![self.start];
        while let Some((r, c)) = frontier.pop() {
            let nr = r + self.shortcut[[r, c]];
            if nr >= height || !visited.insert((nr, c)) {
                continue;
            }
            [-1, 1]
                .iter()
                .filter_map(|&side| {
                    let nc = c.wrapping_add_signed(side);
                    (nc < width).then_some((nr, nc))
                })
                .for_each(|pos| frontier.push(pos));
        }
        visited.len().to_string()
    }

    /// Similar to part 1, but we additionally keep track of the number of ways
    /// to reach each position in the frontier. When we reach the bottom row,
    /// those are counts of unique paths reaching the bottom through that
    /// position. We sum those counts to get the total number of unique paths to
    /// the bottom.
    fn part2(&self) -> String {
        let width = self.shortcut.ncols();
        let height = self.shortcut.nrows();
        let mut count = 0usize;
        let mut frontier = vec![(self.start, 1)];
        while !frontier.is_empty() {
            let mut next_layer = BTreeMap::new();
            for ((r, c), n) in frontier {
                let nr = r + self.shortcut[[r, c]];
                if nr >= height {
                    // Reached bottom row, add to count
                    count += n;
                    continue;
                }
                [-1, 1]
                    .iter()
                    .filter_map(|&side| {
                        let nc = c.wrapping_add_signed(side);
                        (nc < width).then_some((nr, nc))
                    })
                    .for_each(|pos| {
                        next_layer.entry(pos).and_modify(|e| *e += n).or_insert(n);
                    });
            }
            frontier = next_layer.into_iter().collect();
        }
        count.to_string()
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
        assert_eq!(puzzle.part1(), "21");
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        let puzzle = Puzzle::new(true)?;
        assert_eq!(puzzle.part2(), "40");
        Ok(())
    }
}
