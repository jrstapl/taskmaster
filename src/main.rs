use chrono::{Duration, Local, NaiveDate};
use std::env;
use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};

use crate::app::App;

use crate::todo::ToDo;

mod app;
mod event;
mod ui;

mod db;
mod input;
mod todo;

fn main() {
    let default_db_path: PathBuf = match env::current_exe() {
        Ok(exe_path) => {
            let this_file: PathBuf = exe_path.clone();
            // TODO: current_exe handles symlinks differently on different systems,
            // add check for is_symlink and handle appropriately if the returned path
            // is a symlink rather than the actual exe
            this_file.with_file_name("todo_db.csv")
        }
        Err(e) => panic!("{e} Unable to get current_exe path, not recoverable"),
    };

    let cmd = Command::new("Taskmaster")
        .author("Joshua Stapleton, joshrstapleton@gmail.com")
        .version("0.1.1")
        .about("TUI and CLI for ToDo list management")
        .arg(
            Arg::new("db_path")
                .short('p')
                .long("db_path")
                .aliases(["dbpath"])
                .help("Select which database to use")
                .required(false)
                .default_value(default_db_path.into_os_string())
                .value_parser(value_parser!(PathBuf))
                .global(true),
        )
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .help("Name of the ToDo item")
                .required(false)
                .value_parser(value_parser!(String))
                .default_value(String::new())
                .global(true),
        )
        .arg(
            Arg::new("report_to")
                .short('r')
                .long("report_to")
                .help("Who the of the ToDo item is reported to")
                .required(false)
                .value_parser(value_parser!(String))
                .default_value(String::new())
                .global(true),
        )
        .arg(
            Arg::new("due")
                .short('d')
                .long("due")
                .help("When the ToDo item should be completed by")
                .required(false)
                .value_parser(value_parser!(NaiveDate))
                .default_value((Local::now().date_naive() - Duration::days(1)).to_string())
                .global(true),
        )
        .arg(
            Arg::new("id")
                .long("id")
                .help("ID of the of the ToDo item, found with `taskmaster list`")
                .required(false)
                .value_parser(value_parser!(i32))
                .default_value("-1")
                .global(true),
        )
        .subcommand(Command::new("add").about("Add an item to the todo list"))
        .subcommand(Command::new("list").about("List current ToDos"))
        .subcommand(
            Command::new("complete").about("Mark item as complete, move to completed task list"),
        )
        .subcommand(
            Command::new("remove")
                .about("Remove an item from the ToDo list without marking as complete"),
        );

    let matches = cmd.clone().get_matches();

    let _db_path = match matches.get_one::<PathBuf>("db_path") {
        Some(db_path) => db_path,
        None => panic!("Unable to find a db path"),
    };

    match matches.subcommand() {
        Some(("add", _sub_matches)) => add_todo(&matches),
        Some(("list", _sub_matches)) => list_todo(&matches),
        Some(("complete", _sub_matches)) => mark_complete(&matches),
        Some(("remove", _sub_matches)) => remove_todo(&matches),
        _ => {
            gui_main();
        }
    }
}

#[tokio::main]
async fn gui_main() {
    color_eyre::install().expect("Unable to get color");
    let term = ratatui::init();
    let _ = App::new().run(term).await;
    ratatui::restore();
}

fn add_todo(matches: &ArgMatches) {
    let mut todo = ToDo::new();

    let todo_name: String = matches
        .get_one::<String>("name")
        .expect("name specifier must not be empty (default value should be given in parser)")
        .clone();

    let report_to: String = matches
        .get_one::<String>("report_to")
        .expect("report_to must not be empty (default value should be given in parser)")
        .clone();

    let due_date: NaiveDate = matches
        .get_one::<NaiveDate>("due")
        .expect("Due date must not be empty (default value should be given in parser)")
        .clone();

    todo.set_name(todo_name);
    todo.set_report_to(report_to);
    todo.set_due(due_date);

    let conn = db::get_todo_db_conn();

    db::add_todo_to_db(&todo, &conn);
}

fn list_todo(matches: &ArgMatches) {
    let conn = db::get_todo_db_conn();

    let mut todo_name: String = matches
        .get_one::<String>("name")
        .expect("name specifier must not be empty (default value should be given in parser)")
        .clone();

    todo_name.push('%');
    todo_name.insert(0, '%');

    let todo_list = db::get_todo_list_from_db(&conn, &todo_name);

    for todo in todo_list {
        println!("{:?}", todo);
    }
}

fn mark_complete(matches: &ArgMatches) {
    // these happen in order to make sure that the todo is not removed until it is sucessfully
    // added to completed

    // first we ensure we can get the paths and open connection
    let todo_db_conn = db::get_todo_db_conn();
    let completed_db_conn = db::get_completed_db_conn();

    // Use the actual id to delete because name might be long
    let todo_id: i32 = matches
        .get_one::<i32>("id")
        .expect("id specifier must not be empty (default value should be given in parser)")
        .clone();
    db::mark_todo_complete(todo_id, &todo_db_conn, &completed_db_conn);
}

fn remove_todo(matches: &ArgMatches) {
    let todo_id: i32 = matches
        .get_one::<i32>("id")
        .expect("id specifier must not be empty (default value should be given in parser)")
        .clone();

    if todo_id < 0 {
        println!(
            "Must remove a todo by id, use `taskmaster list` to find the id of the todo you'd like to remove"
        );
        return;
    }

    let conn = db::get_todo_db_conn();

    match db::remove_todo_from_db(todo_id, &conn) {
        Ok(updated) => println!("{} rows were updated", updated),
        Err(err) => println!("update failed: {}", err),
    }
    conn.close().expect("Unable to close db...");
}
