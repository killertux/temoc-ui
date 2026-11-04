# Themes

Back to the [specification index](../README.md).

temoc-ui has a dark theme and a light theme, each in two kinds of colour. A theme paints
every cell of the screen, background included, so that a page looks the same on any
terminal and its contrast is known.

| Theme       | Page  | Colours                 |
| ----------- | ----- | ----------------------- |
| `dark`      | Dark  | 24-bit                  |
| `light`     | White | 24-bit                  |
| `dark-256`  | Dark  | The 256-colour palette  |
| `light-256` | White | The 256-colour palette  |

The 256-colour themes only use palette entries 16 to 255, which terminals leave fixed.
Entries 0 to 15 are the ones each terminal changes to suit itself.

## Which theme opens

`--theme NAME` opens that theme, whatever the terminal is:

```sh
temoc-ui [PATH] --theme light-256
```

Otherwise temoc-ui matches the terminal, in two steps.

- **The kind of colour.** The 24-bit themes are used when the `COLORTERM` environment
  variable says `truecolor` or `24bit`. Anything else gets a 256-colour theme.
- **Dark or light.** temoc-ui asks the terminal for its background colour with the
  OSC 11 escape sequence, and waits a fifth of a second for the answer. A background is
  light when black text would have more contrast on it than white text. Without an
  answer, the theme is dark.

The `color term` column is the value of `COLORTERM`, and empty means that it is not
set. The `terminal background` column is what the terminal answers.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Tui"

| # situation            | workspace | color term | terminal background | arguments           | theme?    |
| ---------------------- | --------- | ---------- | ------------------- | ------------------- | --------- |
| Nothing is known       | docs      |            | no answer           |                     | dark-256  |
| 24-bit, no answer      | docs      | truecolor  | no answer           |                     | dark      |
| 24-bit on black        | docs      | truecolor  | #000000             |                     | dark      |
| 24-bit on white        | docs      | truecolor  | #FFFFFF             |                     | light     |
| The other spelling     | docs      | 24bit      | #FFFFFF             |                     | light     |
| Palette on black       | docs      |            | #000000             |                     | dark-256  |
| Palette on white       | docs      |            | #FFFFFF             |                     | light-256 |
| Another COLORTERM      | docs      | yes        | #FFFFFF             |                     | light-256 |
| Dark grey is dark      | docs      | truecolor  | #404040             |                     | dark      |
| Middle grey is light   | docs      | truecolor  | #808080             |                     | light     |
| The option wins        | docs      | truecolor  | #000000             | --theme light       | light     |
| It can pick a palette  | docs      | truecolor  | #FFFFFF             | --theme dark-256    | dark-256  |
| It can pick 24-bit     | docs      |            | no answer           | --theme light       | light     |

[//]: # "decisionTable Tui"

| # situation        | workspace | arguments     | still open? | status?              |
| ------------------ | --------- | ------------- | ----------- | -------------------- |
| An unknown theme   | docs      | --theme blue  | no          | Unknown theme: blue  |
| Theme without name | docs      | --theme       | no          | --theme needs a name |

## Swapping themes

`T` swaps between dark and light and keeps the kind of colour, so a terminal that cannot
show 24-bit colour never gets it. The choice lasts until temoc-ui closes.

`T` works everywhere but the search box, where it is typed like any other letter, and
help, where other keys do nothing. Swapping changes nothing but the colours.

[//]: # "decisionTable Tui"

| # situation       | workspace | color term | arguments         | keys         | theme?    |
| ----------------- | --------- | ---------- | ----------------- | ------------ | --------- |
| Dark to light     | docs      | truecolor  |                   | `T`          | light     |
| And back          | docs      | truecolor  |                   | `T T`        | dark      |
| Palette to light  | docs      |            |                   | `T`          | light-256 |
| Palette to dark   | docs      |            | --theme light-256 | `T`          | dark-256  |
| In the file tree  | docs      | truecolor  |                   | `t T`        | light     |
| In the graph      | docs      | truecolor  |                   | `Ctrl-g T`   | light     |
| While selecting   | docs      | truecolor  |                   | `v T`        | light     |
| Typed in search   | docs      | truecolor  |                   | `/ T`        | dark      |
| Ignored in help   | docs      | truecolor  |                   | `? T`        | dark      |

[//]: # "decisionTable Tui"

| # situation           | workspace | keys      | focused link? | tree open? |
| --------------------- | --------- | --------- | ------------- | ---------- |
| The focus stays       | themes    | `Tab T`   | the next page | no         |
| The file tree stays   | themes    | `t T`     | none          | yes        |

## Every cell is painted

No cell is left to the terminal's own colours. This covers the blank cells of the page,
the whole width of the status bar, and everything inside a pop-up. `unpainted cells?`
counts the cells whose foreground or background is the terminal's default.

[//]: # "decisionTable Tui"

| # screen      | workspace  | arguments  | keys     | unpainted cells? |
| ------------- | ---------- | ---------- | -------- | ---------------- |
| A page        | themes     |            |          | 0                |
| A short page  | themes     | last.md    |          | 0                |
| After a run   | calculator | failing.md | `R`      | 0                |
| Search        | themes     |            | `/`      | 0                |
| The file tree | themes     |            | `t`      | 0                |
| The graph     | themes     |            | `Ctrl-g` | 0                |
| Help          | themes     |            | `?`      | 0                |

## Contrast

Contrast is the WCAG ratio between two colours, from 1 for the same colour to 21 for
black on white. These are the limits every theme keeps to.

| What                                   | Against          | At least |
| -------------------------------------- | ---------------- | -------- |
| Text, including the status bar's       | Its background   | 4.5      |
| Lines that are not text, such as links in the graph | Its background | 3 |
| The status bar                         | The page         | 3        |

The status bar is the opposite of the page: a light bar with dark text in the dark
themes, and a dark bar with white text in the light themes. It stands apart from the
markdown whatever the page shows. The file result in the bar uses the bar's own text
colour.

[//]: # "decisionTable Tui"

| # theme   | workspace | arguments         | bar against page? |
| --------- | --------- | ----------------- | ----------------- |
| dark      | themes    | --theme dark      | 7.7               |
| light     | themes    | --theme light     | 11.5              |
| dark-256  | themes    | --theme dark-256  | 7.4               |
| light-256 | themes    | --theme light-256 | 7.0               |

## Colours

Each colour has one meaning. Yellow is only for snoozed results, so inline code is
purple, and red and green are only for results. The note under a row that did not pass
uses a quieter, greyer red, so that it explains the row without competing with it.
The note under a snoozed row uses the quote colour. Bold, italic and struck text add
their emphasis to the plain text colour, and the same goes for bold text in a pop-up.

Focus and selection stay [reversed](../running/running-a-table.md), swapping the
foreground and background of what they cover. That keeps its contrast, and also works
on a failing row.

`foreground?` and `background?` read the screen at the `look for` text. They are hex
codes in the 24-bit themes and palette entries in the others. `contrast?` is the ratio
between the two, rounded to one decimal place. It uses the usual value of a palette
entry.

### dark

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # style        | workspace  | arguments               | keys         | look for          | foreground? | background? | contrast? |
| -------------- | ---------- | ----------------------- | ------------ | ----------------- | ----------- | ----------- | --------- |
| Plain text     | themes     | --theme dark            |              | Plain words       | #E6E8EC     | #16181D     | 14.5      |
| Heading 1      | themes     | --theme dark            |              | Theme Sample      | #5FD7E6     | #16181D     | 10.4      |
| Heading 2      | themes     | --theme dark            |              | Second Heading    | #5FD7E6     | #16181D     | 10.4      |
| Inline code    | themes     | --theme dark            |              | inline code       | #D7A6FF     | #16181D     | 9.1       |
| Code block     | themes     | --theme dark            |              | a line too long   | #D7A6FF     | #16181D     | 9.1       |
| Link           | themes     | --theme dark            |              | the next page     | #8AB4FF     | #16181D     | 8.5       |
| Quote          | themes     | --theme dark            |              | Quoted words      | #A8B0BD     | #16181D     | 8.1       |
| Scroll marker  | themes     | --theme dark            |              | ›                 | #A8B0BD     | #16181D     | 8.1       |
| Status bar     | themes     | --theme dark            | `v`          | Move to the start | #16181D     | #87AFD7     | 7.7       |
| Pop-up title   | themes     | --theme dark            | `t`          | Files             | #E6E8EC     | #16181D     | 14.5      |
| Pop-up border  | themes     | --theme dark            | `t`          | ┌                 | #8D96A5     | #16181D     | 6.0       |
| Graph line     | themes     | --theme dark            | `Ctrl-g l l` | ▶                 | #8D96A5     | #16181D     | 6.0       |
| Lit graph line | themes     | --theme dark            | `Ctrl-g`     | ▶                 | #5FD7E6     | #16181D     | 10.4      |
| Passed table   | calculator | failing.md --theme dark | `R`          | ✓ passed          | #6EDB8F     | #16181D     | 10.3      |
| Failed table   | calculator | failing.md --theme dark | `R`          | ✗ failed          | #FF8A80     | #16181D     | 7.8       |
| Snoozed table  | calculator | snoozed.md --theme dark | `R`          | ~ snoozed         | #F5D76E     | #16181D     | 12.5      |
| Passing row    | calculator | failing.md --theme dark | `R`          | 2 │ 3 │ 6         | #6EDB8F     | #16181D     | 10.3      |
| Failing row    | calculator | failing.md --theme dark | `R`          | 2 │ 2 │ 5         | #FF8A80     | #16181D     | 7.8       |
| Snoozed row    | calculator | snoozed.md --theme dark | `R`          | 2 │ 2 │ 5         | #F5D76E     | #16181D     | 12.5      |
| Failure note   | calculator | failing.md --theme dark | `R`          | mul expected 5    | #BF8A85     | #16181D     | 6.1       |
| Snoozed note   | calculator | snoozed.md --theme dark | `R`          | mul expected 5    | #A8B0BD     | #16181D     | 8.1       |

### light

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # style        | workspace  | arguments                | keys         | look for          | foreground? | background? | contrast? |
| -------------- | ---------- | ------------------------ | ------------ | ----------------- | ----------- | ----------- | --------- |
| Plain text     | themes     | --theme light            |              | Plain words       | #1F2328     | #FFFFFF     | 15.8      |
| Heading 1      | themes     | --theme light            |              | Theme Sample      | #0A6C74     | #FFFFFF     | 6.2       |
| Heading 2      | themes     | --theme light            |              | Second Heading    | #0A6C74     | #FFFFFF     | 6.2       |
| Inline code    | themes     | --theme light            |              | inline code       | #7A3EC8     | #FFFFFF     | 6.3       |
| Code block     | themes     | --theme light            |              | a line too long   | #7A3EC8     | #FFFFFF     | 6.3       |
| Link           | themes     | --theme light            |              | the next page     | #0B57D0     | #FFFFFF     | 6.4       |
| Quote          | themes     | --theme light            |              | Quoted words      | #57606A     | #FFFFFF     | 6.4       |
| Scroll marker  | themes     | --theme light            |              | ›                 | #57606A     | #FFFFFF     | 6.4       |
| Status bar     | themes     | --theme light            | `v`          | Move to the start | #FFFFFF     | #1F3A5F     | 11.5      |
| Pop-up title   | themes     | --theme light            | `t`          | Files             | #1F2328     | #FFFFFF     | 15.8      |
| Pop-up border  | themes     | --theme light            | `t`          | ┌                 | #6E7781     | #FFFFFF     | 4.5       |
| Graph line     | themes     | --theme light            | `Ctrl-g l l` | ▶                 | #6E7781     | #FFFFFF     | 4.5       |
| Lit graph line | themes     | --theme light            | `Ctrl-g`     | ▶                 | #0A6C74     | #FFFFFF     | 6.2       |
| Passed table   | calculator | failing.md --theme light | `R`          | ✓ passed          | #116329     | #FFFFFF     | 7.4       |
| Failed table   | calculator | failing.md --theme light | `R`          | ✗ failed          | #B3261E     | #FFFFFF     | 6.5       |
| Snoozed table  | calculator | snoozed.md --theme light | `R`          | ~ snoozed         | #7D4E00     | #FFFFFF     | 7.1       |
| Passing row    | calculator | failing.md --theme light | `R`          | 2 │ 3 │ 6         | #116329     | #FFFFFF     | 7.4       |
| Failing row    | calculator | failing.md --theme light | `R`          | 2 │ 2 │ 5         | #B3261E     | #FFFFFF     | 6.5       |
| Snoozed row    | calculator | snoozed.md --theme light | `R`          | 2 │ 2 │ 5         | #7D4E00     | #FFFFFF     | 7.1       |
| Failure note   | calculator | failing.md --theme light | `R`          | mul expected 5    | #8C5A55     | #FFFFFF     | 5.7       |
| Snoozed note   | calculator | snoozed.md --theme light | `R`          | mul expected 5    | #57606A     | #FFFFFF     | 6.4       |

### dark-256

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # style        | workspace  | arguments                   | keys         | look for          | foreground? | background? | contrast? |
| -------------- | ---------- | --------------------------- | ------------ | ----------------- | ----------- | ----------- | --------- |
| Plain text     | themes     | --theme dark-256            |              | Plain words       | 254         | 234         | 13.4      |
| Heading 1      | themes     | --theme dark-256            |              | Theme Sample      | 80          | 234         | 9.9       |
| Heading 2      | themes     | --theme dark-256            |              | Second Heading    | 80          | 234         | 9.9       |
| Inline code    | themes     | --theme dark-256            |              | inline code       | 183         | 234         | 9.3       |
| Code block     | themes     | --theme dark-256            |              | a line too long   | 183         | 234         | 9.3       |
| Link           | themes     | --theme dark-256            |              | the next page     | 111         | 234         | 7.8       |
| Quote          | themes     | --theme dark-256            |              | Quoted words      | 249         | 234         | 8.0       |
| Scroll marker  | themes     | --theme dark-256            |              | ›                 | 249         | 234         | 8.0       |
| Status bar     | themes     | --theme dark-256            | `v`          | Move to the start | 234         | 110         | 7.4       |
| Pop-up title   | themes     | --theme dark-256            | `t`          | Files             | 254         | 234         | 13.4      |
| Pop-up border  | themes     | --theme dark-256            | `t`          | ┌                 | 246         | 234         | 5.6       |
| Graph line     | themes     | --theme dark-256            | `Ctrl-g l l` | ▶                 | 246         | 234         | 5.6       |
| Lit graph line | themes     | --theme dark-256            | `Ctrl-g`     | ▶                 | 80          | 234         | 9.9       |
| Passed table   | calculator | failing.md --theme dark-256 | `R`          | ✓ passed          | 78          | 234         | 9.4       |
| Failed table   | calculator | failing.md --theme dark-256 | `R`          | ✗ failed          | 210         | 234         | 7.4       |
| Snoozed table  | calculator | snoozed.md --theme dark-256 | `R`          | ~ snoozed         | 221         | 234         | 12.3      |
| Passing row    | calculator | failing.md --theme dark-256 | `R`          | 2 │ 3 │ 6         | 78          | 234         | 9.4       |
| Failing row    | calculator | failing.md --theme dark-256 | `R`          | 2 │ 2 │ 5         | 210         | 234         | 7.4       |
| Snoozed row    | calculator | snoozed.md --theme dark-256 | `R`          | 2 │ 2 │ 5         | 221         | 234         | 12.3      |
| Failure note   | calculator | failing.md --theme dark-256 | `R`          | mul expected 5    | 138         | 234         | 5.4       |
| Snoozed note   | calculator | snoozed.md --theme dark-256 | `R`          | mul expected 5    | 249         | 234         | 8.0       |

### light-256

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # style        | workspace  | arguments                    | keys         | look for          | foreground? | background? | contrast? |
| -------------- | ---------- | ---------------------------- | ------------ | ----------------- | ----------- | ----------- | --------- |
| Plain text     | themes     | --theme light-256            |              | Plain words       | 235         | 231         | 15.1      |
| Heading 1      | themes     | --theme light-256            |              | Theme Sample      | 23          | 231         | 7.5       |
| Heading 2      | themes     | --theme light-256            |              | Second Heading    | 23          | 231         | 7.5       |
| Inline code    | themes     | --theme light-256            |              | inline code       | 91          | 231         | 7.9       |
| Code block     | themes     | --theme light-256            |              | a line too long   | 91          | 231         | 7.9       |
| Link           | themes     | --theme light-256            |              | the next page     | 26          | 231         | 5.8       |
| Quote          | themes     | --theme light-256            |              | Quoted words      | 59          | 231         | 6.4       |
| Scroll marker  | themes     | --theme light-256            |              | ›                 | 59          | 231         | 6.4       |
| Status bar     | themes     | --theme light-256            | `v`          | Move to the start | 231         | 24          | 7.0       |
| Pop-up title   | themes     | --theme light-256            | `t`          | Files             | 235         | 231         | 15.1      |
| Pop-up border  | themes     | --theme light-256            | `t`          | ┌                 | 243         | 231         | 4.5       |
| Graph line     | themes     | --theme light-256            | `Ctrl-g l l` | ▶                 | 243         | 231         | 4.5       |
| Lit graph line | themes     | --theme light-256            | `Ctrl-g`     | ▶                 | 23          | 231         | 7.5       |
| Passed table   | calculator | failing.md --theme light-256 | `R`          | ✓ passed          | 22          | 231         | 8.0       |
| Failed table   | calculator | failing.md --theme light-256 | `R`          | ✗ failed          | 124         | 231         | 7.4       |
| Snoozed table  | calculator | snoozed.md --theme light-256 | `R`          | ~ snoozed         | 94          | 231         | 5.7       |
| Passing row    | calculator | failing.md --theme light-256 | `R`          | 2 │ 3 │ 6         | 22          | 231         | 8.0       |
| Failing row    | calculator | failing.md --theme light-256 | `R`          | 2 │ 2 │ 5         | 124         | 231         | 7.4       |
| Snoozed row    | calculator | snoozed.md --theme light-256 | `R`          | 2 │ 2 │ 5         | 94          | 231         | 5.7       |
| Failure note   | calculator | failing.md --theme light-256 | `R`          | mul expected 5    | 95          | 231         | 5.5       |
| Snoozed note   | calculator | snoozed.md --theme light-256 | `R`          | mul expected 5    | 59          | 231         | 6.4       |
