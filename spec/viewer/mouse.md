# Mouse

Back to the [specification index](../README.md).

temoc-ui captures the mouse. Terminals turn off their own text selection when an
application does this, so temoc-ui
[selects and copies text](selecting-text.md) itself.

## Scrolling

The wheel scrolls the document three lines at a time, stopping at either end.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Tui"

| # action         | workspace | arguments          | keys                | look for  | visible? |
| ---------------- | --------- | ------------------ | ------------------- | --------- | -------- |
| Wheel down       | docs      | guide/long-page.md | `WheelDown`         | Long Page | no       |
| Wheel back up    | docs      | guide/long-page.md | `WheelDown WheelUp` | Long Page | yes      |
| Stops at the top | docs      | guide/long-page.md | `WheelUp`           | Long Page | yes      |

A sideways wheel [scrolls wide tables and code blocks](navigation.md#scrolling-sideways),
as does holding `Shift` while turning the wheel. Each step moves the same distance
as pressing `Left` or `Right`.

[//]: # "decisionTable Tui"

| # action             | workspace   | keys                                   | look for     | visible? |
| -------------------- | ----------- | -------------------------------------- | ------------ | -------- |
| Sideways wheel       | wide-blocks | `WheelRight`                           | First Column | no       |
| Sideways wheel back  | wide-blocks | `WheelRight WheelLeft`                 | First Column | yes      |
| Shift and wheel      | wide-blocks | `Shift-WheelDown`                      | First Column | no       |
| Shift and wheel back | wide-blocks | `Shift-WheelDown Shift-WheelUp`        | First Column | yes      |

## Following links

Clicking a link focuses it and follows it, just like
[pressing `Enter`](navigation.md#following-links). Dragging from a link
[selects text](selecting-text.md) instead.

[//]: # "decisionTable Tui"

| # action             | workspace | keys                       | document?                | focused link?   |
| -------------------- | --------- | -------------------------- | ------------------------ | --------------- |
| Click a link         | docs      | `Click "Reference"`        | reference.md             | none            |
| Click a relative link | docs     | `Click "Getting started"`  | guide/getting-started.md | none            |
| Going back           | docs      | `Click "Reference" h`      | README.md                | Reference       |
| Click plain text     | docs      | `Click "Welcome"`          | README.md                | none            |
| Drag from a link     | docs      | `Drag "Getting" "started"` | README.md                | none            |

Links behave as they do from the keyboard, including heading anchors and links that
cannot be opened.

[//]: # "decisionTable Tui"

| # action      | workspace | keys                      | document? | status?                                           |
| ------------- | --------- | ------------------------- | --------- | ------------------------------------------------- |
| Missing file  | docs      | `Click "Missing page"`    | README.md | Link target not found: missing.md                 |
| External link | docs      | `Click "Temoc on GitHub"` | README.md | External link: https://github.com/killertux/temoc |

[//]: # "decisionTable Tui"

| # action | workspace | keys                       | document?          | look for | visible? |
| -------- | --------- | -------------------------- | ------------------ | -------- | -------- |
| Anchor   | docs      | `Click "Long page ending"` | guide/long-page.md | The End  | yes      |

## Selecting a table

Clicking a test table focuses it, so that `r`
[runs it](../running/running-a-table.md). Clicking anything else leaves the focus
where it was.

[//]: # "decisionTable Tui"

| # action             | workspace  | arguments  | keys                     | focused table? |
| -------------------- | ---------- | ---------- | ------------------------ | -------------- |
| Click a table        | calculator | passing.md | `Click "mul?"`           | Multiplication |
| Click back           | calculator | passing.md | `] Click "sum?"`         | Addition       |
| Click outside tables | calculator | passing.md | `] Click "Passing Tests"` | Multiplication |
| Page without tables  | docs       |            | `Click "Welcome"`        | none           |

[//]: # "decisionTable Tui"

| # action           | workspace  | arguments  | keys             | temoc arguments?               |
| ------------------ | ---------- | ---------- | ---------------- | ------------------------------ |
| Click, then run it | calculator | passing.md | `Click "mul?" r` | -c temoc.toml -s passing.md:15 |

## In the file tree

In the [file tree](file-tree.md), clicking a file opens it, and clicking a folder
opens or closes it. Clicking outside the pop-up closes it without doing anything
else. The wheel scrolls the tree and leaves the document behind it alone.

[//]: # "decisionTable Tui"

| # action           | workspace | keys                                  | document?                | tree open? | tree?                                                                        |
| ------------------ | --------- | ------------------------------------- | ------------------------ | ---------- | ---------------------------------------------------------------------------- |
| Click a file       | docs      | `t Click "notes.MD"`                  | notes.MD                 | no         | none                                                                         |
| Open a folder      | docs      | `t Click "guide"`                     | README.md                | yes        | ▾ guide, getting-started.md, long-page.md, notes.MD, README.md, reference.md |
| Close a folder     | docs      | `t Click "guide" Click "guide"`       | README.md                | yes        | ▸ guide, notes.MD, README.md, reference.md                                   |
| Click inside a folder | docs   | `t Click "guide" Click "long-page.md"` | guide/long-page.md      | no         | none                                                                         |
| Click outside      | docs      | `t ClickOutside`                      | README.md                | no         | none                                                                         |

The [`many-pages`](../../samples/many-pages/README.md) sample has more files than
the pop-up can show.

[//]: # "decisionTable Tui"

| # action                | workspace  | arguments          | keys                  | look for   | visible? | tree selection? |
| ----------------------- | ---------- | ------------------ | --------------------- | ---------- | -------- | --------------- |
| Top of the tree         | many-pages | page-02.md         | `t`                   | page-01.md | yes      | page-02.md      |
| Wheel scrolls the tree  | many-pages | page-02.md         | `t WheelDown`         | page-01.md | no       | page-02.md      |
| Wheel scrolls back      | many-pages | page-02.md         | `t WheelDown WheelUp` | page-01.md | yes      | page-02.md      |

[//]: # "decisionTable Tui"

| # action                 | workspace | arguments          | keys            | look for  | visible? |
| ------------------------ | --------- | ------------------ | --------------- | --------- | -------- |
| The document stays still | docs      | guide/long-page.md | `t WheelDown t` | Long Page | yes      |

## In the graph view

In the [graph view](graph.md), clicking a file selects it, so that you can read its
links in the panel. Clicking the selected file opens it. Clicking the panel does
nothing.

The wheel scrolls the graph without changing the selection. To scroll sideways, use
a sideways wheel or hold `Shift` while turning the wheel. Each step moves one column.

[//]: # "decisionTable Tui"

| # action           | workspace | keys                                              | document?          | graph open? | selected file?     |
| ------------------ | --------- | ------------------------------------------------- | ------------------ | ----------- | ------------------ |
| Select a file      | docs      | `Ctrl-g Click "notes.MD"`                         | README.md          | yes         | notes.MD           |
| Select another     | docs      | `Ctrl-g Click "notes.MD" Click "reference.md"`    | README.md          | yes         | reference.md       |
| Open the selection | docs      | `Ctrl-g Click "notes.MD" Click "notes.MD"`        | notes.MD           | no          | none               |
| Open the start file | docs     | `Ctrl-g Click "README.md"`                        | README.md          | no          | none               |
| Click the panel    | docs      | `Ctrl-g Click "missing.md (missing)"`             | README.md          | yes         | README.md          |
| Go back after opening | docs   | `Ctrl-g Click "notes.MD" Click "notes.MD" h`      | README.md          | no          | none               |

[//]: # "decisionTable Tui"

| # action               | workspace  | keys                        | look for   | visible? | selected file? |
| ---------------------- | ---------- | --------------------------- | ---------- | -------- | -------------- |
| Top of the column      | many-pages | `Ctrl-g l`                  | page-02.md | yes      | page-01.md     |
| Wheel scrolls the graph | many-pages | `Ctrl-g l WheelDown`       | page-02.md | no       | page-01.md     |
| Wheel scrolls back     | many-pages | `Ctrl-g l WheelDown WheelUp` | page-02.md | yes     | page-01.md     |

[//]: # "decisionTable Tui"

| # action             | workspace  | keys                          | look for  | in graph? | selected file? |
| -------------------- | ---------- | ----------------------------- | --------- | --------- | -------------- |
| Sideways wheel       | link-chain | `Ctrl-g WheelRight`           | README.md | no        | README.md      |
| Sideways wheel back  | link-chain | `Ctrl-g WheelRight WheelLeft` | README.md | yes       | README.md      |
| Stops at the left    | link-chain | `Ctrl-g WheelLeft`            | README.md | yes       | README.md      |
| Shift and wheel      | link-chain | `Ctrl-g Shift-WheelDown`      | README.md | no        | README.md      |
| Shift and wheel back | link-chain | `Ctrl-g Shift-WheelDown Shift-WheelUp` | README.md | yes | README.md      |
