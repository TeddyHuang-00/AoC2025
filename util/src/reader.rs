use std::{fs::File, io::Read};

use anyhow::Result;

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

pub fn parse_lines_from_file<T>(
    day: u8,
    example: bool,
    parser: fn(&str) -> Result<T>,
) -> Result<Vec<T>> {
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
    contents.lines().map(parser).collect()
}
