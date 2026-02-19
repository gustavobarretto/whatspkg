//! Minimal decoder for the binary node format.
//! Supports nodes with string tag, string attrs, and content as bytes or list of child nodes.

use crate::binary::token;
use crate::Error;
use std::collections::HashMap;

fn check_eos(data: &[u8], position: usize, len: usize) -> crate::Result<()> {
    if position + len <= data.len() {
        Ok(())
    } else {
        Err(Error::Binary("unexpected eof".into()))
    }
}

/// Decodes binary protocol nodes (no dictionary tokens; strings as BINARY_8/BINARY_20).
pub fn decode(data: &[u8]) -> crate::Result<super::Node> {
    let mut d = Decoder::new(data);
    d.read_node()
}

pub(super) struct Decoder<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Decoder<'a> {
    pub(super) fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn read_u8(&mut self) -> crate::Result<u8> {
        check_eos(self.data, self.pos, 1)?;
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    fn read_u16_be(&mut self) -> crate::Result<u16> {
        check_eos(self.data, self.pos, 2)?;
        let b = &self.data[self.pos..self.pos + 2];
        self.pos += 2;
        Ok(u16::from_be_bytes([b[0], b[1]]))
    }

    fn read_u20_be(&mut self) -> crate::Result<u32> {
        check_eos(self.data, self.pos, 3)?;
        let b = &self.data[self.pos..self.pos + 3];
        self.pos += 3;
        Ok(((b[0] as u32 & 0x0F) << 16) | ((b[1] as u32) << 8) | (b[2] as u32))
    }

    fn read_bytes(&mut self, len: usize) -> crate::Result<Vec<u8>> {
        check_eos(self.data, self.pos, len)?;
        let out = self.data[self.pos..self.pos + len].to_vec();
        self.pos += len;
        Ok(out)
    }

    fn read_string(&mut self) -> crate::Result<String> {
        let tag = self.read_u8()?;
        match tag {
            token::LIST_EMPTY => Ok(String::new()),
            token::BINARY_8 => {
                let len = self.read_u8()? as usize;
                let bytes = self.read_bytes(len)?;
                String::from_utf8(bytes).map_err(|e| Error::Binary(e.to_string()))
            }
            token::BINARY_20 => {
                let len = self.read_u20_be()? as usize;
                let bytes = self.read_bytes(len)?;
                String::from_utf8(bytes).map_err(|e| Error::Binary(e.to_string()))
            }
            _ => Err(Error::Binary(format!("unsupported string token {}", tag))),
        }
    }

    fn read_list_size(&mut self, list_tag: u8) -> crate::Result<usize> {
        match list_tag {
            token::LIST_8 => Ok(self.read_u8()? as usize),
            token::LIST_16 => Ok(self.read_u16_be()? as usize),
            _ => Err(Error::Binary(format!(
                "unsupported list token {}",
                list_tag
            ))),
        }
    }

    fn read_node(&mut self) -> crate::Result<super::Node> {
        let list_tag = self.read_u8()?;
        let list_size = self.read_list_size(list_tag)?;
        if list_size == 0 {
            return Err(Error::Binary("empty list size for node".into()));
        }
        let tag = self.read_string()?;
        let attr_count = (list_size - 1) / 2;
        let has_content = (list_size % 2) == 0;

        let mut attrs = HashMap::new();
        for _ in 0..attr_count {
            let k = self.read_string()?;
            let v = self.read_string()?;
            attrs.insert(k, v);
        }

        let content = if has_content {
            self.read_content()?
        } else {
            super::NodeContent::Empty
        };

        Ok(super::Node {
            tag,
            attrs,
            content,
        })
    }

    fn read_content(&mut self) -> crate::Result<super::NodeContent> {
        let tag = self.read_u8()?;
        match tag {
            token::LIST_EMPTY => Ok(super::NodeContent::Empty),
            token::BINARY_8 => {
                let len = self.read_u8()? as usize;
                Ok(super::NodeContent::Bytes(self.read_bytes(len)?))
            }
            token::BINARY_20 => {
                let len = self.read_u20_be()? as usize;
                Ok(super::NodeContent::Bytes(self.read_bytes(len)?))
            }
            token::LIST_8 | token::LIST_16 => {
                let n = self.read_list_size(tag)?;
                let mut children = Vec::with_capacity(n);
                for _ in 0..n {
                    children.push(self.read_node()?);
                }
                Ok(super::NodeContent::Nodes(children))
            }
            _ => Err(Error::Binary(format!("unsupported content token {}", tag))),
        }
    }
}
