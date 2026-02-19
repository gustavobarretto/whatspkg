//! Minimal encoder for the binary node format.
//! Writes nodes with string tag, string attrs, and content as bytes or list of child nodes.

use crate::binary::token;
use crate::Error;
use std::io::Write;

fn write_u8(w: &mut impl Write, v: u8) -> Result<(), Error> {
    w.write_all(&[v]).map_err(|e| Error::Binary(e.to_string()))
}

fn write_u16_be(w: &mut impl Write, v: u16) -> Result<(), Error> {
    w.write_all(&v.to_be_bytes())
        .map_err(|e| Error::Binary(e.to_string()))
}

fn write_u20_be(w: &mut impl Write, v: u32) -> Result<(), Error> {
    let b = [((v >> 16) & 0x0F) as u8, (v >> 8) as u8, v as u8];
    w.write_all(&b).map_err(|e| Error::Binary(e.to_string()))
}

fn write_string(w: &mut impl Write, s: &str) -> Result<(), Error> {
    let bytes = s.as_bytes();
    let len = bytes.len();
    if len <= u8::MAX as usize {
        write_u8(w, token::BINARY_8)?;
        write_u8(w, len as u8)?;
    } else if len <= 0x0F_FFFF {
        write_u8(w, token::BINARY_20)?;
        write_u20_be(w, len as u32)?;
    } else {
        return Err(Error::Binary("string too long for BINARY_20".into()));
    }
    w.write_all(bytes).map_err(|e| Error::Binary(e.to_string()))
}

fn write_bytes_with_len(w: &mut impl Write, b: &[u8]) -> Result<(), Error> {
    let len = b.len();
    if len <= u8::MAX as usize {
        write_u8(w, token::BINARY_8)?;
        write_u8(w, len as u8)?;
    } else if len <= 0x0F_FFFF {
        write_u8(w, token::BINARY_20)?;
        write_u20_be(w, len as u32)?;
    } else {
        return Err(Error::Binary("bytes too long for BINARY_20".into()));
    }
    w.write_all(b).map_err(|e| Error::Binary(e.to_string()))
}

/// Encode a node to the binary format.
pub fn encode_node(node: &super::Node, out: &mut Vec<u8>) -> Result<(), Error> {
    let attr_count = node.attrs.len();
    let has_content = !matches!(node.content, super::NodeContent::Empty);
    let list_size = 1 + 2 * attr_count + if has_content { 1 } else { 0 };

    if list_size <= u8::MAX as usize {
        write_u8(out, token::LIST_8)?;
        write_u8(out, list_size as u8)?;
    } else {
        write_u8(out, token::LIST_16)?;
        write_u16_be(out, list_size as u16)?;
    }

    write_string(out, &node.tag)?;
    for (k, v) in &node.attrs {
        write_string(out, k)?;
        write_string(out, v)?;
    }

    if has_content {
        encode_content(out, &node.content)?;
    }
    Ok(())
}

fn encode_content(out: &mut Vec<u8>, content: &super::NodeContent) -> Result<(), Error> {
    match content {
        super::NodeContent::Empty => {
            write_u8(out, token::LIST_EMPTY)?;
        }
        super::NodeContent::Bytes(b) => {
            write_bytes_with_len(out, b)?;
        }
        super::NodeContent::Nodes(children) => {
            let n = children.len();
            if n <= u8::MAX as usize {
                write_u8(out, token::LIST_8)?;
                write_u8(out, n as u8)?;
            } else {
                write_u8(out, token::LIST_16)?;
                write_u16_be(out, n as u16)?;
            }
            for child in children {
                encode_node(child, out)?;
            }
        }
    }
    Ok(())
}
