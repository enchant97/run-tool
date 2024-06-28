# Usage
Whilst the application has in-build help (by running `run-tool help`), this section will provide a rough overview of available commands.


## Running A Target

```
run-tool run <target name>
```

Or add the ability to watch for file/folder changes:

```
run-tool run -w <target name>
```

You can also provide extra arguments to the targets executable appending to any specified in the config.

```
run-tool run <target name> -- --release
```


## Viewing Config
To view the currently loaded configuration in a human readable format use this command:

```
run-tool config
```

Or view a more minimal version:

```
run-tool config -m
```


## Tips
- Add an alias in your shell, e.g. `alias rt='run-tool run'`
- You can name your config either: `.run-tool.yaml` or `.run-tool.yml`
