# Links and History

Back to the [specification index](../README.md).

## Following links

Following a link opens what it points at. Relative paths are resolved from the
current document.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Core"

| # action                | workspace | path                     | steps                             | document?                |
| ----------------------- | --------- | ------------------------ | --------------------------------- | ------------------------ |
| Link into a subfolder   | docs      |                          | `follow guide/getting-started.md` | guide/getting-started.md |
| Link in the same folder | docs      | guide/getting-started.md | `follow long-page.md`             | guide/long-page.md       |
| Link to a parent folder | docs      | guide/getting-started.md | `follow ../README.md`             | README.md                |

A link with a heading anchor opens the page at that heading.

[//]: # "decisionTable Core"

| # action            | workspace | steps                               | document?                | anchor? |
| ------------------- | --------- | ----------------------------------- | ------------------------ | ------- |
| Goes to the heading | docs      | `follow guide/long-page.md#the-end` | guide/long-page.md       | the-end |
| Another page        | docs      | `follow guide/getting-started.md#install` | guide/getting-started.md | install |
| No anchor           | docs      | `follow reference.md`               | reference.md             | none    |

Links that cannot be opened leave you on the same page and explain why. temoc-ui
does not launch a web browser.

[//]: # "decisionTable Core"

| # action      | workspace | steps                                     | document? | problem?                                          |
| ------------- | --------- | ----------------------------------------- | --------- | ------------------------------------------------- |
| Missing file  | docs      | `follow missing.md`                       | README.md | Link target not found: missing.md                 |
| External link | docs      | `follow https://github.com/killertux/temoc` | README.md | External link: https://github.com/killertux/temoc |

## History

Going back and forward works like a web browser. Following a new link clears the
forward history.

[//]: # "decisionTable Core"

| # action                 | workspace | steps                                                         | document?                |
| ------------------------ | --------- | ------------------------------------------------------------- | ------------------------ |
| Back                     | docs      | `follow guide/getting-started.md, back`                       | README.md                |
| Forward                  | docs      | `follow guide/getting-started.md, back, forward`              | guide/getting-started.md |
| Nothing to go back to    | docs      | `back`                                                        | README.md                |
| Nothing to go forward to | docs      | `forward`                                                     | README.md                |
| New link clears forward  | docs      | `follow guide/long-page.md, back, follow reference.md, forward` | reference.md             |
| Opening a file counts    | docs      | `open reference.md, back`                                     | README.md                |
