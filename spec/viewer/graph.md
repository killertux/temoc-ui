# Graph View

Back to the [specification index](../README.md).

`Ctrl-g` leaves the document and enters a separate screen showing every markdown
file in the workspace and the links between them. `Esc` or `Ctrl-g` returns to the
document.

## Layout

Files are arranged in columns by how many links away they are from the start page,
which is the workspace's `README.md`, or else its first file alphabetically. Each
column is in alphabetical order ignoring case. Files that cannot be reached from
the start page are in a final column of their own.

Lines connect linked files, with an arrowhead at the target. Two files that link to
each other share one line with an arrowhead at each end. Lines touching the selected
file are highlighted.

There are as many columns as the links need. A file sits at its shortest distance
from the start page, so a longer route to it does not move it.

In `columns?` below, columns are separated by semicolons.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation         | workspace  | arguments | keys         | graph open? | columns?                                                                          |
| ------------------- | ---------- | --------- | ------------ | ----------- | --------------------------------------------------------------------------------- |
| Closed at first     | docs       |           |              | no          | none                                                                              |
| Unreachable last    | docs       |           | `Ctrl-g`     | yes         | README.md; guide/getting-started.md, guide/long-page.md, reference.md; notes.MD   |
| Test files included | calculator |           | `Ctrl-g`     | yes         | README.md; failing.md, passing.md; broken.md, mixed.md, snoozed.md                |
| No README           | docs       | guide     | `Esc Ctrl-g` | yes         | getting-started.md; long-page.md                                                  |
| Many hops           | link-chain |           | `Ctrl-g`     | yes         | README.md; shortcut.md, stop-one.md; stop-two.md; stop-three.md; stop-four.md; stop-five.md; stop-six.md; island.md |
| Closes with Esc     | docs       |           | `Ctrl-g Esc` | no          | none                                                                              |
| Closes with Ctrl-g  | docs       |           | `Ctrl-g Ctrl-g` | no       | none                                                                              |

The [`link-chain`](../../samples/link-chain/README.md) sample is a chain of pages with
a shortcut and an island. `distance?` is how many links the selected file is from
the start page.

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation                | workspace  | arguments     | keys     | distance?     |
| -------------------------- | ---------- | ------------- | -------- | ------------- |
| The start page             | link-chain |               | `Ctrl-g` | 0             |
| One link away              | link-chain | stop-one.md   | `Ctrl-g` | 1             |
| A second hop               | link-chain | stop-two.md   | `Ctrl-g` | 2             |
| Links back do not move it  | link-chain | stop-three.md | `Ctrl-g` | 3             |
| Far along the chain        | link-chain | stop-six.md   | `Ctrl-g` | 6             |
| The shortest route wins    | link-chain | shortcut.md   | `Ctrl-g` | 1             |
| Links out are not enough   | link-chain | island.md     | `Ctrl-g` | not reachable |

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation     | workspace | keys     | look for  | visible? |
| --------------- | --------- | -------- | --------- | -------- |
| Files are shown | docs      | `Ctrl-g` | notes.MD  | yes      |
| Links are drawn | docs      | `Ctrl-g` | ▶         | yes      |

## Links

Each pair of files has at most one link in each direction, however many times one
mentions the other. Links to a heading count as links to its file. Links to a
website, to the same page, or out of the workspace are not shown.

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation | workspace | keys     | links?                                                                                                                                                                                                                       |
| ----------- | --------- | -------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| All links   | docs      | `Ctrl-g` | guide/getting-started.md -> README.md, guide/getting-started.md -> reference.md, README.md -> guide/getting-started.md, README.md -> guide/long-page.md, README.md -> reference.md, reference.md -> guide/getting-started.md |

A panel below the graph lists the links out of the selected file and into it. A link
to a file that does not exist has no place in the graph, so it appears only in this
panel.

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation          | workspace | arguments          | keys         | selected file?           | links out?                                                                     | links in?                         |
| -------------------- | --------- | ------------------ | ------------ | ------------------------ | ------------------------------------------------------------------------------ | --------------------------------- |
| Includes broken links | docs     |                    | `Ctrl-g`     | README.md                | guide/getting-started.md, guide/long-page.md, missing.md (missing), reference.md | guide/getting-started.md          |
| Links both ways      | docs      | reference.md       | `Ctrl-g`     | reference.md             | guide/getting-started.md                                                       | guide/getting-started.md, README.md |
| Only links in        | docs      | guide/long-page.md | `Ctrl-g`     | guide/long-page.md       | none                                                                           | README.md                         |
| No links at all      | docs      | notes.MD           | `Ctrl-g`     | notes.MD                 | none                                                                           | none                              |
| Links out of the workspace | docs | guide             | `Esc Ctrl-g` | getting-started.md       | none                                                                           | none                              |

## Moving around

The open document starts selected. `j` and `k` or `Down` and `Up` move within a
column, and `h` and `l` or `Left` and `Right` move between columns, landing on the
nearest file. `g` and `G` go to the top and bottom of the column. Movement stops at
the edges, and the view follows the selected file. The
[mouse](mouse.md#in-the-graph-view) works here too.

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation        | workspace | keys             | selected file?           |
| ------------------ | --------- | ---------------- | ------------------------ |
| Next column        | docs      | `Ctrl-g l`       | guide/getting-started.md |
| Arrow keys         | docs      | `Ctrl-g Right`   | guide/getting-started.md |
| Down a column      | docs      | `Ctrl-g l j`     | guide/long-page.md       |
| Stops at the bottom | docs     | `Ctrl-g l j j j` | reference.md             |
| Stops at the top   | docs      | `Ctrl-g l k`     | guide/getting-started.md |
| Bottom of column   | docs      | `Ctrl-g l G`     | reference.md             |
| Top of column      | docs      | `Ctrl-g l G g`   | guide/getting-started.md |
| Last column        | docs      | `Ctrl-g l l`     | notes.MD                 |
| Stops at the right | docs      | `Ctrl-g l l l`   | notes.MD                 |
| Previous column    | docs      | `Ctrl-g l h`     | README.md                |
| Stops at the left  | docs      | `Ctrl-g h`       | README.md                |

The [`many-pages`](../../samples/many-pages/README.md) sample is too tall for the
screen, and [`link-chain`](../../samples/link-chain/README.md) too wide, so the view
has to follow the selection in both directions. `in graph?` looks only at the graph,
because the panel below it always names the selected file.

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation            | workspace  | keys         | look for   | in graph? |
| ---------------------- | ---------- | ------------ | ---------- | --------- |
| Below the screen       | many-pages | `Ctrl-g l`   | page-30.md | no        |
| View follows downwards | many-pages | `Ctrl-g l G` | page-30.md | yes       |
| View follows upwards   | many-pages | `Ctrl-g l G g` | page-01.md | yes     |

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation            | workspace  | arguments   | keys                          | look for    | in graph? |
| ---------------------- | ---------- | ----------- | ----------------------------- | ----------- | --------- |
| Beyond the right edge  | link-chain |             | `Ctrl-g`                      | stop-six.md | no        |
| View follows right     | link-chain |             | `Ctrl-g l l l l l l`          | stop-six.md | yes       |
| Start scrolls off      | link-chain |             | `Ctrl-g l l l l l l`          | README.md   | no        |
| View follows left      | link-chain |             | `Ctrl-g l l l l l l h h h h h h` | README.md | yes      |
| Starts on the document | link-chain | stop-six.md | `Ctrl-g`                      | stop-six.md | yes       |

## Opening a file

`Enter` opens the selected file and returns to the document screen. Going
[back](navigation.md#history) returns to the previous document.

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation      | workspace | keys                 | document?    | graph open? |
| ---------------- | --------- | -------------------- | ------------ | ----------- |
| Open a file      | docs      | `Ctrl-g l G Enter`   | reference.md | no          |
| Go back          | docs      | `Ctrl-g l G Enter h` | README.md    | no          |
| Leave without opening | docs | `Ctrl-g l G Esc`     | README.md    | no          |
