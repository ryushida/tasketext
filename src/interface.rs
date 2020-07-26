use crate::sql;
use crate::Task;
use dialoguer::Input;
use dialoguer::{theme::ColorfulTheme, Select};
use rusqlite::{Connection, Result};
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
};
use term_table::{Table, TableStyle};
use std::fs::File;
use std::io::{Write, Error};
use std::fs::OpenOptions;

use crate::datetime;

pub fn main_menu(conn: &Connection, main_dir: String) -> Result<()> {
    let selected = &[
        "Add a Task to Today's Plan",
        "Add a Task",
        "View Tasks",
        "Generate Plan",
        "Markdown Log to Database",
        "quit",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Action")
        .items(&selected[..])
        .default(0)
        .interact();

    match selection {
        Ok(0) => add_task_today(main_dir)?,
        Ok(1) => call_add_task(&conn)?,
        Ok(2) => view_tasks_menu(&conn)?,
        Ok(3) => call_generate_daily_plan(&conn, main_dir)?,
        Ok(4) => markdown_log_to_database(&conn, main_dir)?,
        Ok(5) => (),
        Ok(_) => println!("Something went wrong"),
        Err(_err) => println!("Error"),
    }

    Ok(())
}

/// Ask user for input and return entered String
/// Displays given string
fn user_input(displayed_text: &str) -> String {
    let value: String = Input::new()
        .with_prompt(displayed_text)
        .validate_with(|input: &str| -> Result<(), &str> {
            if !input.is_empty() {
                Ok(())
            } else {
                Err("This is not a valid input")
            }
        })
        .interact()
        .unwrap();
    value
}

fn user_input_allow_empty(displayed_text: &str) -> String {
    let value: String = Input::new()
        .allow_empty(true)
        .with_prompt(displayed_text)
        .interact()
        .unwrap();
    value
}

fn user_input_repeat_string() -> String {
    let repeat: String = Input::new()
        .allow_empty(true)
        .with_prompt("Repeat")
        .validate_with(|input: &str| -> Result<(), &str> {
            if input.contains('+') {
                Ok(())
            } else if input.is_empty() {
                Ok(())
            } else {
                Err("This is not a valid repeat string")
            }
        })
        .interact()
        .unwrap();

    repeat
}

/// Displays given string
fn user_input_date(displayed_text: &str) -> String {
    let date: String = Input::new()
        .with_prompt(displayed_text)
        .validate_with(|input: &str| -> Result<(), &str> {
            if input.contains("2020") {
                Ok(())
            } else {
                Err("This is not a 2020 date")
            }
        })
        .interact()
        .unwrap();

    date
}

fn user_input_int(displayed_text: &str) -> i32 {
    let value: i32 = Input::new().with_prompt(displayed_text).interact().unwrap();
    value
}

fn add_task_today(dir: String) -> Result<()> {

    println!("Adding Task...");
    let name = user_input("Name");
    let notes = user_input_allow_empty("Notes");
    let project = user_input("Project");
    let start = user_input("Start Time");
    let estimate = user_input_int("Estimate (Minutes)");

    let t = Task {
        id: 0,
        status: "ACTIVE".to_string(),
        name: name.trim().to_string(),
        notes: notes.trim().to_string(),
        project: project.trim().to_string(),
        start: start.trim().to_string(),
        estimate: estimate,
        repeat: "".to_string(),
        next: "".to_string(),
    };

    let today = datetime::yyyymmdd_today_plus_n(0).replace("-", "");
    let file_path = [&dir, &today, ".md"].join("");

    append_line_to_file(&file_path, t.to_string())?;

    Ok(())
}

fn append_line_to_file(path: &str, line: String) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(path)
        .unwrap();

    if let Err(e) = writeln!(file, "\n{}", line) {
        eprintln!("Couldn't write to file: {}", e);
    }

    Ok(())
}

fn call_add_task(conn: &Connection) -> Result<()> {
    println!("Adding Task...");
    let name = user_input("Name");
    let notes = user_input_allow_empty("Notes");
    let project = user_input("Project");
    let start = user_input("Start Time");
    let estimate = user_input_int("Estimate (Minutes)");
    let repeat = user_input_repeat_string();
    let next = user_input_date("Date");

    let t = Task {
        id: 0,
        status: "ACTIVE".to_string(),
        name: name.trim().to_string(),
        notes: notes.trim().to_string(),
        project: project.trim().to_string(),
        start: start.trim().to_string(),
        estimate: estimate,
        repeat: repeat.trim().to_string(),
        next: next.trim().to_string(),
    };

    sql::add_task(conn, t)?;

    Ok(())
}

/// Presents user with options for viewing tasks
fn view_tasks_menu(conn: &Connection) -> Result<()> {
    let selected = &[
        "Active Tasks",
        "Filter by Due Date",
        "Filter by Project",
        "Filter Routines",
        "Filter by Repeat",
        "quit",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("View Tasks")
        .items(&selected[..])
        .default(0)
        .interact()
        .unwrap();

    match selection {
        0 => filter_by_print(&conn, Ok("active"))?,
        1 => filter_by_print(&conn, Ok("date"))?,
        2 => filter_by_print(&conn, Ok("project"))?,
        3 => filter_by_print(&conn, Ok("routine"))?,
        4 => filter_by_print(&conn, Ok("repeat"))?,
        5 => (),
        _ => println!("Something went wrong"),
    }

    user_input_action_id(&conn)?;

    Ok(())
}

fn filter_by_print(conn: &Connection, by: Result<&str>) -> Result<()> {
    let task_vector = match by {
        Ok("active") => sql::filter_status_active(conn)?,
        Ok("date") => sql::filter_by_date(conn, &user_input_date("Date"))?,
        Ok("project") => sql::filter_by_project(conn, user_input("Project"))?,
        Ok("routine") => sql::filter_by_routine(conn)?,
        Ok("repeat") => sql::filter_by_repeat(conn, user_input("Repeat"))?,
        Ok(_) => panic!(),
        Err(_err) => panic!(),
    };

    print_task_vector(task_vector)?;

    Ok(())
}

fn user_input_action_id(conn: &Connection) -> Result<()> {
    let multiselected = &["Perform Action on a Task", "quit"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select")
        .items(&multiselected[..])
        .default(0)
        .interact()
        .unwrap();

    if selection == 0 {
        let task_id = user_input_int("Task ID");
        task_actions_menu(conn, task_id)?;
    }

    Ok(())
}

fn task_actions_menu(conn: &Connection, task_id: i32) -> Result<()> {
    let task_vector = sql::task_vector_from_task_id(conn, task_id)?;
    print_task_vector(task_vector)?;

    let selected = &[
        "Modify Date",
        "Modify Project",
        "Modify Notes",
        "Modify Estimates",
        "Delete Task",
        "quit",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Action")
        .items(&selected[..])
        .interact();

    match selection {
        Ok(0) => user_input_modify_date(conn, task_id)?,
        Ok(1) => user_input_modify_project(conn, task_id)?,
        Ok(4) => sql::delete_task_by_id(conn, task_id)?,
        Ok(_) => println!("Something went wrong"),
        Err(_err) => println!("Error"),
    }

    Ok(())
}

fn user_input_modify_date(conn: &Connection, task_id: i32) -> Result<()> {
    let value = user_input_date("Date");
    sql::modify_date(conn, task_id, value)?;
    let task_vector = sql::task_vector_from_task_id(conn, task_id)?;
    print_task_vector(task_vector)?;
    Ok(())
}

fn user_input_modify_project(conn: &Connection, task_id: i32) -> Result<()> {
    let value = user_input("Project");
    sql::modify_project(conn, task_id, value)?;
    let task_vector = sql::task_vector_from_task_id(conn, task_id)?;
    print_task_vector(task_vector)?;
    Ok(())
}

fn call_generate_daily_plan(conn: &Connection, dir: String) -> Result<()> {
    let date_vec = datetime::days_range(-1, 5);
    let date_slice: &[String] = &date_vec;

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Generate Plan for Date")
        .items(date_slice)
        .default(1)
        .interact();

    let n = match selection {
        Ok(0) => -1,
        Ok(1) => 0,
        Ok(2) => 1,
        Ok(3) => 2,
        Ok(4) => 3,
        Ok(_) => panic!("Something wrong"),
        Err(_err) => panic!("Something Wrong"),
    };

    let target_date = datetime::yyyymmdd_today_plus_n(n);
    let plan_string = sql::generate_daily_plan(conn, &target_date)?;    

    let file_path = [dir, target_date.replace("-", ""), ".md".to_string()].join("");
    println!("{}", file_path);
    save_string_to_file(plan_string, &file_path)?;

    Ok(())
}

/// Save given string at given path
fn save_string_to_file(s: String, path: &str) -> Result<()> {
    let mut file = match File::create(path) {
        Err(why) => panic!("couldn't create: {}", why),
        Ok(file) => file,
    };
    match file.write_all(s.as_bytes()) {
        Err(why) => panic!("couldn't write: {}", why),
        Ok(_) => println!("successfully wrote: {:?}", file),
    }
    Ok(())
}

/// Prints tasks given a vector with Task structures
fn print_task_vector(task_vector: Vec<Task>) -> Result<()> {
    let mut table = Table::new();
    table.style = TableStyle::extended();
    table.add_row(Row::new(vec![
        TableCell::new_with_alignment("ID", 1, Alignment::Left),
        TableCell::new_with_alignment("Name", 1, Alignment::Left),
        TableCell::new_with_alignment("Project", 2, Alignment::Center),
        TableCell::new_with_alignment("Date", 2, Alignment::Center),
    ]));
    for task in task_vector {
        let t = task;
        table.add_row(Row::new(vec![
            TableCell::new_with_alignment(t.id, 1, Alignment::Left),
            TableCell::new_with_alignment(t.name, 1, Alignment::Left),
            TableCell::new_with_alignment(t.project, 2, Alignment::Center),
            TableCell::new_with_alignment(t.next, 2, Alignment::Center),
        ]));
    }
    println!("{}", table.render());

    Ok(())
}

fn markdown_log_to_database(conn: &Connection, dir: String) -> Result<()> {
    let date = user_input_date("Date to save to database");
    let log_filename = date.replace("-", "") + ".md";
    let log_path = dir + &log_filename;
    sql::log_to_database(conn, log_path, date).ok();

    Ok(())
}