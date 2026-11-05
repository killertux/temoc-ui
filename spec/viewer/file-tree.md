# File Tree

Back to the [specification index](../README.md).

The file tree shows the workspace's markdown files. It lists the same files as
[search](../search.md): hidden folders and build folders are skipped.

Folders come first, then files, each in alphabetical order ignoring case. The folders
above the open document start open, and the others closed.

In `tree?`, a folder ends with `/`, and the rows of an open folder follow it.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Core"

| # situation          | workspace | path                     | tree?                                                                     |
| -------------------- | --------- | ------------------------ | ------------------------------------------------------------------------- |
| Folders come first   | docs      |                          | guide/, notes.MD, README.md, reference.md                                 |
| Reveals the document | docs      | guide/getting-started.md | guide/, getting-started.md, long-page.md, notes.MD, README.md, reference.md |

The [terminal](../tui/file-tree.md) shows it in a pop-up.
