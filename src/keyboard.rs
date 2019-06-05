use crate::*;
use std::fs::File;
use std::process::{Command,Child,Stdio};
use std::thread;
use crossbeam::channel as xbc;

pub trait Keyboard {
    fn next_key(&mut self) -> Result<Option<char>>;
    fn switch(&mut self, layout: Layout) -> EResult;
}

type KeyCode = i64;

/// Read events from a subprocess: xinput test-xi2 --root
pub struct KeyboardXInput {
    recv: xbc::Receiver<Result<KeyCode>>
}

impl KeyboardXInput {
    pub fn new() -> Result<Self> {
        let child = Command::new("xinput")
            .args(&["test-xi2", "--root"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let (send, recv) = xbc::bounded(0);
        thread::spawn(move || {
            if let Err(err) = Self::processor(child, send.clone()) {
                let _ = send.send(Err(err));
            }
        });
        Ok(Self{recv})
    }

    fn processor(xinput: Child, ch: xbc::Sender<Result<KeyCode>>) -> EResult {
        #[derive(PartialEq)]
        enum State {Cruising, LookForDetail};
        use State::*;
        let mut state = Cruising;
        let stdout = BufReader::new(xinput.stdout.ok_or_else(|| format_err!("missing xinput process stdout"))?);
        for line in stdout.lines() {
            if line.is_err() {
                return line.map(|_|()).map_err(|err|err.into());
            }
            if let Ok(line) = line {
                if line.starts_with("EVENT") {
                    // "EVENT type 2 (KeyPress)"
                    const EV_KEY_PRESS: &str = "2";
                    if line.split(" ").skip(2).next() == Some(&EV_KEY_PRESS) {
                        state = LookForDetail;
                    }
                }
                if state == LookForDetail && line.trim_start().starts_with("detail:") {
                    // "    detail: 54"
                    if let Some(code_str) = line.split(": ").skip(1).next() {
                        let code: KeyCode = code_str.parse()?;
                        ch.send(Ok(code))?;
                        state = Cruising;
                    }
                }
            }
        }
        bail!("xinput exited");
    }

    fn key_code_to_char(key_code: i64) -> Option<char> {
        println!("{}", key_code);
        match key_code {
            38 => Some('a'),
            56 => Some('b'),
            54 => Some('c'),
            40 => Some('d'),
            26 => Some('e'),
            41 => Some('f'),
            42 => Some('g'),
            43 => Some('h'),
            31 => Some('i'),
            44 => Some('j'),
            45 => Some('k'),
            46 => Some('l'),
            58 => Some('m'),
            57 => Some('n'),
            32 => Some('o'),
            33 => Some('p'),
            24 => Some('q'),
            27 => Some('r'),
            39 => Some('s'),
            28 => Some('t'),
            30 => Some('u'),
            55 => Some('v'),
            25 => Some('w'),
            53 => Some('x'),
            29 => Some('y'),
            52 => Some('z'),
            10 => Some('1'),
            11 => Some('2'),
            12 => Some('3'),
            13 => Some('4'),
            14 => Some('5'),
            15 => Some('6'),
            16 => Some('7'),
            17 => Some('8'),
            18 => Some('9'),
            19 => Some('0'),
            20 => Some('-'),
            21 => Some('='),
            34 => Some('['),
            35 => Some(']'),
            51 => Some('\\'),
            49 => Some('`'),
            59 => Some(','),
            60 => Some('.'),
            61 => Some('/'),
            65 => Some(' '),
            _ => None,
        }
    }
}

impl Keyboard for KeyboardXInput {
    fn next_key(&mut self) -> Result<Option<char>> {
        match self.recv.try_recv() {
            Ok(Ok(key_code)) => Ok(Self::key_code_to_char(key_code)),
            Ok(Err(err)) => Err(err),
            Err(xbc::TryRecvError::Empty) => Ok(None),
            Err(xbc::TryRecvError::Disconnected) => bail!("kb receiver disconnected"),
        }
    }

    fn switch(&mut self, layout: Layout) -> EResult {switch(layout)}

}

/// Use evdev_rs to read /dev/input devices.
/// Doesn't seem to work when keys are pressed at speed.
/// "word" comes outt "worr" or "wood".
pub struct KeyboardDevInput {
    devices: Vec<evdev_rs::Device>,
    alternate: bool,
}

impl Keyboard for KeyboardDevInput {
    fn next_key(&mut self) -> Result<Option<char>> {
        loop {
            for dev in &self.devices {
                if !dev.has_event_pending() {
                    continue
                }
                let (_, ev) = dev.next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING)?;
                self.alternate = !self.alternate;
                if self.alternate { continue }
                println!("Event: time {:?} {}.{}, ++++++++++++++++++++ {}    {} +++++++++++++++",
                                dev.name(),
                                ev.time.tv_sec,
                                ev.time.tv_usec,
                                ev.event_type,
                                ev.event_code);
                use evdev_rs::enums::*;
                if let EventCode::EV_KEY(key) = ev.event_code {
                   println!("dev {:?}", dev.name());
                   return Ok(Self::key_to_char(key))
                }
                return Ok(None)
            }
        }
    }

    fn switch(&mut self, layout: Layout) -> EResult {switch(layout)}
}

impl KeyboardDevInput {
    #[allow(dead_code)]
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
        let kb_devices = devices.into_iter().enumerate().filter(|(i, dev)| {
            let sample_key = evdev_rs::enums::EventCode::EV_KEY(evdev_rs::enums::EV_KEY::KEY_A);
            let kb = dev.has(&sample_key);
            println!("{}: {:?} {:?}", i, dev.name(), kb);
            kb
        }).map(|(_, dev)| dev).collect::<Vec<_>>();
        println!("selected {} devices", kb_devices.len());
        Ok(Self{ devices: kb_devices, alternate: false })
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
            KEY_MINUS => Some('-'),
            KEY_SEMICOLON => Some(','),
            KEY_APOSTROPHE => Some('\''),
            KEY_GRAVE => Some('`'),
            KEY_BACKSLASH => Some('\\'),
            KEY_EQUAL => Some('='),
            KEY_LEFTBRACE => Some('{'),
            KEY_RIGHTBRACE => Some('}'),
            KEY_COMMA => Some(','),
            KEY_DOT => Some('.'),
            KEY_SLASH => Some('/'),
            _ => None,
        }
    }
}


fn switch(layout: Layout) -> EResult {
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
