# Search

Back to the [specification index](../README.md).

`/` opens the search box over the document. Typing narrows
[what can be found](../search.md). `Enter` opens the top result, and `Esc` closes
search without moving.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Tui"

| # situation     | workspace | keys             | search open? | results?                                                                        |
| --------------- | --------- | ---------------- | ------------ | ------------------------------------------------------------------------------- |
| Closed at first | docs      |                  | no           | none                                                                            |
| All files       | docs      | `/`              | yes          | guide/getting-started.md, guide/long-page.md, notes.MD, README.md, reference.md |
| Typing narrows  | docs      | `/ "install"`    | yes          | guide/getting-started.md > Install                                              |

[//]: # "decisionTable Tui"

| # situation           | workspace | keys             | first result?            |
| --------------------- | --------- | ---------------- | ------------------------ |
| Files before headings | docs      | `/ "reference"`  | reference.md             |

## Opening a result

`Down` and `Up` choose a result. Opening a heading scrolls to it. Going
[back](navigation.md#history) returns to where you were.

[//]: # "decisionTable Tui"

| # situation           | workspace | keys                    | document?                | search open? |
| --------------------- | --------- | ----------------------- | ------------------------ | ------------ |
| Open the top result   | docs      | `/ "reference" Enter`   | reference.md             | no           |
| Choose a result       | docs      | `/ Down Enter`          | guide/long-page.md       | no           |
| Choose and come back  | docs      | `/ Down Up Enter`       | guide/getting-started.md | no           |
| Heading result        | docs      | `/ "install" Enter`     | guide/getting-started.md | no           |
| Enter with no results | docs      | `/ "zzz" Enter`         | README.md                | yes          |
| Cancel                | docs      | `/ "reference" Esc`     | README.md                | no           |
| Back to the start     | docs      | `/ "reference" Enter h` | README.md                | no           |

[//]: # "decisionTable Tui"

| # situation            | workspace | keys               | look for  | visible? |
| ---------------------- | --------- | ------------------ | --------- | -------- |
| Scrolls to the heading | docs      | `/ "middle" Enter` | Middle    | yes      |
| Leaves the top behind  | docs      | `/ "middle" Enter` | Long Page | no       |

## Showing only test files

`Ctrl-t` shows only [test files](../search.md#showing-only-test-files), and pressing it
again shows everything. The choice is remembered until temoc-ui closes.

[//]: # "decisionTable Tui"

| # situation          | workspace  | keys               | results?                                                           |
| -------------------- | ---------- | ------------------ | ------------------------------------------------------------------ |
| Only test files      | calculator | `/ Ctrl-t`         | broken.md, failing.md, mixed.md, passing.md, snoozed.md            |
| Back to all files    | calculator | `/ Ctrl-t Ctrl-t`  | broken.md, failing.md, mixed.md, passing.md, README.md, snoozed.md |
| Combines with typing | calculator | `/ Ctrl-t "pass"`  | passing.md, passing.md > Passing Tests                             |
| Remembered next time | calculator | `/ Ctrl-t Esc /`   | broken.md, failing.md, mixed.md, passing.md, snoozed.md            |
