mod error;
use error::*;

mod keyboard;
use keyboard::*;

#[macro_use] extern crate failure;

fn main() {
    if let Err(err) = main2() {
        // eprintln!("{}", pretty_error(&err));
        eprint_error(&err);
        std::process::exit(1);
    }
}

fn main2() -> EResult {
    let mut kb = Keyboard::new();
    let mut recognizer = Recognizer::new();
    loop {
        let c = kb.next_key()?;
        recognizer.feed(c);
        recognizer.recommend();
    }
}

fn eprint_error(err: &failure::Error) {
    eprintln!("");
    for err in err.iter_chain() {
        eprintln!("{}", err);
    }
    eprintln!("\n{:?}", err.backtrace());
}

pub enum Layout {
    US,
    Dvorak,
}

pub struct Recognizer {
    buf: Option<char>,
}

impl Recognizer {
    pub fn new() -> Self { Self{
        buf: Default::default(),
    } }

    pub fn feed(&mut self, c: char) {
        self.buf = Some(c);
    }

    pub fn recommend(&self) -> Option<Layout> {
        if let Some(c) = self.buf {
            return match c {
                'q' => Some(Layout::US),
                'd' => Some(Layout::Dvorak),
                _ => None,
            }
        }
        None
    }
}
