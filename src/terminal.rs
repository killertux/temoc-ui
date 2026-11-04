//! Asking the real terminal about itself.

use crate::theme::Rgb;
use crossterm::terminal;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// How long a terminal has to answer.
const PATIENCE: Duration = Duration::from_millis(200);
/// Asks for the background colour with OSC 11, then for the device attributes.
/// Terminals answer in order and nearly all know the second question, so its
/// answer means that there is no need to wait any longer.
const QUESTIONS: &[u8] = b"\x1b]11;?\x1b\\\x1b[c";
/// The start of the answer to the background colour.
const BACKGROUND_ANSWER: &str = "\x1b]11;";
/// The start of the answer to the device attributes.
const ATTRIBUTES_ANSWER: &str = "\x1b[?";

/// The terminal's background colour. A terminal that cannot be reached or does
/// not answer in time has no answer, which is not an error: temoc-ui goes on
/// with a dark theme.
#[must_use]
pub fn background() -> Option<Rgb> {
    let Ok(tty) = OpenOptions::new().read(true).write(true).open("/dev/tty") else {
        return None;
    };
    if terminal::enable_raw_mode().is_err() {
        return None;
    }
    let answer = ask(tty);
    terminal::disable_raw_mode().map_or(None, |()| answer)
}

fn ask(mut tty: File) -> Option<Rgb> {
    tty.write_all(QUESTIONS).ok()?;
    let answers = listen(tty)?;
    let deadline = Instant::now() + PATIENCE;
    let mut heard = Vec::new();
    // This ends when the terminal has finished answering, or time runs out.
    while let Ok(latest) = answers.recv_timeout(deadline.saturating_duration_since(Instant::now()))
    {
        heard = latest;
    }
    background_in(&heard)
}

/// Passes on everything heard so far, each time more arrives, and stops at the
/// end of the answers so that the application gets the keys pressed after them.
/// If the terminal never finishes answering, reading blocks until the first
/// key, and that key is lost. An answer that comes too late is kept away from
/// the application the same way.
fn listen(mut tty: impl Read + Send + 'static) -> Option<mpsc::Receiver<Vec<u8>>> {
    let (sender, receiver) = mpsc::channel();
    let listening = thread::Builder::new().spawn(move || {
        let mut heard = Vec::new();
        let mut buffer = [0; 64];
        while !has_finished(&heard) {
            let Ok(count @ 1..) = tty.read(&mut buffer) else {
                break;
            };
            heard.extend(buffer.get(..count).unwrap_or_default());
            if sender.send(heard.clone()).is_err() {
                break;
            }
        }
    });
    listening.is_ok().then_some(receiver)
}

/// Whether the answer to the last question has arrived.
fn has_finished(heard: &[u8]) -> bool {
    String::from_utf8_lossy(heard)
        .split_once(ATTRIBUTES_ANSWER)
        .is_some_and(|(_, rest)| rest.contains('c'))
}

/// The colour in an answer such as `ESC ] 11 ; rgb:ffff/ffff/ffff BEL`.
fn background_in(heard: &[u8]) -> Option<Rgb> {
    let heard = String::from_utf8_lossy(heard);
    let (_, answer) = heard.split_once(BACKGROUND_ANSWER)?;
    let (color, _) = answer.split_once(['\x07', '\x1b'])?;
    let channels = color
        .strip_prefix("rgb:")
        .or_else(|| color.strip_prefix("rgba:"))?;
    let mut channels = channels.split('/').map(channel);
    Some(Rgb(channels.next()??, channels.next()??, channels.next()??))
}

/// A channel of one to four hexadecimal digits, scaled to eight bits.
fn channel(digits: &str) -> Option<u8> {
    let count = u32::try_from(digits.len()).ok()?;
    let top = 16_u32.checked_pow(count).filter(|top| *top <= 0x1_0000)? - 1;
    let value = u32::from_str_radix(digits, 16).ok()?;
    u8::try_from(value * 255 / top.max(1)).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::io;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A terminal that sends one chunk of bytes for each read.
    struct Chunks {
        chunks: VecDeque<&'static [u8]>,
        reads: Arc<AtomicUsize>,
    }

    impl Read for Chunks {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            self.reads.fetch_add(1, Ordering::SeqCst);
            let chunk = self.chunks.pop_front().unwrap_or_default();
            buffer[..chunk.len()].copy_from_slice(chunk);
            Ok(chunk.len())
        }
    }

    #[test]
    fn listening_leaves_the_key_after_the_answers_alone() {
        let reads = Arc::new(AtomicUsize::new(0));
        let tty = Chunks {
            chunks: VecDeque::from([&b"\x1b]11;rgb:ff/ff/ff\x07"[..], b"\x1b[?62;c", b"q"]),
            reads: Arc::clone(&reads),
        };
        let heard: Vec<Vec<u8>> = listen(tty).unwrap().iter().collect();
        assert_eq!(
            background_in(heard.last().unwrap()),
            Some(Rgb(255, 255, 255))
        );
        assert_eq!(reads.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn reads_four_digit_channels() {
        let heard = b"\x1b]11;rgb:ffff/8080/0000\x1b\\\x1b[?62;c";
        assert_eq!(background_in(heard), Some(Rgb(255, 128, 0)));
    }

    #[test]
    fn reads_an_answer_ended_with_bell() {
        let heard = b"\x1b]11;rgb:16/18/1d\x07";
        assert_eq!(background_in(heard), Some(Rgb(0x16, 0x18, 0x1D)));
    }

    #[test]
    fn scales_short_channels() {
        assert_eq!(
            background_in(b"\x1b]11;rgb:f/8/0\x07"),
            Some(Rgb(255, 136, 0))
        );
    }

    #[test]
    fn ignores_the_alpha_channel() {
        let heard = b"\x1b]11;rgba:ffff/ffff/ffff/8000\x1b\\";
        assert_eq!(background_in(heard), Some(Rgb(255, 255, 255)));
    }

    #[test]
    fn keys_pressed_before_the_answer_do_not_matter() {
        let heard = b"jk\x1b]11;rgb:00/00/00\x07";
        assert_eq!(background_in(heard), Some(Rgb(0, 0, 0)));
    }

    #[test]
    fn something_else_is_no_answer() {
        for heard in [
            &b""[..],
            b"\x1b[?62;c",
            b"\x1b]11;rgb:ffff/ffff",
            b"\x1b]11;rgb:ffff/ffff\x07",
            b"\x1b]11;rgb:zz/00/00\x07",
            b"\x1b]11;rgb:/00/00\x07",
            b"\x1b]11;rgb:fffff/00/00\x07",
            b"\x1b]11;white\x07",
        ] {
            assert_eq!(background_in(heard), None);
        }
    }

    #[test]
    fn finishes_with_the_answer_to_the_last_question() {
        assert!(!has_finished(b""));
        assert!(!has_finished(b"\x1b]11;rgb:ff/ff/ff\x07"));
        assert!(!has_finished(b"\x1b[?62;"));
        assert!(has_finished(b"\x1b]11;rgb:ff/ff/ff\x07\x1b[?62;c"));
    }
}
