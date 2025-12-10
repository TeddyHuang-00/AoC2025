use std::collections::BTreeMap;

use anyhow::Result;
use good_lp::{
    Expression, ProblemVariables, Solution as LpSolution, SolverModel, VariableDefinition,
    default_solver,
};
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

    /// When the goal state changes to a count of presses for each light, this
    /// forms a linear system $Ax = b$, where:
    /// - A is the matrix formed by the button transitions, either 0 or 1
    ///   depending on whether a button affects a light
    /// - x is the vector of button presses (to be solved), non-negative
    ///   integers
    /// - b is the goal vector of required presses for each light, also
    ///   non-negative integers
    ///
    /// This forms a problem of integer linear programming. I did try an A-star
    /// search for this (taking the Chebyshev distance as heuristic), but it
    /// took too long to run on the full input and requires more RAM than anyone
    /// would reasonably have, even with further compression optimizations.
    ///
    /// Thus, a proper ILP solver is needed. And to keep my own sanity, I
    /// resorted to introduce a solver library, `good_lp`, only for this part.
    /// At least it is a pure Rust library (with some backend) with no external
    /// dependencies, and I learned a new thing in the process.
    fn integer_programming(goal: &[Count], transition: &[LightState]) -> Option<u16> {
        let mut problem = ProblemVariables::new();
        // Define variables
        let vars = (0..transition.len())
            .map(|_| problem.add(VariableDefinition::new().integer().min(0)))
            .collect::<Vec<_>>();
        // Define constraints
        let constraints = goal
            .iter()
            .enumerate()
            .map(|(i, &g)| {
                transition
                    .iter()
                    .enumerate()
                    .fold(Expression::default(), |mut expr, (j, &t)| {
                        if (t & (1 << i)) != 0 {
                            expr += vars[j];
                        }
                        expr
                    })
                    .eq(g)
            })
            .collect::<Vec<_>>();
        // Define objective
        let objective = vars
            .iter()
            .fold(Expression::default(), |acc, &var| acc + var);
        // Finally we have every component to build the problem
        let problem = problem
            .minimise(objective)
            .using(default_solver)
            .with_all(constraints);
        let solution = problem.solve().ok()?;
        let int_solution = vars
            .iter()
            .map(|&x| {
                let v = solution.eval(x);
                // The solver should always return integer values for integer variables,
                // but we add checks just in case, so that we can reasonably allow the
                // conversion from f64 to u16.
                assert!(
                    (v - v.round()).abs() <= 1e-6,
                    "Non-integer solution found: {v}"
                );
                assert!(v <= f64::from(u16::MAX), "Solution value too large: {v}");
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                Some(v.round() as u16)
            })
            .collect::<Option<Vec<_>>>()?;
        Some(int_solution.iter().sum())
    }
}

impl Solution for Puzzle {
    const DAY: u8 = 10;

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
                Self::integer_programming(&machine.count, &machine.buttons)
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
