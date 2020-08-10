use crate::sql;
use crate::Task;
use comfy_table::presets::ASCII_MARKDOWN;
use dialoguer::Input;
use dialoguer::{theme::ColorfulTheme, Select, Confirm};
use rusqlite::{Connection, Result};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Error, Write};
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
};
use term_table::{Table, TableStyle};

use crate::datetime;
use crate::Log;
use crate::Note;
use std::path::Path;

pub fn main_menu(conn: &Connection, main_dir: String) -> Result<()> {
    let selected = &[
        "Add a Task to Today's Plan",
        "Add a Task",
        "View Tasks",
        "Generate Plan",
        "Markdown Log to Database",
        "Generate Daily Report",
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
        Ok(5) => generate_daily_report(&conn, main_dir)?,
        Ok(6) => (),
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
    let id = sql::get_last_id(conn)?;
    sql::add_note(conn, id, next.trim(), "", notes.trim())?;

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

    print_task_vector(&task_vector)?;
    select_task_action(conn, &task_vector)?;

    Ok(())
}

fn select_task_action(conn: &Connection, task_vector: &Vec<Task>) -> Result<()> {
    let selected = &["Perform action on tasks", "quit"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select")
        .items(&selected[..])
        .default(0)
        .interact()
        .unwrap();

    if selection == 0 {
        bulk_edit_menu(conn, &task_vector)?;
    }

    Ok(())
}

fn bulk_edit_menu(conn: &Connection, task_vector: &Vec<Task>) -> Result<()> {
    let mut all_ids: Vec<i32> = Vec::new();
    for entry in task_vector.iter() {
        all_ids.push(entry.id);
    }

    let selections = user_input("Enter 'all' or space seperated ID numbers");

    let mut selected_ids = Vec::new();
    if selections == "all" {
        selected_ids = all_ids;
    } else {
        let mapped = selections.split_whitespace().map(|s| s.parse::<i32>().unwrap());
        selected_ids = mapped.collect();
    }

    multiple_task_actions_menu(conn, &selected_ids)?;

    let modified_tasks = sql::filter_by_id(conn, selected_ids)?;
    print_task_vector(&modified_tasks)?;

    Ok(())
}

fn multiple_task_actions_menu(conn: &Connection, id_vector: &Vec<i32>) -> Result<()> {
    let selected = &[
        "Modify Date",
        "Modify Start Time",
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
        Ok(0) => user_input_bulk_edit_date(conn, &id_vector)?,
        Ok(1) => user_input_bulk_edit_start(conn, &id_vector)?,
        Ok(2) => user_input_bulk_edit_project(conn, &id_vector)?,
        Ok(3) => user_input_bulk_edit_notes(conn, &id_vector)?,
        Ok(4) => user_input_bulk_edit_estimates(conn, &id_vector)?,
        Ok(5) => bulk_delete(conn, &id_vector)?,
        Ok(_) => println!("Something went wrong"),
        Err(_err) => println!("Error"),
    }

    Ok(())
}

fn user_input_bulk_edit_date(conn: &Connection, id_vec: &Vec<i32>) -> Result<()> {
    let date = user_input_date("New Date");

    for id in id_vec.iter() {
        sql::modify_date(conn, id, &date)?;
    }

    Ok(())
}

fn user_input_bulk_edit_start(conn: &Connection, id_vec: &Vec<i32>) -> Result<()> {
    let start = user_input("New Start Time");

    for id in id_vec.iter() {
        sql::modify_start(conn, id, &start)?;
    }

    Ok(())
}

fn user_input_bulk_edit_project(conn: &Connection, id_vec: &Vec<i32>) -> Result<()> {
    let project = user_input("New Project");

    for id in id_vec.iter() {
        sql::modify_project(conn, id, &project)?;
    }

    Ok(())
}

fn user_input_bulk_edit_notes(conn: &Connection, id_vec: &Vec<i32>) -> Result<()> {
    let notes = sql::get_all_notes(conn, &id_vec.to_vec())?;
    
    print_note_vector(&notes)?;

    Ok(())
}

fn user_input_bulk_edit_estimates(conn: &Connection, id_vec: &Vec<i32>) -> Result<()> {
    let notes = user_input("Estimates");

    for id in id_vec.iter() {
        sql::modify_estimates(conn, id, &notes)?;
    }

    Ok(())
}

fn bulk_delete(conn: &Connection, id_vec: &Vec<i32>) -> Result<()> {
    for id in id_vec.iter() {
        sql::delete_task_by_id(conn, id)?;
    }
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
    
    if Path::new(&file_path).exists() {
        let message = format!("Do you want to overwrite {}?", &file_path);
        if Confirm::new().with_prompt(message).interact().unwrap() {
            save_string_to_file(plan_string, &file_path)?;
        } else {
            println!("nevermind then :(");
        }
    } else {
        println!("{}", file_path);
        save_string_to_file(plan_string, &file_path)?;
    }

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
fn print_task_vector(task_vector: &Vec<Task>) -> Result<()> {
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
            TableCell::new_with_alignment(&t.id, 1, Alignment::Left),
            TableCell::new_with_alignment(&t.name, 1, Alignment::Left),
            TableCell::new_with_alignment(&t.project, 2, Alignment::Center),
            TableCell::new_with_alignment(&t.next, 2, Alignment::Center),
        ]));
    }
    println!("{}", table.render());

    Ok(())
}

/// Prints notes given a vector with Note structures
fn print_note_vector(note_vector: &Vec<Note>) -> Result<()> {
    let mut table = Table::new();
    table.style = TableStyle::extended();
    table.set_max_column_widths(vec![(0, 5), (1, 40), (2, 31), (3, 10)]);
    table.add_row(Row::new(vec![
        TableCell::new_with_alignment("ID", 1, Alignment::Left),
        TableCell::new_with_alignment("Note", 1, Alignment::Center),
        TableCell::new_with_alignment("Name", 2, Alignment::Center),
        TableCell::new_with_alignment("Start Date", 2, Alignment::Center),
    ]));
    for note in note_vector {
        let n = note;
        table.add_row(Row::new(vec![
            TableCell::new_with_alignment(&n.id, 1, Alignment::Left),
            TableCell::new_with_alignment(&n.notetext, 1, Alignment::Left),
            TableCell::new_with_alignment(&n.name, 2, Alignment::Left),
            TableCell::new_with_alignment(&n.start, 2, Alignment::Center),
            
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

fn generate_daily_report(conn: &Connection, dir: String) -> Result<()> {
    let date_vec = datetime::days_range(-7, 1);
    let date_slice: &[String] = &date_vec;

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Generate Plan for Date")
        .items(date_slice)
        .default(6)
        .interact();

    let mut n = -1;
    match &selection {
        Ok(0) => n = -7,
        Ok(1) => n = -6,
        Ok(2) => n = -5,
        Ok(3) => n = -4,
        Ok(4) => n = -3,
        Ok(5) => n = -2,
        Ok(6) => n = -1,
        Ok(7) => n = 0,
        Ok(_) => println!("Something went wrong"),
        Err(_err) => println!("Error"),
    }

    let log_vector = sql::daily_report_log_vector(conn, selection, date_slice)?;
    let table_string = log_vector_to_markdown_table_string(log_vector);

    let filename = datetime::yyyymmdd_today_plus_n(n).replace("-", "");
    let path = format!("{}{}{}{}", dir, "log\\", filename, "_log.md");

    save_string_to_file(table_string, &path)?;

    Ok(())
}

fn log_vector_to_markdown_table_string(log_vector: Vec<Log>) -> String {
    let mut table = comfy_table::Table::new();
    table
        .load_preset(ASCII_MARKDOWN)
        .set_header(vec!["Start", "End", "Duration", "Task", "Review"]);

    for log in log_vector {
        let mut tmp_vec = Vec::new();
        tmp_vec.push(&log.start);
        tmp_vec.push(&log.end);
        let duration = datetime::get_duration(&log.start, &log.end);
        tmp_vec.push(&duration);
        tmp_vec.push(&log.name);
        tmp_vec.push(&log.review);
        table.add_row(tmp_vec);
    }

    table.to_string()
}
