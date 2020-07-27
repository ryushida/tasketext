use rusqlite::{Connection, Result};
use serde_derive::Deserialize;
use std::fmt;
use std::fs;
use structopt::StructOpt;
use toml;

mod datetime;
mod interface;
mod sql;

extern crate chrono;

#[derive(Debug, Clone)]
pub struct Task {
    pub id: i32,
    pub status: String,
    pub name: String,
    pub notes: String,
    pub project: String,
    pub start: String,
    pub estimate: i32,
    pub repeat: String,
    pub next: String,
}

#[derive(Debug)]
pub struct Log {
    pub id: i32,
    pub name: String,
    pub notes: String,
    pub project: String,
    pub date: String,
    pub start: String,
    pub end: String,
    pub review: String,
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "- {} ({}) [{}] {}： {}",
            self.start, self.estimate, self.project, self.name, self.notes
        )
    }
}

impl fmt::Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "- {} ({})", self.id, self.name)
    }
}

#[derive(Deserialize)]
struct Config {
    main_dir: String,
    database_file_name: String,
}

// Define inputs
#[derive(StructOpt)]
struct Cli {
    /// Config File Path: -c "config.toml"
    #[structopt(short = "c", long = "config_file")]
    config: String,
}

fn main() -> Result<()> {
    // Read command line arguments
    let args = Cli::from_args();

    let filename = args.config.as_str();
    let contents = fs::read_to_string(filename).expect("Error");
    let config: Config = toml::from_str(&contents).unwrap();

    let main_dir = config.main_dir.to_string();
    let database_file_name = config.database_file_name;
    let database_path = main_dir.clone() + &database_file_name;

    let conn = Connection::open(database_path)?;
    sql::init(&conn).unwrap();

    interface::main_menu(&conn, main_dir)?;

    Ok(())
}
