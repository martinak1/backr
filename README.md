
Backr - Backs up user data
===

[![Crates.io](https://img.shields.io/crates/l/backr.svg)](https://crates.io/crates/backr) [![Crates.io](https://img.shields.io/crates/v/backr.svg)](https://crates.io/crates/backr) [![Build Status](https://travis-ci.org/martinak1/backr.svg?branch=master)](https://travis-ci.org/martinak1/backr) [![Crates.io](https://img.shields.io/crates/d/backr.svg)](https://crates.io/crates/backr)

Change Log: 0.4.0
---

    Updated backrs documentation
    Implemented a progress bar
    Implemented an 'all' option to back up everything

Change Log: 0.3.0
---

    This a major rewrite of the previous version. This version is now 
    multi-threaded and implements the update option.

USAGE
---

    $ backr [OPTIONS] --destination DESTINATION_PATH

FLAGS
---

       -a, --backup-all
             Backup all files found, overriding the regex. Because of this, it conflicts with the 
             regex option.

       -h, --help 
             Prints help information 

       -p, --progress
             Displays a progress bar during the backup.

       -V, --version 
             Prints version information


OPTIONS
---

    -d, --destination <DESTINATION_PATH>
             The path to the location you want the data saved to.

    -o, --output_file <output_file>
             Specifies the location that failed transfer paths are written to
             [default: "<DESTINATION_PATH>/backr_log.txt"]

    -r, --regex <regex>
             Passes a regex to the program to only backup matching files
             and directories.
             [default: "Documents|Downloads|Movies|Music|Pictures|Videos"]

    -s, --source <SOURCE_PATH>
             The path to the User directory you want to backup.
             [default: <CURRENT_WORKING_DIRECTORY>]

    -t, --threads <NUM>
             Number of threads that will be used to backup files 
             [default: 2]

    -u, --update <update>
             If this flag is set, backr will check the metadata of the
             source file and the already existing destination
             file, and will keep the newest one.
             [default: false]