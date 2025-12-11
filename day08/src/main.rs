use std::{
    cmp::Reverse,
    collections::{BTreeMap, BinaryHeap},
};

use anyhow::Result;
use ndarray::{parallel::prelude::*, prelude::*};
use util::{
    Solution,
    reader::{parse_grid, read_file},
};

struct DisjointSet {
    /// Root of each element
    parent: Vec<usize>,
    /// Map from root to component size (only for part 1)
    sizes: BTreeMap<usize, u64>,
}

impl DisjointSet {
    /// Initialize Disjoint Set with n disjoint sets
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
            sizes: (0..size).map(|i| (i, 1)).collect::<BTreeMap<_, _>>(),
        }
    }

    /// Find the root of the set containing x with path compression
    fn find(&mut self, x: usize) -> usize {
        let mut root = x;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        let mut curr = x;
        let mut next = self.parent[curr];
        while next != root {
            next = self.parent[curr];
            self.parent[curr] = root;
            curr = next;
        }
        root
    }

    /// Union the sets containing x and y
    fn union(&mut self, x: usize, y: usize) {
        let root_x = self.find(x);
        let root_y = self.find(y);
        if root_x != root_y {
            // Set the parent of root_y to root_x
            self.parent[root_y] = root_x;
            // Then update sizes map
            let size_y = self.sizes.remove(&root_y).unwrap_or(1);
            self.sizes
                .entry(root_x)
                .and_modify(|s| *s += size_y)
                .or_insert(size_y);
        }
    }
}

struct Puzzle {
    /// Maximum number of steps to connect nodes (only for part 1)
    max_steps: usize,
    /// Coordinates of nodes: [N, 3]
    nodes: Array2<i64>,
}

impl Puzzle {
    fn new(example: bool) -> Result<Self> {
        let content = read_file(Self::DAY, example)?.replace(',', " ");
        let nodes = parse_grid(content, str::parse)?;
        let max_steps = if example { 10 } else { 1000 };
        Ok(Self { max_steps, nodes })
    }

    /// Helper function to compute squared Euclidean distance between nodes i
    /// and j
    fn dist(&self, i: usize, j: usize) -> i64 {
        if i >= self.nodes.nrows() || j >= self.nodes.nrows() {
            return i64::MAX;
        }
        (&self.nodes.row(i) - &self.nodes.row(j))
            .mapv(|x| x * x)
            .sum()
    }
}

impl Solution for Puzzle {
    const DAY: u8 = 8;

    /// Since we only need to find top `max_steps` smallest edges, we can use a
    /// max-heap to keep track while iterating through all pairs of nodes.
    fn part1(&self) -> String {
        let mut dsu = DisjointSet::new(self.nodes.nrows());
        (0..self.nodes.nrows())
            .into_par_iter()
            .flat_map_iter(|i| {
                // Generate all pairs (i, j) with j > i for upper triangular matrix
                ((i + 1)..self.nodes.nrows())
                    .map(|j| (i, j))
                    .collect::<Vec<_>>()
            })
            .fold(BinaryHeap::new, |mut heap, (i, j)| {
                // Push the distance and the pair into the heap, pop the largest if exceeding
                // max_steps to keep only smallest distances
                let d = self.dist(i, j);
                heap.push((d, i, j));
                if heap.len() > self.max_steps {
                    heap.pop();
                }
                heap
            })
            .reduce(BinaryHeap::new, |mut acc, mut heap| {
                // Further reduce between threads to get global smallest distances
                acc.extend(heap.drain());
                while acc.len() > self.max_steps {
                    acc.pop();
                }
                acc
            })
            .into_iter()
            // Finally, perform the unions
            .for_each(|(_, i, j)| dsu.union(i, j));
        // Get the first three largest components
        dsu.sizes
            .values()
            .fold(BinaryHeap::new(), |mut heap, &size| {
                heap.push(Reverse(size));
                if heap.len() > 3 {
                    heap.pop();
                }
                heap
            })
            .iter()
            .map(|&Reverse(x)| x)
            .product::<u64>()
            .to_string()
    }

    /// We can ignore connections that has no effect, i.e., connections between
    /// already connected components. Since we already have the disjoint set,
    /// this is easily achievable. On top of that, we can always keep track of
    /// the closest neighbor for each node, and only update when a connection is
    /// made, so that we don't have to consider all pairs every time.
    fn part2(&self) -> String {
        // Initialize closest neighbor for each node, stored in a min-heap
        let mut closest_neighbor = (0..self.nodes.nrows())
            .into_par_iter()
            .map(|i| {
                (0..self.nodes.nrows())
                    .filter_map(|j| (j != i).then_some((self.dist(i, j), i, j)))
                    .min_by_key(|&(dist, _, _)| dist)
                    .map_or_else(
                        || unreachable!("There should be at least one other node"),
                        Reverse,
                    )
            })
            .collect::<BinaryHeap<_>>();
        let mut dsu = DisjointSet::new(self.nodes.nrows());
        loop {
            // We greedily process the closest edge
            let Some(Reverse((_, i, j))) = closest_neighbor.pop() else {
                panic!("No more edges to process");
            };
            let root_i = dsu.find(i);
            let root_j = dsu.find(j);
            // If they belong to different components, connect them
            if root_i != root_j {
                dsu.union(i, j);
            }
            // If we find that all nodes are connected after this union,
            // we can return the product of the X coordinates of this last edge
            if dsu.sizes.len() == 1 {
                return (self.nodes[[i, 0]] * self.nodes[[j, 0]]).to_string();
            }
            // Otherwise, we need to continue updating the closest neighbor for node i
            closest_neighbor.push(
                (0..self.nodes.nrows())
                    // Filter out nodes in the same component as i
                    .filter_map(|k| (root_i != dsu.find(k)).then_some((self.dist(i, k), i, k)))
                    .min_by_key(|&(dist, _, _)| dist)
                    .map_or_else(
                        || unreachable!("At least one different component should exist"),
                        Reverse,
                    ),
            );
        }
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
        assert_eq!(puzzle.part1(), "40");
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        let puzzle = Puzzle::new(true)?;
        assert_eq!(puzzle.part2(), "25272");
        Ok(())
    }
}
