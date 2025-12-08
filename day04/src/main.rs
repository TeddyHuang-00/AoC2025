use anyhow::Result;
use ndarray::{Zip, parallel::prelude::*, prelude::*};
use util::{Solution, reader::parse_grid_from_file};

struct Puzzle {
    grid: Array2<u8>,
}

impl Puzzle {
    fn new(example: bool) -> Result<Self> {
        let grid = parse_grid_from_file(Self::DAY, example, |c| match c {
            '.' => Ok(0),
            '@' => Ok(1),
            _ => anyhow::bail!("Invalid character in grid: {c}"),
        })?;
        Ok(Self { grid })
    }

    /// Find removable items in the grid. An item is removable if it is
    /// non-empty and has less than 4 non-empty neighbors in the 8 directions.
    /// Returns a boolean grid indicating which items are removable. This is a
    /// helper function used in both parts.
    fn find_removable(grid: &Array2<u8>) -> Array2<bool> {
        let mut extended = Array2::from_elem([grid.nrows() + 2, grid.ncols() + 2], 0);
        extended
            .slice_mut(s![1..=grid.nrows(), 1..=grid.ncols()])
            .assign(grid);
        let count = extended
            .windows([grid.nrows(), grid.ncols()])
            .into_iter()
            .enumerate()
            // Only consider neighborhoods in 8 directions
            .filter_map(|(idx, window)| (idx != 4).then_some(window))
            .fold(Array2::zeros([grid.nrows(), grid.ncols()]), |mut acc, x| {
                acc += &x;
                acc
            });

        Zip::from(&count)
            .and(grid)
            .par_map_collect(|&cnt, &v| cnt < 4 && v > 0)
    }
}

impl Solution for Puzzle {
    const DAY: u8 = 4;

    /// Count the number of removable items in the initial grid. Nothing fancy,
    /// just simulate the removal once.
    fn part1(&self) -> String {
        Self::find_removable(&self.grid)
            .par_iter()
            .filter(|&&removable| removable)
            .count()
            .to_string()
    }

    /// Repeatedly remove removable items until no more can be removed. Count
    /// the total number of removed items. Also straightforward simulation.
    fn part2(&self) -> String {
        let mut grid = self.grid.clone();
        let mut count = 0;
        loop {
            let removable = Self::find_removable(&grid);
            let num_removable = removable.par_iter().filter(|&&r| r).count();
            if num_removable == 0 {
                break;
            }
            count += num_removable;
            Zip::from(&mut grid).and(&removable).par_for_each(|g, &r| {
                if r {
                    *g = 0;
                }
            });
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
        assert_eq!(puzzle.part1(), "13");
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        let puzzle = Puzzle::new(true)?;
        assert_eq!(puzzle.part2(), "43");
        Ok(())
    }
}
