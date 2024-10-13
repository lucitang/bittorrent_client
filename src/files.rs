use anyhow::{Context, Error};
use std::fs;
use std::io::Write;
use std::path::Path;

pub fn write_file(file_path: &str, data: &[u8]) -> Result<(), Error> {
    // Check that the directory exists
    let parent_dir = Path::new(file_path).parent();
    if Some(parent_dir) != None {
        fs::create_dir_all(parent_dir.unwrap())?;
    }

    let mut file = fs::File::create(file_path).context("Creating file")?;
    file.write_all(data).context("Writing to file")?;
    Ok(())
}
