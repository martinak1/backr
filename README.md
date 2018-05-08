 <h1>Backr - Backs up user data.</h1>
[![Github file size](https://img.shields.io/github/size/webcaetano/craft/build/phaser-craft.min.js.svg)](https://github.com/martinak1/backr)[![Build Status](https://travis-ci.org/martinak1/backr.svg?branch=master)](https://travis-ci.org/martinak1/backr)[![Crates.io](https://img.shields.io/crates/v/rustc-serialize.svg)](https://crates.io/crates/backr)
 

 <h2>USAGE:</h2>

    $ backr [OPTIONS] --destination DESTINATION_PATH

<h2>FLAGS:</h2>

       -h, --help 
             Prints help information 

       -V, --version 
             Prints version information


 <h2>OPTIONS:</h2>

    -d, --destination <DESTINATION_PATH>
             The path to the location you want the data saved to.

    -o, --output_file <output_file>
             Specifies the location that failed transfer paths are written to
             [default: "<DESTINATION_PATH>/backr_log.txt"]

    -r, --regex <regex>
             Passes a regex to the program to only backup matching files
             and directories.
             [default: "Documents|Downloads|Movies|Music|Pictures|Videos"]

    -s, --source <USER_PROFILE>
             The path to the User directory you want to backup.
             [default: <CURRENT_WORKING_DIRECTORY>]

 <h2>Unimplemented:</h2>

    -u, --update <update>
             If this flag is set, backr will check the metadata of the
             source file and the already existing destination
             file, and will keep the newest one.
             [default: false]

 <h2>TODO:</h2>

 1) Add a second thread with a progress bar
 2) Implement the update option
