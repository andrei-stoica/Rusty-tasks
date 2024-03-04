use chrono::Datelike;
use crate::TaskGroup;
use crate::NaiveDate;
use crate::todo::{Status as TaskStatus,File as TodoFile};
use comrak::nodes::{AstNode, NodeValue};
use comrak::{Arena, ComrakExtensionOptions, ComrakOptions, ComrakParseOptions};
use std::str;
use std::io::Write;
use std::collections::HashMap;
use comrak::parse_document;
use std::path::{Path, PathBuf};
use std::fs::{read, read_dir, File};

pub fn get_filepath(data_dir: &PathBuf, date: &NaiveDate) -> PathBuf {
    let file_name = format!("{}-{:02}-{:02}.md", date.year(), date.month(), date.day());
    let mut file_path = data_dir.clone();
    file_path.push(file_name);

    file_path
}

pub fn generate_file_content(data: &Vec<TaskGroup>, date: &NaiveDate) -> String {
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

pub fn write_file(path: &PathBuf, content: &String) {
    let mut new_file = File::create(&path).expect("Could not open today's file: {today_file_path}");
    write!(new_file, "{}", content).expect("Could not write to file: {today_file_path}");
}

pub fn load_file(file: &TodoFile) -> String {
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

pub fn parse_todo_file<'a>(contents: &String, arena: &'a Arena<AstNode<'a>>) -> &'a AstNode<'a> {
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

pub fn extract_secitons<'a>(
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

pub fn get_latest_file(dir: &Path) -> Result<TodoFile, String> {
    let dir = read_dir(dir).expect(format!("Could not find notes folder: {:?}", dir).as_str());
    dir.filter_map(|f| f.ok())
        .filter_map(|file| TodoFile::try_from(file).ok())
        .reduce(|a, b| TodoFile::latest_file(a, b))
        .ok_or("Could not reduce items".to_string())
}
