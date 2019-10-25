use chip8_core::program::Program as InnerProgram;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Program {
    inner: InnerProgram
}

#[derive(Serialize)]
pub struct Screen(Vec<Vec<bool>>);

#[wasm_bindgen]
impl Program {
    pub fn new() -> Self {
        console_error_panic_hook::set_once();

        Program {
            inner: InnerProgram::new()
        }
    }

    pub fn load(&mut self, rom: &[u8]) {
        self.inner.load(rom);
    }

    pub fn tick(&mut self) {
        self.inner.run();
    }

    pub fn screen(&self) -> JsValue {
        let screen: Vec<_> = self.inner.screen.iter()
            .map(|line| line.to_vec())
            .collect();
        let screen = Screen(screen);

        JsValue::from_serde(&screen).unwrap()
    }

    pub fn pc(&self) -> u16 {
        self.inner.program_counter
    }

    pub fn memory(&self) -> Vec<u8> {
        self.inner.memory.to_vec()
    }

    pub fn keydown(&mut self, key: usize) {
        self.inner.keydown(key);
    }

    pub fn keyup(&mut self, key: usize) {
        self.inner.keyup(key);
    }

    pub fn delay_timer(&self) -> u8 {
        self.inner.delay_timer
    }

    pub fn sound_timer(&self) -> u8 {
        self.inner.sound_timer
    }

    pub fn decrement_timers(&mut self) {
        if self.inner.delay_timer > 0 {
            self.inner.delay_timer -= 1;
        }
        if self.inner.sound_timer > 0 {
            self.inner.sound_timer -= 1;
        }
    }
}
