//! Backs up user profile data.
//! ```
//! USAGE:
//!    backr [OPTIONS] --destination <DESTINATION_PATH>
//!
//!FLAGS:
//!    -h, --help
//!             Prints help information
//!
//!    -V, --version
//!             Prints version information
//!
//!
//! OPTIONS:
//!
//!    -d, --destination <DESTINATION_PATH>
//!             The path to the location you want the data saved to.
//!
//!    -l, --log <FILE_PATH>
//!             Specifies the log location that errors are written to
//!             [default: "<DESTINATION_PATH>/backr_log.txt"]
//!
//!    -r, --regex <REGEX>
//!             Passes a regex to the program to only backup matching files
//!             and directories.
//!             [default: "Documents|Downloads|Movies|Music|Pictures|Videos"]
//!
//!    -s, --source <SOURCE_PATH>
//!             The path to the User directory you want to backup.
//!             [default: <CURRENT_WORKING_DIRECTORY>]
//!
//!    -t, --threads <NUM>
//!
//!    -u, --update <update>
//!             If this flag is set, backr will check the metadata of the
//!             source file and the already existing destination
//!             file, and will keep the newest one.
//!             [default: false]
//! ```
//!
//!
//!
//! Note: When copying files from a Linux/Unix (ext4, apfs) filesystem to a
//! Windows (ntfs) the program will report that sucessfull transfers failed.
//! This is due to the way fs::copy is implemented. It first creates a copy of
//! the files, then copies and set the permissions on the new file. Copying the
//! permissions is the cause of your error. Your files will still be transfered,
//! but the permissions will not.
//#![feature(rustc_private)]

// for cli partsing
extern crate clap;
use clap::{App, Arg};

// for interacting with the filesystem
use std::env;
use std::fs::{self, DirBuilder};
use std::io::prelude::*;
use std::path::PathBuf;

// for filtering the files to be backedup
extern crate regex;
use regex::Regex;

// for multi-threading
use std::sync::mpsc;
use std::thread;
use std::time;

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

        let regex: &str =
            &format_args!(r"{}", cli.value_of("regex").unwrap_or_default()).to_string();

        let threads: i32 = cli.value_of("threads").unwrap().parse::<i32>().unwrap();

        let update: bool = cli.value_of("update").unwrap().parse::<bool>().unwrap();

        // create the new struct that will hold data
        GlobalVars {
            source,
            destination,
            log,
            regex: Regex::new(regex).unwrap(),
            threads,
            update,
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

    pub fn get_threads(&self) -> i32 {
        self.threads
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
    /* TODO
     * 1) Add a progress bar
     */

    // get the current working dir
    let cwd = env::current_dir().unwrap();
    let cwd = cwd.to_str().unwrap();

    let cli = App::new("Backr")
        .version("0.2.2")
        .author("AM <martinak@mymail.vcu.edu>")
        .about("Backs up user profile data.")
        .arg(
            Arg::with_name("source")
                .short("s")
                .long("source")
                .value_name("SOURCE_PATH")
                .help("The path to the User directory you want to backup.")
                .takes_value(true)
                .default_value(cwd),
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
        "** Found {} files to backup and {} read errors. \n** Starting backup",
        queue.len(),
        errors.len()
    );

    let queue_len = &queue.len();

    // collect the write errors
    errors.extend(backup(queue, gvars.get_threads()).into_iter());

    // Summarize
    println!("** Files Backed Up: {}", queue_len - errors.len());
    println!("** Total errors {}", errors.len());

    // write log if needed
    write_log(errors, gvars.get_log());
}

fn backup(queue: Vec<(PathBuf, PathBuf)>, max: i32) -> Vec<String> {
    // returned to main
    let mut errors: Vec<String> = vec![];

    // for error communication
    let (err_send, err_recv): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();

    // for thread communication and handeling
    let mut channels: Vec<mpsc::Sender<Option<(PathBuf, PathBuf)>>> = vec![];
    let mut handels = vec![];

    // create threads
    for i in 0..max {
        // create a copy of the send channel for the thread
        let child_err_send = err_send.clone();

        // create the channel to send and recieve jobs
        let (send_path, recv_path): (
            mpsc::Sender<Option<(PathBuf, PathBuf)>>,
            mpsc::Receiver<Option<(PathBuf, PathBuf)>>,
        ) = mpsc::channel();

        // create the thread and handel
        let mut handel = thread::spawn(move || 'main: loop {
            match recv_path.try_recv() {
                Ok(option) => match option {
                    Some((src, dest)) => {
                        let dest_prnt = dest.parent().unwrap();
                        if !dest_prnt.is_dir() {
                            DirBuilder::new().recursive(true).create(dest_prnt).unwrap();
                        }

                        match fs::copy(&src, &dest) {
                            Ok(_) => (),
                            Err(error) => {
                                println!("Thread {}: ERROR: {}", i, error);
                                child_err_send
                                    .send(format!("Thread {}: {}", i, error))
                                    .unwrap();
                            }
                        }
                    }
                    None => break 'main,
                },
                Err(_) => thread::sleep(time::Duration::from_secs(2)),
            }
        });

        // push( thread #, send channel, thread handel)
        channels.push(send_path);
        handels.push(handel);
    }

    // iter the queue till you reach the end sending jobs to the threads
    let mut queue_iter = queue.into_iter();
    // cycling iter for job distribution to the threads
    let channels_iter = channels.clone();
    let mut channels_iter = channels_iter.iter().cycle();

    while let Some(job) = queue_iter.next() {
        let job_chnl = channels_iter.next().unwrap();
        job_chnl.send(Some(job)).unwrap();
    }

    // Send a None to the threads to break them from their loops
    channels.into_iter().for_each(|channel| {
        channel.send(None).unwrap();
    });

    // join the threads
    handels.into_iter().for_each(|handel| {
        handel.join().unwrap();
    });
    println!("** All threads rejoined");

    // collect errors
    errors.extend(err_recv.try_recv().into_iter());
    errors
}

fn walk(
    mut queue: Vec<(PathBuf, PathBuf)>,
    mut errors: Vec<String>,
    source: &PathBuf,
    dest: &PathBuf,
    regex: &Regex,
    update: bool,
) -> (Vec<(PathBuf, PathBuf)>, Vec<(String)>) {
    // Verify the source dir
    //println!("** Reading dir {:?}", &source);
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
        if regex.is_match(src.clone().to_str().unwrap())
        //&& !src.symlink_metadata().unwrap().file_type().is_symlink() // is this redundant?
        {
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
