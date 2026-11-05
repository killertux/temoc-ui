# Glossary

Back to the [specification index](README.md).

**Workspace.** The folder temoc-ui is working in. Search lists the markdown files
inside it, and `temoc.toml` is looked up at its root. See
[Opening a workspace](viewer/opening.md).

**Document.** The markdown file currently open.

**Directive.** A hidden markdown line that gives temoc an instruction, such as
`[//]: # "decisionTable Calculator"`.

**Test file.** A markdown file with at least one `decisionTable` directive. See
[Finding tests](running/finding-tests.md).

**Test table.** A markdown table that follows a `decisionTable` directive. Each
row is one test case. The table's name is the nearest heading above it.

**Fixture.** Code that connects a test table to the software being tested.

**Slim server.** The process that hosts fixtures. temoc starts it and talks to it
using the Slim protocol.

**Snoozed.** A table marked `-- snooze until <date>`. Until that date, its failures
are reported but do not fail the run.

**Core.** What temoc-ui does whatever its interface is.

**Interface.** A way of using temoc-ui, such as the terminal or the GUI. Each one has
its own folder in the specification.

**Focused link**, **focused table** and **focused row.** The link, test table or row of
that table that keys act on.

**Selection mode.** In the terminal, the state entered with `v`, where a cursor moves
through the text. See [Selecting text](tui/selecting-text.md).
