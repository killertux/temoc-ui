# Opening temoc-ui

Back to the [specification index](../README.md).

temoc-ui takes one optional path, plus the `--config` option described in
[Configuration](../running/configuration.md):

```sh
temoc-ui [PATH] [--config FILE]
```

## The workspace

The workspace decides which files [search](../search.md) can find.

- With no path, or a file inside the current folder, it is the current folder.
- With a folder, it is that folder.
- With a file outside the current folder, it is the folder holding that file.

## What opens first

With a file path, that file opens. Otherwise the workspace's `README.md` opens. If
there is no `README.md`, search opens so that you can pick a file.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Tui"

| # situation            | workspace | arguments                   | document?                | search open? |
| ---------------------- | --------- | --------------------------- | ------------------------ | ------------ |
| No path                | docs      |                             | README.md                | no           |
| File in the workspace  | docs      | reference.md                | reference.md             | no           |
| File in a subfolder    | docs      | guide/getting-started.md    | guide/getting-started.md | no           |
| File outside the folder | docs     | ../calculator/passing.md    | passing.md               | no           |
| Folder with a README   | detection | ../docs                     | README.md                | no           |
| Folder without a README | docs     | guide                       | none                     | yes          |

The workspace follows the same rules. Search shows what the workspace contains:

[//]: # "decisionTable Tui"

| # situation             | workspace | arguments                | keys | results?                                 |
| ----------------------- | --------- | ------------------------ | ---- | ---------------------------------------- |
| File inside keeps it    | docs      | guide/getting-started.md | `/`  | guide/getting-started.md, guide/long-page.md, notes.MD, README.md, reference.md |
| Folder replaces it      | docs      | guide                    |      | getting-started.md, long-page.md         |

## Paths that cannot be opened

temoc-ui prints a message and exits without opening.

[//]: # "decisionTable Tui"

| # situation     | workspace | arguments | still open? | status?                       |
| --------------- | --------- | --------- | ----------- | ----------------------------- |
| Missing path    | docs      | nope.md   | no          | Path not found: nope.md       |
| Not markdown    | docs      | data.txt  | no          | Not a markdown file: data.txt |
