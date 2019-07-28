//! Redraw optimisation
/*
Idx // byte index
Prompt: String
Text: String // edited text
Cursor: Idx

String ->* Grapheme -> Width
           (Idx, Grapheme)
// we suppose that unicode width = rendered width

Position = (Column, Row) // relative row (prompt starts at row = 0)
Screen ->* Cell -> Position
Screen -> Size (columns, rows) // impact line wrapping
Span: Range<Position>

We want to cache the association between:
  Idx <-> Position
Maybe
  Vec<(Idx, Position)> or Vec<(Idx, Width, Position)>
If we keep elements sorted by Idx (which also implies by Row), we can perform a binary search.

Insert { idx, text }
  => all elements in cache with Idx >= idx must be updated
  new_idx = old_idx + text.len() if old_idx > idx
  (new_col, new_row) = shift_pos((old_col, old_row), text.width(), screen.columns)
  And new graphemes must be inserted
  And try to remember the range/span impacted (new_col <> old_col or new_row <> old_row)
Delete { idx, text }
  => all elements in cache with Idx >= idx must be updated
  old graphemes must be removed
  new_idx = old_idx - text.len() if old_idx >= idx
  (new_col, new_row) = shift_pos((old_col, old_row), -text.width(), screen.columns)
Replace {idx, old, new }
  Delete { idx, old } + Insert { idx, new }
  or optimize
*/
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Position on the screen
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Position {
    pub col: usize,
    pub row: usize, // Relative row (prompt starts at row = 0)
}

impl Position {
    fn shift_mut(&mut self, s: &Shift) {
        if s.width != 0 {
            self.col = shift(self.col, s.width, s.right);
        }
        if s.height != 0 {
            self.row = shift(self.row, s.height, s.down);
        }
    }

    #[allow(dead_code)]
    fn shift(&self, s: &Shift) -> Position {
        let mut new = *self;
        new.shift_mut(s);
        new
    }
}

/// Layout cache
#[allow(dead_code)]
pub struct LayoutCache {
    prompt: Vec<Entry>, // current prompt. prompt_size = prompt.last().pos
    text: Vec<Entry>,   /* edited text. cursor position = text.binary_search_by_key(cursor
                         * index, |e| e.idx) or text.last().pos + width */
    columns: usize, // number of columns in terminal screen
    //rows: usize,    // number of rows in terminal screen
    tab_stop: usize,
    dirty: bool, // `true` when `text` is not up to date
}

/// Layout cache entry
pub struct Entry {
    idx: usize, // grapheme byte position
    width: usize, /* grapheme width (current.pos.col + current.width) = next.pos.col when no
                 * line break */
    pos: Position, // (column, row) position on screen. Relative row (prompt starts at row = 0)
}

impl Entry {
    fn new(idx: usize, width: usize) -> Self {
        debug_assert!(width > 0); // zero width grapheme does not impact layout => should be ignored
        Entry {
            idx,
            width,
            pos: Position::default(),
        }
    }
}

/// Used to shift position
struct Shift {
    width: usize,
    right: bool,
    height: usize,
    down: bool,
}

const ZERO: Shift = Shift {
    width: 0,
    right: true,
    height: 0,
    down: true,
};

impl Shift {
    #[allow(dead_code)]
    fn right(width: usize, columns: usize) -> Self {
        if width == 0 {
            ZERO
        } else if width < columns {
            Shift {
                width,
                right: true,
                height: 0,
                down: true,
            }
        } else {
            // line wrapping
            Shift {
                width: width.checked_rem(columns).unwrap(),
                right: true,
                height: width.checked_div(columns).unwrap(),
                down: true,
            }
        }
    }

    fn delta(old: Position, new: Position) -> Self {
        if old == new {
            return ZERO;
        }
        let (width, right) = delta(old.col, new.col);
        let (height, down) = delta(old.row, new.row);
        Shift {
            width,
            right,
            height,
            down,
        }
    }

    fn is_empty(&self) -> bool {
        self.width == 0 && self.height == 0
    }
}

impl LayoutCache {
    #[allow(dead_code)]
    pub fn new(tab_stop: usize) -> Self {
        LayoutCache {
            prompt: Vec::new(),
            text: Vec::new(),
            columns: 0,
            //rows: 0,
            tab_stop,
            dirty: false,
        }
    }

    /// Compute the cursor position.
    #[allow(dead_code)]
    pub fn cursor_position(&self, cursor: usize) -> Position {
        if self.text.is_empty() || cursor == 0 {
            self.prompt_size()
        } else {
            match self.text.binary_search_by_key(&cursor, |e| e.idx) {
                Ok(i) => self.text[i].pos,
                Err(i) => {
                    if i == 0 {
                        // we don't store zero width grapheme
                        Position::default()
                    } else if i == self.text.len() {
                        // cursor at the end
                        let e = self.text.last().unwrap();
                        self.shift_entry(e)
                    } else {
                        // we don't store zero width grapheme
                        self.text[i].pos
                    }
                }
            }
        }
    }

    fn prompt_size(&self) -> Position {
        if let Some(e) = self.prompt.last() {
            let p = self.shift_entry(e);
            debug_assert!(p.col < self.columns);
            p
        } else {
            Position::default()
        }
    }
}

impl LayoutCache {
    fn shift_entry(&self, e: &Entry) -> Position {
        if e.width == 0 {
            return e.pos;
        }
        assert!(
            e.pos.col + e.width < self.columns,
            "Invalid entry, col: {} + width: {} >= {}",
            e.pos.col,
            e.width,
            self.columns
        );
        Position {
            col: e.pos.col + e.width,
            row: e.pos.row,
        }
    }

    /// Return `true` if `entry` position is modified.
    // TODO handle the case when a grapheme has width:2 at col:79 and columns:80
    fn update_entry(e: &mut Entry, shift: Shift, columns: usize) -> Shift {
        debug_assert!(shift.width < columns);
        let _ = e;
        unimplemented!()
    }
}

// TODO: handle the case where columns == 0 > impossible to paint
impl LayoutCache {
    // TODO: handle scrolling when self.rows > new rows and absolute(cursor row) >
    // new rows
    #[allow(dead_code)]
    pub fn window_resized(&mut self, columns: usize, _rows: usize) -> bool {
        if self.columns == columns {
            return false;
        }
        let mut shift = ZERO;
        for e in self.prompt.iter_mut().chain(self.text.iter_mut()) {
            shift = Self::update_entry(e, shift, columns);
        }
        self.columns = columns;
        !shift.is_empty()
    }

    #[allow(dead_code)]
    pub fn prompt_updated(&mut self, prompt: String) {
        let old_size = self.prompt_size();
        // TODO: optimize: diff old/new prompt
        self.prompt.clear();
        if self.columns > 0 {
            for (i, s) in prompt.grapheme_indices(true) {
                let width = s.width();
                if width == 0 {
                    continue;
                }
                self.prompt.push(Entry::new(i, width));
            }
        }
        let new_size = self.prompt_size();
        let shift = Shift::delta(old_size, new_size);
        if !shift.is_empty() {
            // shift self.text entries position
        }
        unimplemented!()
    }
}

/*
impl DeleteListener for LayoutCache {
    fn start_killing(&mut self) {
        unimplemented!()
    }

    fn delete(&mut self, idx: usize, string: &str, dir: Direction) {
        unimplemented!()
    }

    fn stop_killing(&mut self) {
        unimplemented!()
    }
}

impl ChangeListener for LayoutCache {
    fn insert_char(&mut self, idx: usize, c: char) {
        unimplemented!()
    }

    fn insert_str(&mut self, idx: usize, string: &str) {
        unimplemented!()
    }

    fn replace(&mut self, idx: usize, old: &str, new: &str) {
        unimplemented!()
    }
}
*/

fn shift(u: usize, shift: usize, plus: bool) -> usize {
    if shift == 0 {
        u
    } else if plus {
        u.checked_add(shift).unwrap()
    } else {
        u.checked_sub(shift).unwrap()
    }
}
fn delta(o: usize, n: usize) -> (usize, bool) {
    if n > o {
        (n - o, true)
    } else if n < o {
        (o - n, false)
    } else {
        (0, true)
    }
}
