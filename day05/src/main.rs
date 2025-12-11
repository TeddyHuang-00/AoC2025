use anyhow::Result;
use rayon::prelude::*;
use util::{
    Solution,
    reader::{parse_lines, read_file},
};

type ID = u64;
type Range = (ID, ID);

struct Puzzle {
    ranges: Vec<Range>,
    ids: Vec<ID>,
}

impl Puzzle {
    fn new(example: bool) -> Result<Self> {
        let content = read_file(Self::DAY, example)?;
        let (ranges, ids) = content
            .split_once("\n\n")
            .ok_or_else(|| anyhow::anyhow!("Expected header and body separated by a blank line"))?;
        let mut ranges = parse_lines(ranges.trim(), |line| {
            let (start, end) = line
                .split_once('-')
                .ok_or_else(|| anyhow::anyhow!("Invalid range format in header: {line}"))?;
            let start: ID = start.parse()?;
            let end: ID = end.parse()?;
            anyhow::Ok((start, end))
        })?;
        let mut ids = parse_lines(ids.trim(), |line| {
            let id: ID = line.trim().parse()?;
            anyhow::Ok(id)
        })?;
        // Sort ranges and ids for easier processing later
        ranges.sort_unstable();
        ids.sort_unstable();
        // Merge overlapping or contiguous ranges
        ranges = ranges
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
        Ok(Self { ranges, ids })
    }

    /// Binary search for a range in the sorted list of IDs.
    fn binary_search_ids(&self, range: Range) -> Option<(usize, usize)> {
        let (start, end) = range;

        let mut left_idx = None;
        let mut low = 0;
        let mut high = self.ids.len() - 1;
        while low <= high {
            let mid = usize::midpoint(low, high);
            if self.ids[mid] >= start {
                left_idx = Some(mid);
                high = mid - 1;
            } else {
                low = mid + 1;
            }
        }
        let Some(left_idx) = left_idx else {
            // No IDs >= start, so no IDs in range
            return None;
        };

        let mut right_idx = None;
        let mut low = left_idx;
        let mut high = self.ids.len() - 1;
        while low <= high {
            let mid = usize::midpoint(low, high);
            if self.ids[mid] <= end {
                right_idx = Some(mid);
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }
        let Some(right_idx) = right_idx else {
            // No IDs <= end, so no IDs in range
            return None;
        };

        Some((left_idx, right_idx))
    }
}

impl Solution for Puzzle {
    const DAY: u8 = 5;

    /// There are two ways to solve part 1:
    /// 1. Iterate through all IDs and check if they are in any range
    /// 2. Iterate through ranges and count how many IDs fall into them
    ///
    /// Given M ranges and N IDs, the first approach is O(M log N) while the
    /// second is O(N log M). Since M is expected to be much smaller than N,
    /// like a magnitude smaller, we choose the second approach.
    fn part1(&self) -> String {
        self.ranges
            .par_iter()
            .map(|&range| {
                let (start, end) = range;
                match self.binary_search_ids((start, end)) {
                    Some((left_idx, right_idx)) => right_idx - left_idx + 1,
                    None => 0,
                }
            })
            .sum::<usize>()
            .to_string()
    }

    /// For part 2, we simply sum up the sizes of all ranges.
    ///
    /// I don't know why it is actually easier than part 1...
    /// But well, let's just go with it.
    fn part2(&self) -> String {
        self.ranges
            .par_iter()
            .map(|&(start, end)| end - start + 1)
            .sum::<ID>()
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
        assert_eq!(puzzle.part1(), "3");
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        let puzzle = Puzzle::new(true)?;
        assert_eq!(puzzle.part2(), "14");
        Ok(())
    }
}
