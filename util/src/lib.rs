//! Utilities for Advent of Code challenges

pub mod reader;

/// A trait that defines the structure for an Advent of Code solution.
pub trait Solution {
    /// The day of the Advent of Code challenge this solution corresponds to.
    const DAY: u8;

    /// Solve part 1 of the day's challenge.
    ///
    /// Should handle errors internally and return the result as a String.
    fn part1(&self) -> String;

    /// Solve part 2 of the day's challenge.
    ///
    /// Should handle errors internally and return the result as a String.
    fn part2(&self) -> String;
}
