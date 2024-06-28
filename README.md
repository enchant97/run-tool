# Run Tool
A multi-purpose tool to aid developers in executing common tasks. Aiming to suit modern requirements, whilst not replicating make.

> Whilst fairly stable, it is still in development and features may change

Docs can be found here: [run-tool.docs.enchantedcode.co.uk](https://run-tool.docs.enchantedcode.co.uk/)

## Features
- Configuration uses YAML
- Per project configuration file (if added at root of project)
- Supporting a global configuration (per user)
- Customisable targets
    - Arguments
    - Environment variables
    - Settable current working directory
    - Hooks (before and after target run)
    - Conditional runs (only run target when conditions are met)
- Watch for file/folder changes


## Use Case
- Running a project
- Building a project
- Run tests
- One-off commands, e.g. downloading test data


## Goals
- Fast and easy to use
- Human readable configuration
- Cross-platform (for core functionality)
- Support mono-repos
- Support use in CI/CD


## Non-Goals
- Be a complete replacement to make


## Install
Currently the only way to install is using Cargo:

```
cargo install run-tool
```


## License
This project is Copyright (c) 2024 Leo Spratt, licence shown below:

    Apache-2.0. Full license found in `LICENSE.txt`
