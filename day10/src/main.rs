use std::collections::{BTreeMap, HashMap};

use anyhow::Result;
use rayon::prelude::*;
use util::{
    Solution,
    reader::{parse_lines, read_file},
};

type LightState = u16;
type Count = u8;

struct Machine {
    /// Target light configuration, compressed
    goal: LightState,
    /// Button configurations, compressed
    buttons: Vec<LightState>,
    /// Press count for each light
    count: Vec<Count>,
}

struct Puzzle {
    machines: Vec<Machine>,
}

impl Puzzle {
    /// Parse a single machine definition from input line
    ///
    /// We represent the light states and button effects as bitmasks within a
    /// u16 as there are at most 10 lights. This allows for efficient state
    /// manipulation using bitwise operations.
    ///
    /// The count of presses for part 2 is stored as a vector of u8,
    /// representing the required number of presses for each light to reach the
    /// goal state.
    ///
    /// Note that the goal state for part 1 and part 2 are not connected, so we
    /// cannot reuse the same goal representation for both parts.
    fn parse_machine(input: &str) -> Result<Machine> {
        let mut goal = None;
        let mut buttons = Vec::new();
        let mut count = None;
        for part in input.split_whitespace() {
            match part.chars().next() {
                Some('[') => {
                    goal = Some(
                        part.trim_matches(|c| c == '[' || c == ']')
                            .chars()
                            .map(|c| match c {
                                '.' => Ok(0),
                                '#' => Ok(1),
                                _ => anyhow::bail!("Unexpected character in goal: {c}"),
                            })
                            .collect::<Result<Vec<_>>>()?
                            .into_iter()
                            .rev()
                            .fold(0, |acc, b| (acc << 1) | b),
                    );
                }
                Some('(') => buttons.push(
                    part.trim_matches(|c| c == '(' || c == ')')
                        .split(',')
                        .map(|s| s.parse::<u8>().map_err(Into::into))
                        .collect::<Result<Vec<_>>>()?
                        .into_iter()
                        .fold(0, |acc, b| acc | (1 << b)),
                ),
                Some('{') => {
                    count = Some(
                        part.trim_matches(|c| c == '{' || c == '}')
                            .split(',')
                            .map(|s| s.parse().map_err(Into::into))
                            .collect::<Result<Vec<_>>>()?,
                    );
                }
                _ => anyhow::bail!("Unexpected part in machine definition: {part}"),
            }
        }
        // Some good-to-have validations
        match (goal, buttons.is_empty(), count) {
            (None, _, _) => anyhow::bail!("Missing goal definition"),
            (_, true, _) => anyhow::bail!("Missing button definitions"),
            (_, _, None) => anyhow::bail!("Missing joltage definition"),
            (Some(goal), false, Some(count)) => Ok(Machine {
                goal,
                buttons,
                count,
            }),
        }
    }

    fn new(example: bool) -> Result<Self> {
        let machines = parse_lines(read_file(Self::DAY, example)?, Self::parse_machine)?;
        Ok(Self { machines })
    }

    /// For any given goal state and button transitions, find the minimum number
    /// of button presses as a binary backpack problem, solved with dynamic
    /// programming.
    ///
    /// This is feasible since pressing a button twice is equivalent to not
    /// pressing it at all (XOR operation), and thus each button can only be
    /// pressed 0 or 1 time in the final solution.
    ///
    /// The state space is limited to 2^n where n is the number of lights (at
    /// most 10), making this approach efficient.
    fn binary_backpack(goal: LightState, transition: &[LightState]) -> Option<u16> {
        let mut dp = BTreeMap::from_iter([(0, 0)]);
        for &t in transition {
            // Not pressing the button is implicitly handled by carrying over existing
            // states
            dp = dp.iter().fold(dp.clone(), |mut acc, (&state, &cost)| {
                // Try pressing the button, resulting in a new state and increased cost
                let state = state ^ t;
                let cost = cost + 1;
                acc.entry(state)
                    .and_modify(|c| {
                        if *c > cost {
                            *c = cost;
                        }
                    })
                    .or_insert(cost);
                acc
            });
        }
        // Return the cost to reach the goal state, if achievable
        dp.get(&goal).copied()
    }

    /// The original solution for this is to use a integer linear programming
    /// solver which I didn't implement myself. The solution is fast, but
    /// involves introducing an extra dependency dedicated to solving linear
    /// programming problems. If you are interested, please check this out:
    /// <https://github.com/TeddyHuang-00/AoC2025/blob/1d136c914936ae3f4c17cc11d0643650d31f9a4a/day10>
    ///
    /// The current solution is inspired by @tenthmascot on Reddit:
    /// <https://www.reddit.com/r/adventofcode/comments/1pk87hl/2025_day_10_part_2_bifurcate_your_way_to_victory>
    ///
    /// It is basically a divide-and-conquer approach to solve the problem. Here
    /// is a brief conceptual explanation: If we have an optimal solution
    /// (number of button presses) for a given target state, the solution can
    /// always be split into two parts:
    /// 1. The residual state that each button press is 0 or 1, which reaches
    ///    the same light state as the target state.
    /// 2. The remaining state that each button press is an even number, and can
    ///    be seen as twice the optimal solution for the subproblem of the
    ///    remaining state (by halving the count of presses for each light).
    ///
    /// The proof to it is also simple: The split between the residual state and
    /// the remaining state is always possible. The remaining state will always
    /// be even so that the two parts cancel each other out. We can demonstrate
    /// the optimality by contradiction: if for the given split in optimal
    /// solution, we are able to find a better solution for its subproblem, we
    /// can always move that part into the residual state, and the original
    /// solution split is not optimal.
    ///
    /// Although the branching factor is upper bounded by `2^n` where `n` is the
    /// number of buttons, the actual branching factor is much smaller in
    /// practice due to the constraints of the problem (need to constitute to
    /// the goal state, and number of presses on lights should not exceed the
    /// goal). Also, the number of recursion is bounded by `log_2 N` where `N`
    /// is the maximum goal state, and in this case, `log_2 2^8` gives 8. Also
    /// note that we are using cache to avoid redundant calculations, so that we
    /// don't recalculate the solution for the same state multiple times. These
    /// all make the solution much faster in practice.
    ///
    /// To implement this, we actually use a dynamic programming approach to
    /// find the optimal solution. But the key idea is the same, we just need to
    /// test all possible splits and use caching to avoid redundant
    /// calculations.
    fn divide_and_conquer(goal: &[Count], transition: &[LightState]) -> Option<u16> {
        let transition = transition
            .iter()
            .map(|&(mut t)| {
                // Quick conversion from bitmasks to vectors of 0 or 1
                let mut bits = vec![0; goal.len()];
                while t != 0 {
                    let i = t.trailing_zeros() as usize;
                    bits[i] = 1;
                    t ^= 1 << i;
                }
                bits
            })
            .collect::<Vec<_>>();
        let mut cache = HashMap::new();
        Self::try_divide_cached(&mut cache, goal, &transition)
    }

    /// Compress the goal state into a single integer for caching
    fn compress(goal: &[Count]) -> u128 {
        goal.iter().fold(0, |acc, &g| (acc << 8) | u128::from(g))
    }

    /// Try to solve the subproblem with caching
    fn try_divide_cached(
        cache: &mut HashMap<u128, Option<u16>>,
        goal: &[Count],
        transition: &[Vec<u8>],
    ) -> Option<u16> {
        // Check cache first
        if let Some(&res) = cache.get(&Self::compress(goal)) {
            return res;
        }
        // Base case: if all counts are 0, no button press is needed
        if goal.iter().all(|&g| g == 0) {
            return Some(0);
        }
        // Try splitting the problem into two parts
        let mut optimal = None;
        for (cnt, residual) in transition
            .iter()
            .fold(vec![(0, vec![0; goal.len()])], |mut acc, t| {
                let new = acc
                    .iter()
                    .map(|(cnt, a)| {
                        (
                            cnt + 1,
                            a.iter()
                                .zip(t.iter())
                                .map(|(&a, &b)| a + b)
                                .collect::<Vec<_>>(),
                        )
                    })
                    .filter(|(_, s)| s.iter().zip(goal.iter()).all(|(x, g)| x <= g))
                    .collect::<Vec<_>>();
                acc.extend(new);
                acc
            })
            .into_iter()
            .filter(|(_, s)| s.iter().zip(goal).all(|(a, b)| a % 2 == b % 2))
        {
            let remaining = goal
                .iter()
                .zip(residual.iter())
                .map(|(&g, &r)| (g - r) / 2)
                .collect::<Vec<_>>();
            if let Some(subsolution) = Self::try_divide_cached(cache, &remaining, transition) {
                let solution = cnt + 2 * subsolution;
                optimal = optimal.map_or(Some(solution), |s: u16| Some(s.min(solution)));
            }
        }
        // `None` means the current state is not achievable, `Some(x)` means we found a
        // solution which guarantees to be optimal
        cache.insert(Self::compress(goal), optimal);
        optimal
    }
}

impl Solution for Puzzle {
    const DAY: u8 = 10;

    fn parse(example: bool) -> Self {
        Self::new(example).unwrap_or_else(|e| panic!("Failed to parse input: {e}"))
    }

    fn part1(&self) -> String {
        self.machines
            .par_iter()
            .map(|machine| {
                Self::binary_backpack(machine.goal, &machine.buttons)
                    // The problem guarantees that a solution exists for every machine
                    .unwrap_or_else(|| unreachable!("No solution found for machine"))
            })
            .sum::<u16>()
            .to_string()
    }

    fn part2(&self) -> String {
        self.machines
            .par_iter()
            .map(|machine| {
                Self::divide_and_conquer(&machine.count, &machine.buttons)
                    // The problem guarantees that a solution exists for every machine
                    .unwrap_or_else(|| unreachable!("No solution found for machine"))
            })
            .sum::<u16>()
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
        assert_eq!(puzzle.part1(), "7");
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        let puzzle = Puzzle::new(true)?;
        assert_eq!(puzzle.part2(), "33");
        Ok(())
    }
}
