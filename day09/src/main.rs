use anyhow::Result;
use ndarray::{parallel::prelude::*, prelude::*};
use util::{
    Solution,
    reader::{parse_grid, read_file},
};

struct Puzzle {
    nodes: Array2<i64>,
}

type Edge = (usize, usize);

impl Puzzle {
    fn new(example: bool) -> Result<Self> {
        let content = read_file(Self::DAY, example)?.replace(',', " ");
        let nodes = parse_grid(content, str::parse)?;
        Ok(Self { nodes })
    }

    fn measure(&self, i: usize, j: usize) -> u64 {
        (&self.nodes.row(i) - &self.nodes.row(j))
            .mapv(|v| v.unsigned_abs() + 1)
            .product()
    }

    /// Get all vertical and horizontal edges of the polygon
    ///
    /// Returns a tuple of two vectors:
    /// - First vector contains vertical edges as pairs of node indices
    /// - Second vector contains horizontal edges as pairs of node indices
    fn get_edges(&self) -> (Vec<Edge>, Vec<Edge>) {
        let n = self.nodes.nrows();
        (0..n)
            .into_par_iter()
            .map(|i| (i, (i + 1) % n))
            .partition(|&(i, j)| self.nodes[[i, 0]] == self.nodes[[j, 0]])
    }

    /// Check if the space between two sides of a rectangle contains any part of
    /// edges (except the corners)
    ///
    /// For example, given a pair of vertical sides and a horizontal edge from
    /// the polygon, if the condition is met, we get something like this:
    /// ```raw
    ///       |  Rect  |
    /// ------+------  |
    ///       |        |
    /// ```
    /// or this:
    /// ```raw
    ///       |  Rect  |
    ///       |  ----  |
    ///       |        |
    /// ```
    /// or this:
    /// ```raw
    ///       |  Rect  |
    ///  -----+--------+----
    ///       |        |
    /// ```
    /// Any of these cases means that some part of the edge falls within the
    /// rectangle. As edges are the interfaces of valid and invalid areas, this
    /// means that the rectangle cannot be fully contained within a valid area,
    /// i.e., at least some part of the rectangle is invalid.
    fn intersect_edge(
        &self,
        xs: (i64, i64),
        ys: (i64, i64),
        edges: &[Edge],
        transpose: bool,
    ) -> bool {
        let (x1, x2) = xs;
        let (x1, x2) = (x1.min(x2), x1.max(x2));
        let (y1, y2) = ys;
        let (y1, y2) = (y1.min(y2), y1.max(y2));
        edges.into_par_iter().any(|&(i, j)| {
            let i = self.nodes.row(i);
            let j = self.nodes.row(j);
            let (ex1, ex2, ey) = if transpose {
                (i[1].min(j[1]), i[1].max(j[1]), j[0])
            } else {
                (i[0].min(j[0]), i[0].max(j[0]), i[1])
            };

            ey > y1 && ey < y2 && ex1 < x2 && ex2 > x1
        })
    }
}

impl Solution for Puzzle {
    const DAY: u8 = 9;

    fn parse(example: bool) -> Self {
        Self::new(example).unwrap_or_else(|e| panic!("Failed to parse input: {e}"))
    }

    /// Find the largest area defined by any two nodes, without any constraints,
    /// so we can brute-force the search and just measure all unique pairs.
    fn part1(&self) -> String {
        (0..self.nodes.nrows())
            .into_par_iter()
            .flat_map_iter(|i| {
                (i + 1..self.nodes.nrows())
                    .map(|j| (i, j))
                    .collect::<Vec<_>>()
            })
            .map(|(i, j)| self.measure(i, j))
            .max()
            .unwrap_or_else(|| unreachable!("Must have at least one pair of nodes"))
            .to_string()
    }

    /// Find the largest area defined by any two nodes, such that the rectangle
    /// defined by those nodes can fit entirely within the polygon defined by
    /// the nodes.
    ///
    /// This is done by checking that the rectangle does not strictly contain
    /// any parts of edges of the polygon. Otherwise, the rectangle would cross
    /// into invalid areas, so we discard it.
    fn part2(&self) -> String {
        let (vertical_edges, horizontal_edges) = self.get_edges();

        (0..self.nodes.nrows())
            .into_par_iter()
            .flat_map_iter(|i| {
                (i + 1..self.nodes.nrows())
                    .map(|j| (i, j))
                    .collect::<Vec<_>>()
            })
            .filter_map(|(i, j)| {
                let (px1, py1) = (self.nodes[[i, 0]], self.nodes[[j, 1]]);
                let (px2, py2) = (self.nodes[[j, 0]], self.nodes[[i, 1]]);
                if !self.intersect_edge((px1, px2), (py1, py2), &horizontal_edges, false)
                    && !self.intersect_edge((py1, py2), (px1, px2), &vertical_edges, true)
                {
                    Some(self.measure(i, j))
                } else {
                    None
                }
            })
            .max()
            .unwrap_or_else(|| unreachable!("Must have at least one pair of nodes"))
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
        assert_eq!(puzzle.part1(), "50");
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        let puzzle = Puzzle::new(true)?;
        assert_eq!(puzzle.part2(), "24");
        Ok(())
    }
}
