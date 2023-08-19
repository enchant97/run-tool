# Configuration
This chapter will go through everything supported in a "Run Tool" configuration file.

When running "Run Tool" *unless overridden* it will search the current directory for a file matching the defined configuration file names. If it does not find one it will move on to the parent directory; continuing until either no parents are left or a config has been located.

After a configuration file has been found the target will run from the current working directory of the configuration file, this allows for a mono-repo to have one file at the root of the project responsible for the whole project that can be loaded from any child directory.

## Configuration Files
By default these will be either `.run-tool.yml` or `.run-tool.yaml`. You may adjust this globally by using the `RUN_TOOL_FILENAME` environment variable or by passing as an argument when launching the app.
