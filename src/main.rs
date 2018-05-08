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
//!    -o, --output_file <output_file>
//!             Specifies the location that failed transfer paths are written to
//!             [default: "<DESTINATION_PATH>/backr_log.txt"]
//!
//!    -r, --regex <regex>
//!             Passes a regex to the program to only backup matching files
//!             and directories.
//!             [default: "Documents|Downloads|Movies|Music|Pictures|Videos"]
//!
//!    -s, --source <USER_PROFILE>
//!             The path to the User directory you want to backup.
//!             [default: <CURRENT_WORKING_DIRECTORY>]
//!
//! Unimplemented:
//!
//!    -u, --update <update>
//!             If this flag is set, backr will check the metadata of the
//!             source file and the already existing destination
//!             file, and will keep the newest one.
//!             [default: false]
//! ```
//!
//! # TODO
//! ```
//! 1) Add a second thread with a progress bar
//! 2) Implement the update option
//! ```

// for cli partsing
extern crate clap;
use clap::{App, Arg};

// for interacting with the filesystem
use std::fs;
use std::path::PathBuf;
use std::env;

// for filtering the files to be backedup
extern crate regex;
use regex::Regex;

#[derive(Debug)]
/// Encapsulates information that is used throughout the program.
/// This includes usefull stats and the source and destination paths.
pub struct GlobalVars {
    // Stats
    /// The amount of files that have been moved
    pub files_moved: i32,
    /// The number of files that failed to transfer
    pub files_failed: i32,

    /// A list of paths to files that failed to transfer and the associated
    /// error
    pub failed_list: Vec<(PathBuf, std::io::Error)>,

    /// The optional output file that failed_list is written to.
    /// If it output_file is empty, then failed_list is instead written
    /// to stdout
    pub output_file: PathBuf,

    // Path info
    /// The path to the source
    pub source: PathBuf,
    /// The path to the destination
    pub destination: PathBuf,

    /// Regex specifying what files/folders should be ignored
    pub regex: Regex,
}

/// # Functions
impl GlobalVars {
    fn new(source: PathBuf, destination: PathBuf, pattern: &str) -> GlobalVars {
        GlobalVars {
            files_moved: 0,
            files_failed: 0,
            failed_list: Vec::new(),
            output_file: PathBuf::new(),
            source,
            destination,
            regex: Regex::new(pattern).unwrap(),
        }
    }

    /// Generates the GlobalVars struct from params captured by clap
    fn from(cli: &clap::ArgMatches) -> GlobalVars {
        // set the source path
        let source = PathBuf::from(cli.value_of("source").unwrap_or_default());

        // generate the dest path
        let dest: PathBuf = match cli.value_of("destination") {
            Some(path) => {
                let mut path = PathBuf::from(path);
                path.push(source.file_name().unwrap());
                path
            }
            _ => panic!("Failed to extract the destination from the CLI"),
        };
        // add the root source file/folder name to the dest

        let output_file = match cli.value_of("output_file") {
            Some(path) => PathBuf::from(path),
            _ => PathBuf::new(),
        };

        let regex: &str =
            &format_args!(r"{}", cli.value_of("regex").unwrap_or_default()).to_string();

        // create the new struct that will hold data
        let mut gvars = GlobalVars::new(source, dest, regex);
        gvars.set_of(output_file);
        gvars
    }
}

/// # Methods
impl GlobalVars {
    /// Add failed paths to the failed_list
    pub fn append(&mut self, paths: &mut Vec<(PathBuf, std::io::Error)>) {
        self.failed_list.append(paths);
    }

    /// Returns the source path
    pub fn get_source(&self) -> &PathBuf {
        &self.source
    }

    /// Returns the destination path
    pub fn get_dest(&self) -> &PathBuf {
        &self.destination
    }

    /// Returns the output_file path
    pub fn get_of(&self) -> &PathBuf {
        &self.output_file
    }

    /// Returns the regex for filtering
    pub fn get_regex(&self) -> &Regex {
        &self.regex
    }

    /// Writes the failed paths to a file if an output_file is set
    /// or prints them to stdout
    // TODO FIx hangle_failed_files, it does not write to file successfully
    pub fn handle_failed_files(&self) {
        use std::io::prelude::*;
        use std::fs::File;

        match File::create(self.get_of()) {
            Ok(mut of) => for file_path in self.failed_list.iter() {
                of.write_fmt(format_args!("{}\n{}", file_path.0.display(), file_path.1))
                    .unwrap()
            },
            _ => {
                println!(
                    "Error: Failed to create log file. \
                     \n Failed transfers will instead be printed to stdout."
                );
                for i in self.failed_list.iter() {
                    println!("\t{:?}", i);
                }
            }
        }
    }

    /// Increments the number of files successfully backedup
    pub fn add_moved(&mut self, fc: i32) {
        self.files_moved += fc
    }

    /// Increments the number of files that failed to transfer
    pub fn get_failed_count(&self) -> usize {
        self.failed_list.len()
    }

    /// Checks if a path matches the desired regex
    pub fn is_match(&self, path: &str) -> bool {
        // Makes it so the regex is only compiled once
        self.get_regex().is_match(path)
    }

    /// Sets the output_file
    pub fn set_of(&mut self, of: PathBuf) {
        if of == PathBuf::from("") {
            let mut path = self.destination.clone();
            path.push("backr_log.txt");
            self.output_file = path;
        } else {
            self.output_file = of;
        }
    }

    pub fn summarize(&self) {
        println!(
            "\n\nSummary: \
             \n-------------------------------------------------------- \
             \n\tSource: {} \
             \n\tDestination: {} \
             \n\tSuccessfull Transfers: {} \
             \n\tFailed Transfers: {} \
             \n\tLog File: {}",
            self.source.display(),
            self.destination.display(),
            self.files_moved,
            self.get_failed_count(),
            self.output_file.display()
        );
    }

    /// Returns the total number of files processed
    pub fn total_files(&self) -> i32 {
        self.get_failed_count() as i32 + self.files_moved
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
        .version("0.1")
        .author("AM <martinak@mymail.vcu.edu>")
        .about("Backs up user profile data.")
        .arg(
            Arg::with_name("source")
                .short("s")
                .long("source")
                .value_name("USER_PROFILE")
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
            Arg::with_name("output_file")
                .short("o")
                .long("output_file")
                .help(
                    "Specifies the location that failed transfer paths are \
                     written to",
                )
                .takes_value(true)
                .default_value(""),
        )
        .arg(
            Arg::with_name("regex")
                .short("r")
                .long("regex")
                .help(
                    "Passes a regex to the program to \
                     only backup matching files and directories.",
                )
                .takes_value(true)
                .default_value("Documents|Downloads|Movies|Music|Pictures|Videos"),
        )
        .get_matches();

    let mut gvars = GlobalVars::from(&cli);

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
    create_dest(&gvars);
    //let (mut failed_paths, total_file_count) = walk(gvars.get_source(), gvars.get_dest());

    let (mut failed_paths, total_file_count) =
        walk(gvars.get_source(), gvars.get_dest(), gvars.get_regex());
    // Update stats and output, then handle them
    gvars.append(&mut failed_paths);
    gvars.add_moved(total_file_count);
    gvars.handle_failed_files();
    gvars.summarize();
}

/// Creates the backup destination folder if it does not already exist
fn create_dest(gvars: &GlobalVars) {
    if !gvars.get_dest().is_dir() {
        println!(
            "** Creating root destination dir at {}",
            gvars.get_dest().display()
        );
        match fs::create_dir_all(gvars.get_dest()) {
            Ok(_) => println!("** Root destination folder created"),
            Err(error) => panic!(
                "\nERROR: Failed to create the root destination folder. \
                 Run backr again with elevated privleges\n {}",
                error
            ),
        }
    }
}

/// # Functions
/// Recursivly itterates over files and directories backing them up
fn walk(source: &PathBuf, dest: &PathBuf, regex: &Regex) -> (Vec<(PathBuf, std::io::Error)>, i32) {
    let mut failed_list: Vec<(PathBuf, std::io::Error)> = Vec::new();
    let itter = match fs::read_dir(source) {
        Ok(itter) => itter,
        Err(error) => {
            println!(
                "\nERROR: Failed to read {}. SKIPPING! \
                 \nTry running backr again as root if you want this to be copied.\
                 \n{}",
                source.display(),
                error
            );
            failed_list.push((source.clone(), error));
            return (failed_list, 0);
        }
    };
    let mut file_count: i32 = 0;
    for entry in itter {
        let path = entry.unwrap().path();

        // Copy if it matches the regex and is not a symlink
        if regex.is_match(path.clone().into_os_string().to_str().unwrap())
            && !fs::symlink_metadata(&path)
                .unwrap()
                .file_type()
                .is_symlink()
        {
            //let tmp_dest = PathBuff::new(dest).join(path.file_name());
            let mut tmp_dest: PathBuf = PathBuf::from(&dest);
            //tmp_dest.push(&path.file_name().unwrap().to_os_string());
            tmp_dest.push(&path.file_name().unwrap());

            // TODO add a flag for update vs skip
            if path.is_file() && !tmp_dest.exists() {
                // TODO Add a section to check for verbosity
                println!("** {:?} -> {:?}", path, tmp_dest);
                // TODO add error handeling
                match fs::copy(&path, &tmp_dest) {
                    Err(error) => {
                        failed_list.push((path, error));
                    }
                    Ok(_) => {
                        file_count += 1;
                        ()
                    }
                }
            } else if path.is_dir() {
                // TODO add error handeling
                if !tmp_dest.exists() {
                    fs::create_dir(&tmp_dest).unwrap();
                }
                //failed_list.append(walk(&path, &tmp_dest).as_mut());
                let (mut fl, fc) = walk(&path, &tmp_dest, &regex);
                file_count += fc;
                failed_list.append(&mut fl);
            }
        }
    }
    (failed_list, file_count)
}

/* fn demo_walk(
    source: &PathBuf,
    dest: &PathBuf,
    regex: &Regex,
) -> (Vec<(PathBuf, std::io::Error)>, i32) {
    let mut failed_list: Vec<(PathBuf, std::io::Error)> = Vec::new();

    let itter = match fs::read_dir(source) {
        Ok(itter) => itter,
        Err(error) => {
            println!(
                "\nERROR: Failed to read {}. SKIPPING! \
                 \nTry running backr again as root if you want this to be copied.\
                 \n{}",
                source.display(),
                error
            );
            failed_list.push((source.clone(), error));
            return (failed_list, 0);
        }
    };
    let mut file_count: i32 = 0;
    for entry in itter {
        let path = entry.unwrap().path();

        if regex.is_match(path.clone().into_os_string().to_str().unwrap()) {
            //let tmp_dest = PathBuff::new(dest).join(path.file_name());
            let mut tmp_dest: PathBuf = PathBuf::from(&dest);
            //tmp_dest.push(&path.file_name().unwrap().to_os_string());
            tmp_dest.push(&path.file_name().unwrap());

            // TODO add a flag for update vs skip
            if path.is_file() && !tmp_dest.exists() {
                // TODO Add a section to check for verbosity
                println!("** {:?} -> {:?}", path, tmp_dest);
            // TODO add error handeling
            } else if path.is_dir() {
                // TODO add error handeling
                if !tmp_dest.exists() {
                    fs::create_dir(&tmp_dest).unwrap();
                }
                //failed_list.append(walk(&path, &tmp_dest).as_mut());
                let (mut fl, fc) = demo_walk(&path, &tmp_dest, &regex);
                file_count += fc;
                failed_list.append(&mut fl);
            }
        }
    }
    (failed_list, file_count)
}
 */
