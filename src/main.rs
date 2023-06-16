mod todo_file;

use crate::todo_file::TodoFile;
use chrono::naive::NaiveDate;
use chrono::{Datelike, Local};
use comrak::nodes::{AstNode, NodeValue};
use comrak::{
    format_commonmark, parse_document, Arena, ComrakExtensionOptions, ComrakOptions,
    ComrakParseOptions,
};
use std::borrow::Borrow;
use std::env;
use std::fs::{read, read_dir, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

//TODO handle unwraps and errors more uniformly
//TODO clean up verbose printing
//TODO create config for passing options to different files

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

            let arena = Arena::new();
            let root = parse_todo_file(&latest_file, &arena);
            cleanup_sections(&root, &["Hello world!", "Hi there"], 2);

            let mut new_doc = vec![];
            format_commonmark(root, &ComrakOptions::default(), &mut new_doc).unwrap();
            let mut new_file = File::create(today_file_path.clone()).unwrap();
            new_file.write_all(&new_doc).unwrap();

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

fn cleanup_sections<'a>(
    root: &'a AstNode<'a>,
    sections: &[&str],
    target_level: u8,
) -> &'a AstNode<'a> {
    for node in root.children() {
        let node_ref = &node.data.borrow();
        if let NodeValue::Heading(heading) = node_ref.value {
            if heading.level != target_level {
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

            if !sections.contains(&title.as_str()) {
                let level = heading.level;

                let mut following = node.following_siblings();
                following.next(); // Skip self
                for node in following {
                    // remove everthing under this heading
                    match &node.data.borrow().value {
                        NodeValue::Heading(heading) if heading.level <= level => break,
                        _ => node.detach(),
                    }
                }
                node.detach(); // remove heading as well
            }
        };
    }
    root
}

fn get_editor(fallback: String) -> String {
    match env::var("EDITOR") {
        Ok(editor) => editor,
        _ => fallback,
    }
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
