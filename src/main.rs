mod todo_file;

use crate::todo_file::TodoFile;
use chrono::naive::NaiveDate;
use chrono::{Datelike, Local};
use std::env;
use std::fs::{copy, read_dir};
use std::path::{Path, PathBuf};
use std::process::Command;

//TODO handle unwraps and errors more uniformly
//TODO clean up verbose printing

fn main() {
    let data_dir = get_data_dir("notes");
    println!("{}", data_dir.to_str().unwrap());

    let latest_file =
        get_latest_file(&data_dir).expect(format!("Could not find any notes files").as_str());
    println!("Latest file: {:?}", latest_file);

    let mut editor = Command::new(get_editor("vim".to_string()));

    let now = Local::now();
    let today = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day());
    match today {
        Some(today) if latest_file.date < today => {
            println!("Today's file does not exist, creating");
            let today_file_name = format!(
                "{}-{:02}-{:02}.md",
                today.year(),
                today.month(),
                today.day()
            );
            let mut today_file_path = data_dir.clone();
            today_file_path.push(today_file_name);

            copy(latest_file.file.path(), today_file_path.clone()).unwrap();

            editor
                .args([today_file_path])
                .status()
                .expect(format!("failed to launch editor {}", "vim").as_str());
        }
        Some(_) => {
            println!("Todays file was created");
            editor
                .args([latest_file.file.path()])
                .status()
                .expect(format!("failed to launch editor {}", "vim").as_str());
        }
        _ => println!("Could not get today's date"),
    }
}

fn get_editor(fallback: String) -> String {
    match env::var("EDITOR") {
        Ok(editor) => editor,
        _ => fallback,
    }
}

fn get_data_dir(dir_name: &str) -> PathBuf {
    let mut dir = if let Ok(home) = env::var("HOME") {
        let mut x = PathBuf::new();
        x.push(home);
        x
    } else {
        env::current_dir().expect("PWD environment variable not set")
    };
    dir = dir.join(dir_name);
    dir
}

fn get_latest_file(dir: &Path) -> Result<TodoFile, String> {
    let dir = read_dir(dir).expect(format!("Could not find notes folder: {:?}", dir).as_str());
    dir.filter_map(|f| f.ok())
        .filter_map(|file| TodoFile::try_from(file).ok())
        .reduce(|a, b| TodoFile::latest_file(a, b))
        .ok_or("Could not reduce items".to_string())
}
