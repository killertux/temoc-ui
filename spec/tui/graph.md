# Graph View

Back to the [specification index](../README.md).

`Ctrl-g` leaves the document and enters a separate screen showing the workspace's
[graph](../viewer/graph.md). `Esc` or `Ctrl-g` returns to the document.

## Layout

Lines connect linked files, with an arrowhead at the target. Two files that link to
each other share one line with an arrowhead at each end. Lines touching the selected
file are highlighted.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Tui"

| # situation        | workspace | arguments | keys            | graph open? | columns?                                                                        |
| ------------------ | --------- | --------- | --------------- | ----------- | ------------------------------------------------------------------------------- |
| Closed at first    | docs      |           |                 | no          | none                                                                            |
| Opens              | docs      |           | `Ctrl-g`        | yes         | README.md; guide/getting-started.md, guide/long-page.md, reference.md; notes.MD |
| No README          | docs      | guide     | `Esc Ctrl-g`    | yes         | getting-started.md; long-page.md                                                |
| Closes with Esc    | docs      |           | `Ctrl-g Esc`    | no          | none                                                                            |
| Closes with Ctrl-g | docs      |           | `Ctrl-g Ctrl-g` | no          | none                                                                            |

[//]: # "decisionTable Tui"

| # situation     | workspace | keys     | look for | visible? |
| --------------- | --------- | -------- | -------- | -------- |
| Files are shown | docs      | `Ctrl-g` | notes.MD | yes      |
| Links are drawn | docs      | `Ctrl-g` | ▶        | yes      |

## The panel

A panel below the graph lists the links out of the selected file and into it. A list
too long for the panel ends by saying how many more there are.

`distance?` is how many links the selected file is from the start page.

[//]: # "decisionTable Tui"

| # situation          | workspace  | arguments     | keys     | selected file? | distance? | links out?                                                                       | links in?                |
| -------------------- | ---------- | ------------- | -------- | -------------- | --------- | -------------------------------------------------------------------------------- | ------------------------ |
| The open document    | docs       |               | `Ctrl-g` | README.md      | 0         | guide/getting-started.md, guide/long-page.md, missing.md (missing), reference.md | guide/getting-started.md |
| Follows the document | docs       | notes.MD      | `Ctrl-g` | notes.MD       | not reachable | none                                                                         | none                     |

[//]: # "decisionTable Tui"

| # situation     | workspace | keys     | links?                                                                                                                                                                                                                       |
| --------------- | --------- | -------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| All links       | docs      | `Ctrl-g` | guide/getting-started.md -> README.md, guide/getting-started.md -> reference.md, README.md -> guide/getting-started.md, README.md -> guide/long-page.md, README.md -> reference.md, reference.md -> guide/getting-started.md |

[//]: # "decisionTable Tui"

| # situation            | workspace  | keys     | look for    | visible? |
| ---------------------- | ---------- | -------- | ----------- | -------- |
| Too long for the panel | many-pages | `Ctrl-g` | and 26 more | yes      |

## Moving around

The open document starts selected. `j` and `k` or `Down` and `Up` move within a
column, and `h` and `l` or `Left` and `Right` move between columns, landing on the
nearest file. `g` and `G` go to the top and bottom of the column. Movement stops at
the edges, and the view follows the selected file. The
[mouse](mouse.md#in-the-graph-view) works here too.

[//]: # "decisionTable Tui"

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

[//]: # "decisionTable Tui"

| # situation            | workspace  | keys         | look for   | in graph? |
| ---------------------- | ---------- | ------------ | ---------- | --------- |
| Below the screen       | many-pages | `Ctrl-g l`   | page-30.md | no        |
| View follows downwards | many-pages | `Ctrl-g l G` | page-30.md | yes       |
| View follows upwards   | many-pages | `Ctrl-g l G g` | page-01.md | yes     |

[//]: # "decisionTable Tui"

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

[//]: # "decisionTable Tui"

| # situation      | workspace | keys                 | document?    | graph open? |
| ---------------- | --------- | -------------------- | ------------ | ----------- |
| Open a file      | docs      | `Ctrl-g l G Enter`   | reference.md | no          |
| Go back          | docs      | `Ctrl-g l G Enter h` | README.md    | no          |
| Leave without opening | docs | `Ctrl-g l G Esc`     | README.md    | no          |
