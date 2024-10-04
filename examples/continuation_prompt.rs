use std::borrow::Cow;

use rustyline::highlight::Highlighter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Cmd, Editor, EventHandler, Helper, KeyCode, KeyEvent, Modifiers, Result};
use rustyline::{Completer, Hinter};

#[derive(Completer, Hinter)]
struct InputValidator {
    bracket_level: i32,
    /// re-render only when input just changed
    /// not render after cursor moving
    need_render: bool,
}

impl Helper for InputValidator {
    fn update_after_edit(&mut self, line: &str, _pos: usize, _forced_refresh: bool) {
        self.bracket_level = line.chars().fold(0, |level, c| {
            if c == '(' {
                level + 1
            } else if c == ')' {
                level - 1
            } else {
                level
            }
        });
        self.need_render = true;
    }
}

impl Validator for InputValidator {
    fn validate(&mut self, _ctx: &mut ValidationContext) -> Result<ValidationResult> {
        if self.bracket_level > 0 {
            Ok(ValidationResult::Incomplete)
            // Ok(ValidationResult::Incomplete(2))
        } else if self.bracket_level < 0 {
            Ok(ValidationResult::Invalid(Some(format!(
                " - excess {} close bracket",
                -self.bracket_level
            ))))
        } else {
            Ok(ValidationResult::Valid(None))
        }
    }
}

impl Highlighter for InputValidator {
    fn highlight_char(&mut self, _line: &str, _pos: usize, _forced: bool) -> bool {
        self.need_render
    }
    fn highlight<'l>(&mut self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.need_render = false;
        Cow::Borrowed(&line)
    }
}

fn main() -> Result<()> {
    let h = InputValidator {
        bracket_level: 0,
        need_render: true,
    };
    let mut rl = Editor::new()?;
    rl.set_helper(Some(h));
    rl.bind_sequence(
        KeyEvent(KeyCode::Char('s'), Modifiers::CTRL),
        EventHandler::Simple(Cmd::Newline),
    );

    let input = rl.readline(">> ")?;
    println!("Input: {input}");

    Ok(())
}
