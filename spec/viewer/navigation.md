# Navigation

Back to the [specification index](../README.md).

## Scrolling

These rows use a page longer than the screen.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Tui"

| # action             | workspace | arguments          | keys              | look for  | visible? |
| -------------------- | --------- | ------------------ | ----------------- | --------- | -------- |
| The top shows first  | docs      | guide/long-page.md |                   | Long Page | yes      |
| The rest is below    | docs      | guide/long-page.md |                   | Line 80   | no       |
| Scroll down          | docs      | guide/long-page.md | `j`               | Long Page | no       |
| Scroll back up       | docs      | guide/long-page.md | `j k`             | Long Page | yes      |
| Stop at the top      | docs      | guide/long-page.md | `k k j k`         | Long Page | yes      |
| Arrow keys scroll    | docs      | guide/long-page.md | `Down`            | Long Page | no       |
| Page down            | docs      | guide/long-page.md | `PageDown`        | Long Page | no       |
| Space pages down     | docs      | guide/long-page.md | `Space`           | Long Page | no       |
| Page up              | docs      | guide/long-page.md | `PageDown PageUp` | Long Page | yes      |
| Go to the bottom     | docs      | guide/long-page.md | `G`               | Line 80   | yes      |
| Go to the top        | docs      | guide/long-page.md | `G g`             | Long Page | yes      |

## Focusing links

`Tab` moves through the links on the page and wraps around at either end. The
focused link is highlighted and scrolled into view.

[//]: # "decisionTable Tui"

| # action                 | workspace | keys                          | focused link?   |
| ------------------------ | --------- | ----------------------------- | --------------- |
| Nothing is focused first | docs      |                               | none            |
| First link               | docs      | `Tab`                         | Getting started |
| Next link                | docs      | `Tab Tab`                     | Install steps   |
| Previous link            | docs      | `Tab Tab Shift-Tab`           | Getting started |
| Backwards from the start | docs      | `Shift-Tab`                   | Temoc on GitHub |
| Wraps past the last link | docs      | `Tab Tab Tab Tab Tab Tab Tab Tab` | Getting started |

## Following links

`Enter` opens the focused link, and [clicking](mouse.md#following-links) a link does
the same. Relative paths are resolved from the current document.

[//]: # "decisionTable Tui"

| # action                  | workspace | keys                  | document?                |
| ------------------------- | --------- | --------------------- | ------------------------ |
| Enter with nothing focused | docs     | `Enter`               | README.md                |
| Link into a subfolder     | docs      | `Tab Enter`           | guide/getting-started.md |
| Link in the same folder   | docs      | `Tab Tab Tab Enter`   | reference.md             |
| Link to a parent folder   | docs      | `Tab Enter Tab Enter` | README.md                |

A link with a heading anchor opens the page and scrolls to that heading.

[//]: # "decisionTable Tui"

| # action                | workspace | keys                      | document?          | look for | visible? |
| ----------------------- | --------- | ------------------------- | ------------------ | -------- | -------- |
| Scrolls to the heading  | docs      | `Tab Tab Tab Tab Tab Enter` | guide/long-page.md | The End  | yes      |
| Leaves the top behind   | docs      | `Tab Tab Tab Tab Tab Enter` | guide/long-page.md | Long Page | no      |

Links that cannot be opened leave you on the same page and explain why. temoc-ui
does not launch a web browser.

[//]: # "decisionTable Tui"

| # action      | workspace | keys                                | document? | status?                                             |
| ------------- | --------- | ----------------------------------- | --------- | --------------------------------------------------- |
| Missing file  | docs      | `Tab Tab Tab Tab Tab Tab Enter`     | README.md | Link target not found: missing.md                   |
| External link | docs      | `Tab Tab Tab Tab Tab Tab Tab Enter` | README.md | External link: https://github.com/killertux/temoc   |

## Moving between tables

`]` moves to the next [test table](../glossary.md) and `[` to the previous one,
scrolling it into view. The first table is focused when a test file opens. Focus
stops at the first and last tables and does not wrap around.

[//]: # "decisionTable Tui"

| # action           | workspace  | arguments  | keys    | focused table? |
| ------------------ | ---------- | ---------- | ------- | -------------- |
| First table        | calculator | passing.md |         | Addition       |
| Next table         | calculator | passing.md | `]`     | Multiplication |
| Previous table     | calculator | passing.md | `] [`   | Addition       |
| Stops at the end   | calculator | passing.md | `] ]`   | Multiplication |
| Stops at the start | calculator | passing.md | `[`     | Addition       |
| Page without tests | docs       | reference.md | `]`   | none           |

[//]: # "decisionTable Tui"

| # action                | workspace | arguments     | keys  | look for  | visible? |
| ----------------------- | --------- | ------------- | ----- | --------- | -------- |
| Far table is off screen | detection | far-tables.md |       | Far Table | no       |
| Scrolls to the table    | detection | far-tables.md | `]`   | Far Table | yes      |
| Scrolls back            | detection | far-tables.md | `] [` | Near Table | yes     |

## History

`h` or `Backspace` goes back and `l` goes forward, like a web browser. Following a
new link clears the forward history.

[//]: # "decisionTable Tui"

| # action                 | workspace | keys                    | document?                |
| ------------------------ | --------- | ----------------------- | ------------------------ |
| Back                     | docs      | `Tab Enter h`           | README.md                |
| Backspace goes back      | docs      | `Tab Enter Backspace`   | README.md                |
| Forward                  | docs      | `Tab Enter h l`         | guide/getting-started.md |
| Nothing to go back to    | docs      | `h`                     | README.md                |
| Nothing to go forward to | docs      | `l`                     | README.md                |
| New link clears forward  | docs      | `Tab Enter h Tab Tab Enter l` | reference.md       |

Going back restores the scroll position and the focused link.

[//]: # "decisionTable Tui"

| # action          | workspace | keys          | focused link?   |
| ----------------- | --------- | ------------- | --------------- |
| Focus is restored | docs      | `Tab Enter h` | Getting started |

[//]: # "decisionTable Tui"

| # action           | workspace | keys                                            | look for | visible? |
| ------------------ | --------- | ----------------------------------------------- | -------- | -------- |
| Scroll is restored | docs      | `Tab Tab Tab Tab Enter G / "reference" Enter h` | Line 80  | yes      |
