
# Backr - Backs up user data

[![Crates.io](https://img.shields.io/crates/l/backr.svg)](https://crates.io/crates/backr) [![Crates.io](https://img.shields.io/crates/v/backr.svg)](https://crates.io/crates/backr) [![Build Status](https://travis-ci.org/martinak1/backr.svg?logo=travis&branch=master)](https://travis-ci.org/martinak1/backr) [![Crates.io](https://img.shields.io/crates/d/backr.svg)](https://crates.io/crates/backr)

## Please feel free to contribute, leave feedback, fork, or open an issue

## Examples

    Backup all data in the Home directory
    $ backr -a -s $HOME -d backup_dir

    Update an existing backup, showing a progress bar and using 5 threads
    $ backr -aupt 5 -s $HOME -d backup_dir

    Passing a custom regex so that only files/folders that match will be copied
    $ backr -r ".*(\.bak|\.cpp|\.rs)" -s $HOME -d backup_dir

## Flags

    -a, --backup-all
        Backup all files found, overriding the regex. Because of this, it
        conflicts with the regex option.

    -h, --help
        Prints help information

    -p, --progress
        Displays a progress bar during the backup.

    -u, --update
        If this flag is set, backr will check the metadata of the source 
        file and the already existing destination file, and will keep
        the newest one.

    -V, --version
        Prints version information

## Options

    -d, --destination <DESTINATION_PATH>
        The path to the location you want the data saved to.

    -o, --output_file <output_file>
        Specifies the location that failed transfer paths are written to
        [default: "<DESTINATION_PATH>/backr_log.txt"]

    -r, --regex <regex>
        Passes a regex to the program to only backup matching files and directories.
        [default: "Documents|Downloads|Movies|Music|Pictures|Videos"]

    -s, --source <SOURCE_PATH>
        The path to the User directory you want to backup.
        [default: <CURRENT_WORKING_DIRECTORY>]

    -t, --threads <NUM>
        Number of threads that will be used to backup files
        [default: 2]

## Goals

    * [ ] Make backup and walk functions concurrent
    * [ ] Create better benchmarks
    * [ ] Remove as many unwraps as I can from the source
    * [ ] Implement hash check options
    * [ ] Implement compression options
    * [ ] Implement incremental updates of compressed backups
    * [ ] Re-implement fs::copy so that it doesn't attempt to set permission bits