# Design

Back to [how the GUI pages work](../writing-specs.md).

These pictures show how the GUI looks. The [pages next to this folder](../writing-specs.md)
say how it behaves, and their tables are what is tested. Nothing here is tested, so when
a picture and a table disagree, the table is right and the picture is out of date.

The pictures of [the window](#the-window) are screenshots of temoc-ui itself. The
[reference](#reference) boards are drawn from the design canvas. The source of each one is
in [`canvas/`](canvas). The `.dc.html` files only render on the canvas, so use the pictures
here to read the design, and the
[canvas](https://claude.ai/artifact/J27Jmv6xTmwGsbUiBehfFj), if you have access to it, to
click through the prototype or change it. After changing the canvas, copy its files here
and draw the reference boards again.

## The window

| Picture                                      | Shows                                                        | Prototype                                        |
| -------------------------------------------- | ------------------------------------------------------------ | ------------------------------------------------ |
| [After a run](images/main.png)               | The window on a test page, with its results.                 | [Main](canvas/Main.dc.html)                      |
| [A run in progress](images/running.png)      | A row running, and the run buttons dimmed.                   | [Running](canvas/Running.dc.html)                |
| [Every kind of failure](images/results.png)  | Errors, a skipped row, and a failure outside any row.        | [Results](canvas/Results.dc.html)                |
| [Hovering a row](images/row-hover.png)       | The Run row button, in place of the mark.                    | [RowHover](canvas/RowHover.dc.html)              |
| [A page without tests](images/reader.png)    | Reading, without the tests panel.                            | [Reader](canvas/Reader.dc.html)                  |
| [Search](images/search.png)                  | The search field, with the results under it.                 | [Search](canvas/Search.dc.html)                  |
| [Graph](images/graph.png)                    | The graph of this `spec/` folder.                            | [Graph](canvas/Graph.dc.html)                    |
| [Graph after a run](images/graph-results.png) | Results on the graph, and files that cannot be reached.     | [GraphResults](canvas/GraphResults.dc.html)      |
| [Light mode](images/light.png)               | The light colours.                                           | [Light](canvas/Light.dc.html)                    |
| [Help](images/help.png)                      | The list of keys.                                            | [Help](canvas/Help.dc.html)                      |
| [No configuration](images/no-config.png)     | The banner on a test page that cannot run.                   | [NoConfig](canvas/NoConfig.dc.html)              |
| [temoc could not finish](images/failed.png)  | The banner after a run that stopped.                         | [Failed](canvas/Failed.dc.html)                  |

## Reference

| Picture                                     | Shows                                                             | Source                                  |
| ------------------------------------------- | ----------------------------------------------------------------- | --------------------------------------- |
| [Markdown elements](images/markdown.png)    | Every markdown element, with how it is built in egui.             | [Markdown](canvas/Markdown.dc.html)     |
| [Tokens and components](images/tokens.png)  | Colours, type, row states, graph nodes, and each part in egui.    | [System](canvas/System.dc.html)         |
| [Work list](images/work-list.png)           | Each terminal feature, and what happens to it in the GUI.         | [Parity](canvas/Parity.dc.html)         |
