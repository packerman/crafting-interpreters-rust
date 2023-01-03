use anyhow::{anyhow, Result};

pub fn error<T>(line: usize, message: &str) -> Result<T> {
    self::error_at(line, "", message)
}

pub fn error_at<T>(line: usize, where_part: &str, message: &str) -> Result<T> {
    Err(anyhow!("[line {}] Error{}: {}", line, where_part, message))
}
