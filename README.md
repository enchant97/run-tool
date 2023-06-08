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
    cwd: "api/"
    # ~ optional
    before_hooks:
      - another_config
    # ~ optional
    after_hooks:
      - another_config
```


## Install
Currently the only way to install is using Cargo:

```
cargo install --git https://github.com/enchant97/run-tool.git
```

> Add `--tag vx.x.x` to install a specific version


## License
This project is Copyright (c) 2023 Leo Spratt, licence shown below:

    Apache-2.0. Full license found in `LICENSE.txt`
