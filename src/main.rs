mod cli;
mod config;
mod file;
mod logging;
mod todo;

use chrono::naive::NaiveDate;
use chrono::{Local, TimeDelta};
use clap::Parser;
use cli::Args;
use comrak::Arena;
use config::Config;
use log;
use logging::get_logging_level;
use resolve_path::PathResolveExt;
use simple_logger::init_with_level;
use std::fs;
use std::path::Path;
use std::process::Command;
use todo::{File as TodoFile, TaskGroup};

fn main() {
    // setup
    let args = Args::parse();
    let _logger = init_with_level(get_logging_level(args.verbose)).unwrap();
    log::debug!("{:?}", args);

    // getting config location
    let expected_cfg_files = match Config::expected_locations() {
        Ok(cfg_files) => cfg_files,
        Err(e) => panic!("{:?}", e),
    };

    // getting exising config files
    let cfg_files: Vec<&Path> = expected_cfg_files
        .iter()
        .map(|file| Path::new(file))
        .filter(|file| file.exists())
        .collect();

    // writing default config if non exist
    if cfg_files.len() <= 0 && args.config.is_none() {
        if let Err(e) = Config::write_default(match expected_cfg_files[0].to_str() {
            Some(s) => s,
            None => panic!("Could not resolve expected cfg file paths"),
        }) {
            panic!("Could not write config: {:?}", e);
        }
    }

    // set witch config file to load
    let cfg_file = match args.config {
        Some(file) => file,
        None => match cfg_files.last() {
            None => expected_cfg_files[0].to_string_lossy().to_string(),
            Some(file) => file.to_string_lossy().to_string(),
        },
    };

    // show current config file or just log it based on args
    if args.current_config {
        print!("{}", &cfg_file);
        return;
    } else {
        log::debug!("config file: {}", &cfg_file);
    }

    // load config file
    let cfg = match Config::load(&cfg_file) {
        Ok(cfg) => cfg,
        Err(_e) => panic!("could not load config: {}", cfg_file),
    };
    log::debug!("{:#?}", cfg);

    // resolve data directory and create it if it does not exisit
    let data_dir = cfg.notes_dir.resolve().to_path_buf();
    if !fs::metadata(&data_dir).is_ok() {
        match fs::create_dir_all(&data_dir) {
            Err(_e) => panic!("Could not create default directory: {:?}", &data_dir),
            _ => log::info!("created dir {}", &data_dir.to_string_lossy()),
        };
    }

    // get file paths of notes
    let files = fs::read_dir(&data_dir)
        .expect(format!("Could not find notes folder: {:?}", &data_dir).as_str())
        .filter_map(|f| f.ok())
        .map(|file| file.path());
    // list all notes
    if args.list_all {
        files
            .into_iter()
            .for_each(|f| println!("{}", f.canonicalize().unwrap().to_string_lossy()));
        return ();
    }

    // get clossest files to specified date
    let today = Local::now().date_naive();
    let target = today - TimeDelta::try_days(args.previous.into()).unwrap();
    let closest_files = TodoFile::get_closest_files(files.collect(), target, args.number);
    // list files
    if args.list {
        closest_files
            .into_iter()
            .for_each(|f| println!("{}", f.file.canonicalize().unwrap().to_string_lossy()));
        return ();
    }

    let latest_file = closest_files.first();
    let current_file = match latest_file {
        // copy old file if the user specifies today's notes but it does not exist
        Some(todo_file) if todo_file.date < today && args.previous == 0 => {
            let sections = &cfg.sections;
            log::info!("looking for sections: {:?}", sections);
            let arena = Arena::new();

            // attempt to load file
            let root = {
                log::info!(
                    "loading and parsing file: {}",
                    todo_file.file.to_string_lossy()
                );
                let contents = file::load_file(&todo_file);
                let root = file::parse_todo_file(&contents, &arena);
                root
            };
            log::trace!("file loaded");
            // extract sections specified in config
            let groups = file::extract_secitons(root, sections);
            log::trace!("sections extracted");
            // create new sections and generate empty sections for any that are missing
            let level = groups.values().map(|group| group.level).min().unwrap_or(2);
            let data = sections
                .iter()
                .map(|section| match groups.get(section) {
                    Some(group) => group.clone(),
                    None => TaskGroup::empty(section.to_string(), level),
                })
                .collect();

            // generate string for new file and write to filesystem
            let content = file::generate_file_content(&data, &today);
            let file_path = file::get_filepath(&data_dir, &today);
            log::info!("writing to file: {}", file_path.to_string_lossy());
            file::write_file(&file_path, &content);
            // return file name
            file_path
        }
        // returning the selected file
        Some(todo_file) => todo_file.file.to_owned(),
        // no note files exist creating based on template from config
        None => {
            // generate empty file
            let sections = &cfg.sections;
            log::info!("creating new empty file with sections: {:?}", sections);
            let data = sections
                .iter()
                .map(|sec| TaskGroup::empty(sec.clone(), 2))
                .collect();
            let content = file::generate_file_content(&data, &today);
            let file_path = file::get_filepath(&data_dir, &today);
            file::write_file(&file_path, &content);
            log::info!("writing to file: {}", file_path.to_string_lossy());
            // return file name
            file_path
        }
    };

    // opening file
    log::info!(
        "Opening {} in {}",
        current_file.to_string_lossy(),
        cfg.editor
    );
    Command::new(&cfg.editor)
        .args([current_file])
        .status()
        .expect(format!("failed to launch editor {}", &cfg.editor).as_str());
}
