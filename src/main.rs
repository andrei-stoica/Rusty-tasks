use chrono::naive::NaiveDate;
use chrono::{Datelike, Local};
use std::env;
use std::fs::{copy, read_dir};
use std::path::{Path, PathBuf};
use std::process::Command;
use todo_file::TodoFile;

//TODO handle unwraps and errors more uniformly
//TODO move TodoFile into its file
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

mod todo_file {
    use chrono::naive::NaiveDate;
    use regex::Regex;
    use std::convert::TryFrom;
    use std::fs::DirEntry;
    use std::str::FromStr;

    #[derive(Debug)]
    pub struct TodoFile {
        pub file: DirEntry,
        pub date: NaiveDate,
    }

    impl TodoFile {
        fn capture_as_number<T: FromStr>(
            capture: &regex::Captures,
            name: &str,
        ) -> Result<T, String> {
            Ok(capture
                .name(name)
                .unwrap()
                .as_str()
                .parse::<T>()
                .ok()
                .ok_or("Something went wrong".to_owned())?)
        }

        pub fn latest_file(a: TodoFile, b: TodoFile) -> TodoFile {
            if a.date > b.date {
                a
            } else {
                b
            }
        }

        fn get_file_regex() -> Regex {
            //TODO This would ideally be configurable
            Regex::new(r"(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2}).md")
                .expect("could not create regex")
        }
    }

    impl TryFrom<DirEntry> for TodoFile {
        type Error = String;

        fn try_from(direntry: DirEntry) -> Result<Self, Self::Error> {
            let re = TodoFile::get_file_regex();
            println!("{:?}", re);
            let file_name = direntry.file_name();
            let file_name_str = match file_name.to_str() {
                Some(name) => name,
                _ => "",
            };
            println!("{:?}", file_name_str);

            if let Some(caps) = re.captures(file_name_str) {
                let year: i32 = Self::capture_as_number(&caps, "year").unwrap();
                let month: u32 = Self::capture_as_number(&caps, "month").unwrap();
                let day: u32 = Self::capture_as_number(&caps, "day").unwrap();

                return Ok(Self {
                    file: direntry,
                    date: NaiveDate::from_ymd_opt(year, month, day).unwrap(),
                });
            };
            Err(format!(
                "Could not parse file name => {{ name: {:?}, re: {:?} }}",
                file_name, re
            )
            .to_string())
        }
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
