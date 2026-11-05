# Search

Back to the [specification index](README.md).

Search is the quickest way to jump anywhere in the workspace. The
[terminal](tui/search.md) shows it in a box over the document.

## What can be found

With nothing typed, every markdown file in the workspace is listed in alphabetical
order, ignoring case. Files count as markdown when their extension is `.md` in any
case. Hidden folders and folders named `target` or `node_modules` are skipped.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Core"

| # situation     | workspace | steps    | results?                                                                        |
| --------------- | --------- | -------- | ------------------------------------------------------------------------------- |
| Before a search | docs      |          | none                                                                            |
| All files       | docs      | `search` | guide/getting-started.md, guide/long-page.md, notes.MD, README.md, reference.md |

Typing narrows the results with a fuzzy match on file paths and headings. Letters
must appear in order but need not be next to each other, and case is ignored. A
heading result is written as `file > heading`. File matches come before heading
matches.

[//]: # "decisionTable Core"

| # situation          | workspace | steps            | results?                           |
| -------------------- | --------- | ---------------- | ---------------------------------- |
| Matches a heading    | docs      | `search install` | guide/getting-started.md > Install |
| Ignores case         | docs      | `search INSTALL` | guide/getting-started.md > Install |
| Letters may be apart | docs      | `search instl`   | guide/getting-started.md > Install |
| No matches           | docs      | `search zzz`     | none                               |
| Hidden folders       | docs      | `search secret`  | none                               |
| Build folders        | docs      | `search build`   | none                               |
| Not markdown         | docs      | `search data`    | none                               |

[//]: # "decisionTable Core"

| # situation           | workspace | steps              | first result?            |
| --------------------- | --------- | ------------------ | ------------------------ |
| Files before headings | docs      | `search reference` | reference.md             |
| Fuzzy file match      | docs      | `search getstart`  | guide/getting-started.md |

## Opening a result

Opening a heading goes to that heading. Going [back](viewer/links.md#history)
returns to where you were.

[//]: # "decisionTable Core"

| # situation        | workspace | steps                                  | document?                | anchor? |
| ------------------ | --------- | -------------------------------------- | ------------------------ | ------- |
| Open the top result | docs     | `search reference, open result 1`      | reference.md             | none    |
| Choose a result    | docs      | `search, open result 2`                | guide/long-page.md       | none    |
| Heading result     | docs      | `search install, open result 1`        | guide/getting-started.md | install |
| No such result     | docs      | `search zzz, open result 1`            | README.md                | none    |
| Back to the start  | docs      | `search reference, open result 1, back` | README.md               | none    |

## Showing only test files

Search can show only [test files](running/finding-tests.md).

[//]: # "decisionTable Core"

| # situation          | workspace  | steps                                    | results?                                                           |
| -------------------- | ---------- | ---------------------------------------- | ------------------------------------------------------------------ |
| All files            | calculator | `search`                                 | broken.md, failing.md, mixed.md, passing.md, README.md, snoozed.md |
| Only test files      | calculator | `search, only test files`                | broken.md, failing.md, mixed.md, passing.md, snoozed.md            |
| Back to all files    | calculator | `search, only test files, only test files` | broken.md, failing.md, mixed.md, passing.md, README.md, snoozed.md |
| Combines with typing | calculator | `search pass, only test files`           | passing.md, passing.md > Passing Tests                             |
| Excludes other files | calculator | `search readme, only test files`         | none                                                               |
| No test files        | docs       | `search, only test files`                | none                                                               |
