use log::Level;

pub fn get_logging_level(verbose_level: u8) -> Level {
    match verbose_level {
        ..=0 => Level::Error,
        1 => Level::Warn,
        2 => Level::Info,
        3 => Level::Debug,
        4.. => Level::Trace,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_logging_level() {
        assert_eq!(get_logging_level(0), Level::Error);
        assert_eq!(get_logging_level(1), Level::Warn);
        assert_eq!(get_logging_level(2), Level::Info);
        assert_eq!(get_logging_level(3), Level::Debug);
        assert_eq!(get_logging_level(4), Level::Trace);
        assert_eq!(get_logging_level(5), Level::Trace);
    }
}
