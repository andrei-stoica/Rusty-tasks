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
}

#[derive(Debug, PartialEq, Clone)]
pub enum Status {
    Done(char),
    Todo(char),
    Empty,
}

pub enum TaskErorr {
    ParsingError(&'static str),
}

impl Task {
    fn find_text<'a>(node: &'a AstNode<'a>) -> String {
        let mut text = String::new();
        for child in node.descendants() {
            let data_ref = child.data.borrow();
            if let NodeValue::Text(contents) = &data_ref.value {
                text.push_str(format!("{}\n       ", &contents.clone()).as_str());
            };
        }
        text
    }
}

impl ToString for Task {
    fn to_string(&self) -> String {
        let ch = match self.status {
            Status::Done(ch) => ch,
            Status::Todo(ch) => ch,
            Status::Empty => ' ',
        };
        format!(" - [{}] {}\n", ch, self.text.trim())
    }
}

impl<'a> TryFrom<&'a AstNode<'a>> for Task {
    type Error = TaskErorr;
    fn try_from(node: &'a AstNode<'a>) -> Result<Self, Self::Error> {
        let data_ref = &node.data.borrow();
        if let NodeValue::TaskItem(ch) = data_ref.value {
            let text = Self::find_text(node);
            let status = match ch {
                Some(c) if c == 'x' || c == 'X' => Status::Done(c),
                Some(c) => Status::Todo(c),
                _ => Status::Empty,
            };

            Ok(Self { status, text })
        } else {
            Err(TaskErorr::ParsingError(
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
    type Error = TaskErorr;
    fn try_from(node: &'a AstNode<'a>) -> Result<Self, Self::Error> {
        let node_ref = &node.data.borrow();
        if let NodeValue::Heading(heading) = node_ref.value {
            let level = heading.level;
            let first_child_ref = &node.first_child();
            let first_child = if let Some(child) = first_child_ref.borrow() {
                child
            } else {
                return Err(TaskErorr::ParsingError("Node has no children"));
            };

            let data_ref = &first_child.data.borrow();
            let name = if let NodeValue::Text(value) = &data_ref.value {
                value.to_string()
            } else {
                return Err(TaskErorr::ParsingError(
                    "Could not get title from heading node",
                ));
            };

            let next_sib = node
                .next_sibling()
                .ok_or(TaskErorr::ParsingError("Empty section at end of file"))?;

            if let NodeValue::List(_list_meta) = next_sib.data.borrow().value {
                let tasks = next_sib
                    .children()
                    .into_iter()
                    .filter_map(|item_node| Task::try_from(item_node).ok())
                    .collect();

                Ok(TaskGroup { name, tasks, level })
            } else {
                Err(TaskErorr::ParsingError(
                    "Next sibling of node is not a list",
                ))
            }
        } else {
            Err(TaskErorr::ParsingError("Node is not a section heading"))
        }
    }
}
