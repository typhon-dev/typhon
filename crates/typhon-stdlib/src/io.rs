//! Input/Output functionality for the Typhon language.

use std::fs::File;
use std::io::{self, Read, Write};

/// Read the contents of a file as a string.
pub fn read_file(path: &str) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

/// Write a string to a file.
pub fn write_file(path: &str, content: &str) -> Result<(), io::Error> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Read a line of text from standard input.
pub fn input(prompt: &str) -> Result<String, io::Error> {
    if !prompt.is_empty() {
        print!("{prompt}");
        io::stdout().flush()?;
    }

    let mut line = String::new();
    io::stdin().read_line(&mut line)?;

    // Trim the trailing newline
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }

    Ok(line)
}
