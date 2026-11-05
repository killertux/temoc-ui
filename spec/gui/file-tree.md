# File Tree

Back to the [specification index](../README.md).

The [file tree](../viewer/file-tree.md) is a side panel that stays open while you read.
`Ctrl-b` or its toolbar button shows or hides it.

The open document is selected, and the folders above it start open.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Gui"

| # situation          | workspace | arguments                | actions         | tree open? | tree?                                                                       | tree selection?    |
| -------------------- | --------- | ------------------------ | --------------- | ---------- | --------------------------------------------------------------------------- | ------------------ |
| Open at first        | docs      |                          |                 | yes        | guide/, notes.MD, README.md, reference.md                                   | README.md          |
| Reveals the document | docs      | guide/getting-started.md |                 | yes        | guide/, getting-started.md, long-page.md, notes.MD, README.md, reference.md | getting-started.md |
| The key hides        | docs      |                          | `Ctrl-b`        | no         | none                                                                        | none               |
| The key shows it     | docs      |                          | `Ctrl-b Ctrl-b` | yes        | guide/, notes.MD, README.md, reference.md                                   | README.md          |

## Using it

The tree is for the mouse: [search](search.md) is the way to a file from the keyboard.
Clicking a folder opens or closes it, and clicking a file opens that file. The tree
stays open, and follows the document however you got there. Going
[back](navigation.md#history) returns to the previous document.

[//]: # "decisionTable Gui"

| # situation       | workspace | actions                                        | document?          | tree open? | tree selection? |
| ----------------- | --------- | ---------------------------------------------- | ------------------ | ---------- | --------------- |
| Open a file       | docs      | `Click tree "reference.md"`                    | reference.md       | yes        | reference.md    |
| From a folder     | docs      | `Click tree "guide" Click tree "long-page.md"` | guide/long-page.md | yes        | long-page.md    |
| Go back           | docs      | `Click tree "reference.md" Alt-Left`           | README.md          | yes        | README.md       |
| It follows a link | docs      | `Click "Reference"`                            | reference.md       | yes        | reference.md    |

[//]: # "decisionTable Gui"

| # situation    | workspace | actions                                 | tree?                                                                       |
| -------------- | --------- | --------------------------------------- | --------------------------------------------------------------------------- |
| Open a folder  | docs      | `Click tree "guide"`                    | guide/, getting-started.md, long-page.md, notes.MD, README.md, reference.md |
| Close it again | docs      | `Click tree "guide" Click tree "guide"` | guide/, notes.MD, README.md, reference.md                                   |

## Tests in the tree

[Test files](../running/finding-tests.md) are marked, so that you can tell them apart
from other pages. A file that has a result shows
[its mark](row-results.md#marks), and keeps it while you read something else.

[//]: # "decisionTable Gui"

| # situation  | workspace  | arguments  | actions                                 | tree test files?                                        | tree marks?                  |
| ------------ | ---------- | ---------- | --------------------------------------- | ------------------------------------------------------- | ---------------------------- |
| Before a run | calculator | failing.md |                                         | broken.md, failing.md, mixed.md, passing.md, snoozed.md | none                         |
| After a run  | calculator | failing.md | `Ctrl-r`                                | broken.md, failing.md, mixed.md, passing.md, snoozed.md | failing.md: ✗                |
| Two files    | calculator | failing.md | `Ctrl-r Click tree "passing.md" Ctrl-r` | broken.md, failing.md, mixed.md, passing.md, snoozed.md | failing.md: ✗, passing.md: ✓ |
