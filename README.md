# tasketext
Task Management Tool built with Rust and a SQLite Database.

Features
- Daily Recurring Tasks
- Generate Daily Plans in a Markdown File
- Markdown Reports
- Works Completely Offline

````config.toml````
```toml
main_dir = "C:\\tasks\\"
database_file_name = "mydatabase.db"
```

You can perform operations on tasks and generate plans/reports from the command line:

`.\tasketext.exe --config_file "C:\tasks\config.toml"`
