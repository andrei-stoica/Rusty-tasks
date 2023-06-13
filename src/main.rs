use chrono::naive::NaiveDate;
use chrono::{Local, Datelike};
use regex::Regex;
use std::convert::TryFrom;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::env;

//TODO handle unwraps and errors more uniformly

#[derive(Debug)]
struct TodoFile {
    file: DirEntry,
    date: NaiveDate,
}

fn main() {
    let data_dir = get_data_dir("notes");

    println!("{}", data_dir.to_str().unwrap());
    let latest_file = get_latest_file(&data_dir).unwrap();
    //.expect(
    //    format!(
    //        "Could not find any notes files please use format: {}",
    //        get_file_regex().to_string()
    //    )
    //    .as_str(),
    //);
    println!("Latest file: {:?}", latest_file);
    let now = Local::now();
    let today = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day());
    match today {
        Some(today) if latest_file.date == today => println!("Todays file was created"),
        Some(today) if latest_file.date < today => println!("Todays file was not created"),
        Some(today) if latest_file.date > today => println!("Future files were created"),

        _ => println!("Today never happend!")
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
    let dir = fs::read_dir(dir).expect(format!("Could not find notes folder: {:?}", dir).as_str());
    dir.filter_map(|f| f.ok())
        .filter_map(|file| TodoFile::try_from(file).ok())
        .reduce(|a, b| TodoFile::latest_file(a, b))
        .ok_or("Could not reduce items".to_string())
}

fn get_file_regex() -> Regex {
    Regex::new(r"(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2}).md")
        .expect("could not create regex")
}

impl TodoFile {
    fn capture_as_number<T: FromStr>(capture: &regex::Captures, name: &str) -> Result<T, String> {
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
}

impl TryFrom<DirEntry> for TodoFile {
    type Error = String;

    fn try_from(direntry: DirEntry) -> Result<Self, Self::Error> {
        let re = get_file_regex();
        println!("{:?}", re);
        let file_name = direntry.file_name();
        let file_name_str = match file_name.to_str() {
            Some(name) => name,
            _ => ""
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
        Err(format!("Could not parse file name => {{ name: {:?}, re: {:?} }}", file_name, re).to_string())
    }
}
