mod cli;
mod config;
mod todo;

use crate::cli::Args;
use clap::Parser;

use crate::config::Config;
use crate::todo::File as TodoFile;
use crate::todo::{Status as TaskStatus, TaskGroup};
use chrono::naive::NaiveDate;
use chrono::{Datelike, Local};
use comrak::nodes::{AstNode, NodeValue};
use comrak::{parse_document, Arena};
use comrak::{ComrakExtensionOptions, ComrakOptions, ComrakParseOptions};
use resolve_path::PathResolveExt;
use std::collections::HashMap;
use std::fs::{create_dir_all, metadata, read, read_dir, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

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

    if !metadata(&data_dir).is_ok() {
        match create_dir_all(&data_dir) {
            Err(_e) => panic!("Could not create defult directory: {:?}", &data_dir),
            _ => (),
        };
    }

    let latest_file = get_latest_file(&data_dir);

    let now = Local::now();
    let today = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap();
    let current_file = match latest_file {
        Ok(todo_file) if todo_file.date < today => {
            let arena = Arena::new();
            let root = {
                let contents = load_file(&todo_file);
                let root = parse_todo_file(&contents, &arena);
                root
            };

            let sections = &cfg.sections;

            let groups = extract_secitons(root, sections);

            let level = groups.values().map(|group| group.level).min().unwrap_or(2);

            let data = sections
                .iter()
                .map(|section| match groups.get(section) {
                    Some(group) => group.clone(),
                    None => TaskGroup::empty(section.to_string(), level),
                })
                .collect();

            let content = generate_file_content(&data, &today);
            let file_path = get_filepath(&data_dir, &today);
            write_file(&file_path, &content);
            file_path
        }
        Err(_) => {
            let sections = &cfg.sections;
            let data = sections
                .iter()
                .map(|sec| TaskGroup::empty(sec.clone(), 2))
                .collect();
            let content = generate_file_content(&data, &today);
            let file_path = get_filepath(&data_dir, &today);
            write_file(&file_path, &content);
            file_path
        }
        Ok(todo_file) => todo_file.file.path(),
    };

    Command::new(cfg.editor)
        .args([current_file])
        .status()
        .expect(format!("failed to launch editor {}", "vim").as_str());
}

fn get_filepath(data_dir: &PathBuf, date: &NaiveDate) -> PathBuf {
    let file_name = format!("{}-{:02}-{:02}.md", date.year(), date.month(), date.day());
    let mut file_path = data_dir.clone();
    file_path.push(file_name);

    file_path
}

fn generate_file_content(data: &Vec<TaskGroup>, date: &NaiveDate) -> String {
    let mut content = format!(
        "# Today's tasks {}-{:02}-{:02}\n",
        date.year(),
        date.month(),
        date.day()
    );
    data.iter()
        .for_each(|task_group| content.push_str(format!("\n{}", task_group.to_string()).as_str()));

    content
}

fn write_file(path: &PathBuf, content: &String) {
    let mut new_file = File::create(&path).expect("Could not open today's file: {today_file_path}");
    write!(new_file, "{}", content).expect("Could not write to file: {today_file_path}");
}

fn load_file(file: &TodoFile) -> String {
    let contents_utf8 = read(file.file.path())
        .expect(format!("Could not read file {}", file.file.path().to_string_lossy()).as_str());
    str::from_utf8(&contents_utf8)
        .expect(
            format!(
                "failed to convert contents of file to string: {}",
                file.file.path().to_string_lossy()
            )
            .as_str(),
        )
        .to_string()
}

fn parse_todo_file<'a>(contents: &String, arena: &'a Arena<AstNode<'a>>) -> &'a AstNode<'a> {
    let options = &ComrakOptions {
        extension: ComrakExtensionOptions {
            tasklist: true,
            ..ComrakExtensionOptions::default()
        },
        parse: ComrakParseOptions {
            relaxed_tasklist_matching: true,
            ..ComrakParseOptions::default()
        },
        ..ComrakOptions::default()
    };
    parse_document(arena, contents, options)
}

fn extract_secitons<'a>(
    root: &'a AstNode<'a>,
    sections: &Vec<String>,
) -> HashMap<String, TaskGroup> {
    let mut groups: HashMap<String, TaskGroup> = HashMap::new();
    for node in root.reverse_children() {
        let node_ref = &node.data.borrow();
        if let NodeValue::Heading(heading) = node_ref.value {
            if heading.level < 2 {
                continue;
            }

            let first_child_ref = &node.first_child();
            let first_child = if let Some(child) = first_child_ref {
                child
            } else {
                continue;
            };

            let data_ref = &first_child.data.borrow();
            let title = if let NodeValue::Text(value) = &data_ref.value {
                value
            } else {
                continue;
            };

            if sections.iter().any(|section| section.eq(title)) {
                if let Ok(mut group) = TaskGroup::try_from(node) {
                    group.tasks = group
                        .tasks
                        .into_iter()
                        .filter(|task| !matches!(task.status, TaskStatus::Done(_)))
                        .collect();
                    groups.insert(title.to_string(), group);
                }
            }
        };
    }
    groups
}

fn get_latest_file(dir: &Path) -> Result<TodoFile, String> {
    let dir = read_dir(dir).expect(format!("Could not find notes folder: {:?}", dir).as_str());
    dir.filter_map(|f| f.ok())
        .filter_map(|file| TodoFile::try_from(file).ok())
        .reduce(|a, b| TodoFile::latest_file(a, b))
        .ok_or("Could not reduce items".to_string())
}
