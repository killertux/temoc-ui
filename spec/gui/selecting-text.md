# Selecting Text

Back to the [specification index](../README.md).

Dragging over the page selects text, as in any window. `Ctrl-c` copies it, and the
status bar says how much. What is copied is the text as shown, not the markdown source.
A click clears the selection. There is no selection mode: `v` does nothing.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Gui -- snooze until 2099-12-31"

| # action        | workspace | arguments                | actions                                | selection?        | clipboard?        | status?              |
| --------------- | --------- | ------------------------ | -------------------------------------- | ----------------- | ----------------- | -------------------- |
| Drag            | docs      | guide/getting-started.md | `Drag "Run" "installer"`               | Run the installer | none              | none                 |
| Copy            | docs      | guide/getting-started.md | `Drag "Run" "installer" Ctrl-c`        | Run the installer | Run the installer | Copied 17 characters |
| A click clears  | docs      | guide/getting-started.md | `Drag "Run" "installer" Click "Install"` | none            | none              | none                 |
| Nothing to copy | docs      | guide/getting-started.md | `Ctrl-c`                               | none              | none              | none                 |
| No mode         | docs      | guide/getting-started.md | `v`                                    | none              | none              | none                 |

Text in a table can be selected too, and dragging from a link selects text without
following it.

[//]: # "decisionTable Gui -- snooze until 2099-12-31"

| # action        | workspace | arguments    | actions                                     | document?    | selection?   |
| --------------- | --------- | ------------ | ------------------------------------------- | ------------ | ------------ |
| In a table      | docs      | reference.md | `Drag "Opens" "file"`                       | reference.md | Opens a file |
| From a link     | docs      |              | `Drag "Getting" "started"`                  | README.md    | Getting started |
