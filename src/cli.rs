use clap::{Arg, App, SubCommand, AppSettings};

pub fn build_cli() -> App<'static, 'static> {
    App::new("meta-task")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(SubCommand::with_name("new")
            .about("creates a new task")
            .display_order(1)
            .arg(
                Arg::with_name("task-name")
                .help("the name of the task")
                .required(true)
            ))
        .subcommand(SubCommand::with_name("focus")
            .about("focuses on a task")
            .display_order(2)
            .help("This will open the tmux session and checkout the branch")
            .arg(
                Arg::with_name("task-name")
                .help("the name of the task")
                .required(true)
            ))
        .subcommand(SubCommand::with_name("done")
            .about("finishes a task")
            .display_order(3)
            .help("This will checkout the master branch and delete this branch and stop the tmux session")
            .arg(
                Arg::with_name("task-name")
                .help("the name of the task")
                .required(true)
            ))
        .subcommand(SubCommand::with_name("list")
            .about("lists all known tasks"))
}
