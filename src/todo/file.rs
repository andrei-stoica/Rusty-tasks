use chrono::naive::NaiveDate;
use std::cmp::min;
use std::convert::TryFrom;
use std::path::PathBuf;

use crate::file::FileNameParseError;

#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub file: PathBuf,
    pub date: NaiveDate,
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
        let expected_res = vec![File::try_from(PathBuf::from("./2024-01-01.md")).unwrap()];
        assert_eq!(res, expected_res);
    }
}
