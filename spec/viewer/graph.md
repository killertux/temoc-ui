# Graph

Back to the [specification index](../README.md).

The graph shows every markdown file in the workspace and the links between them. The
[terminal](../tui/graph.md) draws it on a screen of its own.

## Columns

Files are arranged in columns by how many links away they are from the start page,
which is the workspace's `README.md`, or else its first file alphabetically. Each
column is in alphabetical order ignoring case. Files that cannot be reached from
the start page are in a final column of their own.

There are as many columns as the links need. A file sits at its shortest distance
from the start page, so a longer route to it does not move it.

In `columns?` below, columns are separated by semicolons.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Core"

| # situation         | workspace  | path  | columns?                                                                                                            |
| ------------------- | ---------- | ----- | ------------------------------------------------------------------------------------------------------------------- |
| Unreachable last    | docs       |       | README.md; guide/getting-started.md, guide/long-page.md, reference.md; notes.MD                                     |
| Test files included | calculator |       | README.md; failing.md, passing.md; broken.md, mixed.md, snoozed.md                                                  |
| No README           | docs       | guide | getting-started.md; long-page.md                                                                                    |
| Many hops           | link-chain |       | README.md; shortcut.md, stop-one.md; stop-two.md; stop-three.md; stop-four.md; stop-five.md; stop-six.md; island.md |

The [`link-chain`](../../samples/link-chain/README.md) sample is a chain of pages with
a shortcut and an island. `distance?` is how many links the open document is from
the start page.

[//]: # "decisionTable Core"

| # situation               | workspace  | path          | distance?     |
| ------------------------- | ---------- | ------------- | ------------- |
| The start page            | link-chain |               | 0             |
| One link away             | link-chain | stop-one.md   | 1             |
| A second hop              | link-chain | stop-two.md   | 2             |
| Links back do not move it | link-chain | stop-three.md | 3             |
| Far along the chain       | link-chain | stop-six.md   | 6             |
| The shortest route wins   | link-chain | shortcut.md   | 1             |
| Links out are not enough  | link-chain | island.md     | not reachable |

## Links

Each pair of files has at most one link in each direction, however many times one
mentions the other. Links to a heading count as links to its file. Links to a
website, to the same page, or out of the workspace are not shown.

[//]: # "decisionTable Core"

| # situation | workspace | links?                                                                                                                                                                                                                       |
| ----------- | --------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| All links   | docs      | guide/getting-started.md -> README.md, guide/getting-started.md -> reference.md, README.md -> guide/getting-started.md, README.md -> guide/long-page.md, README.md -> reference.md, reference.md -> guide/getting-started.md |

Each file knows its links out and its links in. A link to a file that does not exist
has no place in the graph, so it only appears among the links out. Links to anything
else that exists but is not in the graph are not listed. `links out?` and `links in?`
read the open document.

[//]: # "decisionTable Core"

| # situation                | workspace | path               | steps                       | links out?                                                                       | links in?                           |
| -------------------------- | --------- | ------------------ | --------------------------- | -------------------------------------------------------------------------------- | ----------------------------------- |
| Includes broken links      | docs      |                    |                             | guide/getting-started.md, guide/long-page.md, missing.md (missing), reference.md | guide/getting-started.md            |
| Links both ways            | docs      | reference.md       |                             | guide/getting-started.md                                                         | guide/getting-started.md, README.md |
| Only links in              | docs      | guide/long-page.md |                             | none                                                                             | README.md                           |
| No links at all            | docs      | notes.MD           |                             | none                                                                             | none                                |
| Links out of the workspace | docs      | guide              | `open getting-started.md`   | none                                                                             | none                                |
