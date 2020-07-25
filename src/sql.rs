use crate::Task;
use rusqlite::types::ToSql;
use rusqlite::NO_PARAMS;
use rusqlite::{params, Connection, Result};
use std::str;
use std::io::{BufReader, BufRead, Error};
use std::fs::File;

/// Creates tables in SQLite Database
pub fn init(conn: &Connection) -> Result<()> {
    conn.execute(
        "create table if not exists tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT, status TEXT, name TEXT,
            notes TEXT, project TEXT, start TEXT,
            estimate INTEGER, repeat TEXT, next TEXT
         )",
        NO_PARAMS,
    )?;
    conn.execute(
        "create table if not exists log (
            id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, notes TEXT,
            project TEXT, date TEXT, start TEXT, end TEXT,
            estimate INTEGER, review TEXT
         )",
        NO_PARAMS,
    )?;

    Ok(())
}

fn execute_insert_query(conn: &Connection, query: &str, param_slice: &[&ToSql]) -> Result<()> {
    conn.execute(query, param_slice)?;

    Ok(())
}

pub fn add_task(conn: &Connection, t: Task) -> Result<()> {
    let query = "INSERT INTO tasks (status, name, notes, project, start,
        estimate, repeat, next)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7,?8)";

    let param_slice = params![
        t.status, t.name, t.notes, t.project, t.start, t.estimate, t.repeat, t.next
    ];
    execute_insert_query(conn, query, param_slice)?;

    Ok(())
}

pub fn task_vector_from_task_id(conn: &Connection, task_id: i32) -> Result<Vec<Task>> {
    let query = format!(
        "SELECT id, name, project, start, estimate, repeat, next,
                 notes, status FROM tasks WHERE id = '{}'",
        task_id
    );
    let task_vector = query_to_vec_task(conn, &query).unwrap();
    Ok(task_vector)
}

pub fn modify_date(conn: &Connection, task_id: i32, value: String) -> Result<()> {
    let query = format!(
        "UPDATE tasks SET next = '{}' WHERE id = '{}'",
        value, task_id
    );
    conn.execute(&query, NO_PARAMS)?;

    Ok(())
}

pub fn modify_project(conn: &Connection, task_id: i32, value: String) -> Result<()> {
    let query = format!(
        "UPDATE tasks SET project = '{}' WHERE id = '{}'",
        value, task_id
    );
    conn.execute(&query, NO_PARAMS)?;

    Ok(())
}

pub fn delete_task_by_id(conn: &Connection, id: i32) -> Result<()> {
    let query = format!("DELETE FROM tasks WHERE id={}", id);
    conn.execute(&query, NO_PARAMS)?;

    Ok(())
}

fn query_to_vec_task(conn: &Connection, query: &str) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare(&query)?;

    let task_iter = stmt.query_map(params![], |row| {
        Ok(Task {
            id: row.get(0)?,
            name: row.get(1)?,
            project: row.get(2)?,
            start: row.get(3)?,
            estimate: row.get(4)?,
            repeat: row.get(5)?,
            next: row.get(6)?,
            notes: row.get(7)?,
            status: row.get(8)?,
        })
    })?;

    let mut vec = Vec::new();
    for task in task_iter {
        let t = task.unwrap();
        vec.push(t);
    }

    Ok(vec)
}

pub fn filter_status_active(conn: &Connection) -> Result<Vec<Task>> {
    let query = "SELECT id, name, project, start, estimate, repeat, next,
                 notes, status FROM tasks WHERE status = 'ACTIVE'";
    let task_vector = query_to_vec_task(conn, query)?;

    Ok(task_vector)
}

pub fn filter_by_date(conn: &Connection, date: &str) -> Result<Vec<Task>> {
    let query = format!(
        "SELECT id, name, project, start, estimate,
                         repeat, next, notes, status FROM tasks
                         WHERE next = '{}' ORDER BY start",
        date
    );
    let task_vector = query_to_vec_task(conn, &query)?;

    Ok(task_vector)
}

pub fn filter_by_project(conn: &Connection, date: String) -> Result<Vec<Task>> {
    let query = format!(
        "SELECT id, name, project, start, estimate,
    repeat, next, notes, status FROM tasks
    WHERE project = '{}' ORDER BY start",
        date
    );

    let task_vector = query_to_vec_task(conn, &query)?;

    Ok(task_vector)
}

pub fn filter_by_routine(conn: &Connection) -> Result<Vec<Task>> {
    let query = "SELECT id, name, project, start, estimate,
    repeat, next, notes, status FROM tasks
    WHERE repeat <> '' ORDER BY start";

    let task_vector = query_to_vec_task(conn, &query)?;

    Ok(task_vector)
}

pub fn filter_by_repeat(conn: &Connection, date: String) -> Result<Vec<Task>> {
    let query = format!(
        "SELECT id, name, project, start, estimate,
    repeat, next, notes, status FROM tasks
    WHERE repeat = '{}' ORDER BY start",
        date
    );

    let task_vector = query_to_vec_task(conn, &query)?;

    Ok(task_vector)
}

pub fn generate_daily_plan(conn: &Connection, target_date: &str) -> Result<String> {
    let vec = filter_by_date(conn, target_date);
    let plan_string = vector_to_daily_plan(vec)?;

    Ok(plan_string)

}

fn vector_to_daily_plan(vec: Result<Vec<Task>>) -> Result<String> {
    let vec2 = vec.unwrap();
    let mut output_string = "".to_owned();
    for task in vec2 {
        let tmp = task.to_string().to_owned();
        output_string.push_str(&tmp);
    }

    Ok(output_string)
}

pub fn log_to_database(conn: &Connection, log_path: String)-> Result<(), Error> {

    let input = File::open(log_path)?;
    let buffered = BufReader::new(input);

    for line in buffered.lines() {
    }

    Ok(())
}