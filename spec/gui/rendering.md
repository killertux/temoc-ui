# Rendering

Back to the [specification index](../README.md).

The GUI shows the same markdown as the [terminal](../tui/rendering.md), and shows some
of it better. The [markdown elements](design/images/markdown.png) picture shows every
element, and how to build it. These tables cover what behaves differently.

The [`elements`](../../samples/elements/README.md) sample has one of each.

## What stays hidden

temoc directives and HTML comments draw nothing. Any other HTML shows as its source, and
none of it is interpreted.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Gui -- snooze until 2099-12-31"

| # element       | workspace  | arguments  | look for               | visible? |
| --------------- | ---------- | ---------- | ---------------------- | -------- |
| Table directive | calculator | passing.md | decisionTable          | no       |
| Import          | calculator | passing.md | Sample.Fixtures        | no       |
| Comment         | elements   |            | A hidden comment       | no       |
| Other HTML      | elements   |            | `<details>raw html</details>` | yes |

## Alerts

A quote that starts with `[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]` or `[!CAUTION]`
is an alert. It shows its kind as a title, and the marker itself is not shown. The
terminal shows these as plain quotes.

[//]: # "decisionTable Gui -- snooze until 2099-12-31"

| # element        | workspace | look for        | visible? |
| ---------------- | --------- | --------------- | -------- |
| The title        | elements  | Note            | yes      |
| Another title    | elements  | Warning         | yes      |
| The text         | elements  | A note to read. | yes      |
| No marker        | elements  | [!NOTE]         | no       |
| A plain quote    | elements  | A plain quote.  | yes      |

## Task lists

Items show a box that is ticked or not. It only shows what the file says: clicking it
does nothing, and temoc-ui never changes a file.

[//]: # "decisionTable Gui -- snooze until 2099-12-31"

| # situation    | workspace | actions             | checked?                      |
| -------------- | --------- | ------------------- | ----------------------------- |
| As in the file | elements  |                     | Done item: yes, Open item: no |
| Clicking       | elements  | `Click "Open item"` | Done item: yes, Open item: no |

## Images

A picture in the workspace is drawn on the page, scaled down to fit and never up. Only
files in the workspace are loaded, so that opening a page never reaches the network.
Anything that cannot be drawn shows its alt text and its path instead.

[//]: # "decisionTable Gui -- snooze until 2099-12-31"

| # situation       | workspace | look for                           | image shown? | visible? |
| ----------------- | --------- | ---------------------------------- | ------------ | -------- |
| A file            | elements  | A small picture                    | picture.png  | no       |
| A missing file    | elements  | A lost picture                     | picture.png  | yes      |
| Its path          | elements  | lost.png                           | picture.png  | yes      |
| A remote picture  | elements  | A remote picture                   | picture.png  | yes      |
| A remote path     | elements  | https://example.com/remote.png     | picture.png  | yes      |

## Code blocks

Code keeps its text exactly as written, without syntax colours, and shows its language
if it has one. `Copy` copies the whole block.

[//]: # "decisionTable Gui -- snooze until 2099-12-31"

| # situation  | workspace | actions        | look for    | visible? | clipboard?  | status?              |
| ------------ | --------- | -------------- | ----------- | -------- | ----------- | -------------------- |
| Its text     | elements  |                | echo copied | yes      | none        | none                 |
| Its language | elements  |                | sh          | yes      | none        | none                 |
| Copying      | elements  | `Click "Copy"` | echo copied | yes      | echo copied | Copied 11 characters |

## Footnotes

A footnote reference is a link to its definition.

[//]: # "decisionTable Gui -- snooze until 2099-12-31"

| # situation | workspace | actions     | look for                 | visible? |
| ----------- | --------- | ----------- | ------------------------ | -------- |
| Following   | elements  | `Click "1"` | The source of the claim. | yes      |
