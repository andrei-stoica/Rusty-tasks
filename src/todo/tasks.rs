use std::borrow::Borrow;

use comrak::nodes::AstNode;
use comrak::nodes::NodeValue;

#[derive(Debug, Clone)]
pub struct TaskGroup {
    pub name: String,
    pub tasks: Vec<Task>,
    pub level: u8,
}

// This does not support subtasks, need to figure out best path forward
#[derive(Debug, Clone)]
pub struct Task {
    pub status: Status,
    pub text: String,
    pub subtasks: Option<Vec<Task>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Status {
    Done(char),
    Todo(char),
    Empty,
}

pub enum TaskError {
    ParsingError(&'static str),
}

impl Task {
    fn extract_text<'a>(node: &'a AstNode<'a>) -> Result<String, TaskError> {
        let data_ref = node.data.borrow();
        if let NodeValue::Text(contents) = &data_ref.value {
            Ok(contents.to_string())
        } else {
            Err(TaskError::ParsingError("Could not get text from element"))
        }
    }

    fn extract_text_from_task<'a>(node: &'a AstNode<'a>) -> Result<String, TaskError> {
        let mut text = String::new();
        let data_ref = node.data.borrow();
        if let NodeValue::Paragraph = data_ref.value {
            for child in node.children() {
                let child_data_ref = child.data.borrow();
                let t = match &child_data_ref.borrow().value {
                    NodeValue::Text(contents) => contents.clone(),
                    NodeValue::Emph if child.first_child().is_some() => {
                        format!("*{}*", Self::extract_text(child.first_child().unwrap())?)
                    }
                    NodeValue::Strong if child.first_child().is_some() => {
                        format!("**{}**", Self::extract_text(child.first_child().unwrap())?)
                    }
                    NodeValue::SoftBreak => {
                        format!("\n{}", " ".repeat(data_ref.sourcepos.start.column))
                    }
                    _ => "".into(),
                };
                text.push_str(&t);
            }
            Ok(text)
        } else {
            Err(TaskError::ParsingError("First child is not Paragraph"))
        }
    }
}

impl ToString for Task {
    fn to_string(&self) -> String {
        let ch = match self.status {
            Status::Done(ch) => ch,
            Status::Todo(ch) => ch,
            Status::Empty => ' ',
        };

        let subtasks = if let Some(subtasks) = &self.subtasks {
            let mut text = subtasks
                .iter()
                .map(|task| task.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            text.insert(0, '\n');
            text.trim_end().to_string()
        } else {
            "".into()
        };

        format!(
            "- [{}] {}{}\n",
            ch,
            self.text.trim(),
            subtasks.replace("\n", "\n  ")
        )
    }
}

impl<'a> TryFrom<&'a AstNode<'a>> for Task {
    type Error = TaskError;
    fn try_from(node: &'a AstNode<'a>) -> Result<Self, Self::Error> {
        let data_ref = &node.data.borrow();
        if let NodeValue::TaskItem(ch) = data_ref.value {
            let text = Self::extract_text_from_task(
                node.first_child()
                    .ok_or(TaskError::ParsingError("No childern of node found"))?,
            )?;
            let status = match ch {
                Some(c) if c == 'x' || c == 'X' => Status::Done(c),
                Some(c) => Status::Todo(c),
                _ => Status::Empty,
            };
            let subtasks = node
                .children()
                .filter_map(|child| {
                    if let NodeValue::List(_) = child.data.borrow().value {
                        Some(child)
                    } else {
                        None
                    }
                })
                .map(|child| {
                    child
                        .children()
                        .into_iter()
                        .filter_map(|item_node| Task::try_from(item_node).ok())
                        .collect()
                })
                .reduce(|a: Vec<Task>, b: Vec<Task>| [a, b].concat());

            Ok(Self {
                status,
                text,
                subtasks,
            })
        } else {
            Err(TaskError::ParsingError(
                "Node being parsed is not a TaskItem",
            ))
        }
    }
}

impl TaskGroup {
    pub fn empty(name: String, level: u8) -> TaskGroup {
        TaskGroup {
            name,
            tasks: Vec::new(),
            level,
        }
    }
}
impl ToString for TaskGroup {
    fn to_string(&self) -> String {
        let mut output = String::new();
        output.push_str(format!("{} {}\n", "#".repeat(self.level.into()), self.name).as_str());
        self.tasks
            .iter()
            .for_each(|task| output.push_str(task.to_string().as_str()));

        output
    }
}

impl<'a> TryFrom<&'a AstNode<'a>> for TaskGroup {
    type Error = TaskError;
    fn try_from(node: &'a AstNode<'a>) -> Result<Self, Self::Error> {
        let node_ref = &node.data.borrow();
        if let NodeValue::Heading(heading) = node_ref.value {
            let level = heading.level;
            let first_child_ref = &node.first_child();
            let first_child = if let Some(child) = first_child_ref.borrow() {
                child
            } else {
                return Err(TaskError::ParsingError("Node has no children"));
            };

            let data_ref = &first_child.data.borrow();
            let name = if let NodeValue::Text(value) = &data_ref.value {
                value.to_string()
            } else {
                return Err(TaskError::ParsingError(
                    "Could not get title from heading node",
                ));
            };

            let next_sib = node
                .next_sibling()
                .ok_or(TaskError::ParsingError("Empty section at end of file"))?;

            if let NodeValue::List(_list_meta) = next_sib.data.borrow().value {
                let tasks = next_sib
                    .children()
                    .into_iter()
                    .filter_map(|item_node| Task::try_from(item_node).ok())
                    .collect();

                Ok(TaskGroup { name, tasks, level })
            } else {
                Err(TaskError::ParsingError(
                    "Next sibling of node is not a list",
                ))
            }
        } else {
            Err(TaskError::ParsingError("Node is not a section heading"))
        }
    }
}
