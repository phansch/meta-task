// TODO: Maybe assume a clean master branch checkout for now

use tmux_interface::TmuxInterface;
use dialoguer::Confirmation;

mod cli;
mod database;
mod tmux_git;

/// I want a system that helps me keep track of the various programming tasks I have
/// This is meant to be called from _inside_ a project, i.e. from `~/code/rust-clippy/`.

fn main() {
    let mut db = database::Database::from_disk();

    let matches = cli::build_cli().get_matches();

    // TODO: Do our stuff last, because we don't want to persist something and then undo it
    if let Some(matches) = matches.subcommand_matches("new") {
        let task_name = matches.value_of("task-name").unwrap();
        if db.task_exists(task_name) {
            eprintln!("Task '{}' already exists", task_name);
            std::process::exit(1);
        }
        match tmux_git::create_tmux_session_and_branch(task_name) {
            Ok(()) => {
                db.add_task(task_name);
                println!("Created new task: {}", task_name);
            },
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
    }
    if let Some(matches) = matches.subcommand_matches("done") {
        let task_name = matches.value_of("task-name").unwrap();
        if !db.task_exists(task_name) {
            eprintln!("Task '{}' not found", task_name);
            std::process::exit(1);
        }

        if Confirmation::with_theme(&dialoguer::theme::ColorfulTheme::default()).with_text("Are you sure are done with this task? This will remove the git branch and kill the tmux session!").interact().unwrap() {
            match tmux_git::delete_tmux_session_and_branch(task_name) {
                Ok(()) => {
                    db.remove_task(task_name);
                    println!("Task done: {}", task_name);
                },
                Err(err) => {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        } else {
            std::process::exit(0);
        }
    }
    if let Some(matches) = matches.subcommand_matches("focus") {
        let task_name = matches.value_of("task-name").unwrap();
        if !db.task_exists(task_name) {
            eprintln!("Task '{}' not found", task_name);
            std::process::exit(1);
        }
        match tmux_git::focus_tmux_session_and_branch(task_name) {
            Ok(()) => {},
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
    }
    if let Some(_) = matches.subcommand_matches("list") {
        let mut tmux = TmuxInterface::new();
        println!("{}", tmux.list_sessions(None).unwrap());
        for t in db.list_tasks() {
            println!("{}", t);
        }
    }
    db.save();
}
