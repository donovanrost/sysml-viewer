use crate::{Position, SourceRange};

/// Precomputed character boundaries avoid repeated prefix scans for source maps.
pub struct SourceIndex {
    document_id: String,
    text: String,
    positions: Vec<Position>,
}

impl SourceIndex {
    pub fn new(document_id: &str, text: &str) -> Self {
        let mut positions = Vec::new();
        let (mut utf16, mut line, mut column) = (0, 1, 1);
        for (count, (byte, ch)) in text.char_indices().enumerate() {
            if count % 128 == 0 {
                positions.push(Position {
                    byte_offset: byte as u32,
                    utf16_offset: utf16,
                    line,
                    column,
                });
            }
            utf16 += ch.len_utf16() as u32;
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += ch.len_utf16() as u32;
            }
        }
        positions.push(Position {
            byte_offset: text.len() as u32,
            utf16_offset: utf16,
            line,
            column,
        });
        Self {
            document_id: document_id.into(),
            text: text.into(),
            positions,
        }
    }

    pub fn position(&self, byte_offset: u32) -> Position {
        let index = self
            .positions
            .partition_point(|p| p.byte_offset <= byte_offset)
            .saturating_sub(1);
        let mut position = self.positions[index].clone();
        for ch in self.text[position.byte_offset as usize..].chars() {
            let next = position.byte_offset + ch.len_utf8() as u32;
            if next > byte_offset {
                break;
            }
            position.byte_offset = next;
            position.utf16_offset += ch.len_utf16() as u32;
            if ch == '\n' {
                position.line += 1;
                position.column = 1;
            } else {
                position.column += ch.len_utf16() as u32;
            }
        }
        position
    }

    pub fn range(&self, start: u32, end: u32) -> SourceRange {
        SourceRange {
            document_id: self.document_id.clone(),
            start: self.position(start),
            end: self.position(end),
        }
    }
}
