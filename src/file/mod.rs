use crate::todo::{File as TodoFile, Status as TaskStatus};
use crate::NaiveDate;
use crate::TaskGroup;
use chrono::Datelike;
use comrak::nodes::{AstNode, NodeValue};
use comrak::parse_document;
use comrak::{Arena, ComrakExtensionOptions, ComrakOptions, ComrakParseOptions};
use std::collections::HashMap;
use std::fs::{read, File};
use std::io::Write;
use std::path::PathBuf;
use std::str;

#[derive(Debug)]
pub enum FileNameParseError {
    TypeConversionError(&'static str),
    ParseError(chrono::ParseError),
}

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
    let contents_utf8 = read(file.file.clone())
        .expect(format!("Could not read file {}", file.file.to_string_lossy()).as_str());
    str::from_utf8(&contents_utf8)
        .expect(
            format!(
                "failed to convert contents of file to string: {}",
                file.file.to_string_lossy()
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::todo::Task;

    #[test]
    fn test_extract_sections() {
        let test_md = "\
# Test
## Content
 - [ ] something
 - [x] done
 - [!] other
## Unused
### Sub section
 - [ ] task
## Unrealated Stuff
 - [ ] something else
    + [ ] subtask";

        let arena = Arena::new();
        let root = parse_todo_file(&test_md.to_string(), &arena);

        let result = extract_secitons(root, &vec![]);
        assert_eq!(result.keys().count(), 0);

        let result = extract_secitons(root, &vec!["Not There".to_string()]);
        assert_eq!(result.keys().count(), 0);

        let sections = vec!["Unused".to_string()];
        let result = extract_secitons(root, &sections);
        assert_eq!(result.keys().count(), 0);

        let sections = vec!["Sub section".to_string()];
        let result = extract_secitons(root, &sections);
        println!("{:#?}", root);
        assert_eq!(result.keys().count(), 1);
        assert!(result.get(sections.first().unwrap()).is_some());
        assert_eq!(result.get(sections.first().unwrap()).unwrap().level, 3);

        let sections = vec!["Content".to_string()];
        let result = extract_secitons(root, &sections);
        println!("{:#?}", root);
        assert_eq!(result.keys().count(), 1);
        assert!(result.get(sections.first().unwrap()).is_some());
        assert_eq!(
            result
                .get(sections.first().unwrap())
                .expect("No Value for \"Content\""),
            &TaskGroup {
                name: sections.first().unwrap().clone(),
                tasks: vec![
                    Task {
                        status: TaskStatus::Empty,
                        text: "something".to_string(),
                        subtasks: None
                    },
                    Task {
                        status: TaskStatus::Todo('!'),
                        text: "other".to_string(),
                        subtasks: None
                    },
                ],
                level: 2
            }
        );

        let sections = vec!["Unrealated Stuff".to_string()];
        let result = extract_secitons(root, &sections);
        assert_eq!(result.keys().count(), 1);
        assert!(result.get(sections.first().unwrap()).is_some());
        assert_eq!(
            result
                .get(sections.first().unwrap())
                .expect("No Value for \"Content\""),
            &TaskGroup {
                name: sections.first().unwrap().clone(),
                tasks: vec![Task {
                    status: TaskStatus::Empty,
                    text: "something else".to_string(),
                    subtasks: Some(vec![Task {
                        status: TaskStatus::Empty,
                        text: "subtask".to_string(),
                        subtasks: None
                    }]),
                }],
                level: 2
            }
        );

        let result = extract_secitons(
            root,
            &vec!["Content".to_string(), "Sub section".to_string()],
        );
        assert_eq!(result.keys().count(), 2);
    }
}
