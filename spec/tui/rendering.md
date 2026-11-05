# Rendering

Back to the [specification index](../README.md).

Markdown is shown formatted, not as raw source. The terminal supports GitHub Flavored
Markdown, the same format temoc reads.

## Inline and block elements

The `MarkdownRendering` fixture renders a piece of markdown by itself. `text?` is
what appears on screen, and `style?` is how it is emphasized. A `text?` of
`nothing` means that nothing is drawn.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable MarkdownRendering"

| # element       | markdown                             | text?        | style?    |
| --------------- | ------------------------------------ | ------------ | --------- |
| Heading         | `# Title`                            | Title        | heading 1 |
| Subheading      | `## Section`                         | Section      | heading 2 |
| Paragraph       | `Plain text`                         | Plain text   | plain     |
| Bold            | `**strong**`                         | strong       | bold      |
| Italic          | `*soft*`                             | soft         | italic    |
| Inline code     | `` `code` ``                         | code         | code      |
| Link            | `[Docs](docs.md)`                    | Docs         | link      |
| Bullet          | `- item`                             | • item       | plain     |
| Numbered item   | `1. first`                           | 1. first     | plain     |
| Quote           | `> wise words`                       | │ wise words | quote     |
| Temoc directive | `[//]: # "decisionTable Calculator"` | nothing      | none      |

## Tables and code blocks

Tables are drawn with their columns aligned, and code blocks keep their text
exactly as written. One wider than the screen is not wrapped:
[scroll it sideways](navigation.md#scrolling-sideways) to see the rest.

[//]: # "decisionTable Tui"

| # element        | workspace | arguments    | look for             | visible? |
| ---------------- | --------- | ------------ | -------------------- | -------- |
| Table cell       | docs      | reference.md | Opens a file         | yes      |
| Table separators | docs      | reference.md | \| ---               | no       |
| Code block       | docs      | reference.md | temoc-ui docs/       | yes      |
| Code fence       | docs      | reference.md | ```                  | no       |

## Long words

Text wraps between words, and never runs off the screen. A word too long for one row
starts on a row of its own and continues on the rows below, so that all of it can be
read. Text does not [scroll sideways](navigation.md#scrolling-sideways), and has no
markers.

[//]: # "decisionTable Tui"

| # element                | workspace   | arguments    | keys    | look for         | visible? |
| ------------------------ | ----------- | ------------ | ------- | ---------------- | -------- |
| The start of the word    | wide-blocks | long-word.md |         | wordstart        | yes      |
| The end of the word      | wide-blocks | long-word.md |         | wordend          | yes      |
| It starts on its own row | wide-blocks | long-word.md |         | before wordstart | no       |
| The text after it        | wide-blocks | long-word.md |         | and some text    | yes      |
| No marker                | wide-blocks | long-word.md |         | ›                | no       |
| It does not scroll       | wide-blocks | long-word.md | `Right` | wordstart        | yes      |

## Test files

Temoc directives stay hidden, and test tables appear like any other table.

[//]: # "decisionTable Tui"

| # element       | workspace  | arguments  | look for      | visible? |
| --------------- | ---------- | ---------- | ------------- | -------- |
| Test table      | calculator | passing.md | sum?          | yes      |
| Table directive | calculator | passing.md | decisionTable | no       |
| Import          | calculator | passing.md | Sample.Fixtures | no     |
