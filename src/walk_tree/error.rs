use anyhow::{anyhow, Result};

pub fn error<T>(line: usize, message: &str) -> Result<T> {
    self::report(line, "", message)
}

fn report<T>(line: usize, where_part: &str, message: &str) -> Result<T> {
    Err(anyhow!("[line {}] Error{}: {}", line, where_part, message))
}

pub fn try_until_ok<T, F>(mut f: F) -> Option<T>
where
    F: FnMut() -> Option<Result<T>>,
{
    loop {
        match f() {
            Some(result) => match result {
                Ok(value) => return Some(value),
                Err(err) => eprintln!("{}", err),
            },
            None => return None,
        }
    }
}
