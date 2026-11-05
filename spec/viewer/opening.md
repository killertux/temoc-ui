# Opening a Workspace

Back to the [specification index](../README.md).

temoc-ui starts from one optional path, and an optional
[configuration file](../running/configuration.md). How an interface is given them is
up to that interface: [the terminal](../tui/opening.md) reads its command line.

## The workspace

The workspace decides which files [search](../search.md) can find.

- With no path, or a file inside the current folder, it is the current folder.
- With a folder, it is that folder.
- With a file outside the current folder, it is the folder holding that file.

## What opens first

With a file path, that file opens. Otherwise the workspace's `README.md` opens. If
there is no `README.md`, nothing opens, and it is up to the interface to offer the
files of the workspace.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Core"

| # situation             | workspace | path                     | document?                |
| ----------------------- | --------- | ------------------------ | ------------------------ |
| No path                 | docs      |                          | README.md                |
| File in the workspace   | docs      | reference.md             | reference.md             |
| File in a subfolder     | docs      | guide/getting-started.md | guide/getting-started.md |
| File outside the folder | docs      | ../calculator/passing.md | passing.md               |
| Folder with a README    | detection | ../docs                  | README.md                |
| Folder without a README | docs      | guide                    | none                     |

The workspace follows the same rules. Search shows what the workspace contains:

[//]: # "decisionTable Core"

| # situation          | workspace | path                     | steps    | results?                                                                        |
| -------------------- | --------- | ------------------------ | -------- | ------------------------------------------------------------------------------- |
| File inside keeps it | docs      | guide/getting-started.md | `search` | guide/getting-started.md, guide/long-page.md, notes.MD, README.md, reference.md |
| Folder replaces it   | docs      | guide                    | `search` | getting-started.md, long-page.md                                                |

## Paths that cannot be opened

temoc-ui refuses to start, and says why.

[//]: # "decisionTable Core"

| # situation  | workspace | path     | document? | problem?                      |
| ------------ | --------- | -------- | --------- | ----------------------------- |
| Missing path | docs      | nope.md  | none      | Path not found: nope.md       |
| Not markdown | docs      | data.txt | none      | Not a markdown file: data.txt |
