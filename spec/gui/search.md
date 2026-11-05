# Search

Back to the [specification index](../README.md).

`/` or the search field in the toolbar opens the search box over the page. Typing
narrows [what can be found](../search.md). `Up` and `Down` choose a result, `Enter`
opens it, and `Esc` closes search without moving.

![Search](design/images/search.png)

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Gui -- snooze until 2099-12-31"

| # situation      | workspace | actions                               | search open? | results?                                                                        |
| ---------------- | --------- | ------------------------------------- | ------------ | ------------------------------------------------------------------------------- |
| Closed at first  | docs      |                                       | no           | none                                                                            |
| The key          | docs      | `/`                                   | yes          | guide/getting-started.md, guide/long-page.md, notes.MD, README.md, reference.md |
| The toolbar      | docs      | `Click "Search files and headings"`   | yes          | guide/getting-started.md, guide/long-page.md, notes.MD, README.md, reference.md |
| Typing narrows   | docs      | `/ "install"`                         | yes          | guide/getting-started.md > Install                                              |
| Esc closes       | docs      | `/ Esc`                               | no           | none                                                                            |

## Opening a result

Clicking a result opens it too. Going [back](navigation.md#history) returns to where
you were.

[//]: # "decisionTable Gui -- snooze until 2099-12-31"

| # situation           | workspace | actions                                     | document?                | search open? |
| --------------------- | --------- | ------------------------------------------- | ------------------------ | ------------ |
| The top result        | docs      | `/ "reference" Enter`                       | reference.md             | no           |
| Choose a result       | docs      | `/ Down Enter`                              | guide/long-page.md       | no           |
| Click a result        | docs      | `/ Click "guide/long-page.md"`              | guide/long-page.md       | no           |
| Enter with no results | docs      | `/ "zzz" Enter`                             | README.md                | yes          |
| Back to the start     | docs      | `/ "reference" Enter h`                     | README.md                | no           |

## Letters stay letters

While the search box has the keyboard, letters are typed and do nothing else.

[//]: # "decisionTable Gui -- snooze until 2099-12-31"

| # situation     | workspace | actions        | search open? | still open? | mode? | help open? |
| --------------- | --------- | -------------- | ------------ | ----------- | ----- | ---------- |
| q is typed      | docs      | `/ q`          | yes          | yes         | dark  | no         |
| T is typed      | docs      | `/ T`          | yes          | yes         | dark  | no         |
| ? is typed      | docs      | `/ ?`          | yes          | yes         | dark  | no         |

## Showing only test files

`Ctrl-t` or the `Test files only` button shows only
[test files](../search.md#showing-only-test-files), and doing it again shows everything.
The choice is remembered until temoc-ui closes.

[//]: # "decisionTable Gui -- snooze until 2099-12-31"

| # situation          | workspace  | actions                     | results?                                                           |
| -------------------- | ---------- | --------------------------- | ------------------------------------------------------------------ |
| The key              | calculator | `/ Ctrl-t`                  | broken.md, failing.md, mixed.md, passing.md, snoozed.md            |
| The button           | calculator | `/ Click "Test files only"` | broken.md, failing.md, mixed.md, passing.md, snoozed.md            |
| Back to all files    | calculator | `/ Ctrl-t Ctrl-t`           | broken.md, failing.md, mixed.md, passing.md, README.md, snoozed.md |
| Remembered next time | calculator | `/ Ctrl-t Esc /`            | broken.md, failing.md, mixed.md, passing.md, snoozed.md            |
