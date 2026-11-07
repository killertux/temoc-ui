# Reloading

Back to the [specification index](../README.md).

The page follows its file as it [changes on disk](../reloading.md), without a word in the
status bar. The file changes are written in the `keys` column, as described in
[how these pages work](writing-specs.md#changing-files).

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Tui"

| # situation      | workspace | arguments | keys                                       | look for                            | visible? | status? |
| ---------------- | --------- | --------- | ------------------------------------------ | ----------------------------------- | -------- | ------- |
| Before a change  | changing  | page.md   |                                            | A paragraph added above the tables. | no       | none    |
| After a change   | changing  | page.md   | `Write file "page.md" from "page-moved.md"`     | A paragraph added above the tables. | yes      | none    |

## Where the page is

The page stays scrolled where it was. If it is now too short for that, it shows its end.

[//]: # "decisionTable Tui"

| # situation    | workspace | arguments | keys                                     | look for                              | visible? |
| -------------- | --------- | --------- | ---------------------------------------- | ------------------------------------- | -------- |
| Stays put      | changing  | long.md   | `G Write file "long.md" from "long-edited.md"` | This is the new end of the long page. | yes      |
| Got shorter    | changing  | long.md   | `G Write file "long.md" from "long-short.md"`  | Short now.                            | yes      |

The focused table stays focused if a table of the same name is still there, and the
focused row stays focused if that table still has a row in the same place. Otherwise
the first table is focused, as when the page opens, and no row is.

[//]: # "decisionTable Tui"

| # situation      | workspace | arguments | keys                                           | focused table? | focused row? |
| ---------------- | --------- | --------- | ---------------------------------------------- | -------------- | ------------ |
| Lines moved      | changing  | page.md   | `] } } Write file "page.md" from "page-moved.md"`   | Multiplication | 22           |
| The table changed | changing | page.md   | `] } } Write file "page.md" from "page-fixed.md"`   | Multiplication | 20           |
| The tables gone  | changing  | page.md   | `] } } Write file "page.md" from "page-no-tests.md"` | none          | none         |

Results move with their rows on screen.

[//]: # "decisionTable Tui"

| # situation  | workspace | arguments | keys                                     | row results?                       |
| ------------ | --------- | --------- | ---------------------------------------- | ---------------------------------- |
| Lines moved  | changing  | page.md   | `R Write file "page.md" from "page-moved.md"` | 13: passed, 21: passed, 22: failed |

## A deleted file

The status bar says that the file is gone, and keeps saying so until it comes back.
The page stays as it was.

[//]: # "decisionTable Tui"

| # situation | workspace | arguments | keys                                                 | document? | status?               |
| ----------- | --------- | --------- | ---------------------------------------------------- | --------- | --------------------- |
| Deleted     | changing  | page.md   | `Delete file "page.md"`                                   | page.md   | File deleted: page.md |
| Still gone  | changing  | page.md   | `Delete file "page.md" j j`                               | page.md   | File deleted: page.md |
| Running it  | changing  | page.md   | `Delete file "page.md" R`                                 | page.md   | File deleted: page.md |
| Comes back  | changing  | page.md   | `Delete file "page.md" Write file "page.md" from "new.md"`     | page.md   | none                  |

## Search and the file tree

Both update while they are showing.

[//]: # "decisionTable Tui"

| # situation    | workspace | arguments | keys                             | search open? | results?                                        |
| -------------- | --------- | --------- | -------------------------------- | ------------ | ----------------------------------------------- |
| Search showing | changing  |           | `/ Write file "new.md" from "new.md"` | yes          | hanging.md, long.md, new.md, page.md, README.md |

[//]: # "decisionTable Tui"

| # situation  | workspace | arguments | keys                 | tree open? | tree?                          |
| ------------ | --------- | --------- | -------------------- | ---------- | ------------------------------ |
| Tree showing | changing  |           | `t Delete file "long.md"` | yes        | hanging.md, page.md, README.md |
