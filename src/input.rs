use sdl2::event::{Event};
use sdl2::keyboard::{Keycode, Mod, Scancode};

pub struct Input {
    data: Vec<(Keycode, KeyData, Vec<CheckEvent>)>,
    window_id: Option<u32>,
}
impl Input {
    pub fn new() -> Input {
        Input{
            data: Vec::new(),
            window_id: None,
        }
    }
    pub fn event(&mut self, e: Event) {
        match e {
            sdl2::event::Event::KeyDown {timestamp, window_id, keycode, scancode, keymod, repeat} => {
                if let None = self.window_id { self.key_down(timestamp, keycode, scancode, keymod, repeat); }
                else if let Some(id) = self.window_id {
                    if id == window_id { self.key_down(timestamp, keycode, scancode, keymod, repeat); }
                }
            },
            sdl2::event::Event::KeyUp {timestamp, window_id, keycode, scancode, keymod, repeat} => {
                if let None = self.window_id { self.key_up(timestamp, keycode, scancode, keymod, repeat); }
                else if let Some(id) = self.window_id {
                    if id == window_id { self.key_up(timestamp, keycode, scancode, keymod, repeat); }
                }
            },
            _ => {}
        }

    }
    fn key_down(&mut self, timestamp: u32, keycode: Option<Keycode>, _scancode: Option<Scancode>, _keymod: Mod, _repeat: bool) {
        if let None = keycode { return; }
        for i in 0..self.data.len() {
            if self.data[i].0 == keycode.unwrap() {
                self.data[i].1.pressed = true;
                self.data[i].1.last_pressed = timestamp;
                return;
            }
        }
        self.data.push(
            (
                keycode.unwrap(),
                KeyData{ pressed:true, last_pressed: timestamp, last_released: 0 },
                Vec::new()
            )
        );
    }
    fn key_up(&mut self, timestamp: u32, keycode: Option<Keycode>, _scancode: Option<Scancode>, _keymod: Mod, _repeat: bool) {
        if let None = keycode { return; }
        for i in 0..self.data.len() {
            if self.data[i].0 == keycode.unwrap() {
                self.data[i].1.pressed = false;
                self.data[i].1.last_released = timestamp;
                return;
            }
        }
        self.data.push(
            (
                keycode.unwrap(),
                KeyData{ pressed:true, last_pressed: timestamp, last_released: 0 },
                Vec::new(),
            )
        );
    }

    pub fn key_pressed(&self, keycode: Keycode) -> bool {
        for i in 0..self.data.len() {
            if self.data[i].0 == keycode { return self.data[i].1.pressed; }
        }
        false
    }
    /** Позволяет реагировать на нажатие/отпускание кнопки через конструкцию
        if input.on_pressed(LShift, N) {...}
        Где вместо N нужно подставить уникальный айди чекера. Если несколько блоков if
        будут использовать один айди - на ивент среагирует первый выполнившийся if, остальные не среагируют. */
    pub fn on_pressed(&mut self, keycode: Keycode, checker_id: usize) -> bool {
        for (kc, state, checkers) in self.data.iter_mut() {
            if *kc == keycode {
                for i in 0..checkers.len() {
                    if checkers[i].id == checker_id { //Если такой чекер существует
                        let result = checkers[i].prev != state.pressed;
                        checkers[i].prev = state.pressed;
                        return result && state.pressed;
                    }
                }
                //Если его не существует
                checkers.push(CheckEvent{id: checker_id, prev: state.pressed});
                return state.pressed;
            }
        }
        //Если этой клавиши еще нет в базе
        self.data.push(
            (
                keycode,
                KeyData{ pressed: false, last_pressed: 0, last_released: 0 },
                vec![ CheckEvent{id: checker_id, prev: false} ],
                )
        );
        false
    }

    /** Позволяет реагировать на нажатие/отпускание кнопки через конструкцию
       if input.on_released(LShift, N) {...}
       Где вместо N нужно подставить уникальный айди чекера. Если несколько блоков if
       будут использовать один айди - на ивент среагирует первый выполнившийся if, остальные не среагируют. */
    pub fn on_released(&mut self, keycode: Keycode, checker_id: usize) -> bool {
        for (kc, state, checkers) in self.data.iter_mut() {
            if *kc == keycode {
                for i in 0..checkers.len() {
                    if checkers[i].id == checker_id { //Если такой чекер существует
                        let result = checkers[i].prev != state.pressed;
                        checkers[i].prev = state.pressed;
                        return result && !state.pressed;
                    }
                }
                //Если его не существует
                checkers.push(CheckEvent{id: checker_id, prev: state.pressed});
                return false;
            }
        }
        //Если этой клавиши еще нет в базе
        self.data.push(
            (
                keycode,
                KeyData{ pressed: false, last_pressed: 0, last_released: 0 },
                vec![ CheckEvent{id: checker_id, prev: false} ],
            )
        );
        false
    }

}

#[derive(Copy, Clone)]
struct KeyData {
    pressed: bool,
    last_pressed: u32,
    last_released: u32,
}
#[derive(Copy, Clone)]
struct CheckEvent {
    id: usize,
    prev: bool,
}