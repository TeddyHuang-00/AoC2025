//! Common reading and parsing utilities

use std::{fs::File, io::Read};

use anyhow::Result;
use ndarray::Array2;

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

/// Convert a nested Vec (`Vec<Vec<T>>`) into a 2D ndarray `Array2<T>`
///
/// # Errors
/// This function will return an error if the nested Vec does not have a
/// consistent number of columns in each row.
fn nested_vec_to_array2<T>(grid: Vec<Vec<T>>) -> Result<Array2<T>> {
    let row_count = grid.len();
    let col_count = grid.first().map_or(0, Vec::len);
    let flat_data = grid.into_iter().flatten().collect::<Vec<T>>();
    Ok(Array2::from_shape_vec((row_count, col_count), flat_data)?)
}

/// Read the input file for a given day and example flag
///
/// # Errors
/// This function will return an error if:
/// - the day is not between 1 and 25, or
/// - the workspace root cannot be determined, or
/// - the file cannot be read.
pub fn read_file(day: u8, example: bool) -> Result<String> {
    if day == 0 || day > 25 {
        anyhow::bail!("Day must be between 1 and 25");
    }
    let file_path = get_workspace_root()?.join(format!(
        "inputs/day{:02}{}.txt",
        day,
        if example { "-example" } else { "" }
    ));
    let mut file = File::open(&file_path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to open file '{}': {}",
            file_path.to_string_lossy(),
            e
        )
    })?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Parse lines of input using a provided parser function
///
/// # Errors
/// This function will return any errors produced by the parser function.
pub fn parse_lines<T, E>(
    input: impl AsRef<str>,
    parser: fn(&str) -> Result<T, E>,
) -> Result<Vec<T>, E> {
    input.as_ref().lines().map(parser).collect()
}

/// Parse comma-separated values using a provided parser function
///
/// # Errors
/// This function will return any errors produced by the parser function.
pub fn parse_comma_separated<T, E>(
    input: impl AsRef<str>,
    parser: fn(&str) -> Result<T, E>,
) -> Result<Vec<T>, E> {
    input
        .as_ref()
        .trim()
        .split(',')
        .map(|s| parser(s.trim()))
        .collect()
}

/// Parse whitespace-separated values using a provided parser function
///
/// # Errors
/// This function will return an error if the parser function returns an error.
pub fn parse_whitespace_separated<T, E>(
    input: impl AsRef<str>,
    parser: fn(&str) -> Result<T, E>,
) -> Result<Vec<T>, E> {
    input.as_ref().split_whitespace().map(parser).collect()
}

/// Parse a grid of characters using a provided parser function
///
/// # Errors
/// This function will return an error if:
/// - any line has a different number of columns, or
/// - the parser function returns an error.
pub fn parse_char_grid<T, E>(
    input: impl AsRef<str>,
    parser: fn(char) -> Result<T, E>,
) -> Result<Array2<T>>
where
    E: Into<anyhow::Error>,
{
    let content = input.as_ref();
    let grid = content
        .lines()
        .map(|line| line.chars().map(parser).collect())
        .collect::<Result<Vec<Vec<T>>, E>>()
        .map_err(Into::into)?;
    nested_vec_to_array2(grid)
}

/// Parse a grid of whitespace-separated values using a provided parser function
///
/// # Errors
/// This function will return an error if:
/// - any line has a different number of columns, or
/// - the parser function returns an error.
pub fn parse_grid<T, E>(
    input: impl AsRef<str>,
    parser: fn(&str) -> Result<T, E>,
) -> Result<Array2<T>>
where
    E: Into<anyhow::Error>,
{
    let content = input.as_ref();
    let grid = content
        .lines()
        .map(|line| parse_whitespace_separated(line, parser))
        .collect::<Result<Vec<Vec<T>>, E>>()
        .map_err(Into::into)?;
    nested_vec_to_array2(grid)
}

/// Parse a fixed-width grid using a provided parser function.
///
/// The widths of each column must be specified.
///
/// # Errors
/// This function will return an error if:
/// - the specified column widths do not match the input data, or
/// - the parser function returns an error, or
/// - the resulting nested Vec cannot be converted into an Array2.
pub fn parse_fixed_width_grid<T, E>(
    input: impl AsRef<str>,
    column_widths: impl AsRef<[usize]>,
    parser: fn(&str) -> Result<T, E>,
) -> Result<Array2<T>>
where
    E: Into<anyhow::Error>,
{
    let content = input.as_ref();
    let column_widths = column_widths.as_ref();
    let grid = content
        .lines()
        .map(|line| {
            let mut cols = Vec::with_capacity(column_widths.len());
            let mut start = 0;
            for &width in column_widths {
                if start >= line.len() {
                    anyhow::bail!("Line is shorter than expected based on column widths");
                }
                let end = start + width;
                let slice = &line[start..end];
                cols.push(parser(slice).map_err(Into::into)?);
                start = end;
            }
            // Handle any remaining characters in the line as the last column
            if start < line.len() {
                cols.push(parser(&line[start..]).map_err(Into::into)?);
            }
            Ok(cols)
        })
        .collect::<Result<Vec<Vec<T>>>>()?;
    nested_vec_to_array2(grid)
}

#[cfg(test)]
mod tests {
    use ndarray::prelude::*;

    use super::*;

    fn int_parser(s: &str) -> Result<i32> {
        s.parse().map_err(Into::into)
    }

    #[test]
    fn test_read_file() {
        // We don't test successful file reading here since it depends on external
        // files. Instead, we test error handling for invalid days
        let result = read_file(0, false);
        assert!(result.is_err());
        let result = read_file(26, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_vec_to_array2() {
        let vec = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let array = nested_vec_to_array2(vec)
            .unwrap_or_else(|e| panic!("Failed to convert nested vec to array2: {e}"));
        assert_eq!(array.shape(), &[2, 3]);
        assert_eq!(array, array![[1, 2, 3], [4, 5, 6]]);

        let vec_inconsistent = vec![vec![1, 2], vec![3, 4, 5]];
        let result = nested_vec_to_array2(vec_inconsistent);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_lines() {
        let input = "1\n2\n3";
        let result =
            parse_lines(input, int_parser).unwrap_or_else(|e| panic!("Failed to parse lines: {e}"));
        assert_eq!(result, vec![1, 2, 3]);

        let input_invalid = "1\ntwo\n3";
        let result = parse_lines(input_invalid, int_parser);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_comma_separated() {
        let input = "1,2, 3";
        let result = parse_comma_separated(input, int_parser)
            .unwrap_or_else(|e| panic!("Failed to parse comma-separated values: {e}"));
        assert_eq!(result, vec![1, 2, 3]);

        let input_invalid = "1, two,3";
        let result = parse_comma_separated(input_invalid, int_parser);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_whitespace_separated() {
        let input = "1  2\t3";
        let result = parse_whitespace_separated(input, int_parser)
            .unwrap_or_else(|e| panic!("Failed to parse whitespace-separated values: {e}"));
        assert_eq!(result, vec![1, 2, 3]);

        let input_invalid = "1 two 3";
        let result = parse_whitespace_separated(input_invalid, int_parser);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_grid() {
        let input = "1 2 3\n4 5 6\n7 8 9";
        let array =
            parse_grid(input, int_parser).unwrap_or_else(|e| panic!("Failed to parse grid: {e}"));
        assert_eq!(array.shape(), &[3, 3]);
        assert_eq!(array, array![[1, 2, 3], [4, 5, 6], [7, 8, 9]]);

        let input_invalid = "1 2 3\n4 five 6\n7 8 9";
        let result = parse_grid(input_invalid, int_parser);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_char_grid() {
        let input = "abc\ndef\nghi";
        let array = parse_char_grid(input, anyhow::Ok)
            .unwrap_or_else(|e| panic!("Failed to parse char grid: {e}"));
        assert_eq!(array.shape(), &[3, 3]);
        assert_eq!(
            array,
            array![['a', 'b', 'c'], ['d', 'e', 'f'], ['g', 'h', 'i']]
        );

        let input_invalid = "abc\ndef\ngh";
        let result = parse_char_grid(input_invalid, anyhow::Ok);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_fixed_width_grid() {
        let input = "12 345 6789 9\n01 234 5678 8";
        let column_widths = vec![3, 4, 4, 2];
        let array = parse_fixed_width_grid(input, &column_widths, |s| int_parser(s.trim()))
            .unwrap_or_else(|e| panic!("Failed to parse fixed-width grid: {e}"));
        assert_eq!(array.shape(), &[2, 4]);
        assert_eq!(array, array![[12, 345, 6789, 9], [1, 234, 5678, 8]]);

        let implicit_widths = vec![3, 4, 5];
        let array = parse_fixed_width_grid(input, &implicit_widths, |s| int_parser(s.trim()))
            .unwrap_or_else(|e| {
                panic!("Failed to parse fixed-width grid with implicit last column: {e}")
            });
        assert_eq!(array.shape(), &[2, 4]);
        assert_eq!(array, array![[12, 345, 6789, 9], [1, 234, 5678, 8]]);

        let input_invalid = "12 345 6789\n01 234 5678 8";
        let result =
            parse_fixed_width_grid(input_invalid, &column_widths, |s| int_parser(s.trim()));
        assert!(result.is_err());
    }
}
