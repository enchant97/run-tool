# The File
There are many different configuration values that can be used to provide different functionality, this section documents them.


## Reference

```yaml
targets:
  <target_name>:
    # - optional, string
    # - give the target a description
    description: null
    # - required, string
    # - program to execute
    program: ""
    # - optional, string or array of strings
    # - arguments to give program
    args: []
    # - optional, dictionary (var_key: var_val)
    # - environment variables to give program
    env: null
    # - optional, string or array of strings
    # - path to environment files to load
    env_file: []
    # - optional, string (default to loaded config file cwd)
    # - current working directory to set for application
    cwd: null
    # - optional, array of strings
    # - other targets to run before running this one
    before_hooks: []
    # - optional, array of strings
    # - other targets to run after running this one
    after_hooks: []
    # - optional, array of dictionaries
    # checks to meet before running target (including hooks)
    run_when:
        -
          # - required, string
          # - check name
          when: ""
          # - optional, boolean
          # - whether to invert the check
          invert: false
          # - optional, dictionary
          # - extra fields to give to check (depends on specified check)
          fields: {}
```

### Checks
Checks are used in the `run_when` configuration. They decide whether to run the selected target or not.

> Explanations in this section will assume the check has not been inverted.

#### Execution OK
Will allow target to run when the execution returns a status code of success (zero).

```yaml
when: exec_ok
fields:
  # required, string
  # - program to execute
  program: ""
  # - optional, string or array of strings
  # - arguments to give program
  args: []
  # - optional, dictionary (var_key: var_val)
  # - environment variables to give program
  env: null
  # - optional, string or array of strings
  # - path to environment files to load
  env_file: []
  # - optional, string (default to loaded config file cwd)
  # - current working directory to set for application
  cwd: null
```

#### Path Exists
Will allow target to run when given path exists, could mean it is a directory or file.

```yaml
when: path_exists
fields:
  # - required, string
  # - path to check
  path: ""
```

#### Path Is File
Will allow target to run when given path exists and is a file.

```yaml
when: path_is_file
fields:
  # - required, string
  # - path to check
  path: ""
```

#### Path Is Directory
Will allow target to run when given path exists and is a directory.

```yaml
when: path_is_dir
fields:
  # - required, string
  # - path to check
  path: ""
```


## Example
Here is an example for building a project in a mono-repo.

```yaml
#
# my-project/.run-tool.yml
#

targets:
  clean-build:
    program: cargo
    args:
      - build
      - --release
    env:
        CARGO_HOME: /mnt/cache/.cargo
    cwd: backend/
    before_hooks:
      - clean

  clean:
    program: cargo
    args:
      - clean
    cwd: backend/
```
