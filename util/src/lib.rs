//! Utilities for Advent of Code challenges

pub mod reader;
pub mod timer;
pub mod writer;

use std::time::Duration;

use anyhow::Result;

use crate::timer::{BenchmarkResult, measure_many};
pub use crate::writer::Serializable;

/// Get the root directory of the workspace by looking for Cargo.lock
///
/// Returns a `PathBuf` representing the workspace root directory.
///
/// # Errors
/// This function will return an error if it cannot find the workspace root in
/// any parent directory.
fn get_workspace_root() -> Result<std::path::PathBuf> {
    let mut dir = std::env::current_dir()?;
    // Traverse up the directory tree until we find Cargo.lock,
    // which indicates the workspace root
    while !dir.join("Cargo.lock").exists() {
        if !dir.pop() {
            anyhow::bail!("Could not find workspace root");
        }
    }
    Ok(dir)
}

/// A trait that defines the structure for an Advent of Code solution.
pub trait Solution {
    /// The day of the Advent of Code challenge this solution corresponds to.
    const DAY: u8;

    /// Parse the input data for the day's challenge.
    fn parse(example: bool) -> Self;

    /// Solve part 1 of the day's challenge.
    ///
    /// Should handle errors internally and return the result as a String.
    fn part1(&self) -> String;

    /// Solve part 2 of the day's challenge.
    ///
    /// Should handle errors internally and return the result as a String.
    fn part2(&self) -> String;
}

pub trait Benchmark {
    fn bench_parse(time_limit: Duration) -> BenchmarkResult;
    fn bench_part1(time_limit: Duration) -> BenchmarkResult;
    fn bench_part2(time_limit: Duration) -> BenchmarkResult;
    #[must_use]
    fn bench_all(time_limit: Duration) -> [BenchmarkResult; 3] {
        [
            Self::bench_parse(time_limit),
            Self::bench_part1(time_limit),
            Self::bench_part2(time_limit),
        ]
    }
}

impl<T: Solution> Benchmark for T {
    fn bench_parse(time_limit: Duration) -> BenchmarkResult {
        measure_many("Parse", time_limit, || T::parse(false))
    }

    fn bench_part1(time_limit: Duration) -> BenchmarkResult {
        let puzzle = T::parse(false);
        measure_many("Part 1", time_limit, move || puzzle.part1())
    }

    fn bench_part2(time_limit: Duration) -> BenchmarkResult {
        let puzzle = T::parse(false);
        measure_many("Part 2", time_limit, move || puzzle.part2())
    }
}
