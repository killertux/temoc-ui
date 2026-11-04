# Search

Back to the [specification index](README.md).

`/` opens the search box, which is the quickest way to jump anywhere in the
workspace. `Enter` opens the top result, and `Esc` closes search without moving.

## What can be found

With an empty search box, every markdown file in the workspace is listed in
alphabetical order, ignoring case. Files count as markdown when their extension
is `.md` in any case. Hidden folders and folders named `target` or `node_modules`
are skipped.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation   | workspace | keys | search open? | results?                                                                        |
| ------------- | --------- | ---- | ------------ | ------------------------------------------------------------------------------- |
| Closed at first | docs    |      | no           | none                                                                            |
| All files     | docs      | `/`  | yes          | guide/getting-started.md, guide/long-page.md, notes.MD, README.md, reference.md |

Typing narrows the results with a fuzzy match on file paths and headings. Letters
must appear in order but need not be next to each other, and case is ignored. A
heading result is written as `file > heading`. File matches come before heading
matches.

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation        | workspace | keys             | results?                                                   |
| ------------------ | --------- | ---------------- | ---------------------------------------------------------- |
| Matches a heading  | docs      | `/ "install"`    | guide/getting-started.md > Install                         |
| Ignores case       | docs      | `/ "INSTALL"`    | guide/getting-started.md > Install                         |
| Letters may be apart | docs    | `/ "instl"`      | guide/getting-started.md > Install                         |
| No matches         | docs      | `/ "zzz"`        | none                                                       |
| Hidden folders     | docs      | `/ "secret"`     | none                                                       |
| Build folders      | docs      | `/ "build"`      | none                                                       |
| Not markdown       | docs      | `/ "data"`       | none                                                       |

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation            | workspace | keys             | first result?            |
| ---------------------- | --------- | ---------------- | ------------------------ |
| Files before headings  | docs      | `/ "reference"`  | reference.md             |
| Fuzzy file match       | docs      | `/ "getstart"`   | guide/getting-started.md |

## Opening a result

`Down` and `Up` choose a result. Opening a heading scrolls to it. Going
[back](viewer/navigation.md#history) returns to where you were.

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation        | workspace | keys                     | document?                | search open? |
| ------------------ | --------- | ------------------------ | ------------------------ | ------------ |
| Open the top result | docs     | `/ "reference" Enter`    | reference.md             | no           |
| Choose a result    | docs      | `/ Down Enter`           | guide/long-page.md       | no           |
| Heading result     | docs      | `/ "install" Enter`      | guide/getting-started.md | no           |
| Enter with no results | docs   | `/ "zzz" Enter`          | README.md                | yes          |
| Cancel             | docs      | `/ "reference" Esc`      | README.md                | no           |
| Back to the start  | docs      | `/ "reference" Enter h`  | README.md                | no           |

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation             | workspace | keys                | look for | visible? |
| ----------------------- | --------- | ------------------- | -------- | -------- |
| Scrolls to the heading  | docs      | `/ "middle" Enter`  | Middle   | yes      |
| Leaves the top behind   | docs      | `/ "middle" Enter`  | Long Page | no      |

## Showing only test files

`Ctrl-t` shows only [test files](running/finding-tests.md), and pressing it again
shows everything. The choice is remembered until temoc-ui closes.

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation            | workspace  | keys                       | results?                                                    |
| ---------------------- | ---------- | -------------------------- | ----------------------------------------------------------- |
| All files              | calculator | `/`                        | broken.md, failing.md, mixed.md, passing.md, README.md, snoozed.md |
| Only test files        | calculator | `/ Ctrl-t`                 | broken.md, failing.md, mixed.md, passing.md, snoozed.md     |
| Back to all files      | calculator | `/ Ctrl-t Ctrl-t`          | broken.md, failing.md, mixed.md, passing.md, README.md, snoozed.md |
| Combines with typing   | calculator | `/ Ctrl-t "pass"`          | passing.md, passing.md > Passing Tests                      |
| Excludes other files   | calculator | `/ Ctrl-t "readme"`        | none                                                        |
| Remembered next time   | calculator | `/ Ctrl-t Esc /`           | broken.md, failing.md, mixed.md, passing.md, snoozed.md     |
| No test files          | docs       | `/ Ctrl-t`                 | none                                                        |
