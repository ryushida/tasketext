use crate::Task;
use rusqlite::types::ToSql;
use rusqlite::NO_PARAMS;
use rusqlite::{params, Connection, Result};
use std::str;
use std::io::{BufReader, BufRead, Error};
use std::fs::File;
use std::fmt;

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

#[derive(Debug)]
pub struct LogItem {
    pub name: String,
    pub notes: String,
    pub project: String,
    pub date: String,
    pub start: String,
    pub end: String,
    pub estimate: i32,
    pub review: String,
}

impl LogItem {
    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn set_notes(&mut self, notes: String) {
        self.notes = notes;
    }

    fn set_project(&mut self, project: String) {
        self.project = project;
    }

    fn set_estimate(&mut self, estimate: i32) {
        self.estimate = estimate;
    }

    fn set_start(&mut self, start: String) {
        self.start = start;
    }

    fn set_end(&mut self, end: String) {
        self.end = end;
    }

    fn set_review(&mut self, review: String) {
        self.review = review;
    }
}

impl Default for LogItem {
    fn default () -> LogItem {
        LogItem{
            name: "".to_string(),
            notes: "".to_string(),
            project: "".to_string(),
            date: "".to_string(),
            start: "".to_string(),
            end: "".to_string(),
            estimate: 0,
            review: "".to_string(),
        }
    }
}

impl fmt::Display for LogItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {} {} {} {} {} {}", self.name, self.notes, self.project, self.estimate, self.start, self.end, self.review)
    }
}

pub fn log_to_database(conn: &Connection, log_path: String, date: String) -> Result<(), Error> {

    let input = File::open(log_path)?;
    let buffered = BufReader::new(input);

    let mut one_log = LogItem::default();
    one_log.date = date;

    for line in buffered.lines() {
        let l = match line {
            Ok(line) => line,
            Err(err) => panic!("Error reading line"),
        };
        if l.starts_with("- ") {
            if one_log.start != "" && one_log.end != "" {
                logitem_to_database(conn, &mut one_log);
                reset_review_fields(&mut one_log);
            }
            process_task_line(l, &mut one_log);
        } else if l.starts_with("  -") {
            process_indented_line(l, &mut one_log);
        }
    }
    logitem_to_database(conn, &mut one_log);

    Ok(())
}

fn logitem_to_database(conn: &Connection, one_log: &mut LogItem) -> Result<()> {

    let query = "INSERT INTO log (name, notes, project, date,
        start, end, estimate, review) VALUES
        (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)";
    let param_slice = params![
        one_log.name, one_log.notes, one_log.project, one_log.date, one_log.start,
        one_log.end, one_log.estimate, one_log.review
    ];
    execute_insert_query(conn, query, param_slice)?;

    Ok(())
}

fn process_task_line(line: String, one_log: &mut LogItem) -> Result<()> {

    reset_time_fields(one_log);

    one_log.set_name(get_text_between(&line, "]", "：")?);
    one_log.set_notes(get_text_after(&line, "：")?);
    one_log.set_project(get_text_between(&line, "[", "]")?);

    let estimate_int = match get_text_between(&line, "(", ")").unwrap().parse::<i32>() {
        Ok(estimate) => estimate,
        Err(_) => panic!("Failed to read estimate for task {}", one_log.name),
    };

    one_log.set_estimate(estimate_int);

    Ok(())
    
}

fn reset_time_fields(one_log: &mut LogItem) -> Result<()> {
    one_log.set_start("".to_string());
    one_log.set_end("".to_string());
    Ok(())
}

fn reset_review_fields(one_log: &mut LogItem) -> Result<()> {
    one_log.set_review("".to_string());
    Ok(())
}

fn process_indented_line(line: String, one_log: &mut LogItem) -> Result<()> {
    if get_text_after(&line, "  - ")?.trim().len() <= 5 {
        let time = get_text_after(&line, "  - ")?;
        if one_log.start == "" {
            one_log.set_start(time);
        } else {
            one_log.set_end(time);
        }
    } else {
        one_log.set_review(get_text_after(&line, "  - ")?);
    }
    Ok(())
}

fn get_text_between(s: &str, left: &str, right: &str) -> Result<String> {
    let start_bytes = s.find(left).unwrap_or(0) + 1;
    let end_bytes = s.find(right).unwrap_or(s.len());
    let result = &s[start_bytes..end_bytes];
    Ok(result.trim().to_string())
}

fn get_text_after(s: &str, beginning: &str) -> Result<String> {
    let v: Vec<&str> = s.split(beginning).collect();
    Ok(v[1].trim().to_string())
}