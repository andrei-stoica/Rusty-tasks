use chrono::naive::NaiveDate;
use regex::Regex;
use std::convert::TryFrom;
use std::fs::DirEntry;
use std::str::FromStr;

#[derive(Debug)]
pub struct File {
    pub file: DirEntry,
    pub date: NaiveDate,
}

pub enum FileError{
    //IOError(&'static str),
    ParseError(&'static str)
}

impl File {
    fn capture_as_number<T: FromStr>(capture: &regex::Captures, name: &str) -> Result<T, String> {
        Ok(capture
            .name(name)
            .unwrap()
            .as_str()
            .parse::<T>()
            .ok()
            .ok_or("Something went wrong".to_owned())?)
    }

    pub fn latest_file(a: File, b: File) -> File {
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

impl TryFrom<DirEntry> for File {
    type Error = FileError;

    fn try_from(direntry: DirEntry) -> Result<Self, Self::Error> {
        let re = File::get_file_regex();
//        println!("{:?}", re);
        let file_name = direntry.file_name();
        let file_name_str = match file_name.to_str() {
            Some(name) => name,
            _ => "",
        };
//        println!("{:?}", file_name_str);

        if let Some(caps) = re.captures(file_name_str) {
            let year: i32 = Self::capture_as_number(&caps, "year").unwrap();
            let month: u32 = Self::capture_as_number(&caps, "month").unwrap();
            let day: u32 = Self::capture_as_number(&caps, "day").unwrap();

            return Ok(Self {
                file: direntry,
                date: NaiveDate::from_ymd_opt(year, month, day).unwrap(),
            });
        };
        Err(FileError::ParseError("Could not parse file name"))
    }
}
