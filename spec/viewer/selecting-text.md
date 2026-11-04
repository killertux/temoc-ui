# Selecting Text

Back to the [specification index](../README.md).

Text can be selected with the mouse or the keyboard and copied to the clipboard.
What is copied is the text as shown on screen, not the markdown source, with lines
separated by line breaks.

In the tables below, `\n` in an expected value stands for a line break.

## With the mouse

Dragging selects text. Releasing the button copies it straight away, and the text
stays highlighted until the next click or `Esc`. A selection covers what is on
screen, so use the keyboard for anything longer.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # action     | workspace | arguments                | keys                                  | selection?             | clipboard?                | status?              |
| ------------ | --------- | ------------------------ | ------------------------------------- | ---------------------- | ------------------------- | -------------------- |
| Drag         | docs      | guide/getting-started.md | `Drag "Run" "installer"`              | Run the installer      | Run the installer         | Copied 17 characters |
| Drag upwards | docs      | guide/getting-started.md | `DragBack "installer" "Run"`          | Run the installer      | Run the installer         | Copied 17 characters |
| Several lines | docs     | guide/getting-started.md | `Drag "Install" "installer"`          | `Install\n\nRun the installer` | `Install\n\nRun the installer` | Copied 26 characters |
| Clear by click | docs    | guide/getting-started.md | `Drag "Run" "installer" Click "Install"` | none                | Run the installer         | none                 |
| Clear by Esc | docs      | guide/getting-started.md | `Drag "Run" "installer" Esc`          | none                   | Run the installer         | none                 |
| A click alone | docs     | guide/getting-started.md | `Click "Run"`                         | none                   | none                      | none                 |

## With the keyboard

`v` enters selection mode, with a cursor on the first visible character. Move the
cursor to where the selection should start and press `v` again, then move to where
it should end and press `y` to copy. `Esc` leaves without copying.

| Key                | Moves the cursor              |
| ------------------ | ----------------------------- |
| `h`, `j`, `k`, `l` | One character or line         |
| Arrow keys         | One character or line         |
| `w`, `b`           | To the next or previous word  |
| `0`, `$`           | To the start or end of a line |
| `g`, `G`           | To the top or bottom          |

The view follows the cursor, so a selection can be longer than the screen. The
selection includes the characters at both ends.

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # action            | workspace | arguments          | keys          | clipboard?                       |
| ------------------- | --------- | ------------------ | ------------- | -------------------------------- |
| One character       | docs      |                    | `v v y`       | D                                |
| Character by character | docs   |                    | `v v l l l y` | Docs                             |
| Arrow keys          | docs      |                    | `v v Right y` | Do                               |
| To the end of a line | docs     |                    | `v v $ y`     | Docs Sample                      |
| Next word           | docs      |                    | `v w v $ y`   | Sample                           |
| Previous word       | docs      |                    | `v w b v l y` | Do                               |
| Down to a line      | docs      |                    | `v j j v $ y` | Welcome to the sample documentation. |
| Several lines       | docs      |                    | `v v j j $ y` | `Docs Sample\n\nWelcome to the sample documentation.` |
| Backwards           | docs      |                    | `v $ v 0 y`   | Docs Sample                      |
| From a scrolled view | docs     | guide/long-page.md | `j j v v $ y` | Line 1                           |
| As shown on screen  | docs      |                    | `v j j j j v $ y` | • Getting started            |

A mouse click leaves selection mode without copying, and then acts as normal.

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # action           | workspace | keys                    | document?    | selection? | clipboard? |
| ------------------ | --------- | ----------------------- | ------------ | ---------- | ---------- |
| Click on text      | docs      | `v v w Click "Welcome"` | README.md    | none       | none       |
| Click on a link    | docs      | `v v w Click "Reference"` | reference.md | none     | none       |

The status bar says what to do next, and reports the result.

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # action         | workspace | keys        | selection?  | clipboard? | status?                         |
| ---------------- | --------- | ----------- | ----------- | ---------- | ------------------------------- |
| Mode starts      | docs      | `v`         | none        | none       | Move to the start, then press v |
| Selection starts | docs      | `v v`       | D           | none       | Move to the end, then press y   |
| Selection grows  | docs      | `v v w`     | Docs S      | none       | Move to the end, then press y   |
| Copied           | docs      | `v v $ y`   | none        | Docs Sample | Copied 11 characters           |
| Nothing marked   | docs      | `v y`       | none        | none       | Nothing selected                |
| Cancelled        | docs      | `v v w Esc` | none        | none       | none                            |

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # action           | workspace | arguments          | keys  | look for | visible? |
| ------------------ | --------- | ------------------ | ----- | -------- | -------- |
| View follows cursor | docs     | guide/long-page.md | `v G` | Line 80  | yes      |

## Copying to the clipboard

temoc-ui copies through the terminal using the OSC 52 escape sequence, so it works
over SSH and needs no clipboard tools installed. The text is sent as UTF-8 encoded
in base64. The terminal must allow OSC 52, and some have it turned off by default.

`clipboard sequence?` shows what was written to the terminal, with the escape and
bell control characters written as `ESC` and `BEL`.

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # action       | workspace | arguments                | keys                     | clipboard sequence?                       |
| -------------- | --------- | ------------------------ | ------------------------ | ----------------------------------------- |
| Mouse          | docs      | guide/getting-started.md | `Drag "Run" "installer"` | `ESC]52;c;UnVuIHRoZSBpbnN0YWxsZXI=BEL`       |
| Keyboard       | docs      |                          | `v j j j j v $ y`        | `ESC]52;c;4oCiIEdldHRpbmcgc3RhcnRlZA==BEL`   |
| Nothing copied | docs      |                          | `v v`                    | none                                      |
