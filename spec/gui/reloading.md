# Reloading

Back to the [specification index](../README.md).

The page follows its file as it [changes on disk](../reloading.md), and stays scrolled
where it was. The file changes are written in the `actions` column, as described in
[how these pages work](writing-specs.md#changing-files).

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Gui"

| # situation     | workspace | arguments | actions                                     | look for                            | visible? | page scroll? |
| --------------- | --------- | --------- | ------------------------------------------- | ----------------------------------- | -------- | ------------ |
| After a change  | changing  | page.md   | `Write file "page.md" from "page-moved.md"`      | A paragraph added above the tables. | yes      | top          |
| Stays put       | changing  | long.md   | `End Write file "long.md" from "long-edited.md"` | This is the new end of the long page. | yes    | bottom       |

Results, the tests panel, and the focus follow the same rules as in the
[terminal](../tui/reloading.md#where-the-page-is).

[//]: # "decisionTable Gui"

| # situation | workspace | arguments | actions                                                      | row results?                       | focused table? | focused row? |
| ----------- | --------- | --------- | ------------------------------------------------------------ | ---------------------------------- | -------------- | ------------ |
| Lines moved | changing  | page.md   | `Ctrl-r Alt-Down Alt-Shift-Down Alt-Shift-Down Write file "page.md" from "page-moved.md"` | 13: passed, 21: passed, 22: failed | Multiplication | 22 |

## A deleted file

A banner above the page says that the file is gone, until it comes back, and the page
cannot run.

[//]: # "decisionTable Gui"

| # situation | workspace | arguments | actions                                          | banner?               | run file? |
| ----------- | --------- | --------- | ------------------------------------------------ | --------------------- | --------- |
| Deleted     | changing  | page.md   | `Delete file "page.md"`                               | File deleted: page.md | disabled  |
| Comes back  | changing  | page.md   | `Delete file "page.md" Write file "page.md" from "page-moved.md"` | none           | enabled   |

## The file tree

It updates as the files change.

[//]: # "decisionTable Gui"

| # situation    | workspace | arguments | actions                            | tree open? | tree?                                           |
| -------------- | --------- | --------- | ---------------------------------- | ---------- | ----------------------------------------------- |
| A new file     | changing  |           | `Write file "new.md" from "new.md"` | yes        | hanging.md, long.md, new.md, page.md, README.md |
| A deleted file | changing  |           | `Delete file "long.md"`            | yes        | hanging.md, page.md, README.md                  |
