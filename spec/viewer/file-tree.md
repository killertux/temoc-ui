# File Tree

Back to the [specification index](../README.md).

`t` opens a pop-up over the document showing the workspace's markdown files as a
tree. Pressing `t` again or `Esc` closes it. It lists the same files as
[search](../search.md): hidden folders and build folders are skipped.

## What it shows

Folders come first, then files, each in alphabetical order ignoring case. A closed
folder is marked `▸` and an open one `▾`. The open document is selected, and the
folders above it start open.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Tui"

| # situation        | workspace | arguments                | keys  | tree open? | tree?                                                                     | tree selection?    |
| ------------------ | --------- | ------------------------ | ----- | ---------- | ------------------------------------------------------------------------- | ------------------ |
| Closed at first    | docs      |                          |       | no         | none                                                                      | none               |
| Opens              | docs      |                          | `t`   | yes        | ▸ guide, notes.MD, README.md, reference.md                                | README.md          |
| Closes with t      | docs      |                          | `t t` | no         | none                                                                      | none               |
| Closes with Esc    | docs      |                          | `t Esc` | no       | none                                                                      | none               |
| Reveals the document | docs    | guide/getting-started.md | `t`   | yes        | ▾ guide, getting-started.md, long-page.md, notes.MD, README.md, reference.md | getting-started.md |

## Moving around

`j` and `k` or the arrow keys move the selection, stopping at either end. `l` or
`Right` opens a folder, and `h` or `Left` closes it. `Enter` opens a folder too. The
[mouse](mouse.md#in-the-file-tree) works here as well.

[//]: # "decisionTable Tui"

| # situation        | workspace | keys          | tree?                                                                        | tree selection? |
| ------------------ | --------- | ------------- | ---------------------------------------------------------------------------- | --------------- |
| Move down          | docs      | `t j`         | ▸ guide, notes.MD, README.md, reference.md                                   | reference.md    |
| Move up            | docs      | `t k`         | ▸ guide, notes.MD, README.md, reference.md                                   | notes.MD        |
| Arrow keys         | docs      | `t Up`        | ▸ guide, notes.MD, README.md, reference.md                                   | notes.MD        |
| Stops at the end   | docs      | `t j j`       | ▸ guide, notes.MD, README.md, reference.md                                   | reference.md    |
| Stops at the start | docs      | `t k k k`     | ▸ guide, notes.MD, README.md, reference.md                                   | guide           |
| Open with l        | docs      | `t k k l`     | ▾ guide, getting-started.md, long-page.md, notes.MD, README.md, reference.md | guide           |
| Open with Enter    | docs      | `t k k Enter` | ▾ guide, getting-started.md, long-page.md, notes.MD, README.md, reference.md | guide           |
| Close with h       | docs      | `t k k l h`   | ▸ guide, notes.MD, README.md, reference.md                                   | guide           |

## Opening a file

`Enter` on a file opens it and closes the pop-up. Going
[back](navigation.md#history) returns to the previous document. Closing the pop-up
any other way leaves the document as it was.

[//]: # "decisionTable Tui"

| # situation        | workspace | keys                | document?                | tree open? |
| ------------------ | --------- | ------------------- | ------------------------ | ---------- |
| Open a file        | docs      | `t j Enter`         | reference.md             | no         |
| Open from a folder | docs      | `t k k l j Enter`   | guide/getting-started.md | no         |
| Go back            | docs      | `t j Enter h`       | README.md                | no         |
| Close without opening | docs   | `t j Esc`           | README.md                | no         |
