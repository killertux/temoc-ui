# Navigation

Back to the [specification index](../README.md).

## Scrolling

The wheel and the scroll bar work as in any window. `Up` and `Down` scroll a line,
`PageUp`, `PageDown` and `Space` a page, and `Home` and `End` go to either end.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Gui"

| # action          | workspace | arguments          | actions             | page scroll? |
| ----------------- | --------- | ------------------ | ------------------- | ------------ |
| Starts at the top | docs      | guide/long-page.md |                     | top          |
| One line          | docs      | guide/long-page.md | `Down`              | middle       |
| Back up           | docs      | guide/long-page.md | `Down Up`           | top          |
| Page down         | docs      | guide/long-page.md | `PageDown`          | middle       |
| Space             | docs      | guide/long-page.md | `Space`             | middle       |
| Page up           | docs      | guide/long-page.md | `PageDown PageUp`   | top          |
| Bottom            | docs      | guide/long-page.md | `End`               | bottom       |
| Top               | docs      | guide/long-page.md | `End Home`          | top          |
| The wheel         | docs      | guide/long-page.md | `Wheel "Long Page"` | middle       |

## Wide blocks

Tables and code blocks are never wrapped. One wider than the page has a scroll bar of
its own, and only that block moves. The rest of the page stays where it is, and so does
any other wide block.

Its scroll bar, a trackpad or a tilting wheel scrolls the block under the pointer.
`Left` and `Right` scroll the block under the pointer, or else the
[focused table](running.md#choosing-a-table). A block that fits does not scroll.

[//]: # "decisionTable Gui"

| # action                 | workspace   | arguments    | actions                              | look for               | sideways? |
| ------------------------ | ----------- | ------------ | ------------------------------------ | ---------------------- | --------- |
| Starts at the left       | wide-blocks |              |                                      | First Column           | start     |
| The wheel                | wide-blocks |              | `Wheel sideways "First Column"`      | First Column           | middle    |
| The other block stays    | wide-blocks |              | `Wheel sideways "First Column"`      | start of the long line | start     |
| Code scrolls by itself   | wide-blocks |              | `Wheel sideways "start of the long"` | start of the long line | middle    |
| The table stays          | wide-blocks |              | `Wheel sideways "start of the long"` | First Column           | start     |
| Right, under the pointer | wide-blocks |              | `Hover "First Column" Right`         | First Column           | middle    |
| Left goes back           | wide-blocks |              | `Hover "First Column" Right Left`    | First Column           | start     |
| Keys need a block        | wide-blocks |              | `Leave Right`                        | First Column           | start     |
| The focused table        | elements    | wide-test.md | `Leave Right`                        | a long comment         | middle    |
| A block that fits        | wide-blocks |              | `Wheel sideways "Fits"`              | Fits                   | start     |

[//]: # "decisionTable Gui"

| # action            | workspace   | actions                         | look for        | visible? |
| ------------------- | ----------- | ------------------------------- | --------------- | -------- |
| Text stays in place | wide-blocks | `Wheel sideways "First Column"` | This text stays | yes      |
| Focus shows a link  | wide-blocks | `Tab Tab`                       | Far link        | yes      |

Going back restores how far each block was scrolled, and a new page starts at the left.

[//]: # "decisionTable Gui"

| # action            | workspace   | actions                                                     | look for     | sideways? |
| ------------------- | ----------- | ----------------------------------------------------------- | ------------ | --------- |
| Going back restores | wide-blocks | `Wheel sideways "First Column" Click "other page" Alt-Left` | First Column | middle    |

## Focusing links

`Tab` moves through the links on the page and wraps around at either end, as in the
[terminal](../tui/navigation.md#focusing-links).

[//]: # "decisionTable Gui"

| # action                 | workspace | actions             | focused link?   |
| ------------------------ | --------- | ------------------- | --------------- |
| Nothing is focused first | docs      |                     | none            |
| First link               | docs      | `Tab`               | Getting started |
| Previous link            | docs      | `Tab Tab Shift-Tab` | Getting started |
| Backwards from the start | docs      | `Shift-Tab`         | Temoc on GitHub |

## Following links

Clicking a link follows it, and so does `Enter` on the focused link. See
[how links resolve](../viewer/links.md).

[//]: # "decisionTable Gui"

| # action     | workspace | actions                    | document?                | look for | visible? |
| ------------ | --------- | -------------------------- | ------------------------ | -------- | -------- |
| Click a link | docs      | `Click "Reference"`        | reference.md             | Commands | yes      |
| Enter        | docs      | `Tab Enter`                | guide/getting-started.md | Install  | yes      |
| A heading    | docs      | `Click "Long page ending"` | guide/long-page.md       | The End  | yes      |

A link to a website opens in the web browser, and the page stays as it was. Hovering any
link shows where it goes in the status bar.

A link to a file that does not exist stays on the page and says why.

[//]: # "decisionTable Gui"

| # action       | workspace | actions                   | document? | opened url?                        | status?                                                    |
| -------------- | --------- | ------------------------- | --------- | ---------------------------------- | ---------------------------------------------------------- |
| A website      | docs      | `Click "Temoc on GitHub"` | README.md | https://github.com/killertux/temoc | Opened in your browser: https://github.com/killertux/temoc |
| By keyboard    | docs      | `Shift-Tab Enter`         | README.md | https://github.com/killertux/temoc | Opened in your browser: https://github.com/killertux/temoc |
| A missing file | docs      | `Click "Missing page"`    | README.md | none                               | Link target not found: missing.md                          |
| Hovering       | docs      | `Hover "Reference"`       | README.md | none                               | reference.md                                               |

## History

`Alt-Left` or `Ctrl-[` goes back and `Alt-Right` or `Ctrl-]` goes forward, as do the
back and forward buttons of a mouse, and the toolbar has a button for each. See [how history works](../viewer/links.md#history). Going back restores how far
the page was scrolled and the focused link.

[//]: # "decisionTable Gui"

| # action     | workspace | actions                                                | document?    |
| ------------ | --------- | ------------------------------------------------------ | ------------ |
| Back         | docs      | `Click "Reference" Alt-Left`                           | README.md    |
| Other key    | docs      | `Click "Reference" Ctrl-[`                             | README.md    |
| Mouse button | docs      | `Click "Reference" Mouse back`                         | README.md    |
| Forward      | docs      | `Click "Reference" Alt-Left Alt-Right`                 | reference.md |
| Other key on | docs      | `Click "Reference" Ctrl-[ Ctrl-]`                      | reference.md |
| Mouse on     | docs      | `Click "Reference" Mouse back Mouse forward`           | reference.md |
| The buttons  | docs      | `Click "Reference" Click "Go back"`                    | README.md    |
| And forward  | docs      | `Click "Reference" Click "Go back" Click "Go forward"` | reference.md |

[//]: # "decisionTable Gui"

| # action          | workspace | actions              | focused link?   |
| ----------------- | --------- | -------------------- | --------------- |
| Focus is restored | docs      | `Tab Enter Alt-Left` | Getting started |
