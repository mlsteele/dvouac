use crate::*;
use std::fs::File;
use std::process::Command;

pub struct Keyboard {
    devices: Vec<evdev_rs::Device>
}

impl Keyboard {
    pub fn new() -> Result<Self> {
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
        Ok(Self{ devices })
    }

    pub fn next_key(&mut self) -> Result<Option<char>> {
        loop {
            for dev in &self.devices {
                if !dev.has_event_pending() {
                    continue
                }
                let (_, ev) = dev.next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING)?;
                // println!("Event: time {:?} {}.{}, ++++++++++++++++++++ {}    {} +++++++++++++++",
                //                 dev.name(),
                //                 ev.time.tv_sec,
                //                 ev.time.tv_usec,
                //                 ev.event_type,
                //                 ev.event_code);
                use evdev_rs::enums::*;
                if let EventCode::EV_KEY(key) = ev.event_code {
                    return Ok(Self::key_to_char(key))
                }
                return Ok(None)
            }
        }
    }

    fn key_to_char(k: evdev_rs::enums::EV_KEY) -> Option<char> {
        use evdev_rs::enums::EV_KEY::*;
        match k {
            KEY_A => Some('a'),
            KEY_B => Some('b'),
            KEY_C => Some('c'),
            KEY_D => Some('d'),
            KEY_E => Some('e'),
            KEY_F => Some('f'),
            KEY_G => Some('g'),
            KEY_H => Some('h'),
            KEY_I => Some('i'),
            KEY_J => Some('j'),
            KEY_K => Some('k'),
            KEY_L => Some('l'),
            KEY_M => Some('m'),
            KEY_N => Some('n'),
            KEY_O => Some('o'),
            KEY_P => Some('p'),
            KEY_Q => Some('q'),
            KEY_R => Some('r'),
            KEY_S => Some('s'),
            KEY_T => Some('t'),
            KEY_U => Some('u'),
            KEY_V => Some('v'),
            KEY_W => Some('w'),
            KEY_X => Some('x'),
            KEY_Y => Some('y'),
            KEY_Z => Some('z'),
            KEY_SPACE => Some(' '),
            _ => None,
        }
    }

    pub fn switch(&mut self, layout: Layout) -> EResult {
        use Layout::*;
        let arg = match layout {
            US => "us",
            Dvorak => "us(dvorak)",
        };
        if Command::new("setxkbmap")
            .args(&[arg])
            .status()?.success()
        {
            EOK
        } else {
            bail!("failed to setxkbmap")
        }
        // setxkbmap 'us'
        // setxkbmap 'us(dvorak)'
    }
}

