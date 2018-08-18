# Change Log

## ver: 0.6.1

    * Removed some of the goals to avoid scope creep. It would be better to use this 
      program in conjunction with other technologies that specialize in compression
      and similar instead of implementing such features. KISS and all that.

## ver: 0.6.0 

    * Implemented the -L, --force-log flag, to force write a log even if there
      are no errors to report

    * Unified the function call syntax for gvars

    * removed the unstable `fs::write()` call from check_permissions. It now 
      uses `fs::File::create()`

    * removed the benchmark from the README. It is not thorough enough and I do
      not want the performance to be misleading. I will post more though benchmarks
      when I can create a controlled environment.

## ver: 0.5.0

    * Implemented the -q --quite flag, to stop printing to stdout

    * Added a function to check permissions before a backup is attempted

    * Added a build function to GlobalVars, so the clap related code doesn't 
      clutter main

    * Refactored the README

## ver: 0.4.1

    * Simplified the backup function. It now passes Arc protected mutexes to the
      spawned threads instead of populating thread local queues

    * Created a simple benchmark chart for comparison w/ other backup utils

    * Moved the change log from the README to CHANGELOG (ie. this file)

## ver: 0.4.0

    * Updated the backr documentation

    * Implemented a progress bar

    * Implemented the -a, -all option to back up everything

## ver: 0.3.0

    * This a major rewrite of the previous version. This version is now

    * multi-threaded and implements the update option.