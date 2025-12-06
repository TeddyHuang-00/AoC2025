# Advent of Code 2025

This repository contains my solutions for the Advent of Code 2025 in Rust.

Unlike [last year](https://github.com/TeddyHuang-00/AoC2024), this year, I have decided to take a different approach in organizing my solutions:

- Each day's solution is contained within its own crate (e.g., `day01`, `day02`, etc.), so that multiple files for a single day can be possible.
- A shared utility crate (`util`) is created to hold common functionalities used across different days, such as input reading and parsing.
- No more top-level binary crate; instead, each day's crate is a binary crate itself. This also means no more macro-based solution definition.
- A workspace is set up to manage all the crates together. Testing and building can be done from the workspace root.

The core idea is to leverage Rust's crate system and Cargo's workspace feature to reduce the boilerplate and improve code organization.

## License

This project is licensed under the MIT OR Apache-2.0 License. See the [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) files for details.
