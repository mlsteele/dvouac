use crate::*;
use std::fs::File;

pub struct Keyboard {
}

impl Keyboard {
    pub fn new() -> Self { Self{} }

    pub fn next_key(&mut self) -> Result<char> {
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
                let (_, ev) = dev.next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING)?;
                println!("Event: time {:?} {}.{}, ++++++++++++++++++++ {}    {} +++++++++++++++",
                                dev.name(),
                                ev.time.tv_sec,
                                ev.time.tv_usec,
                                ev.event_type,
                                ev.event_code);
                use evdev_rs::enums::*;
                if let EventCode::EV_KEY(key) = ev.event_code {
                }
                return Ok('t');
            }
        }
    }

    fn key_to_char(k: evdev_rs::enums::EV_KEY) -> Option<char> {
        use evdev_rs::enums::EV_KEY::*;
        match k {
            KEY_A => Some('a'),
            _ => None,
        }
    }

    pub fn switch(&mut self, _layout: Layout) {
        unimplemented!();
    }
}

