//! Prompt and line continuations
use std::borrow::Cow::{self, Borrowed};

/// Prompt and line continuations
pub trait Prompt {
    /// Returns text to be shown as the prompt at the first line.
    /// Or text for the next lines of the input when
    /// `Prompt::has_continuation()`.
    fn get_prompt<'p>(&'p self, ctx: &dyn PromptContext) -> Cow<'p, str>;
    /// Returns `true` when line continuations should be displayed. `false` by
    /// default.
    fn has_continuation(&self) -> bool {
        false
    }
}

/*
We can cache Prompt::get_prompt result(s) if:
 * if `input_mode` is kept untouched
 * or (if `Prompt::has_continuation`), if `wrap_count`/`line_number` are kept untouched

TODO
* Layout impacts
 - compute_layout
 - State.prompt_size
 - ...
* Highlight impacts
 - highlight_prompt
 */

impl PromptContext for () {}

pub trait PromptContext {
    // Current line number. Computed/meaningful only when
    // `Prompt::has_continuation()`. fn line_number(&self) -> usize;
    // Soft wrap count. Computed/meaningful only when `Prompt::has_continuation()`.
    //fn wrap_count(&self) -> usize;
    // Vi input mode. `None` with default emacs editing mode.
    //fn input_mode(&self) -> Option<InputMode>;
}

impl Prompt for str {
    fn get_prompt<'p>(&'p self, _: &dyn PromptContext) -> Cow<'p, str> {
        Borrowed(self)
    }
}
