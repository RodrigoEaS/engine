pub struct InputManager {
    pub(crate) input: u8
}

impl InputManager {
    pub fn new() -> Self {
        Self { 
            input: u8::default()
        }
    }

    pub(crate) fn register(&mut self, input: u8) {
        self.input = input
    }
}