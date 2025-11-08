//! Built-in functions and types for the Typhon language.

/// Print a value to standard output.
pub fn print(value: &str) {
    println!("{}", value);
}

/// Convert a value to a string.
pub fn str<T: std::fmt::Display>(value: T) -> String {
    format!("{}", value)
}
