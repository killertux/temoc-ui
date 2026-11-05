//! A parsed markdown document, before any interface shows it.

use crate::temoc::{self, TestFile};
use markdown::ParseOptions;
use markdown::mdast::Node;
use markdown::unist::Position;
use std::collections::HashMap;

/// A heading of a document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    /// The heading as plain text.
    pub text: String,
    /// What a link uses to point at this heading, as on GitHub.
    pub anchor: String,
    /// The line of the markdown it starts on.
    pub line: usize,
}

/// A parsed markdown document.
#[derive(Debug, Clone)]
pub struct Document {
    root: Node,
    targets: HashMap<usize, LinkTarget>,
}

/// What a link in the markdown says, by its offset in the markdown.
#[derive(Debug, Clone)]
pub struct LinkTarget {
    /// The text of the link, however an interface shows it.
    pub text: Vec<Node>,
    /// Where it points, as written in the markdown.
    pub url: String,
}

impl Document {
    /// Parses markdown the same way temoc does.
    ///
    /// # Errors
    ///
    /// Fails when the markdown cannot be parsed.
    pub fn parse(markdown: &str) -> Result<Self, String> {
        markdown::to_mdast(markdown, &ParseOptions::gfm())
            .map(|root| Self {
                targets: link_targets(&root),
                root,
            })
            .map_err(|message| message.to_string())
    }

    /// What temoc would find in this document.
    #[must_use]
    pub fn tests(&self) -> TestFile {
        temoc::tests_in(&self.root)
    }

    /// The top-level headings, in document order.
    #[must_use]
    pub fn headings(&self) -> Vec<Heading> {
        let mut seen: HashMap<String, usize> = HashMap::new();
        children(&self.root)
            .iter()
            .filter(|node| matches!(node, Node::Heading(_)))
            .map(|node| {
                let text = node.to_string();
                let slug = slug(&text);
                let count = seen.entry(slug.clone()).or_default();
                let anchor = match *count {
                    0 => slug,
                    count => format!("{slug}-{count}"),
                };
                *count += 1;
                Heading {
                    text,
                    anchor,
                    line: line_of(node),
                }
            })
            .collect()
    }

    /// Where the links of the document point, as written in the markdown.
    pub fn link_urls(&self) -> impl Iterator<Item = &str> {
        self.targets.values().map(|target| target.url.as_str())
    }

    /// The root of the markdown.
    #[must_use]
    pub fn root(&self) -> &Node {
        &self.root
    }

    /// What each link says, by the offset of the link in the markdown.
    #[must_use]
    pub fn targets(&self) -> &HashMap<usize, LinkTarget> {
        &self.targets
    }
}

/// Finds what each link says. Links can use a definition written anywhere in
/// the document.
fn link_targets(root: &Node) -> HashMap<usize, LinkTarget> {
    let mut definitions = HashMap::new();
    walk(root, &mut |node| {
        if let Node::Definition(definition) = node {
            definitions
                .entry(definition.identifier.as_str())
                .or_insert(definition.url.as_str());
        }
    });
    let mut targets = HashMap::new();
    walk(root, &mut |node| {
        let found = match node {
            Node::Link(link) => Some((&link.position, &link.children, link.url.as_str())),
            Node::LinkReference(link) => definitions
                .get(link.identifier.as_str())
                .map(|url| (&link.position, &link.children, *url)),
            _ => None,
        };
        if let Some((position, children, url)) = found {
            let target = LinkTarget {
                text: children.clone(),
                url: url.to_owned(),
            };
            targets.insert(link_id(position.as_ref()), target);
        }
    });
    targets
}

fn walk<'node>(node: &'node Node, visit: &mut impl FnMut(&'node Node)) {
    visit(node);
    for child in children(node) {
        walk(child, visit);
    }
}

/// How a link is told apart from the others: its offset in the markdown.
#[must_use]
pub fn link_id(position: Option<&Position>) -> usize {
    position.map_or(0, |position| position.start.offset)
}

/// The line of the markdown a node starts on.
#[must_use]
pub fn line_of(node: &Node) -> usize {
    node.position().map_or(0, |position| position.start.line)
}

/// The anchor GitHub gives a heading with this text.
fn slug(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter_map(|c| match c {
            ' ' => Some('-'),
            c if c.is_alphanumeric() || c == '-' || c == '_' => Some(c),
            _ => None,
        })
        .collect()
}

/// The nodes inside a node.
#[must_use]
pub fn children(node: &Node) -> &[Node] {
    node.children().map_or(&[], Vec::as_slice)
}
