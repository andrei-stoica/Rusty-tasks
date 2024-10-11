use crate::todo::{File as TodoFile, Status as TaskStatus};
use crate::NaiveDate;
use crate::TaskGroup;
use chrono::Datelike;
use comrak::nodes::{Ast, AstNode, LineColumn, NodeHeading, NodeValue};
use comrak::{
    format_commonmark, parse_document, Arena, ComrakOptions, ExtensionOptions, ParseOptions,
};
use indexmap::IndexMap;
use regex::Regex;
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

/// generate strings from TaskGroups and date
pub fn generate_file_content(data: &Vec<TaskGroup>, date: &NaiveDate) -> String {
    // TODO: This should be a type and then I can implement it with From<>
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

/// Load in text file as String
pub fn load_file(file: &TodoFile) -> String {
    // TODO: This could be a TryFrom<>
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

/// Parse contents of markdown file with Comrak ( relaxed tasklist matching is enabled)
pub fn parse_todo_file<'a>(contents: &String, arena: &'a Arena<AstNode<'a>>) -> &'a AstNode<'a> {
    let mut extension_options = ExtensionOptions::default();
    extension_options.tasklist = true;

    let mut parse_options = ParseOptions::default();
    parse_options.relaxed_tasklist_matching = true;

    let options = &ComrakOptions {
        extension: extension_options,
        parse: parse_options,
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

fn remove_heading<'a>(node: &'a AstNode<'a>, level: u8) {
    let mut following = node.following_siblings();
    let _ = following.next().unwrap();
    for sib in following {
        let node_ref = sib.data.borrow();
        if let NodeValue::Heading(heading) = node_ref.value {
            if heading.level == level {
                break;
            }
        } else {
            sib.detach();
        }
    }
    node.detach();
}

/// recursively removes nodes from List
fn remove_task_nodes<'a>(root: &'a AstNode<'a>) {
    for node in root.children() {
        for child_node in node.children() {
            remove_task_nodes(child_node)
        }
        match node.data.borrow().value {
            NodeValue::TaskItem(Some(status)) if status == 'x' || status == 'X' => node.detach(),
            _ => continue,
        }
    }
}

fn create_title<'a>(arena: &'a Arena<AstNode<'a>>, date: &str) -> &'a AstNode<'a> {
    let mut text = String::new();
    text.push_str("Today's tasks ");
    text.push_str(date);

    create_heading(arena, 1, &text)
}

fn create_heading<'a>(arena: &'a Arena<AstNode<'a>>, level: u8, text: &str) -> &'a AstNode<'a> {
    let heading_node = arena.alloc(AstNode::new(
        Ast::new(
            NodeValue::Heading(NodeHeading {
                level,
                setext: false,
            }),
            LineColumn { line: 0, column: 0 },
        )
        .into(),
    ));
    let text_node = arena.alloc(AstNode::new(
        Ast::new(
            NodeValue::Text(text.to_string()),
            LineColumn { line: 0, column: 2 },
        )
        .into(),
    ));

    heading_node.append(text_node);

    heading_node
}

pub fn create_new_doc<'a>(
    arena: &'a Arena<AstNode<'a>>,
    new_date: &str,
    sections: IndexMap<String, Option<Vec<&'a AstNode<'a>>>>,
) -> &'a AstNode<'a> {
    let doc = arena.alloc(AstNode::new(
        Ast::new(NodeValue::Document, LineColumn { line: 0, column: 0 }).into(),
    ));
    let title = create_title(&arena, new_date);
    doc.append(title);

    for (section, value) in sections.iter() {
        let heading = create_heading(arena, 2, &section);
        doc.append(heading);
        match value {
            Some(nodes) => {
                for node in nodes.iter() {
                    doc.append(node);
                }
            }
            _ => (),
        }
    }
    doc
}

pub fn extract_sections<'a>(
    root: &'a AstNode<'a>,
    sections: &Vec<String>,
) -> IndexMap<String, Option<Vec<&'a AstNode<'a>>>> {
    let mut section_map: IndexMap<String, Option<Vec<&'a AstNode<'a>>>> = IndexMap::new();
    sections.iter().for_each(|section| {
        section_map.insert(section.to_string(), None);
    });

    for node in root.reverse_children() {
        let node_ref = node.data.borrow();
        match node_ref.value {
            NodeValue::Heading(heading) => {
                let heading_content_node = if let Some(child) = node.first_child() {
                    child
                } else {
                    continue;
                };

                let mut heading_content_ref = heading_content_node.data.borrow_mut();
                if let NodeValue::Text(text) = &mut heading_content_ref.value {
                    if sections.contains(text) {
                        let mut content = Vec::new();
                        let mut following = node.following_siblings();
                        let _ = following.next().unwrap();

                        for sib in following {
                            remove_task_nodes(sib);
                            let node_ref = sib.data.borrow();
                            if let NodeValue::Heading(inner_heading) = node_ref.value {
                                if heading.level == inner_heading.level {
                                    break;
                                }
                            } else {
                                content.push(sib);
                            }
                        }
                        section_map.insert(text.to_string(), Some(content));
                        remove_heading(node, heading.level);
                    };
                }
            }
            _ => continue,
        }
    }

    section_map
}

pub fn process_doc_tree<'a>(root: &'a AstNode<'a>, new_date: &str, sections: &Vec<String>) {
    for node in root.reverse_children() {
        let node_ref = node.data.borrow();
        match node_ref.value {
            NodeValue::Heading(heading) => {
                let heading_content_node = if let Some(child) = node.first_child() {
                    child
                } else {
                    continue;
                };

                let mut heading_content_ref = heading_content_node.data.borrow_mut();
                if let NodeValue::Text(text) = &mut heading_content_ref.value {
                    let re = Regex::new(r"Today's tasks \d+-\d+-\d+")
                        .expect("title regex is not parsable");
                    if matches!(re.find(text), Some(_)) {
                        text.clear();
                        text.push_str("Today's tasks ");
                        text.push_str(new_date);
                    } else if !sections.contains(text) {
                        remove_heading(node, heading.level);
                    };
                }
            }
            NodeValue::List(_list) => remove_task_nodes(node),
            _ => continue,
        }
    }
    eprintln!("{:#?}", root);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::todo::{Status, Task};
    use std::io::BufWriter;

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
        assert_eq!(result.keys().count(), 1);
        assert!(result.get(sections.first().unwrap()).is_some());
        assert_eq!(result.get(sections.first().unwrap()).unwrap().level, 3);

        let sections = vec!["Content".to_string()];
        let result = extract_secitons(root, &sections);
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

    #[test]
    fn test_generate_file_content() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let mut content: Vec<TaskGroup> = vec![];

        let result = generate_file_content(&content, &date);
        let expected = "# Today's tasks 2024-01-01\n";
        assert_eq!(result, expected);

        content.push(TaskGroup {
            name: "Empty".into(),
            tasks: vec![],
            level: 2,
        });

        let result = generate_file_content(&content, &date);
        let expected = "# Today's tasks 2024-01-01\n\n## Empty\n";
        assert_eq!(result, expected);

        content.push(TaskGroup {
            name: "Subgroup".into(),
            tasks: vec![],
            level: 3,
        });

        let result = generate_file_content(&content, &date);
        let expected = "# Today's tasks 2024-01-01\n\n## Empty\n\n### Subgroup\n";
        assert_eq!(result, expected);

        content.push(TaskGroup {
            name: "Tasks".into(),
            tasks: vec![
                Task {
                    status: Status::Empty,
                    text: "task 1".into(),
                    subtasks: None,
                },
                Task {
                    status: Status::Done('x'),
                    text: "task 2".into(),
                    subtasks: None,
                },
                Task {
                    status: Status::Todo('>'),
                    text: "task 3".into(),
                    subtasks: None,
                },
            ],
            level: 2,
        });

        let result = generate_file_content(&content, &date);
        let expected = "\
# Today's tasks 2024-01-01

## Empty

### Subgroup

## Tasks
- [ ] task 1
- [x] task 2
- [>] task 3
";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_node_removal() {
        let md = "
# Today's tasks 2024-01-01

## Tasks

- [ ] task 1
- [X] task 2
- [x] task 2
- [>] task 3
- [!] task 3

## Long Term

- [ ] task 1
- [X] task 2
    - [ ] all of these subtasks should be removed
        - [x] subtasks
    - [x] sub task to remove
- [!] task 3
    - [ ] sub task to keep
    - [x] sub task to remove

## Todays Notes

- some notes here
- these can go
";
        let new_date = "2024-01-02";
        let groups = vec![
            "Tasks".to_string(),
            "Other".to_string(),
            "Long Term".to_string(),
            "Last".to_string(),
        ];
        let arena = Arena::new();
        let mut extension_options = ExtensionOptions::default();
        extension_options.tasklist = true;

        let mut parse_options = ParseOptions::default();
        parse_options.relaxed_tasklist_matching = true;

        let options = &ComrakOptions {
            extension: extension_options,
            parse: parse_options,
            ..ComrakOptions::default()
        };

        let ast = parse_document(&arena, md, options);

        let sections = extract_sections(ast, &groups);

        let new_doc = create_new_doc(&arena, new_date, sections);

        process_doc_tree(ast, new_date, &groups);

        let mut output = BufWriter::new(Vec::new());

        assert!(format_commonmark(new_doc, options, &mut output).is_ok());

        let bytes = output.into_inner().expect("should be a vec");
        let text = String::from_utf8(bytes).expect("should be convertable to string");
        assert_eq!(
            "\
# Today's tasks 2024-01-02

## Tasks

- [ ] task 1
- [>] task 3
- [!] task 3

## Other

## Long Term

- [ ] task 1
- [!] task 3
  - [ ] sub task to keep

## Last
",
            text
        );
    }
}
