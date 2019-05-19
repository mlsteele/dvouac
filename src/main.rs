mod error;
use error::*;

mod keyboard;
use keyboard::*;

#[macro_use] extern crate failure;

extern crate strum;
#[macro_use]
extern crate strum_macros;

use std::io::{BufReader,BufRead};
use stringreader::StringReader;
use std::collections::VecDeque;
use std::collections::HashSet;
use strum::IntoEnumIterator;

fn main() {
    if let Err(err) = main2() {
        // eprintln!("{}", pretty_error(&err));
        eprint_error(&err);
        std::process::exit(1);
    }
}

fn main2() -> EResult {
    let mut kb = Keyboard::new()?;
    let mut recognizer = Recognizer::new();
    loop {
        let c = kb.next_key()?;
        if let Some(c) = c {
            println!("key {}", c);
            recognizer.feed(c);
            if let Some(layout) = recognizer.recommend() {
                println!("switch {:?}", layout);
                kb.switch(layout)?;
            }
        }
    }
}

fn eprint_error(err: &failure::Error) {
    eprintln!("");
    for err in err.iter_chain() {
        eprintln!("{}", err);
    }
    eprintln!("\n{:?}", err.backtrace());
}

#[derive(Debug, Clone, Copy, PartialEq, EnumIter)]
pub enum Layout {
    US,
    Dvorak,
}

pub struct Recognizer {
    words: HashSet<String>,
    buf: VecDeque<char>,
    buf_max: usize,
}

impl Recognizer {
    pub fn new() -> Self {
        let words = BufReader::new(StringReader::new(
            include_str!("/usr/share/dict/words")))
            .lines().flat_map(|word| if let Ok(word) = word {
                Some(word.to_lowercase())
            } else {
                None
            }).collect();
        Self {
            words,
            buf: Default::default(),
            buf_max: 16,
        }
    }

    pub fn feed(&mut self, c: char) {
        self.buf.push_back(c);
        if self.buf.len() > self.buf_max {
            self.buf.pop_front();
        }
    }

    pub fn feed_str(&mut self, s: &str) {
        for c in s.chars() {
            self.feed(c)
        }
    }

    pub fn recommend(&self) -> Option<Layout> {
        Layout::iter()
            .map(|ly| (ly, self.evaluate(ly)))
            .filter(|(_, score)| *score > 0)
            .max_by_key(|(_, score)| *score)
            .map(|(ly, _)| ly)
    }

    fn evaluate(&self, layout: Layout) -> i64 {
        let buf = self.buf.iter().collect::<String>();
        let buf = Self::transform(layout, &buf);
        let (head, middle, tail) = Self::split_head_middle_tail(&buf);
        let score: i64 = self.suffix(head)
            + middle.iter().map(|w| self.exact(w)).sum::<i64>()
            + self.prefix(tail);
        println!("eval {:?} {:?} {:?} {:?} {:?}", head, middle, tail, layout, score);
        if score > 9 {
            score
        } else {
            0
        }
    }

    // Transform into `layout`
    fn transform(layout: Layout, s: &str) -> String {
        use Layout::*;
        // "-=qwertyuiop[]asdfghjkl;'zxcvbnm,./_+QWERTYUIOP{}ASDFGHJKL:\"ZXCVBNM<>?"
        // "[]',.pyfgcrl/=aoeuidhtns-;qjkxbmwvz{}\"<>PYFGCRL?+AOEUIDHTNS_:QJKXBMWVZ"
        s.chars().map(|c| match layout {
            US => c,
            // US => match c {

            //     '[' => '-',
            //     ']' => '=',
            //     '\'' => 'q',
            //     ',' => 'w',
            //     '.' => 'e',
            //     'p' => 'r',
            //     'y' => 't',
            //     'f' => 'y',
            //     'g' => 'u',
            //     'c' => 'i',
            //     'r' => 'o',
            //     'l' => 'p',
            //     '/' => '[',
            //     '=' => ']',
            //     'a' => 'a',
            //     'o' => 's',
            //     'e' => 'd',
            //     'u' => 'f',
            //     'i' => 'g',
            //     'd' => 'h',
            //     'h' => 'j',
            //     't' => 'k',
            //     'n' => 'l',
            //     's' => ';',
            //     '-' => '\'',
            //     ';' => 'z',
            //     'q' => 'x',
            //     'j' => 'c',
            //     'k' => 'v',
            //     'x' => 'b',
            //     'b' => 'n',
            //     'm' => 'm',
            //     'w' => ',',
            //     'v' => '.',
            //     'z' => '/',
            //     '{' => '_',
            //     '}' => '+',
            //     '"' => 'Q',
            //     '<' => 'W',
            //     '>' => 'E',
            //     'P' => 'R',
            //     'Y' => 'T',
            //     'F' => 'Y',
            //     'G' => 'U',
            //     'C' => 'I',
            //     'R' => 'O',
            //     'L' => 'P',
            //     '?' => '{',
            //     '+' => '}',
            //     'A' => 'A',
            //     'O' => 'S',
            //     'E' => 'D',
            //     'U' => 'F',
            //     'I' => 'G',
            //     'D' => 'H',
            //     'H' => 'J',
            //     'T' => 'K',
            //     'N' => 'L',
            //     'S' => ':',
            //     '_' => '"',
            //     ':' => 'Z',
            //     'Q' => 'X',
            //     'J' => 'C',
            //     'K' => 'V',
            //     'X' => 'B',
            //     'B' => 'N',
            //     'M' => 'M',
            //     'W' => '<',
            //     'V' => '>',
            //     'Z' => '?',

            //     _ => c,
            // }
            Dvorak => match c {

                '-' => '[',
                '=' => ']',
                'q' => '\'',
                'w' => ',',
                'e' => '.',
                'r' => 'p',
                't' => 'y',
                'y' => 'f',
                'u' => 'g',
                'i' => 'c',
                'o' => 'r',
                'p' => 'l',
                '[' => '/',
                ']' => '=',
                'a' => 'a',
                's' => 'o',
                'd' => 'e',
                'f' => 'u',
                'g' => 'i',
                'h' => 'd',
                'j' => 'h',
                'k' => 't',
                'l' => 'n',
                ';' => 's',
                '\'' => '-',
                'z' => ';',
                'x' => 'q',
                'c' => 'j',
                'v' => 'k',
                'b' => 'x',
                'n' => 'b',
                'm' => 'm',
                ',' => 'w',
                '.' => 'v',
                '/' => 'z',
                '_' => '{',
                '+' => '}',
                'Q' => '"',
                'W' => '<',
                'E' => '>',
                'R' => 'P',
                'T' => 'Y',
                'Y' => 'F',
                'U' => 'G',
                'I' => 'C',
                'O' => 'R',
                'P' => 'L',
                '{' => '?',
                '}' => '+',
                'A' => 'A',
                'S' => 'O',
                'D' => 'E',
                'F' => 'U',
                'G' => 'I',
                'H' => 'D',
                'J' => 'H',
                'K' => 'T',
                'L' => 'N',
                ':' => 'S',
                '"' => '_',
                'Z' => ':',
                'X' => 'Q',
                'C' => 'J',
                'V' => 'K',
                'B' => 'X',
                'N' => 'B',
                'M' => 'M',
                '<' => 'W',
                '>' => 'V',
                '?' => 'Z',

                _ => c,
            },
        }).collect()
    }



    fn split_head_middle_tail(s: &str) -> (&str, Vec<&str>, &str) {
        let mut words = s.split(' ');
        if let Some(first) = words.next() {
            let mut words = words.rev();
            if let Some(last) = words.next() {
                (first, words.collect(), last)
            } else {
                ("", Default::default(), first)
            }
        } else {
            ("", Default::default(), "")
        }
    }

    fn exact(&self, s: &str) -> i64 {
        if self.words.contains(s) { s.len() as i64 } else { 0 }
    }

    // Is s the prefix of a word? Returns length of prefix or 0.
    fn prefix(&self, s: &str) -> i64 {
        self.words.iter()
            .map(|word| if word.starts_with(s) { s.len() as i64 } else { 0 } )
            .max().unwrap_or(0)
    }

    // is a suffix of s a word
    fn suffix(&self, s: &str) -> i64 {
        self.words.iter()
            .map(|word| if word.ends_with(s) { s.len() as i64 } else { 0 } )
            .max().unwrap_or(0)
    }

    // fn prefixes<'a>(s: &'a str) -> impl Iterator<Item=String> + 'a {
    //     s.chars().scan("", |acc, c| Some(format!("{}{}", acc, c).to_owned()))
    // }

    // fn rev(s: &str) -> String {
    //     s.chars().rev().collect::<String>()
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_prefix() {
        let r = Recognizer::new();
        assert_eq!(0, r.prefix("qwordq"));
        assert_eq!(0, r.prefix("wordq"));
        assert_eq!(0, r.prefix("qword"));
        assert_eq!(3, r.prefix("wor"));
        assert_eq!(4, r.prefix("word"));
    }

    #[test] fn test_suffix() {
        let r = Recognizer::new();
        assert_eq!(0, r.suffix("qwordq"));
        assert_eq!(0, r.suffix("wordq"));
        assert_eq!(0, r.suffix("qword"));
        assert_eq!(3, r.suffix("ord"));
        assert_eq!(4, r.suffix("word"));
    }

    #[test] fn test_recognizer_us_dv() {
        let mut r = Recognizer::new();
        assert_eq!(None, r.recommend());
        r.feed_str(",soh ,soh ,soh ,soh");
        assert_eq!(Some(Layout::US), r.recommend());
    }

    // In dvorak but then type like you're in qwerty word
    #[test] fn test_recognizer_dv_us() {
        let mut r = Recognizer::new();
        assert_eq!(None, r.recommend());
        r.feed_str(",rpe ,rpe ,rpe ,rpe");
        assert_eq!(Some(Layout::Dvorak), r.recommend());
    }
}
