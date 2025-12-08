use std::{fs::File, io::Read};

use anyhow::Result;
use ndarray::Array2;

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

pub fn parse_lines_from_file<T>(
    day: u8,
    example: bool,
    parser: fn(&str) -> Result<T>,
) -> Result<Vec<T>> {
    let content = read_file(day, example)?;
    content.lines().map(parser).collect()
}

pub fn parse_comma_separated_from_file<T>(
    day: u8,
    example: bool,
    parser: fn(&str) -> Result<T>,
) -> Result<Vec<T>> {
    let content = read_file(day, example)?;
    content
        .trim()
        .split(',')
        .map(|s| parser(s.trim()))
        .collect()
}

pub fn parse_grid_from_file<T>(
    day: u8,
    example: bool,
    parser: fn(char) -> Result<T>,
) -> Result<Array2<T>> {
    let content = read_file(day, example)?;
    let grid = content
        .lines()
        .map(|line| line.chars().map(parser).collect())
        .collect::<Result<Vec<Vec<T>>>>()?;
    let row_count = grid.len();
    let col_count = grid.first().map_or(0, Vec::len);
    let flat_data = grid.into_iter().flatten().collect::<Vec<T>>();
    Ok(Array2::from_shape_vec((row_count, col_count), flat_data)?)
}
