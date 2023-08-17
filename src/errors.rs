use std::process::exit;

use exitcode::ExitCode;

#[derive(Debug)]
pub struct AppError {
    pub msg: String,
    pub exitcode: ExitCode,
}

impl AppError {
    pub fn handle(&self) -> ! {
        eprintln!("{}", self.msg);
        exit(self.exitcode);
    }
}

pub type AppErrorResult<T> = Result<T, AppError>;
