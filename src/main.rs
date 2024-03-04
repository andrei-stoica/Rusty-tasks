mod cli;
mod config;
mod file;
mod todo;

use crate::cli::Args;
use crate::config::Config;
use crate::todo::TaskGroup;
use chrono::naive::NaiveDate;
use chrono::{Datelike, Local};
use clap::Parser;
use comrak::Arena;
use resolve_path::PathResolveExt;
use std::fs;
use std::path::Path;
use std::process::Command;

//TODO refactor creating new file

fn main() {
    let args = Args::parse();

    let expected_cfg_files = match Config::expected_locations() {
        Ok(cfg_files) => cfg_files,
        Err(e) => panic!("{:?}", e),
    };

    let cfg_files: Vec<&Path> = expected_cfg_files
        .iter()
        .map(|file| Path::new(file))
        .filter(|file| file.exists())
        .collect();

    if cfg_files.len() <= 0 {
        if let Err(e) = Config::write_default(match expected_cfg_files[0].to_str() {
            Some(s) => s,
            None => panic!("Could not resolve expected cfg file paths"),
        }) {
            panic!("Could not write config: {:?}", e);
        }
    }

    let cfg_file = match args.config {
        Some(file) => file,
        None => match cfg_files.last() {
            None => expected_cfg_files[0].to_string_lossy().to_string(),
            Some(file) => file.to_string_lossy().to_string(),
        },
    };

    if args.current_config {
        println!("{}", &cfg_file);
        return;
    }

    let cfg = match Config::load(&cfg_file) {
        Ok(cfg) => cfg,
        Err(_e) => panic!("could not load config: {}", cfg_file),
    };

    let data_dir = cfg.notes_dir.resolve().to_path_buf();

    if !fs::metadata(&data_dir).is_ok() {
        match fs::create_dir_all(&data_dir) {
            Err(_e) => panic!("Could not create defult directory: {:?}", &data_dir),
            _ => (),
        };
    }

    let latest_file = file::get_latest_file(&data_dir);

    let now = Local::now();
    let today = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap();
    let current_file = match latest_file {
        Ok(todo_file) if todo_file.date < today => {
            let arena = Arena::new();
            let root = {
                let contents = file::load_file(&todo_file);
                let root = file::parse_todo_file(&contents, &arena);
                root
            };

            let sections = &cfg.sections;

            let groups = file::extract_secitons(root, sections);

            let level = groups.values().map(|group| group.level).min().unwrap_or(2);

            let data = sections
                .iter()
                .map(|section| match groups.get(section) {
                    Some(group) => group.clone(),
                    None => TaskGroup::empty(section.to_string(), level),
                })
                .collect();

            let content = file::generate_file_content(&data, &today);
            let file_path = file::get_filepath(&data_dir, &today);
            file::write_file(&file_path, &content);
            file_path
        }
        Err(_) => {
            let sections = &cfg.sections;
            let data = sections
                .iter()
                .map(|sec| TaskGroup::empty(sec.clone(), 2))
                .collect();
            let content = file::generate_file_content(&data, &today);
            let file_path = file::get_filepath(&data_dir, &today);
            file::write_file(&file_path, &content);
            file_path
        }
        Ok(todo_file) => todo_file.file.path(),
    };

    Command::new(cfg.editor)
        .args([current_file])
        .status()
        .expect(format!("failed to launch editor {}", "vim").as_str());
}
