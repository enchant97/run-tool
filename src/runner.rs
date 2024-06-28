use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

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

    /// Same as `run_interactive()` however will allow the process to be canceled
    pub fn run_interactive_cancelable(&self, cancel: &AtomicBool) -> AppErrorResult<ExitCode> {
        let mut cmd = Command::new(&self.program);
        cmd.envs(&self.vars).args(&self.args);
        if let Some(cwd) = &self.cwd {
            cmd.current_dir(cwd);
        }
        let mut child_process = cmd.spawn().map_err(|err| AppError {
            msg: format!("{}", err),
            exitcode: exitcode::SOFTWARE,
        })?;
        loop {
            match child_process.try_wait() {
                Ok(Some(status)) => return Ok(status.code().unwrap_or_default()),
                Ok(None) => {
                    if cancel.load(Ordering::Relaxed) {
                        child_process.kill().map_err(|_| AppError {
                            msg: String::from("process could not be killed"),
                            exitcode: exitcode::OSERR,
                        })?;
                        return Ok(exitcode::OK);
                    }
                    ()
                }

                Err(_) => {
                    return Err(AppError {
                        msg: String::from("failure"),
                        exitcode: exitcode::OSERR,
                    })
                }
            }
            sleep(Duration::from_millis(1));
        }
    }
}
