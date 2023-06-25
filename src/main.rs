mod config;
mod todo;

use crate::config::Config;
use crate::todo::{Status as TaskStatus, TaskGroup};
use crate::todo::File as TodoFile;
use chrono::naive::NaiveDate;
use chrono::{Datelike, Local};
use comrak::nodes::{AstNode, NodeValue};
use comrak::{parse_document, Arena};
use comrak::{ComrakExtensionOptions, ComrakOptions, ComrakParseOptions};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::env;
use std::fs::{read, read_dir, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

//TODO handle unwraps and errors more uniformly
//TODO refactor creating new file
//TODO clean up verbose printing
//TODO create custom errors for better error handling

fn main() {
    let expected_cfg_files = Config::expected_locations().unwrap();
    println!("{:#?}", expected_cfg_files);
    let cfg_files: Vec<&Path> = expected_cfg_files
        .iter()
        .map(|file| Path::new(file))
        .filter(|file| file.exists())
        .collect();
    println!("{:#?}", cfg_files);

    if cfg_files.len() <= 0 {
        let status = Config::write_default(expected_cfg_files[0].to_str().unwrap());
        if let Err(e) = status {
            println!("Could not write to default cfg location: {:#?}", e);
        }
    }
    let cfg = Config::load(cfg_files.last().unwrap().to_str().unwrap()).unwrap();

    println!("{:#?}", cfg);
    let data_dir = get_data_dir(
        &cfg.notes_dir
            .clone()
            .expect("Could not get notes dir from config"),
    );
    println!("dir = {}", data_dir.to_str().unwrap());

    let latest_file = get_latest_file(&data_dir);
    println!("Latest file: {:?}", latest_file);

    let now = Local::now();
    let today = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap();
    let current_file = match latest_file {
        Ok(file) if file.date < today => {
            println!("Today's file does not exist, creating");

            let arena = Arena::new();
            let root = parse_todo_file(&file, &arena);

            //println!("{:#?}", root);
            //println!("=======================================================");

            let sections = &cfg.sections.unwrap();
            let groups = extract_secitons(root, sections);
            println!("{:#?}", groups);

            let level = groups.values().map(|group| group.level).min().unwrap_or(2);

            let data = sections
                .iter()
                .map(|section| match groups.get(section) {
                    Some(group) => group.clone(),
                    None => TaskGroup::empty(section.to_string(), level),
                })
                .collect();

            let new_file = write_file(&data_dir, &today, &data);

            Some(new_file)
        }
        Err(_) => {
            println!("No files in dir: {:}", cfg.notes_dir.unwrap());

            let sections = &cfg.sections.unwrap();
            let data = sections
                .iter()
                .map(|sec| TaskGroup::empty(sec.clone(), 2))
                .collect();

            let new_file = write_file(&data_dir, &today, &data);

            Some(new_file)
        }
        Ok(file) => {
            println!("Today's file was created");
            Some(file.file.path())
        }
    };

    if let Some(file) = current_file {
        Command::new(cfg.editor.expect("Could not resolve editor from config"))
            .args([file])
            .status()
            .expect(format!("failed to launch editor {}", "vim").as_str());
    };
}

fn write_file(data_dir: &PathBuf, date: &NaiveDate, data: &Vec<TaskGroup>) -> PathBuf {
    let mut content = format!(
        "# Today's tasks {}-{:02}-{:02}\n",
        date.year(),
        date.month(),
        date.day()
    );
    data.iter()
        .for_each(|task_group| content.push_str(format!("\n{}", task_group.to_string()).as_str()));

    let file_name = format!(
        "{}-{:02}-{:02}.md",
        date.year(),
        date.month(),
        date.day()
    );
    let mut file_path = data_dir.clone();
    file_path.push(file_name);

    let mut file = File::create(&file_path).expect("Could not open today's file: {today_file_path}");
    write!(file, "{}", content).expect("Could not write to file: {today_file_path}");

    file_path
}

fn parse_todo_file<'a>(file: &TodoFile, arena: &'a Arena<AstNode<'a>>) -> &'a AstNode<'a> {
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

    let contents_utf8 = read(file.file.path())
        .expect(format!("Could not read file {}", file.file.path().to_string_lossy()).as_str());
    let contents = str::from_utf8(&contents_utf8).expect(
        format!(
            "failed to convert contents of file to string: {}",
            file.file.path().to_string_lossy()
        )
        .as_str(),
    );

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
            let first_child = if let Some(child) = first_child_ref.borrow() {
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

            println!("Attempting to parse {}", title);
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

fn get_data_dir(dir_name: &str) -> PathBuf {
    let mut dir = match env::var("HOME") {
        Ok(home) => {
            let mut x = PathBuf::new();
            x.push(home);
            x
        }
        _ => env::current_dir().expect("PWD environment variable not set"),
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
