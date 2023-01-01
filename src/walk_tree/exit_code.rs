use std::process::ExitCode;

pub fn usage() -> ExitCode {
    ExitCode::from(64)
}

pub fn data_err() -> ExitCode {
    ExitCode::from(65)
}
