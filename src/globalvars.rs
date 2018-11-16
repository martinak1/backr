// for cli parsing
use clap::{App, Arg};
use std::path::PathBuf;
use regex::Regex;

/// Encapsulates information that is used throughout the program.
/// This includes useful stats and the source and destination paths.
#[derive(Debug)]
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
    pub fn from(cli: &clap::ArgMatches) -> GlobalVars {
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
    pub fn build() -> GlobalVars {
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