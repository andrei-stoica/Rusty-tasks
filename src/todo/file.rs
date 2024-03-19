use chrono::naive::NaiveDate;
use regex::Regex;
use std::cmp::min;
use std::convert::TryFrom;
use std::fs::DirEntry;
use std::path::PathBuf;
use std::str::FromStr;

use crate::file::FileNameParseError;

#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub file: PathBuf,
    pub date: NaiveDate,
}

pub enum FileError {
    //IOError(&'static str),
    ParseError(&'static str),
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
                file: direntry.path(),
                date: NaiveDate::from_ymd_opt(year, month, day).unwrap(),
            });
        };
        Err(FileError::ParseError("Could not parse file name"))
    }
}

fn try_get_date(file: &PathBuf) -> Result<NaiveDate, FileNameParseError> {
    let file_name = file
        .file_name()
        .ok_or(FileNameParseError::TypeConversionError(
            "Could not get filename from path: {:?}",
        ))?
        .to_str()
        .ok_or(FileNameParseError::TypeConversionError(
            "Could not get filename from path: {:?}",
        ))?;

    NaiveDate::parse_from_str(file_name, "%Y-%m-%d.md")
        .or_else(|e| Err(FileNameParseError::ParseError(e)))
}

impl TryFrom<PathBuf> for File {
    type Error = FileNameParseError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Ok(Self {
            date: try_get_date(&path)?,
            file: path.into(),
        })
    }
}
impl File {
    pub fn get_closest_files(files: Vec<PathBuf>, target: NaiveDate, n: usize) -> Vec<File> {
        let mut dated_files = files
            .into_iter()
            .filter_map(|file| File::try_from(file).ok())
            .collect::<Vec<_>>();
        dated_files.sort_by_cached_key(|dated_file| (dated_file.date - target).num_days().abs());

        let count = min(n, dated_files.len());
        dated_files[..count].to_vec()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::NaiveDate;
    use std::path::PathBuf;

    #[test]
    fn test_get_closest_date() {
        let files = vec![
            PathBuf::from("./2024-01-01.md"),
            PathBuf::from("./2024-01-02.md"),
            PathBuf::from("./2024-01-03.md"),
            PathBuf::from("./2024-02-01.md"),
            PathBuf::from("./2024-03-01.md"),
            PathBuf::from("./2024-04-01.md"),
            PathBuf::from("./2024-04-02.md"),
            PathBuf::from("./2024-04-03.md"),
            PathBuf::from("./2024-04-04.md"),
        ];

        let res = File::get_closest_files(
            files.clone(),
            NaiveDate::from_ymd_opt(2023, 12, 30).unwrap(),
            3,
        );
        let expected_res = vec![
            File::try_from(PathBuf::from("./2024-01-01.md")).unwrap(),
            File::try_from(PathBuf::from("./2024-01-02.md")).unwrap(),
            File::try_from(PathBuf::from("./2024-01-03.md")).unwrap(),
        ];
        assert_eq!(res, expected_res);

        let res = File::get_closest_files(
            files.clone(),
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            3,
        );
        let expected_res = vec![
            File::try_from(PathBuf::from("./2024-02-01.md")).unwrap(),
            File::try_from(PathBuf::from("./2024-01-03.md")).unwrap(),
            File::try_from(PathBuf::from("./2024-03-01.md")).unwrap(),
        ];
        assert_eq!(res, expected_res);

        let res = File::get_closest_files(
            files.clone(),
            NaiveDate::from_ymd_opt(2024, 5, 2).unwrap(),
            3,
        );
        let expected_res = vec![
            File::try_from(PathBuf::from("./2024-04-04.md")).unwrap(),
            File::try_from(PathBuf::from("./2024-04-03.md")).unwrap(),
            File::try_from(PathBuf::from("./2024-04-02.md")).unwrap(),
        ];
        assert_eq!(res, expected_res);

        let res = File::get_closest_files(
            files[..1].to_vec(),
            NaiveDate::from_ymd_opt(2023, 12, 30).unwrap(),
            3,
        );
        let expected_res = vec![
            File::try_from(PathBuf::from("./2024-01-01.md")).unwrap(),
        ];
        assert_eq!(res, expected_res);
    }
}
