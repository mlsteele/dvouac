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

#[derive(Debug)]
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
    }

    pub fn recommend(&self) -> Option<Layout> {
        let buf = self.buf.iter().collect::<String>();
        let (head, middle, tail) = Self::split_head_middle_tail(&buf);
        let score: i64 = self.suffix(head)
            + middle.iter().map(|w| self.exact(w)).sum::<i64>()
            + self.prefix(tail);
        if score > 16 {
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

    // is a prefix of s a word
    fn prefix(&self, s: &str) -> i64 {
        Self::str_compounder(s).find_map(|w|
            if self.words.contains(w) { Some(w.len() as i64) } else { None }
        ).unwrap_or(0)
    }

    // is a suffix of s a word
    fn suffix(&self, s: &str) -> i64 {
        Self::str_compounder(Self::rev(s)).find_map(|w|
            if self.words.contains(Self::rev(w)) { Some(w.len() as i64) } else { None }
        ).unwrap_or(0)
    }

    fn str_compounder(s: &str) -> impl Iterator<Item=&String> {
        s.chars().scan("", |acc, c| Some(&format!("{}{}", acc, c)))
    }

    fn rev(s: &str) -> &str {
        &s.chars().rev().collect::<String>()
    }
}
