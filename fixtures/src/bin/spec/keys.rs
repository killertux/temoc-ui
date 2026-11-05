//! The keys notation described in `spec/tui/writing-specs.md`.

use crossterm::event::{KeyCode, KeyModifiers, MouseEventKind};

/// One action written in a `keys` column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    Key(KeyCode, KeyModifiers),
    Text(String),
    Click(String),
    /// Press on the first character of `from`, release on the last of `to`.
    Drag {
        from: String,
        to: String,
    },
    /// Press on the last character of `from`, release on the first of `to`.
    DragBack {
        from: String,
        to: String,
    },
    Wheel(MouseEventKind, KeyModifiers),
    ClickOutside,
    /// Stop waiting for the run, and wait until the text is on screen.
    Wait(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Word(String),
    Quoted(String),
}

/// Parses a `keys` cell.
pub fn parse(keys: &str) -> Result<Vec<Input>, String> {
    let mut tokens = tokenize(keys)?.into_iter();
    let mut inputs = Vec::new();
    while let Some(token) = tokens.next() {
        inputs.push(match token {
            Token::Quoted(text) => Input::Text(text),
            Token::Word(word) => match word.as_str() {
                "Click" => Input::Click(quoted(&mut tokens, "Click")?),
                "Drag" => Input::Drag {
                    from: quoted(&mut tokens, "Drag")?,
                    to: quoted(&mut tokens, "Drag")?,
                },
                "DragBack" => Input::DragBack {
                    from: quoted(&mut tokens, "DragBack")?,
                    to: quoted(&mut tokens, "DragBack")?,
                },
                "ClickOutside" => Input::ClickOutside,
                "Wait" => Input::Wait(quoted(&mut tokens, "Wait")?),
                _ => wheel(&word)
                    .or_else(|| key(&word))
                    .ok_or_else(|| format!("unknown key `{word}`"))?,
            },
        });
    }
    Ok(inputs)
}

fn quoted(tokens: &mut impl Iterator<Item = Token>, action: &str) -> Result<String, String> {
    match tokens.next() {
        Some(Token::Quoted(text)) => Ok(text),
        _ => Err(format!("`{action}` needs quoted text")),
    }
}

fn tokenize(keys: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut characters = keys.chars().peekable();
    while let Some(&character) = characters.peek() {
        if character.is_whitespace() {
            characters.next();
        } else if character == '"' {
            characters.next();
            let text: String = characters.by_ref().take_while(|c| *c != '"').collect();
            tokens.push(Token::Quoted(text));
        } else {
            let mut word = String::new();
            while let Some(c) = characters.next_if(|c| !c.is_whitespace()) {
                word.push(c);
            }
            tokens.push(Token::Word(word));
        }
    }
    if keys.chars().filter(|c| *c == '"').count() % 2 == 1 {
        return Err(format!("unbalanced quotes in `{keys}`"));
    }
    Ok(tokens)
}

fn wheel(word: &str) -> Option<Input> {
    let (kind, modifiers) = match word {
        "WheelUp" => (MouseEventKind::ScrollUp, KeyModifiers::NONE),
        "WheelDown" => (MouseEventKind::ScrollDown, KeyModifiers::NONE),
        "WheelLeft" => (MouseEventKind::ScrollLeft, KeyModifiers::NONE),
        "WheelRight" => (MouseEventKind::ScrollRight, KeyModifiers::NONE),
        "Shift-WheelUp" => (MouseEventKind::ScrollUp, KeyModifiers::SHIFT),
        "Shift-WheelDown" => (MouseEventKind::ScrollDown, KeyModifiers::SHIFT),
        _ => return None,
    };
    Some(Input::Wheel(kind, modifiers))
}

fn key(word: &str) -> Option<Input> {
    let named = |code| Some(Input::Key(code, KeyModifiers::NONE));
    match word {
        "Enter" => named(KeyCode::Enter),
        "Esc" => named(KeyCode::Esc),
        "Tab" => named(KeyCode::Tab),
        "Backspace" => named(KeyCode::Backspace),
        "PageUp" => named(KeyCode::PageUp),
        "PageDown" => named(KeyCode::PageDown),
        "Up" => named(KeyCode::Up),
        "Down" => named(KeyCode::Down),
        "Left" => named(KeyCode::Left),
        "Right" => named(KeyCode::Right),
        "Space" => named(KeyCode::Char(' ')),
        "Shift-Tab" => Some(Input::Key(KeyCode::BackTab, KeyModifiers::SHIFT)),
        _ => word
            .strip_prefix("Ctrl-")
            .and_then(single)
            .map(|c| Input::Key(KeyCode::Char(c), KeyModifiers::CONTROL))
            .or_else(|| single(word).map(character)),
    }
}

fn single(word: &str) -> Option<char> {
    let mut characters = word.chars();
    let first = characters.next()?;
    characters.next().is_none().then_some(first)
}

/// The key press a terminal reports for a typed character.
pub fn character(c: char) -> Input {
    let modifiers = if c.is_uppercase() {
        KeyModifiers::SHIFT
    } else {
        KeyModifiers::NONE
    };
    Input::Key(KeyCode::Char(c), modifiers)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(c: char) -> Input {
        Input::Key(KeyCode::Char(c), KeyModifiers::NONE)
    }

    #[test]
    fn empty_cell_has_no_inputs() {
        assert_eq!(parse("").unwrap(), vec![]);
    }

    #[test]
    fn single_keys_include_punctuation_and_digits() {
        assert_eq!(
            parse("j / ] [ $ 0 ?").unwrap(),
            ['j', '/', ']', '[', '$', '0', '?'].map(plain).to_vec()
        );
    }

    #[test]
    fn uppercase_letters_report_shift() {
        assert_eq!(
            parse("G").unwrap(),
            vec![Input::Key(KeyCode::Char('G'), KeyModifiers::SHIFT)]
        );
    }

    #[test]
    fn named_keys_and_modifiers() {
        assert_eq!(
            parse("Enter Shift-Tab Ctrl-g Space").unwrap(),
            vec![
                Input::Key(KeyCode::Enter, KeyModifiers::NONE),
                Input::Key(KeyCode::BackTab, KeyModifiers::SHIFT),
                Input::Key(KeyCode::Char('g'), KeyModifiers::CONTROL),
                plain(' '),
            ]
        );
    }

    #[test]
    fn quoted_text_is_typed_with_its_spaces() {
        assert_eq!(
            parse(r#"/ "two words" Enter"#).unwrap(),
            vec![
                plain('/'),
                Input::Text("two words".into()),
                Input::Key(KeyCode::Enter, KeyModifiers::NONE),
            ]
        );
    }

    #[test]
    fn mouse_actions_take_their_text() {
        assert_eq!(
            parse(r#"Click "mul?" Drag "a b" "c" DragBack "c" "a" ClickOutside WheelDown"#)
                .unwrap(),
            vec![
                Input::Click("mul?".into()),
                Input::Drag {
                    from: "a b".into(),
                    to: "c".into()
                },
                Input::DragBack {
                    from: "c".into(),
                    to: "a".into()
                },
                Input::ClickOutside,
                Input::Wheel(MouseEventKind::ScrollDown, KeyModifiers::NONE),
            ]
        );
    }

    #[test]
    fn wait_takes_its_text() {
        assert_eq!(
            parse(r#"R Wait "a  b""#).unwrap(),
            vec![
                Input::Key(KeyCode::Char('R'), KeyModifiers::SHIFT),
                Input::Wait("a  b".into()),
            ]
        );
        assert_eq!(parse("Wait j").unwrap_err(), "`Wait` needs quoted text");
    }

    #[test]
    fn braces_are_keys() {
        assert_eq!(parse("} {").unwrap(), ['}', '{'].map(plain).to_vec());
    }

    #[test]
    fn shift_wheel_keeps_the_modifier() {
        assert_eq!(
            parse("Shift-WheelUp").unwrap(),
            vec![Input::Wheel(MouseEventKind::ScrollUp, KeyModifiers::SHIFT)]
        );
    }

    #[test]
    fn mistakes_are_reported_not_ignored() {
        assert_eq!(parse("Nope").unwrap_err(), "unknown key `Nope`");
        assert_eq!(parse("Click j").unwrap_err(), "`Click` needs quoted text");
        assert_eq!(
            parse("\"open").unwrap_err(),
            "unbalanced quotes in `\"open`"
        );
    }
}
