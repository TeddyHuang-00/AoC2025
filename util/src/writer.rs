//! Writer for writing data to a file in a specific format.

use std::{
    fs::{self, File},
    io::Write,
    marker::PhantomData,
};

use anyhow::Result;

use super::get_workspace_root;
use crate::timer::BenchmarkResult;

pub struct FileWriter {
    file: File,
}

impl FileWriter {
    fn new(day: u8, extension: impl AsRef<str>) -> Result<Self> {
        let path = get_workspace_root()?.join(format!(
            "outputs/benchmark-day{day:02}.{}",
            extension.as_ref().trim_matches('.')
        ));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = File::create(path)?;
        Ok(Self { file })
    }

    fn write(&mut self, data: &[u8]) -> Result<()> {
        self.file.write_all(data)?;
        Ok(())
    }
}

/// Trait for CSV entries.
pub trait CsvEntry {
    fn columns() -> Vec<String>;
    fn values(&self) -> Vec<String>;
}

pub struct CsvWriter<T: CsvEntry> {
    file_writer: FileWriter,
    _marker: PhantomData<T>,
}

impl<T: CsvEntry> CsvWriter<T> {
    pub fn new(day: u8) -> Result<Self> {
        let file_writer = FileWriter::new(day, "csv")?;
        let mut instance = Self {
            file_writer,
            _marker: PhantomData,
        };
        instance.write_line(&T::columns().join(","))?;
        Ok(instance)
    }

    fn write_line(&mut self, line: &str) -> Result<()> {
        self.file_writer.write(line.as_bytes())?;
        self.file_writer.write(b"\n")?;
        Ok(())
    }

    pub fn write_entry(&mut self, entry: &T) -> Result<()> {
        self.write_line(&entry.values().join(","))?;
        Ok(())
    }
}

pub trait Serializable {
    fn to_csv(&self, day: u8) -> Result<()>;
}

impl<T: AsRef<[BenchmarkResult]>> Serializable for T {
    fn to_csv(&self, day: u8) -> Result<()> {
        let mut writer = CsvWriter::new(day)?;
        for result in self.as_ref() {
            writer.write_entry(result)?;
        }
        Ok(())
    }
}
