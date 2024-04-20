use chrono::{Datelike, NaiveDate};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// set config file to use
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<String>,
    /// show current config file
    #[arg(short = 'C', long)]
    pub current_config: bool,

    /// view a specific date's file (YYYY-MM-DD)
    #[arg(short, long)]
    pub date: Option<String>,
    /// view previous day's notes
    #[arg(short = 'p', long, default_value_t = 0)]
    pub previous: u16,
    /// list closest files to date
    #[arg(short, long)]
    pub list: bool,
    /// number of files to list
    #[arg(short, long, default_value_t = 5)]
    pub number: usize,
    /// list closest files to date
    #[arg(short = 'L', long)]
    pub list_all: bool,

    /// increase logging level
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

pub fn smart_parse_date(date_str: &str, cur_date: &NaiveDate) -> Option<NaiveDate> {
    let full_date_fmt = "%Y-%m-%d";

    if let Ok(date) = NaiveDate::parse_from_str(date_str, &full_date_fmt) {
        return Some(date);
    }
    let parts: Vec<&str> = date_str.split('-').collect();

    match parts.len() {
        1 => cur_date.with_day(parts[0].parse().unwrap_or(cur_date.day())),
        2 => cur_date
            .with_day(parts[1].parse().unwrap_or(cur_date.day()))?
            .with_month(parts[0].parse().unwrap_or(cur_date.month())),
        3 => NaiveDate::from_ymd_opt(
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
        ),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_smart_parse_date() {
        let good_date = NaiveDate::from_ymd_opt(2024, 01, 03).expect("Invalid date specified");
        let good_date_str = good_date.format("%Y-%m-%d").to_string();

        assert_eq!(
            Some(good_date),
            smart_parse_date(&good_date_str, &good_date)
        );
        let no_padding_date_str = "2024-1-3";
        assert_eq!(
            Some(good_date),
            smart_parse_date(no_padding_date_str, &good_date)
        );

        let bad_day_str = "2024-01-99";
        assert_eq!(None, smart_parse_date(bad_day_str, &good_date));
        let no_day_str = "2024-01";
        assert_eq!(None, smart_parse_date(no_day_str, &good_date));

        let bad_month_str = "2024-25-01";
        assert_eq!(None, smart_parse_date(bad_month_str, &good_date));
        let no_month_str = "2024-14";
        assert_eq!(None, smart_parse_date(no_month_str, &good_date));

        let no_year_str = "01-03";
        assert_eq!(Some(good_date), smart_parse_date(no_year_str, &good_date));
        let bad_month_no_year_str = "25-01";
        assert_eq!(None, smart_parse_date(bad_month_no_year_str, &good_date));
        let bad_day_no_year_str = "01-35";
        assert_eq!(None, smart_parse_date(bad_day_no_year_str, &good_date));

        let no_year_month_str = "03";
        assert_eq!(
            Some(good_date),
            smart_parse_date(no_year_month_str, &good_date)
        );
        let bad_day_no_year_month_str = "35";
        assert_eq!(
            None,
            smart_parse_date(bad_day_no_year_month_str, &good_date)
        );
    }
}
