//! Input/Output functionality for the Typhon language.

use std::fs::File;
use std::io::{Error as IOError, Read, Write, stdin, stdout};

/// Read the contents of a file as a string.
pub fn read_file(path: &str) -> Result<String, IOError> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    let _ = file.read_to_string(&mut content)?;

    Ok(content)
}

/// Write a string to a file.
pub fn write_file(path: &str, content: &str) -> Result<(), IOError> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;

    Ok(())
}

/// Read a line of text from standard input.
pub fn input(prompt: &str) -> Result<String, IOError> {
    if !prompt.is_empty() {
        print!("{prompt}");
        stdout().flush()?;
    }

    let mut line = String::new();
    let _ = stdin().read_line(&mut line)?;

    // Trim the trailing newline
    if line.ends_with('\n') {
        let _ = line.pop();

        if line.ends_with('\r') {
            let _ = line.pop();
        }
    }

    Ok(line)
}
