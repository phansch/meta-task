use std::fmt;
use std::process::Command;

use tmux_interface::{TmuxInterface, AttachSession, NewSession, SwitchClient};

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

#[allow(dead_code)]
fn attach_session(task_name: &str) -> Result<(), TaskError> {
    println!("AttachSession");
    let mut tmux = TmuxInterface::new();

    let attach = AttachSession {
        target_session: Some(task_name),
        detach_other: Some(false),
        cwd: None,
        not_update_env: Some(false),
        read_only: Some(false)
    };
    match tmux.attach_session(Some(&attach)) {
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
            return Err(TaskError { status: 100, message: e.message, source: "tmux" });
        }
    }
    Ok(())
}

pub fn focus_tmux_session_and_branch(task_name: &str) -> Result<(), TaskError> {
    if in_tmux_session() {
        switch_to_session(task_name)?;
    } else {
        return attach_session(task_name);
    }
    Ok(())
}

fn switch_to_session(task_name: &str) -> Result<(), TaskError> {
    let mut tmux = TmuxInterface::new();

    let switch = SwitchClient {
        target_session: Some(task_name),
        ..Default::default()
    };
    if let Err(e) = tmux.switch_client(Some(&switch)) {
        return Err(TaskError { status: 100, message: e.message, source: "tmux" });
    }
    Ok(())
}

pub fn create_tmux_session_and_branch(task_name: &str) -> Result<(), TaskError> {
    let mut tmux = TmuxInterface::new();

    let new_session = NewSession {
        session_name: Some(task_name),
        detached: Some(true),
        ..Default::default()
    };
    if let Err(e) = tmux.new_session(Some(&new_session)) {
        return Err(TaskError { status: 100, message: e.message, source: "tmux" });
    }

    if in_tmux_session() {
        switch_to_session(task_name)?;
    } else {
        return attach_session(task_name);
    }

    // TODO: Handle already existing branch, do nothing
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
    let mut tmux = TmuxInterface::new();


    match tmux.kill_session(Some(false), Some(false), Some(task_name)) {
        Ok(output) => {
            println!("{}", &output.status.code().unwrap());
            Ok(())
        },
        Err(e) => {
            return Err(TaskError { status: 100, message: e.message, source: "tmux" });
        }
    }
    // TODO:
    // 0. Check what happens when marking current task as done
    // 1. If git repo, check that working directory is not dirty
    // 2. Detach from tmux session
    // 3. checkout master
    // 4. remove the branch
}

/// Returns true if executed inside a tmux session
pub fn in_tmux_session() -> bool {
    match env::var("TMUX") {
        Err(VarError::NotPresent) => {
            // If we're not connected to a tmux session, we have to attach first.
            // (We can't switch to it)

            return false
        },
        Err(VarError::NotUnicode(_)) => {
            eprintln!("Bailing, TMUX env var contains non-Unicode characters");
            std::process::exit(1);
        },
        Ok(val) => {
            if val.is_empty() {
                return false;
            } else {
                return true;
            }
        }
    }
}
