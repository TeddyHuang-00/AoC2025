pub mod reader;

/// A trait that defines the structure for an Advent of Code solution.
pub trait Solution {
    const DAY: u8;

    fn part1(&self) -> String;
    fn part2(&self) -> String;
}
