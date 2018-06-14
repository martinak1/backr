//! Backs up user profile data.
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
//!     -p, --progress
//!         Displays a progress bar during the backup.
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
//!     -u, --update <update>
//!         If this flag is set, backr will check the metadata of the source
//!         file and the already existing destination file, and will keep the
//!         newest one. [default: false]
//! ```
//!
//!
//!
//! Note: When copying files from a Linux/Unix (ext4, apfs, etc.) filesystem
//! to a Windows (ntfs) the program will report that sucessfull transfers
//! failed.This is due to the way fs::copy is implemented. It first creates a
//! copy of the files, then copies and set the permissions on the new file.
//! Copying the permissions is the cause of your error. Your files will still
//! be transfered, but the permissions will not.

//#![feature(rustc_private)]

// for cli partsing
extern crate clap;
use clap::{App, Arg};

// for interacting with the filesystem
use std::fs::{self, DirBuilder};
use std::io::prelude::Write;
use std::path::PathBuf;

// for filtering the files to be backedup
extern crate regex;
use regex::Regex;

// for multi-threading
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

// for progress bar
extern crate progress;
use progress::Bar;

#[derive(Debug)]
/// Encapsulates information that is used throughout the program.
/// This includes usefull stats and the source and destination paths.
pub struct GlobalVars {
    // Path info
    /// The path to the source
    pub source: PathBuf,
    /// The path to the destination
    pub destination: PathBuf,

    /// If it output_file is empty, then failed_list is instead written
    /// to DESTINATION/backr_log.txt
    pub log: PathBuf,

    /// Regex specifying what files/folders should be ignored
    pub regex: Regex,

    /// i32 representing the number of threads used for backingup files
    pub threads: i32,

    /// Flag that determines overwrite/update behavior
    pub update: bool,

    /// Flag that determines if a progress bar is drawn or not
    pub bar: bool,
}

/// # Functions
impl GlobalVars {
    /// Generates the GlobalVars struct from params captured by clap
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

        //let regex: &str =
        //&format_args!(r"{}", cli.value_of("regex").unwrap_or_default()).to_string();

        let regex = match cli.is_present("all") {
            true => r#"\\*"#,
            false => cli.value_of("regex").unwrap_or_default(),
        };

        let threads: i32 = cli
            .value_of("threads")
            .unwrap_or_default()
            .parse::<i32>()
            .unwrap();

        let update: bool = cli.value_of("update").unwrap().parse::<bool>().unwrap();

        // create the new struct that will hold data
        GlobalVars {
            source,
            destination,
            log,
            regex: Regex::new(regex).unwrap(),
            threads,
            update,
            bar: cli.is_present("progress"),
        }
    }
}

/// # Methods
impl GlobalVars {
    /// Returns the source path
    pub fn get_source(&self) -> &PathBuf {
        &self.source
    }

    /// Returns the destination path
    pub fn get_dest(&self) -> &PathBuf {
        &self.destination
    }

    /// Returns the output_file path
    pub fn get_log(&self) -> &PathBuf {
        &self.log
    }

    /// Returns the regex for filtering
    pub fn get_regex(&self) -> &Regex {
        &self.regex
    }

    /// Returns the number of threads to use when backing up data
    pub fn get_threads(&self) -> i32 {
        self.threads
    }

    /// Returns a boolian saying if the progress bar should be drawn
    pub fn draw_bar(&self) -> bool {
        self.bar
    }
}

impl GlobalVars {
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

fn main() {
    let cli = App::new("Backr")
        .version("0.4.0")
        .author("AM <martinak@mymail.vcu.edu>")
        .about("Backs up user data.")
        .arg(
            Arg::with_name("source")
                .short("s")
                .long("source")
                .value_name("SOURCE_PATH")
                .help("The path to the User directory you want to backup.")
                .takes_value(true)
                .default_value("./"),
        )
        .arg(
            Arg::with_name("destination")
                .short("d")
                .long("destination")
                .value_name("DESTINATION_PATH")
                .help("The path to the location you want the data saved too.")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("update")
                .short("u")
                .long("update")
                .help(
                    "Tells backer to update the files instead of clobering \
                     them.",
                )
                .long_help(
                    "If this flag is set, backr will check the \
                     metadata of the source file and the already existing \
                     destination file, and will keep the newest one.",
                )
                .takes_value(true)
                .default_value("false"),
        )
        .arg(
            Arg::with_name("log_file")
                .short("l")
                .long("log")
                .value_name("FILE_PATH")
                .help(
                    "Specifies the log location that errors are \
                     written to",
                )
                .takes_value(true)
                .default_value(""),
        )
        .arg(
            Arg::with_name("regex")
                .short("r")
                .long("regex")
                .value_name("REGEX")
                .help(
                    "Passes a regex to the program to \
                     only backup matching files and directories.",
                )
                .takes_value(true)
                .default_value("Documents|Downloads|Movies|Music|Pictures|Videos"),
        )
        .arg(
            Arg::with_name("threads")
                .short("t")
                .long("threads")
                .value_name("NUM")
                .help("Number of threads that will be used to backup files")
                .default_value("2"),
        )
        .arg(
            Arg::with_name("all")
                .short("a")
                .long("backup-all")
                .help(
                    "Backup all files found, overriding the regex. Because \
                     of this, it conflicts with the regex option.",
                )
                .conflicts_with("regex"),
        )
        .arg(
            Arg::with_name("progress")
                .short("p")
                .long("progress")
                .help("Displays a progress bar during the backup."),
        )
        .get_matches();

    let gvars = GlobalVars::from(&cli);

    println!(
        "** {:?} is being used as the source directory",
        gvars.get_source()
    );
    println!(
        "** {:?} is being used as the destination directory",
        gvars.get_dest()
    );

    // Create the destination if it does not exist and begin walking through
    // the directories recursively
    //create_dest(&gvars);
    //let (mut failed_paths, total_file_count) = walk(gvars.get_source(), gvars.get_dest());

    // get the job queue and read errors
    println!("** Searching for files to backup...");
    let (queue, mut errors, ..) = walk(
        Vec::<(PathBuf, PathBuf)>::new(),
        Vec::<String>::new(),
        &gvars.source,
        &gvars.destination,
        &gvars.regex,
        gvars.update,
    );

    // Collect the read errors
    println!(
        "** Found {} files to backup and {} read errors.",
        queue.len(),
        errors.len()
    );

    let queue_len = &queue.len();

    // collect the write errors
    errors.extend(backup(queue, gvars.get_threads(), gvars.draw_bar()).into_iter());

    // Summarize
    println!("** Files Backed Up: {}", queue_len - errors.len());
    println!("** Total errors {}", errors.len());

    // write log if needed
    write_log(errors, gvars.get_log());
}

/// Backs up user data, by spawning the specified number of threads and
/// creating a queue for each one. It will collect errors from the
/// spawned threads and keeps track of the backups progress
fn backup(queue: Vec<(PathBuf, PathBuf)>, threads: i32, progress: bool) -> Vec<String> {
    println!("** Starting backup ");

    // Keeps track of progress
    let total = queue.len();

    // returned to main
    //let mut errors: Vec<String> = vec![];

    // to send to threads
    let errors_mutex = Arc::new(Mutex::new(Vec::<String>::new()));
    let queue_mutex = Arc::new(Mutex::new(queue.into_iter()));
    let completed_mutex = Arc::new(Mutex::new(0));

    // to join threads
    let mut handels = vec![];

    // create threads
    for _ in 0..threads {
        let (queue, errors, completed) = (
            queue_mutex.clone(),
            errors_mutex.clone(),
            completed_mutex.clone(),
        );

        let handel = thread::spawn(move || {
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
                                println!("{}", &error);
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

        // collect the thread handels
        handels.push(handel);
    }
    // draw progress bar
    if progress {
        // create progress bar
        let mut bar = Bar::new();
        bar.set_job_title("Backup");

        // loop till percent >= 100
        'bar: loop {
            // get num completed then release the mutex
            let cmplt = *completed_mutex.lock().unwrap();
            let perc = ((cmplt as f32 / total as f32) * 100.0) as i32;
            bar.reach_percent(perc);

            if perc >= 100 {
                break 'bar;
            }
            // sleep so it doesn't interfere with the backup threads
            thread::sleep(time::Duration::from_secs(5));
        }
    }

    // join the threads
    handels.into_iter().for_each(|handel| {
        handel.join().unwrap();
    });

    // unwrap the Arc leaving the mutex
    let errors = Arc::try_unwrap(errors_mutex).unwrap();
    // return the vector that the mutex is holding
    errors.into_inner().unwrap()
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
        let src = path.unwrap().path();

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
fn write_log(errors: Vec<String>, log: &PathBuf) {
    if errors.is_empty() {
        println!("** There are no errors to report, so creating a log will be skipped");
        return ();
    }

    match fs::File::create(log) {
        Ok(mut file) => {
            println!("** Writing log to {:?}", log);
            for error in errors {
                file.write_fmt(format_args!("{}", error)).unwrap();
            }
        }
        Err(error) => {
            println!("ERROR: Failed to create log file \n{}", error);
            println!("** Dumping errors to stdout\n");
            for error in errors {
                println!("{}", error);
            }
        }
    }
}
