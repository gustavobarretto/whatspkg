//! Binary protocol nodes (whatsmeow binary package).
//! WhatsApp uses a custom binary XML-like node format over the Noise socket.

use std::collections::HashMap;

/// Attributes on a node (key-value; values can be string, int, etc. in Go; we use string for simplicity).
pub type Attrs = HashMap<String, String>;

/// Content of a node: either child nodes or raw bytes.
#[derive(Clone, Debug, Default)]
pub enum NodeContent {
    #[default]
    Empty,
    Nodes(Vec<Node>),
    Bytes(Vec<u8>),
}

/// A single binary protocol node (mirrors waBinary.Node).
#[derive(Clone, Debug, Default)]
pub struct Node {
    pub tag: String,
    pub attrs: Attrs,
    pub content: NodeContent,
}

impl Node {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            attrs: Attrs::new(),
            content: NodeContent::Empty,
        }
    }

    pub fn with_attr(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.attrs.insert(k.into(), v.into());
        self
    }

    pub fn with_children(mut self, children: Vec<Node>) -> Self {
        self.content = NodeContent::Nodes(children);
        self
    }

    pub fn with_content(mut self, bytes: Vec<u8>) -> Self {
        self.content = NodeContent::Bytes(bytes);
        self
    }

    pub fn get_child_by_tag(&self, tag: &str) -> Option<&Node> {
        match &self.content {
            NodeContent::Nodes(nodes) => nodes.iter().find(|n| n.tag == tag),
            _ => None,
        }
    }

    pub fn get_children(&self) -> &[Node] {
        match &self.content {
            NodeContent::Nodes(n) => n,
            _ => &[],
        }
    }

    /// Encode to binary form (whatsmeow binary encoding). Stub: real impl would match Go binary.Writer.
    pub fn encode(&self) -> crate::Result<Vec<u8>> {
        // TODO: implement binary encoding to match WhatsApp format
        Err(crate::Error::Binary("encode not yet implemented".into()))
    }

    /// Decode from binary form. Stub.
    pub fn decode(data: &[u8]) -> crate::Result<Self> {
        let _ = data;
        Err(crate::Error::Binary("decode not yet implemented".into()))
    }
}
