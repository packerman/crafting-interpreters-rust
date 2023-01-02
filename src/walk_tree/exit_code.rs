use std::process::ExitCode;

// Source: https://www.freebsd.org/cgi/man.cgi?query=sysexits&apropos=0&sektion=0&manpath=FreeBSD+4.3-RELEASE&format=html

pub fn usage() -> ExitCode {
    ExitCode::from(64)
}

pub fn data_err() -> ExitCode {
    ExitCode::from(65)
}
