# Advent of Code 2025

This repository contains my solutions for the Advent of Code 2025 in Rust.

Unlike [last year](https://github.com/TeddyHuang-00/AoC2024), this year, I have decided to take a different approach in organizing my solutions:

- Each day's solution is contained within its own crate (e.g., `day01`, `day02`, etc.), so that multiple files for a single day can be possible.
- A shared utility crate (`util`) is created to hold common functionalities used across different days, such as input reading and parsing.
- No more top-level binary crate; instead, each day's crate is a binary crate itself. This also means no more macro-based solution definition.
- A workspace is set up to manage all the crates together. Testing and building can be done from the workspace root.

The core idea is to leverage Rust's crate system and Cargo's workspace feature to reduce the boilerplate and improve code organization.

## Performance

Although not the primary focus of my solutions, I tried my best to find the most efficient approach to each day's puzzle, which might take advantage of certain structures of the problem input thus not directly comparable with general solution to the described problem.

Below are the benchmarking result of different part of the solutions, ran on M1 max with 64G RAM. The median $\pm$ MAD of runtimes are reported. The results are quite satisfactory, with a total runtime of under half a second for all 12 days' puzzles on my machine.

| Day | Parsing                 | Part 1                  | Part 2                  | Note |
| --- | ----------------------- | ----------------------- | ----------------------- | ---- |
| 01  | 111.125 $\pm$ 2.167 µs  | 18.250 $\pm$ 0.042 µs   | 20.667 $\pm$ 0.042 µs   |      |
| 02  | 30.292 $\pm$ 1.375 µs   | 16.625 $\pm$ 3.584 µs   | 29.167 $\pm$ 4.750 µs   |      |
| 03  | 158.395 $\pm$ 3.103 µs  | 96.875 $\pm$ 12.750 µs  | 200.000 $\pm$ 15.042 µs |      |
| 04  | 180.209 $\pm$ 2.709 µs  | 111.583 $\pm$ 11.792 µs | 12.038 $\pm$ 0.284 ms   |      |
| 05  | 70.625 $\pm$ 1.375 µs   | 23.542 $\pm$ 5.000 µs   | 19.584 $\pm$ 4.582 µs   |      |
| 06  | 107.375 $\pm$ 2.375 µs  | 40.625 $\pm$ 6.250 µs   | 117.583 $\pm$ 9.625 µs  |      |
| 07  | 290.875 $\pm$ 19.584 µs | 134.125 $\pm$ 2.292 µs  | 1.759 $\pm$ 0.013 ms    |      |
| 08  | 123.083 $\pm$ 2.250 µs  | 21.040 $\pm$ 1.829 ms   | 281.640 $\pm$ 0.541 ms  |      |
| 09  | 67.145 $\pm$ 2.979 µs   | 2.552 $\pm$ 0.243 ms    | 9.382 $\pm$ 0.942 ms    |      |
| 10  | 203.583 $\pm$ 5.291 µs  | 836.958 $\pm$ 70.437 µs | 95.850 $\pm$ 3.624 ms   |      |
| 11  | 433.979 $\pm$ 9.896 µs  | 1.639 $\pm$ 0.117 ms    | 1.725 $\pm$ 0.131 ms    |      |
| 12  | 163.166 $\pm$ 4.333 µs  | 48.458 $\pm$ 7.625 µs   | N/A                     |      |

## License

This project is licensed under the MIT OR Apache-2.0 License. See the [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) files for details.
