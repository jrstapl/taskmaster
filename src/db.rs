use std::env;
use std::fs;
use std::path::PathBuf;

use chrono::{Local, NaiveDate};
use rusqlite::Connection;

use crate::todo::ToDo;

fn get_todui_data_dir() -> PathBuf {
    let home_name: PathBuf = match env::home_dir() {
        Some(p) => p,
        None => panic!(
            "Home directory not found! Make sure $HOME is set on Linux/MacOS or %USERPROFILE% on Windows",
        ),
    };

    let app_dir = home_name.join(".local/todui");
    if !app_dir.exists() {
        fs::create_dir_all(app_dir.clone()).expect("Unable to create $HOME/.local/todui");
    }
    return app_dir;
}

fn get_todui_db_dir() -> PathBuf {
    let app_dir = get_todui_data_dir();

    let db_dir = app_dir.join("db");
    if !db_dir.exists() {
        fs::create_dir_all(db_dir.clone()).expect("Unable to create $HOME/.local/db");
    };

    return db_dir;
}

pub fn get_todo_db_conn() -> Connection {
    let db_dir = get_todui_db_dir();

    let db_file = db_dir.join("todo.db");
    let conn = initialize_todo_db(&db_file);

    return conn;
}

pub fn get_completed_db_conn() -> Connection {
    let db_dir = get_todui_db_dir();

    let db_file = db_dir.join("completed.db");
    let conn = initialize_todo_db(&db_file);

    return conn;
}

fn initialize_todo_db(db_file: &PathBuf) -> Connection {
    let conn = Connection::open(db_file).expect("Unable to open db connection");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            created_on TEXT NOT NULL,
            due_on TEXT NOT NULL,
            report_to TEXT
        );",
        (),
    )
    .expect("Unable to execute table creation");
    return conn;
}

pub fn get_todo_list_from_db(conn: &Connection, name: &String) -> Vec<ToDo> {
    let mut stmt = conn
        .prepare("SELECT * FROM todos WHERE name LIKE ?1")
        .expect("Unable to prepare db query");
    let todo_iter = stmt
        .query_map([name], |row| {
            let id: i32 = row.get(0)?;
            let name: String = row.get(1)?;
            let created_on: String = row.get(2)?;
            let due_on: String = row.get(3)?;
            let report_to = row.get(4)?;
            Ok(ToDo {
                id: id,
                name: name,
                created: NaiveDate::parse_from_str(&created_on, "%Y-%m-%d")
                    .unwrap_or(Local::now().date_naive()),
                due: NaiveDate::parse_from_str(&due_on, "%Y-%m-%d").unwrap(),
                report_to: report_to,
            })
        })
        .expect("Unable to create query map");

    let mut todo_list = Vec::new();
    for todo in todo_iter {
        todo_list.push(todo.unwrap());
    }

    return todo_list;
}

pub fn remove_todo_from_db(todo_id: i32, conn: &Connection) -> Result<usize, rusqlite::Error> {
    return conn.execute("DELETE FROM todos WHERE rowid=?1", [todo_id]);
}

pub fn add_todo_to_db(todo: &ToDo, conn: &Connection) {
    conn.execute(
        "INSERT INTO todos (name, created_on, due_on, report_to) VALUES (?1, ?2, ?3, ?4)",
        (
            &todo.name,
            &todo.created.to_string(),
            &todo.due.to_string(),
            &todo.report_to,
        ),
    )
    .unwrap();
}

pub fn mark_todo_complete(todo_id: i32, todo_db_conn: &Connection, completed_db_conn: &Connection) {
    let todo: ToDo = todo_db_conn
        .query_row("SELECT * FROM todos WHERE rowid = ?1", [todo_id], |row| {
            let row_id: i32 = row.get(0)?;
            let name: String = row.get(1)?;
            let created_on: String = row.get(2)?;
            let due_on: String = row.get(3)?;
            let report_to = row.get(4)?;
            Ok(ToDo {
                id: row_id,
                name: name,
                created: NaiveDate::parse_from_str(&created_on, "%Y-%m-%d")
                    .unwrap_or(Local::now().date_naive()),
                due: NaiveDate::parse_from_str(&due_on, "%Y-%m-%d").unwrap(),
                report_to: report_to,
            })
        })
        .expect(format! {"Unable to get ToDo for id: {}",todo_id}.as_str());

    // then add todo to completed db
    add_todo_to_db(&todo, &completed_db_conn);

    // now we can remove from todo db
    let _ = remove_todo_from_db(todo.id.clone(), &todo_db_conn);
}
