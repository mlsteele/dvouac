mod error;
use error::*;

mod keyboard;
use keyboard::*;

#[macro_use] extern crate failure;

use std::io::{BufReader,BufRead};
use stringreader::StringReader;
use std::collections::VecDeque;
use std::collections::HashSet;

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
            // println!("key {}", c);
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

#[derive(Debug, PartialEq)]
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
        let buf = self.buf.iter().collect::<String>();
        let (head, middle, tail) = Self::split_head_middle_tail(&buf);
        println!("{:?} {:?} {:?}", head, middle, tail);
        let score: i64 = self.suffix(head)
            + middle.iter().map(|w| self.exact(w)).sum::<i64>()
            + self.prefix(tail);
        if score > 9 {
            Some(Layout::US)
        } else {
            None
        }
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

    #[test] fn test_recognizer_us_us() {
        let mut r = Recognizer::new();
        assert_eq!(None, r.recommend());
        r.feed_str("w");
        assert_eq!(None, r.recommend());
        r.feed_str("ord word word word");
        assert_eq!(Some(Layout::US), r.recommend());
    }

    #[test] fn test_recognizer_us_dv() {
        let mut r = Recognizer::new();
        assert_eq!(None, r.recommend());
        r.feed_str(",rpe");
        assert_eq!(Some(Layout::US), r.recommend());
    }

    #[test] fn test_recognizer_dv_us() {
        let mut r = Recognizer::new();
        assert_eq!(None, r.recommend());
        r.feed_str(",soh");
        assert_eq!(Some(Layout::Dvorak), r.recommend());
    }
}
