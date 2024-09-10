use std::cmp::Ordering;

use crate::line_buffer::{ChangeListener, DeleteListener, Direction};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Position {
    pub col: usize, // The leftmost column is number 0.
    pub row: usize, // The highest row is number 0.
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.row.cmp(&other.row) {
            Ordering::Equal => self.col.cmp(&other.col),
            o => o,
        }
    }
}

/// (byte offset in input buffer <=> Screen cell)
pub type Cell = (usize, Position);

/// All positions are relative to start of the prompt == origin (col: 0, row: 0)
#[derive(Debug, Default)]
pub struct Layout {
    /// Prompt Unicode/visible width and last row (relative)
    pub prompt_size: Position,
    pub default_prompt: bool,
    /// Cursor position (relative to the start of the prompt)
    /// - cursor.row >= prompt_size.row
    /// - if cursor.row >= prompt_size.row then cursor.col >=
    ///   prompt_size.col/width
    // FIXME
    pub cursor: Position,
    /// Number of rows used so far (from start of prompt to end
    /// of input or hint)
    /// - cursor <= end
    // FIXME
    pub end: Position,
    /// Input breaked into sorted cells
    // TODO ignore zero-width grapheme (even '\n') ?
    pub cells: Vec<Cell>,
}

impl Layout {
    /// Find the nearest byte / grapheme offset in input buffer
    /// matching `pos`
    fn find_byte_by(&self, pos: Position) -> &Cell {
        match self.cells.binary_search_by_key(&pos, |cell| cell.1) {
            Ok(i) => &self.cells[i],
            Err(i) => {
                if i < self.cells.len() {
                    &self.cells[i]
                } else {
                    todo!()
                }
            }
        }
    }

    /// Find the nearest cell in screen matching `offset`
    fn find_position_by(&self, offset: usize) -> &Cell {
        match self.cells.binary_search_by_key(&offset, |cell| cell.0) {
            Ok(i) => &self.cells[i],
            Err(i) => {
                if i < self.cells.len() {
                    &self.cells[i]
                } else {
                    todo!()
                }
            }
        }
    }
}

// TODO Update (cursor, end, cells) accordingly
// But we need to remember (old cursor/end row)
impl DeleteListener for Layout {
    fn delete(&mut self, idx: usize, string: &str, dir: Direction) {
        todo!()
    }
}
// TODO Update (cursor, end, cells) accordingly
// But we need to remember (old cursor/end row)
impl ChangeListener for Layout {
    fn insert_char(&mut self, idx: usize, c: char) {
        todo!()
    }

    fn insert_str(&mut self, idx: usize, string: &str) {
        todo!()
    }

    fn replace(&mut self, idx: usize, old: &str, new: &str) {
        todo!()
    }
}
