# Reloading

Back to the [specification index](README.md).

temoc-ui keeps up with the files of the workspace as they change on disk, whatever
changes them: an editor, `git`, or a script. Nothing has to be reopened. temoc-ui
notices a change within a second, without being asked, and when a file changes several
times in quick succession it ends up showing the last version.

The [`changing`](../samples/changing/README.md) workspace has pages for this, and a
hidden `.versions` folder holding what the steps write over them. The steps that change
files are described in [how these pages work](writing-specs.md#changing-files).

## The open document

The open document shows what its file holds now. It stays the open document, and
history is untouched. Many editors save by writing a new file and renaming it over the
old one, and that counts as a change too.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Core"

| # situation          | workspace | path | steps                                            | document? | headings?              | links out?       |
| -------------------- | --------- | ---- | ------------------------------------------------ | --------- | ---------------------- | ---------------- |
| Before a change      | changing  |      |                                                  | README.md | Changing, Pages        | long.md, page.md |
| Written over         | changing  |      | `write README.md from readme-fewer-links.md`     | README.md | Changing, Pages, Added | long.md          |
| Saved like an editor | changing  |      | `replace README.md with readme-fewer-links.md`   | README.md | Changing, Pages, Added | long.md          |
| Another file changed | changing  |      | `write page.md from page-moved.md`               | README.md | Changing, Pages        | long.md, page.md |

A page you go back or forward to shows what its file holds when you get there.

[//]: # "decisionTable Core"

| # situation   | workspace | path | steps                                                            | document? | headings?              |
| ------------- | --------- | ---- | ---------------------------------------------------------------- | --------- | ---------------------- |
| Back to it    | changing  |      | `open page.md, write README.md from readme-fewer-links.md, back` | README.md | Changing, Pages, Added |
| Forward to it | changing  |      | `open page.md, back, write page.md from new.md, forward`         | page.md   | New                    |

## Deleted files

When the file of the open document is deleted, the page stays as it was last read, so
that you do not lose your place. It says that its file is gone, and it cannot run. If
the file comes back, the page reloads as if it had changed.

A file renamed to another name counts as deleted. temoc-ui does not follow it to its
new name, which shows up as a new file.

[//]: # "decisionTable Core"

| # situation  | workspace | path    | steps                                             | document? | headings?                      | problem?              |
| ------------ | --------- | ------- | ------------------------------------------------- | --------- | ------------------------------ | --------------------- |
| Deleted      | changing  | page.md | `delete page.md`                                  | page.md   | Page, Addition, Multiplication | File deleted: page.md |
| Cannot run   | changing  | page.md | `delete page.md, run file`                        | page.md   | Page, Addition, Multiplication | File deleted: page.md |
| Renamed      | changing  | page.md | `rename page.md to moved.md`                      | page.md   | Page, Addition, Multiplication | File deleted: page.md |
| Comes back   | changing  | page.md | `delete page.md, write page.md from new.md`       | page.md   | New                            | none                  |

Going back or forward to a file that was deleted is refused, as following a link to a
missing file is, and you stay where you are.

[//]: # "decisionTable Core"

| # situation         | workspace | path | steps                                                 | document? | problem?              |
| ------------------- | --------- | ---- | ----------------------------------------------------- | --------- | --------------------- |
| Forward to deleted  | changing  |      | `open page.md, back, delete page.md, forward`         | README.md | File deleted: page.md |
| Back to deleted     | changing  |      | `open page.md, open long.md, delete page.md, back`    | long.md   | File deleted: page.md |

## Search, the file tree, and the graph

Search, the file tree and the graph show the workspace as it is now. A new file is
there, a deleted one is gone, and so are its headings and links. Search results that
are showing update in place.

[//]: # "decisionTable Core"

| # situation              | workspace | steps                                                         | results?                                         |
| ------------------------ | --------- | ------------------------------------------------------------- | ------------------------------------------------ |
| Before a change          | changing  | `search`                                                      | hanging.md, long.md, page.md, README.md          |
| A new file               | changing  | `write new.md from new.md, search`                            | hanging.md, long.md, new.md, page.md, README.md  |
| A deleted file           | changing  | `delete long.md, search`                                      | hanging.md, page.md, README.md                   |
| A renamed file           | changing  | `rename long.md to longer.md, search`                         | hanging.md, longer.md, page.md, README.md        |
| A new heading            | changing  | `write README.md from readme-fewer-links.md, search added`    | README.md > Added                                |
| While search is showing  | changing  | `search, write new.md from new.md`                            | hanging.md, long.md, new.md, page.md, README.md  |
| No longer a test file    | changing  | `search, only test files, write page.md from page-no-tests.md` | hanging.md                                      |

[//]: # "decisionTable Core"

| # situation     | workspace | steps                                        | tree?                                           | links?                                     |
| --------------- | --------- | -------------------------------------------- | ----------------------------------------------- | ------------------------------------------ |
| Before a change | changing  |                                              | hanging.md, long.md, page.md, README.md         | README.md -> long.md, README.md -> page.md |
| A new folder    | changing  | `write notes/new.md from new.md`             | notes/, hanging.md, long.md, page.md, README.md | README.md -> long.md, README.md -> page.md |
| A deleted file  | changing  | `delete long.md`                             | hanging.md, page.md, README.md                  | README.md -> page.md                       |
| A changed link  | changing  | `write README.md from readme-fewer-links.md` | hanging.md, long.md, page.md, README.md         | README.md -> long.md                       |

## Test results

A result belongs to the text that produced it. When a file changes, a table that is
still there, with the same name and the same text from its directive to its last row,
keeps its results, and they move with it to its new lines. Every other table loses its
results, and the file result is worked out again from what is left. This is the same
whether the file is open or not.

[//]: # "decisionTable Core"

| # situation            | workspace | path    | steps                                                              | row results?                       | table results?                            | file result? |
| ---------------------- | --------- | ------- | ------------------------------------------------------------------ | ---------------------------------- | ----------------------------------------- | ------------ |
| Lines moved            | changing  | page.md | `run file, write page.md from page-moved.md`                       | 13: passed, 21: passed, 22: failed | Addition: passed, Multiplication: failed  | failed       |
| A table changed        | changing  | page.md | `run file, write page.md from page-fixed.md`                       | 11: passed                         | Addition: passed, Multiplication: not run | partial      |
| Tables gone            | changing  | page.md | `run file, write page.md from page-no-tests.md`                    | none                               | none                                      | none         |
| While another is open  | changing  | page.md | `run file, open README.md, write page.md from page-moved.md, back` | 13: passed, 21: passed, 22: failed | Addition: passed, Multiplication: failed  | failed       |
| Deleted and back again | changing  | page.md | `run file, delete page.md, write page.md from page-moved.md`       | 13: passed, 21: passed, 22: failed | Addition: passed, Multiplication: failed  | failed       |

Failures move with their rows.

[//]: # "decisionTable Core"

| # situation | workspace | path    | steps                                        | failures?                                     |
| ----------- | --------- | ------- | -------------------------------------------- | --------------------------------------------- |
| Lines moved | changing  | page.md | `run file, write page.md from page-moved.md` | Multiplication line 22: mul expected 5, got 4 |

Running after a change runs what the file holds now, and lines are those of the new
version.

[//]: # "decisionTable Core"

| # situation       | workspace | path    | steps                                        | row results?                       | temoc arguments?                            |
| ----------------- | --------- | ------- | -------------------------------------------- | ---------------------------------- | ------------------------------------------- |
| The new content   | changing  | page.md | `write page.md from page-fixed.md, run file` | 11: passed, 19: passed, 20: passed | -c temoc.toml --output jsonl page.md        |
| The new lines     | changing  | page.md | `write page.md from page-moved.md, run row 22` | 22: failed                       | -c temoc.toml --output jsonl page.md:22     |

## While a run goes on

temoc read the file when the run started, so the results still to come describe the
tables as they were then. They keep arriving, and go where the rule above says: a table
that did not change gets them on its new lines, and a table that changed gets none.
The run itself goes on until it ends.

`hanging.md` holds its run at line 13, as in [Row results](running/row-results.md#while-a-run-goes-on).

[//]: # "decisionTable Core"

| # situation          | workspace | path       | steps                                                                        | row results?                        | table results?                    | running?   |
| -------------------- | --------- | ---------- | ---------------------------------------------------------------------------- | ----------------------------------- | --------------------------------- | ---------- |
| Lines moved          | changing  | hanging.md | `run file, wait for row 13, write hanging.md from hanging-moved.md`          | 13: passed, 14: failed, 15: running | Stuck: running, Waiting: not run  | hanging.md |
| The running table    | changing  | hanging.md | `run file, wait for row 13, write hanging.md from hanging-stuck-changed.md`  | none                                | Stuck: not run, Waiting: not run  | hanging.md |
