use std::fmt;
use std::process::Command;

use tmux_interface::{AttachSession, NewSession, SwitchClient};
use tmux_interface::TmuxInterface;

use std::env::VarError;
use std::env;

pub struct TaskError<'a> {
    status: usize,
    message: String,
    source: &'a str,
}

impl<'a> fmt::Display for TaskError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} (exit-status: {}, source: {})", self.message, self.status, self.source)
    }
}

fn attach_session(task_name: &str) -> Result<(), TaskError> {
    println!("AttachSession");
    let tmux = TmuxInterface::new();

    let attach = AttachSession {
        target_session: Some(task_name),
        detach_other: Some(false),
        cwd: None,
        not_update_env: Some(false),
        read_only: Some(false)
    };
    match tmux.attach_session(&attach) {
        Ok(output) => {
            if !output.status.success() {
                return Err(
                    TaskError {
                        status: output.status.code().unwrap_or(1) as usize,
                        message: std::str::from_utf8(&output.stderr).unwrap().trim().to_string(),
                        source: "tmux"
                    }
                );
            }
        },
        Err(e) => {
            return Err(TaskError { status: e.err_type, message: e.err_text, source: "tmux" });
        }
    }
    Ok(())
}

pub fn focus_tmux_session_and_branch(task_name: &str) -> Result<(), TaskError> {
    let tmux = TmuxInterface::new();

    match env::var("TMUX") {
        Err(VarError::NotPresent) => {
            // If we're not connected to a tmux session, we have to attach first.
            // (We can't switch to it)

            return attach_session(task_name);
        },
        Err(VarError::NotUnicode(_)) => {
            eprintln!("Bailing, TMUX env var contains non-Unicode characters");
            std::process::exit(1);
        },
        Ok(val) => {
            if val.is_empty() {
                return attach_session(task_name);
            } else {
                println!("SwitchClient");
                let switch = SwitchClient {
                    target_session: Some(task_name),
                    ..Default::default()
                };
                if let Err(e) = tmux.switch_client(&switch) {
                    return Err(TaskError { status: e.err_type, message: e.err_text, source: "tmux" });
                }
            }
        }
    }
    Ok(())
}

pub fn create_tmux_session_and_branch(task_name: &str) -> Result<(), TaskError> {
    let tmux = TmuxInterface::new();

    let new_session = NewSession {
        session_name: Some(task_name),
        detached: Some(true),
        ..Default::default()
    };
    if let Err(e) = tmux.new_session(&new_session) {
        return Err(TaskError { status: e.err_type, message: e.err_text, source: "tmux" });
    }

    let output = Command::new("git")
        .arg("checkout")
        .arg("-b")
        .arg(task_name)
        .output()
        .expect("Failed to run git checkout -b");
    if let Some(code) = output.status.code() {
        if code != 0 {
            let message = format!("git failed to create branch ({})", std::str::from_utf8(&output.stderr).unwrap().trim());
            return Err(TaskError { status: code as usize, message, source: "git" });
        }
    }

    Ok(())
}

pub fn delete_tmux_session_and_branch(task_name: &str) -> Result<(), TaskError> {
    let tmux = TmuxInterface::new();

    match tmux.kill_session(Some(true), Some(false), Some(task_name)) {
        Ok(output) => {
            println!("{}", &output.status.code().unwrap());
            Ok(())
        },
        Err(e) => {
            return Err(TaskError { status: e.err_type, message: e.err_text, source: "tmux" });
        }
    }
    // TODO:
    // 0. Check what happens when marking current task as done
    // 1. If git repo, check that working directory is not dirty
    // 2. Detach from tmux session
    // 3. checkout master
    // 4. remove the branch
}
