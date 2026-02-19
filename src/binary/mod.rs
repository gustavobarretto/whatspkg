//! Binary protocol nodes for the WhatsApp Web protocol.
//! Custom binary XML-like node format over the Noise socket.

mod consts;
mod decoder;
mod encoder;
mod token;

pub use consts::{NOISE_START_PATTERN, WA_CONN_HEADER, WA_MAGIC_VALUE};
use std::collections::HashMap;

/// Attributes on a node (key-value; values are strings).
pub type Attrs = HashMap<String, String>;

/// Content of a node: either child nodes or raw bytes.
#[derive(Clone, Debug, Default)]
pub enum NodeContent {
    #[default]
    Empty,
    Nodes(Vec<Node>),
    Bytes(Vec<u8>),
}

/// A single binary protocol node.
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

    /// Encode to binary form (LIST_8/16 + BINARY_8/20 for strings; no dictionary tokens).
    pub fn encode(&self) -> crate::Result<Vec<u8>> {
        let mut out = Vec::new();
        encoder::encode_node(self, &mut out)?;
        Ok(out)
    }

    /// Decode a single node from binary form. Expects data to start with a list tag (LIST_8 or LIST_16).
    pub fn decode(data: &[u8]) -> crate::Result<Self> {
        decoder::decode(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip_empty() {
        let n = Node::new("iq");
        let data = n.encode().unwrap();
        let decoded = Node::decode(&data).unwrap();
        assert_eq!(decoded.tag, "iq");
        assert!(decoded.attrs.is_empty());
        assert!(matches!(decoded.content, NodeContent::Empty));
    }

    #[test]
    fn encode_decode_roundtrip_with_attrs() {
        let n = Node::new("iq")
            .with_attr("id", "1")
            .with_attr("type", "get");
        let data = n.encode().unwrap();
        let decoded = Node::decode(&data).unwrap();
        assert_eq!(decoded.tag, "iq");
        assert_eq!(decoded.attrs.get("id").map(String::as_str), Some("1"));
        assert_eq!(decoded.attrs.get("type").map(String::as_str), Some("get"));
        assert!(matches!(decoded.content, NodeContent::Empty));
    }

    #[test]
    fn encode_decode_roundtrip_with_children() {
        let child = Node::new("item").with_attr("jid", "123@s.whatsapp.net");
        let n = Node::new("list").with_children(vec![child]);
        let data = n.encode().unwrap();
        let decoded = Node::decode(&data).unwrap();
        assert_eq!(decoded.tag, "list");
        let nodes = decoded.get_children();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].tag, "item");
        assert_eq!(
            nodes[0].attrs.get("jid").map(String::as_str),
            Some("123@s.whatsapp.net")
        );
    }

    #[test]
    fn encode_decode_roundtrip_bytes() {
        let n = Node::new("payload").with_content(b"hello binary".to_vec());
        let data = n.encode().unwrap();
        let decoded = Node::decode(&data).unwrap();
        assert_eq!(decoded.tag, "payload");
        match &decoded.content {
            NodeContent::Bytes(b) => assert_eq!(b.as_slice(), b"hello binary"),
            _ => panic!("expected Bytes content"),
        }
    }
}
