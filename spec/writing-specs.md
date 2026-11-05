# How These Pages Work

Back to the [specification index](README.md).

Every page in this folder is both documentation and a temoc test. We always write
the spec first. The new tests fail because the feature does not exist yet, and
then we write the code that makes them pass.

Run the whole specification from the repository root:

```sh
temoc -c temoc.toml
```

## Core and interfaces

The pages at this level describe what temoc-ui does whatever the interface is, and
their tables never touch a screen, a key, or a colour. Each interface gets a folder
of its own for what only it does, and for proof that it reaches the core behavior.
The first is [`tui/`](tui/writing-specs.md), the terminal.

A behavior belongs in the core if another interface would have to do the same.
How it looks and what you press to get it belong to the interface.

## Reading a test table

Each row is one independent scenario. Columns ending in `?` are what we expect to
see. The other columns are inputs. Columns starting with `#` are comments that
temoc ignores.

Most pages here use the `Core` fixture. For each row it opens a fresh temoc-ui with
no interface, takes the steps, waits for any test run to finish, and then reads what
happened. `TemocDetection` and `TemocOutput` look at one piece of temoc-ui by itself,
as their pages describe.

### Inputs

| Column      | Meaning                                                |
| ----------- | ------------------------------------------------------ |
| `workspace` | The folder under [`samples/`](../samples) to start in. |
| `path`      | The path to open. Empty means none.                    |
| `config`    | The configuration file to use. Empty means none.       |
| `steps`     | What to do, in order, separated by commas.             |

### Steps

| Written as                 | Meaning                                                          |
| -------------------------- | ---------------------------------------------------------------- |
| `open FILE`                | Open a file of the workspace.                                    |
| `follow TARGET`            | Follow a link in the open document, as written in the markdown.  |
| `back`, `forward`          | Go back or forward.                                              |
| `search`, `search TEXT`    | Search for the text, or for nothing to list everything.          |
| `only test files`          | Turn the test file filter of the search on or off.               |
| `open result NUMBER`       | Open that result of the search, counting from 1.                 |
| `run file`                 | Run the open file.                                               |
| `run table LINE`           | Run the table with its directive on that line.                   |
| `run row LINE`             | Run the row on that line.                                        |

Steps wait for any test run to finish before the next one is taken, which leaves
nothing to see of a run in progress. `wait for row` is for looking at one.

| Written as          | Meaning                                                                                                     |
| ------------------- | ----------------------------------------------------------------------------------------------------------- |
| `wait for row LINE` | Stop waiting for the run, and wait until that row is `running`. Later steps do not wait for the run either. |

### Expected values

| Column             | Meaning                                                                                |
| ------------------ | -------------------------------------------------------------------------------------- |
| `document?`        | Path of the open document within the workspace, or `none`.                             |
| `anchor?`          | The heading the document was opened at, or `none`.                                     |
| `problem?`         | Why temoc-ui did not start, why the last step was refused, or what stopped the last run, or `none`. |
| `results?`         | Search results in order, separated by commas, or `none`.                               |
| `first result?`    | The top search result, or `none`.                                                      |
| `tree?`            | The rows of the file tree in order. A folder ends with `/`.                            |
| `columns?`         | The graph's columns, separated by semicolons.                                          |
| `links?`           | Every link in the graph, written `from -> to`.                                         |
| `distance?`        | Links from the start page to the open document, or `not reachable`.                    |
| `links out?`       | The links out of the open document, or `none`.                                         |
| `links in?`        | The links into the open document, or `none`.                                           |
| `file result?`     | The result for the whole document, or `none` when it has no tests.                     |
| `table results?`   | Each table's name and result, separated by commas, or `none`.                          |
| `row results?`     | Each row that has a result, written `line: result` and separated by commas, or `none`. |
| `failures?`        | Each failure with its table and line, separated by semicolons, or `none`.              |
| `temoc arguments?` | The arguments of the last temoc command, or `none`.                                    |
| `running?`         | The file temoc is running, or `none`.                                                  |

## Sample workspaces

The workspaces live in [`samples/`](../samples), outside this folder so that
running the specification does not run them as tests too.

| Workspace                                     | Used for                          |
| --------------------------------------------- | --------------------------------- |
| [`docs`](../samples/docs/README.md)           | Viewing, navigating, and search.  |
| [`calculator`](../samples/calculator/README.md) | Running real tests.             |
| [`outcomes`](../samples/outcomes/README.md)   | Errors, symbols, and runs that hang or stop midway. |
| [`detection`](../samples/detection)           | Recognizing test files.           |
| [`unconfigured`](../samples/unconfigured)     | Running without configuration.    |
| [`server-failure`](../samples/server-failure) | A Slim server that will not start.|
| [`custom-config`](../samples/custom-config)   | Using the `--config` option.      |
| [`many-pages`](../samples/many-pages/README.md) | A graph taller than the screen. |
| [`link-chain`](../samples/link-chain/README.md) | A graph wider than the screen.  |
| [`wide-blocks`](../samples/wide-blocks/README.md) | Content wider than the screen.  |
| [`themes`](../samples/themes/README.md)       | One of each thing a theme colours. |

The `calculator` and `outcomes` workspaces run real temoc against a small Slim server
built by the spec harness.

## Writing a new page

- Describe the behavior in plain language first, then show it with a table.
- Keep each row independent, so that it can run alone.
- Put it with the core unless only one interface would ever do it.
- Add a link to the page from the [index](README.md).
- Link back to the index at the top of the page.
