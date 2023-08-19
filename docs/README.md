# Run Tool
A multi-purpose tool to aid developers in executing common tasks. Aiming to suit modern requirements, whilst not replicating make.

> Whilst fairly stable, it is still in development and features may change


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
