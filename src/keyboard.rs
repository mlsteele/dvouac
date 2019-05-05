
pub struct Keyboard {}

impl Keyboard {
    pub fn new() -> Self {
        // evdev::enumerate()
        //     device.events
        //     .next
        //     .code
        Self{}
    }

    pub fn next_key(&mut self) -> Result<char> {
        unimplemented!();
    }

    pub fn switch(&mut self, _layout: Layout) {
        unimplemented!();
    }
}
