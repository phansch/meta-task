use std::fmt;
use std::process::Command;

use tmux_interface::{NewSession, SwitchClient};
use tmux_interface::TmuxInterface;

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

pub fn focus_tmux_session_and_branch(task_name: &str) -> Result<(), TaskError> {
    let tmux = TmuxInterface::new();

    let switch = SwitchClient {
        target_session: Some(task_name),
        ..Default::default()
    };
    if let Err(e) = tmux.switch_client(&switch) {
        return Err(TaskError { status: e.err_type, message: e.err_text, source: "tmux" });
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
