use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Add,
    str::FromStr,
};

use anyhow::Result;
use rayon::prelude::*;
use util::{
    Solution,
    reader::{parse_lines, parse_whitespace_separated, read_file},
};

struct Puzzle {
    /// Incoming nodes for each node (parents)
    in_nodes: Vec<BTreeSet<usize>>,
    /// Outgoing nodes for each node (children)
    out_nodes: Vec<Vec<usize>>,
    /// Mapping from machine names to node indices (just for convenience)
    names: BTreeMap<String, usize>,
}

impl Puzzle {
    fn new(example: bool) -> Result<Self> {
        let content = read_file(Self::DAY, example)?.replace(':', "");
        let mut machines =
            parse_lines(content, |s| parse_whitespace_separated(s, String::from_str))?;
        // Create an extra out node
        machines.push(vec!["out".to_string()]);
        let names = machines
            .iter()
            .enumerate()
            .map(|(i, m)| {
                Ok((
                    m.first()
                        .ok_or_else(|| anyhow::anyhow!("Empty line"))?
                        .to_owned(),
                    i,
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        let out_nodes = machines
            .iter()
            .map(|m| {
                m.iter()
                    .skip(1)
                    .map(|p| {
                        names
                            .get(p)
                            .ok_or_else(|| anyhow::anyhow!("{p} not found in machine definitions"))
                            .copied()
                    })
                    .collect::<Result<_>>()
            })
            .collect::<Result<Vec<Vec<_>>>>()?;
        let in_nodes = out_nodes.iter().enumerate().fold(
            vec![BTreeSet::new(); out_nodes.len()],
            |mut acc, (i, outs)| {
                for &j in outs {
                    acc[j].insert(i);
                }
                acc
            },
        );
        Ok(Self {
            in_nodes,
            out_nodes,
            names,
        })
    }

    /// Generalized topology dynamic programming framework for DAGs.
    ///
    /// It does a topological traversal of the DAG from `start` to `goal`,
    fn topology_dynamic_programming<T, FT, FU>(
        &self,
        start: usize,
        goal: usize,
        default_state: T,
        start_state: T,
        transit: FT,
        update: FU,
    ) -> T
    where
        T: Clone + Copy + Send + Sync,
        FT: Fn(T, T) -> T + Send + Sync,
        FU: Fn(T, usize) -> T + Send + Sync,
    {
        let mut in_nodes = self.in_nodes.clone();
        let mut count = vec![default_state; in_nodes.len()];
        count[start] = start_state;
        let mut frontier = in_nodes
            .iter()
            .enumerate()
            .filter_map(|(i, ins)| if ins.is_empty() { Some(i) } else { None })
            .collect::<Vec<_>>();
        let mut visited = BTreeSet::<usize>::new();
        // This loop is fail-safe because even the graph is not a DAG, we will just be
        // stuck when there is a cycle and no new nodes can be added to the frontier. So
        // the algorithm will terminate, and the contribution from the cycle will just
        // not be counted.
        while !frontier.is_empty() {
            // Update and finalize the states for all nodes in the frontier
            for &node in &frontier {
                count[node] = update(count[node], node);
            }
            visited.extend(&frontier);
            // Early exit if we have reached the goal
            if visited.contains(&goal) {
                break;
            }
            // Propagate states to outgoing nodes in parallel
            // (to speed up for large graphs, hopefully)
            let edits = frontier
                .into_par_iter()
                .flat_map_iter(|from| {
                    let cnt = count[from];
                    self.out_nodes[from].iter().map(move |&to| (from, to, cnt))
                })
                .collect::<Vec<_>>();
            // Apply the edits
            for (from, to, cnt) in edits {
                count[to] = transit(count[to], cnt);
                in_nodes[to].remove(&from);
            }
            // Find new frontier nodes with no remaining incoming edges
            frontier = in_nodes
                .iter()
                .enumerate()
                .filter_map(|(i, ins)| {
                    if ins.is_empty() && !visited.contains(&i) {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
        }
        // Return the final state at the goal node
        count[goal]
    }
}

impl Solution for Puzzle {
    const DAY: u8 = 11;

    fn parse(example: bool) -> Self {
        Self::new(example).unwrap_or_else(|e| panic!("Failed to parse input: {e}"))
    }

    /// Part 1 we just count the number of paths, no special update or transit
    /// logic needed.
    fn part1(&self) -> String {
        self.topology_dynamic_programming(
            self.names["you"],
            self.names["out"],
            0,
            // Give 1 path at the start
            1,
            Add::add,
            // No special update needed
            |state, _| state,
        )
        .to_string()
    }

    /// Part 2 we need to track different "kinds" of paths based on whether they
    /// visit two special nodes (dac and fft), or not. This gives 4 combinations
    /// of paths. We use a tuple of 4 u64 integers to track the counts of each
    /// kind of path as it turns out that the number of paths can be really
    /// large and any other compact representation (e.g., bitmask) won't work
    /// because we don't have such a large integer type to use.
    ///
    /// The update function will check if the current node is one of the special
    /// nodes, and if so, it will "shift" the counts accordingly to mark that
    /// the paths have visited that node. For example, if (A, B, C, D)
    /// represents the counts of paths that have visited neither node, only dac,
    /// only fft, and both nodes respectively, then visiting dac will transform
    /// the state to (0, A + B, 0, C + D), effectively moving the counts to
    /// reflect that those paths have now visited dac.
    ///
    /// The transit function simply adds the counts from different paths
    /// together as before, we are just adding tuples element-wise instead of
    /// single integers.
    ///
    /// Compared to yesterday's problem, this one is much, much, MUCH more
    /// straightforward and enjoyable. What a nice and relaxing ride!
    fn part2(&self) -> String {
        // State: (--, -+, +-, ++) for 4 combinations of visiting two nodes or not
        type State = (u64, u64, u64, u64);
        // Nodes (checkpoints) to track
        let ckpts = (self.names["dac"], self.names["fft"]);

        self.topology_dynamic_programming(
            self.names["svr"],
            self.names["out"],
            (0, 0, 0, 0),
            // Start with only 1 path (both unvisited)
            (1, 0, 0, 0),
            // Carry over states when merging from different paths
            |a: State, b: State| (a.0 + b.0, a.1 + b.1, a.2 + b.2, a.3 + b.3),
            // Mark paths that visit dac and fft by shifting counts
            move |state: State, node: usize| match ckpts {
                (x, _) if node == x => (0, state.0 + state.1, 0, state.2 + state.3),
                (_, y) if node == y => (0, 0, state.0 + state.2, state.1 + state.3),
                _ => state,
            },
        )
        // Return the count of paths that have visited both checkpoints
        .3
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
        assert_eq!(puzzle.part1(), "5");
        Ok(())
    }

    /// I didn't expect the example to change for part 2, but it did.
    /// Fortunately, we can still tweak the example a little bit so that it
    /// doesn't change the answer for part 1. As for part 2, we will just use
    /// our hand-calculated answer for testing.
    ///
    /// Specifically, we renamed some machines:
    /// - aaa -> svr
    /// - bbb -> dac
    /// - ddd -> fft
    ///
    /// And the rest of the graph remains the same.
    ///
    /// Alternatively, you can also use the example from part 2, and change aaa
    /// to you so that it can also be used for part 1. But you will need to
    /// change the expected answer for part 1.
    #[test]
    fn test_part2() -> Result<()> {
        let puzzle = Puzzle::new(true)?;
        assert_eq!(puzzle.part2(), "1");
        Ok(())
    }
}
