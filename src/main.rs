//! Backs up user data.
//! ```
//! USAGE:
//!     backr [FLAGS] [OPTIONS] --destination <DESTINATION_PATH>
//!
//! FLAGS:
//!     -a, --backup-all
//!         Backup all files found, overriding the regex. Because of this, it conflicts with the regex option.
//!
//!     -h, --help
//!         Prints help information
//!
//!     -L, --force-log
//!         Force a log to be written even if there are no errors to report
//!
//!     -p, --progress
//!         Displays a progress bar during the backup.
//!
//!     -u, --update
//!         If this flag is set, backr will check the metadata of the source
//!         file and the already existing destination file, and will keep the
//!         newest one.
//!
//!     -V, --version
//!         Prints version information
//!
//! OPTIONS:
//!     -d, --destination <DESTINATION_PATH>
//!         The path to the location you want the data saved too.
//!
//!     -l, --log <FILE_PATH>
//!         Specifies the log location that errors are written to [default: ]
//!
//!     -r, --regex <REGEX>
//!         Passes a regex to the program to only backup matching files and
//!         directories.
//!         [default:Documents|Downloads|Movies|Music|Pictures|Videos]
//!
//!     -s, --source <SOURCE_PATH>
//!         The path to the User directory you want to backup. [default: ./]
//!
//!     -t, --threads <NUM>
//!         Number of threads that will be used to backup files [default: 2]
//!
//!     -L, --force-log
//!         Writes a log, even if there are no errors to report
//! ```
//!
//!
//!
//! Note: When copying files from a Linux/Unix (ext4, apfs, etc.) filesystem
//! to a Windows (ntfs) the program will report that successful transfers
//! failed.This is due to the way fs::copy is implemented. It first creates a
//! copy of the files, then copies and set the permissions on the new file.
//! Copying the permissions is the cause of your error. Your files will still
//! be transferred, but the permissions will not.

// for cli parsing
extern crate clap;
use clap::{App, Arg};

// for interacting with the filesystem
use std::fs::{self, DirBuilder};
use std::io::prelude::Write;
use std::path::PathBuf;

// for filtering the files to be backed up
extern crate regex;
use regex::Regex;

// for multi-threading
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

// for progress bar
extern crate progress;
use progress::Bar;

fn main() {
    let gvars = GlobalVars::build();

    match check_permissions(gvars.source(), gvars.dest()) {
        true => {
            if gvars.quite() {
                println!(
                    "** {:?} is being used as the source directory \
                     \n** {:?} is being used as the destination directory \
                     \n** Searching for files to backup...",
                    gvars.source(),
                    gvars.dest()
                );
            }

            // get the job queue and read errors
            let (queue, mut errors, ..) = walk(
                Vec::<(PathBuf, PathBuf)>::new(),
                Vec::<String>::new(),
                gvars.source(),
                gvars.dest(),
                gvars.regex(),
                gvars.update(),
            );

            // note the queues length
            let queue_len = &queue.len();

            // Collect the read errors
            if gvars.quite() {
                println!(
                    "** {} files to backup and {} read errors.",
                    queue_len,
                    errors.len()
                );
            }

            // backup files and collect the errors
            errors.extend(backup(queue, gvars.threads(), gvars.bar(), gvars.quite()).into_iter());

            // Summarize
            if gvars.quite() {
                println!("** Files Backed Up: {}", queue_len - errors.len());
                println!("** Total errors {}", errors.len());
            }

            // write log if needed
            write_log(&mut errors, gvars.log(), gvars.quite(), gvars.force_log());
        }
        false => (),
    }
}

/// Backs up user data, by spawning the specified number of threads and
/// creating a queue for each one. It will collect errors from the
/// spawned threads and keeps track of the backup progress
// TODO setup up an option return type for error handling
fn backup(
    queue: Vec<(PathBuf, PathBuf)>,
    threads: i32,
    progress: bool,
    quite: bool,
) -> Vec<String> {
    if quite {
        println!("** Starting backup ");
    }

    // Keeps track of progress
    let total = queue.len();

    // to send to threads
    let errors_mutex = Arc::new(Mutex::new(Vec::<String>::new()));
    let queue_mutex = Arc::new(Mutex::new(queue.into_iter()));
    let completed_mutex = Arc::new(Mutex::new(0));

    // to join threads
    let mut handles = vec![];

    // create threads
    for _ in 0..threads {
        let (queue, errors, completed) = (
            queue_mutex.clone(),
            errors_mutex.clone(),
            completed_mutex.clone(),
        );

        let handle = thread::spawn(move || {
            // collect local errors
            let mut local_errors = vec![];

            'main: loop {
                // capture the current values then release the mutex
                let next = queue.lock().unwrap().next();

                match next {
                    Some((src, dest)) => {
                        // create parent dir if not already existing
                        if !dest.parent().unwrap().is_dir() {
                            DirBuilder::new()
                                .recursive(true)
                                .create(dest.parent().unwrap())
                                .unwrap();
                        }

                        // copy the file
                        match fs::copy(&src, &dest) {
                            Ok(_) => (),
                            Err(error) => {
                                if quite {
                                    println!("{}", &error);
                                }
                                let mut _errors = errors.lock().unwrap();
                                local_errors.push(format!(
                                    "Error: Failed to copy {:?} -> {:?} \n \
                                     {}",
                                    src, dest, error
                                ));
                            }
                        }
                    }
                    None => {
                        break 'main;
                    }
                }
                let mut completed = completed.lock().unwrap();
                *completed += 1;
            }
            // add all of the local errors to the programs error vec
            // then die
            errors.lock().unwrap().extend(local_errors.into_iter());
        });

        // collect the thread handles
        handles.push(handle);
    }
    // draw progress bar
    if progress {
        // create progress bar
        let mut bar = Bar::new();
        bar.set_job_title("Backup");

        // loop till percent >= 100
        'bar: loop {
            // get num completed then release the mutex
            let completed = *completed_mutex.lock().unwrap();
            let percent = ((completed as f32 / total as f32) * 100.0) as i32;
            bar.reach_percent(percent);

            if percent >= 100 {
                break 'bar;
            }
            // sleep so it doesn't interfere with the backup threads
            thread::sleep(time::Duration::from_secs(5));
        }
    }

    // join the threads
    handles.into_iter().for_each(|handel| {
        handel.join().unwrap();
    });

    // unwrap the Arc leaving the mutex
    let errors = Arc::try_unwrap(errors_mutex).unwrap();
    // return the vector that the mutex is holding
    errors.into_inner().unwrap()
}

/// Verify permissions on the src & dest. It reads the
/// first level of the src dir and creates, then deletes a file in the dest.
fn check_permissions(src: &PathBuf, dest: &PathBuf) -> bool {
    // verify read on src
    let src_read = match fs::read_dir(src) {
        Ok(_) => true,
        Err(error) => {
            println!(
                "Error: Failed to read the source directory {:?} \n{}.",
                src, error
            );
            false
        }
    };

    // verify write on dest
    let write_error_msg = format!("Error: You do not have write permissions for {:?}", dest);

    let dest_write = match dest.exists() {
        // Dest exists try to create a file in it
        true => {
            let tmp_path = dest.join("CanIWriteHere?.txt");
            match fs::File::create(&tmp_path) {
                Ok(_) => {
                    match fs::remove_file(tmp_path) {
                        Ok(_) => (),
                        Err(_) => {
                            println!("Error: Failed to delete the test file. The program will continue, but verify the backup after completion.");
                            ()
                        }
                    }
                    true
                }
                Err(error) => {
                    println!("{} \n{}", write_error_msg, error);
                    false
                }
            }
        }
        // Dest does not exist, attempt to create it
        false => match DirBuilder::new().recursive(true).create(&dest) {
            Ok(_) => true,
            Err(error) => {
                println!("{} \n{}", write_error_msg, error);
                false
            }
        },
    };

    src_read && dest_write
}

/// Iterates through the source directory and adds files that match a regex
/// to a queue. It also collects read errors
fn walk(
    mut queue: Vec<(PathBuf, PathBuf)>,
    mut errors: Vec<String>,
    source: &PathBuf,
    dest: &PathBuf,
    regex: &Regex,
    update: bool,
) -> (Vec<(PathBuf, PathBuf)>, Vec<String>) {
    // Verify the source dir
    let iter = match fs::read_dir(&source) {
        Ok(iter) => iter,
        Err(error) => {
            errors.push(format!("Failed to read {:?}.\n{}", &source, &error));
            return (queue, errors);
        }
    };

    for path in iter {
        //let src = path.unwrap().path();

        let src = match path {
            Ok(path) => path.path(),
            Err(err) => {
                println!("Error: Failed to read a path. Skipping! \n{}", err);
                continue;
            }
        };

        // if it matches the regex and is not a symlink
        if regex.is_match(&src.to_str().unwrap()) {
            let mut tmp_dest: PathBuf = PathBuf::from(&dest);
            tmp_dest.push(src.file_name().unwrap());

            // if src is a file
            if src.is_file() {
                match update {
                    // update flag is set
                    true => {
                        // If the existing destination file is newer than the source file, ignore it and continue looping
                        if tmp_dest.exists()
                            && (src.metadata().unwrap().modified().unwrap()
                                < tmp_dest.metadata().unwrap().modified().unwrap())
                        {
                            continue;
                        } else {
                            queue.push((src, tmp_dest));
                        }
                    }
                    false => {
                        queue.push((src, tmp_dest));
                        continue;
                    }
                }
            // if src is a dir
            } else if src.is_dir() {
                let (child_queue, child_errors) =
                    walk(vec![], vec![], &src, &tmp_dest, regex, update);

                queue.extend(child_queue.into_iter());

                errors.extend(child_errors.into_iter());
            }
        }
    }
    (queue, errors)
}

/// Writes all the read/write errors to a specified file. If there are no
/// errors creating a log will be skipped
fn write_log(errors: &mut Vec<String>, log: &PathBuf, quite: bool, force_log: bool) {
    if errors.is_empty() {
        match force_log {
            true => {
                errors.push(String::from("** Backr completed without error"));
            }
            false => {
                if quite {
                    println!("** There are no errors to report, so creating a log will be skipped");
                }
                return ();
            }
        }
    }

    match fs::File::create(log) {
        Ok(mut file) => {
            if quite {
                println!("** Writing log to {:?}", log);
            }
            for error in errors {
                //file.write_fmt(format_args!("{}", error)).unwrap();
                match file.write_fmt(format_args!("{}", error)) {
                    Ok(_) => (),
                    Err(_) => {
                        println!("Error: {}", error);
                    }
                }
            }
        }
        Err(error) => {
            if quite {
                println!("ERROR: Failed to create log file \n{}", error);
                println!("** Dumping errors to stdout\n");
                for error in errors {
                    println!("{}", error);
                }
            }
        }
    }
}

#[derive(Debug)]
/// Encapsulates information that is used throughout the program.
/// This includes useful stats and the source and destination paths.
pub struct GlobalVars {
    // Path info
    /// The path to the source
    pub source: PathBuf,
    /// The path to the destination
    pub destination: PathBuf,

    /// If it output_file is empty, then errors are instead written
    /// to DESTINATION/backr_log.txt
    pub log: PathBuf,

    /// Regex specifying what files/folders should be ignored
    pub regex: Regex,

    /// i32 representing the number of threads used for backing up files
    pub threads: i32,

    /// Flag that determines overwrite/update behavior
    pub update: bool,

    /// Flag that determines if a progress bar is drawn or not
    pub bar: bool,

    /// Flag that determines if anything is printed to stdout
    pub quite: bool,

    /// Flag that forces a log to be written
    pub force_log: bool,
}

/// # Methods
impl GlobalVars {
    /// Returns the source path
    pub fn source(&self) -> &PathBuf {
        &self.source
    }

    /// Returns the destination path
    pub fn dest(&self) -> &PathBuf {
        &self.destination
    }

    /// Returns the output_file path
    pub fn log(&self) -> &PathBuf {
        &self.log
    }

    /// Returns the regex for filtering
    pub fn regex(&self) -> &Regex {
        &self.regex
    }

    /// Returns the number of threads to use when backing up data
    pub fn threads(&self) -> i32 {
        self.threads
    }

    /// Returns a bool saying if the progress bar should be drawn
    pub fn bar(&self) -> bool {
        self.bar
    }

    /// Returns a bool determining if backr prints to stdout
    pub fn quite(&self) -> bool {
        !self.quite
    }

    /// Returns a bool determining if backr forces a log file
    pub fn force_log(&self) -> bool {
        self.force_log
    }

    /// Returns a bool determining if the latest version of the file should be
    /// kept
    pub fn update(&self) -> bool {
        self.update
    }

    /// Sets the output_file
    pub fn set_of(&mut self, log: PathBuf) {
        if log == PathBuf::from("") {
            let mut path = self.destination.clone();
            path.push("backr_log.txt");
            self.log = path;
        } else {
            self.log = log;
        }
    }
}

/// # Functions
impl GlobalVars {
    /// Generates the GlobalVars struct from params captured by clap
    // TODO SETUP return type a an option to remove unwraps
    fn from(cli: &clap::ArgMatches) -> GlobalVars {
        // set the source path
        let source = PathBuf::from(cli.value_of("source").unwrap_or_default());

        // generate the dest path
        let destination: PathBuf = match cli.value_of("destination") {
            Some(path) => {
                let mut path = PathBuf::from(path);
                path.push(source.file_name().unwrap());
                path
            }
            _ => panic!("Failed to extract the destination from the CLI"),
        };
        // add the root source file/folder name to the dest

        let log = match cli.value_of("log") {
            Some(path) => PathBuf::from(path),
            _ => PathBuf::new(),
        };

        let regex = match cli.is_present("all") {
            true => r#".*"#,
            false => cli.value_of("regex").unwrap_or_default(),
        };

        let threads: i32 = cli
            .value_of("threads")
            .unwrap_or_default()
            .parse::<i32>()
            .unwrap();

        //let update: bool = cli.value_of("update").unwrap().parse::<bool>().unwrap();

        // create the new struct that will hold data
        GlobalVars {
            source,
            destination,
            log,
            regex: Regex::new(regex).unwrap(),
            threads,
            update: cli.is_present("update"),
            bar: cli.is_present("progress"),
            quite: cli.is_present("quite"),
            force_log: cli.is_present("force_log"),
        }
    }

    /// Creates a struct that manages defaults and cli
    fn build() -> GlobalVars {
        GlobalVars::from(
            &App::new("Backr")
                .version("0.5.0")
                .author("martinak1 <https://github.com/martinak1>")
                .about("Backs up user data.")
                .arg(
                    Arg::with_name("source")
                        .short("s")
                        .long("source")
                        .value_name("SOURCE_PATH")
                        .help("The path to the User directory you want to backup.")
                        .takes_value(true)
                        .default_value("./"),
                ).arg(
                    Arg::with_name("destination")
                        .short("d")
                        .long("destination")
                        .value_name("DESTINATION_PATH")
                        .help("The path to the location you want the data saved too.")
                        .takes_value(true)
                        .required(true),
                ).arg(
                    Arg::with_name("update")
                        .short("u")
                        .long("update")
                        .help(
                            "Tells backer to update the files instead of\
                             overwriting them.",
                        ).long_help(
                            "If this flag is set, backr will check the\
                             metadata of the source file and the already existing\
                             destination file, and will keep the newest one.",
                        ),
                ).arg(
                    Arg::with_name("log_file")
                        .short("l")
                        .long("log")
                        .value_name("FILE_PATH")
                        .help(
                            "Specifies the log location that errors are\
                             written to",
                        ).takes_value(true)
                        .default_value(""),
                ).arg(
                    Arg::with_name("regex")
                        .short("r")
                        .long("regex")
                        .value_name("REGEX")
                        .help(
                            "Passes a regex to the program to \
                             only backup matching files and directories.",
                        ).takes_value(true)
                        .default_value("Documents|Downloads|Movies|Music|Pictures|Videos"),
                ).arg(
                    Arg::with_name("threads")
                        .short("t")
                        .long("threads")
                        .value_name("NUM")
                        .help("Number of threads that will be used to backup files")
                        .default_value("2"),
                ).arg(
                    Arg::with_name("all")
                        .short("a")
                        .long("backup-all")
                        .help(
                            "Backup all files found, overriding the regex. Because\
                             of this, it conflicts with -r, --regex.",
                        ).conflicts_with("regex"),
                ).arg(
                    Arg::with_name("progress")
                        .short("p")
                        .long("progress")
                        .help("Displays a progress bar during the backup."),
                ).arg(
                    Arg::with_name("quite")
                        .short("q")
                        .long("quite")
                        .conflicts_with("progress")
                        .help(
                            "Stop backr from printing to stdout. As such it\
                             conflicts with -p, --progress",
                        ),
                ).arg(
                    Arg::with_name("force_log")
                        .short("L")
                        .long("force-log")
                        .help(
                            "Forces a log to be written, even if there are no\
                             errors to report.",
                        ),
                ).get_matches(),
        )
    }
}
