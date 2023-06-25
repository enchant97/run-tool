# Run Tool
A tool to aid developers in running their projects.


## Usage
At your project root; create a `.run-tool.yml` file.

Follow this format:

```yaml
configurations:
  run_name:
    program: "<executable name>"
    # ~ optional
    args:
      - "arg1"
    # optional
    env:
      LOG_LEVEL: "debug"
    # ~ optional
    env_file: ".env"
    # or
    env_file:
      - ".env"
    # ~ optional
    # absolute or relative to config file
    cwd: "api/"
    # ~ optional
    before_hooks:
      - another_config
    # ~ optional
    after_hooks:
      - another_config
```

> You can navigate in child directories and still run the configs, since the app will search parent directories.

### Global
If you have some commands that you want to access everywhere, a global config can be used. Below are the paths that will be searched:

- Linux & Unix
    1. `$XDG_CONFIG_HOME/run-tool/`
    2. `$HOME/.config/run-tool/`
- Windows
    1. `%%USERPROFILE%%/.config/run-tool/`

### Tips
- Add an alias in your shell, I use `alias rt='run-tool run'`
- You can name your config either: `.run-tool.yaml` or `.run-tool.yml`


## Install
Currently the only way to install is using Cargo:

```
cargo install --git https://github.com/enchant97/run-tool.git
```

> Add `--tag vx.x.x` to install a specific version


## Goals
- Fast
- Understandable configuration
- Cross-Platform


## License
This project is Copyright (c) 2023 Leo Spratt, licence shown below:

    Apache-2.0. Full license found in `LICENSE.txt`
