mod error;
use error::*;
use std::fs::File;

#[macro_use] extern crate failure;

fn main() {
    if let Err(err) = main2() {
        // eprintln!("{}", pretty_error(&err));
        eprint_error(&err);
        std::process::exit(1);
    }
}

fn main2() -> EResult {
    // let mut devices = evdev::enumerate();
    // if devices.len() == 0 {
    //     bail!("evdev listed no devices (may need sudo)");
    // }
    // for (i, device) in devices.iter().enumerate() {
    //     println!("{} {:?}", i, device.name());
    // }
    // let device = &mut devices[5];
    // println!("{:?}", device.name());
    // for ev in device.events()? {
    //     println!("{:?}", ev);
    // }
    // return EOK;

    let mut devices = Vec::new();
    if let Ok(dir) = std::fs::read_dir("/dev/input") {
        for entry in dir {
            if let Ok(entry) = entry {
                if let Ok(dev) = evdev_rs::Device::new_from_fd(File::open(&entry.path())?) {
                    devices.push(dev)
                }
            }
        }
    }
    if devices.len() == 0 {
        bail!("no devices opened (may need sudo)");
    }
    for (i, dev) in devices.iter().enumerate() {
        println!("{}: {:?}", i, dev.name())
    }

    loop {
        for dev in &devices {
            if !dev.has_event_pending() {
                continue
            }
            let a = dev.next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING);
            match a {
                Ok(k) => println!("Event: time {:?} {}.{}, ++++++++++++++++++++ {} +++++++++++++++",
                                  dev.name(),
                                k.1.time.tv_sec,
                                k.1.time.tv_usec,
                                k.1.event_type),
                Err(e) => (),
            }
        }
    }
    return EOK;

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

pub struct Recognizer {}

impl Recognizer {
    pub fn new() -> Self { Self{} }

    pub fn feed(&mut self, _c: char) {
        unimplemented!();
    }

    pub fn recommend(&self) -> Option<Layout> {
        unimplemented!();
    }
}
