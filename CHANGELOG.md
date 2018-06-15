# Change Log

## ver: 0.5.0

    * Implemented the -q --quite flag, to stop printing to stdout
    * Added a function to check permissions before a backup is attempted
    * Added a build function to GlobalVars, so the clap related code doen't 
      clutter main
    * Refactored the README

## ver: 0.4.1

    * Simplified the backup function. It now passes Arc protected mutexs to the
      spawned threads instead of populating thread local queues

    * Created a simple benchmark chart for comparison w/ of backup utils

    * Moved the change log from the README to CHANGELOG (ie. this file)

## ver: 0.4.0

    * Updated backrs documentation
    * Implemented a progress bar
    * Implemented an 'all' option to back up everything

## ver: 0.3.0

    * This a major rewrite of the previous version. This version is now
    * multi-threaded and implements the update option.