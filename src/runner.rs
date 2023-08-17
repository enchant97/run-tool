use std::process::Command;

use exitcode::ExitCode;

use crate::errors::{AppError, AppErrorResult};
use crate::helpers::EnvVars;

pub struct ProcessRunner {
    pub program: String,
    pub args: Vec<String>,
    pub vars: EnvVars,
    pub cwd: Option<String>,
}

impl ProcessRunner {
    /// Runs the process interactively allowing user to see stdout and use stdin
    pub fn run_interactive(&self) -> AppErrorResult<ExitCode> {
        let mut cmd = Command::new(&self.program);
        cmd.envs(&self.vars).args(&self.args);
        if let Some(cwd) = &self.cwd {
            cmd.current_dir(cwd);
        }
        Ok(cmd
            .spawn()
            .map_err(|err| AppError {
                msg: format!("{}", err),
                exitcode: exitcode::SOFTWARE,
            })?
            .wait()
            .map_err(|_| AppError {
                msg: String::from("failed to execute"),
                exitcode: exitcode::OSERR,
            })?
            .code()
            .unwrap_or_default())
    }
}
