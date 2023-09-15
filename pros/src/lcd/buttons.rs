extern crate alloc;

use alloc::boxed::Box;

use crate::multitasking::mutex::Mutex;

pub struct ButtonsState {
    pub left_pressed: bool,
    pub middle_pressed: bool,
    pub right_pressed: bool,
}

pub fn read_buttons() -> ButtonsState {
    let bit_mask = unsafe { pros_sys::lcd_read_buttons() };
    ButtonsState {
        left_pressed: bit_mask & 0b001 == bit_mask,
        middle_pressed: bit_mask & 0b010 == bit_mask,
        right_pressed: bit_mask & 0b100 == bit_mask,
    }
}

pub enum Button {
    Left,
    Middle,
    Right,
}


pub struct ButtonCallbacks {
    pub left_cb: Option<Box<dyn Fn() -> ()>>,
    pub middle_cb: Option<Box<dyn Fn() -> ()>>,
    pub right_cb: Option<Box<dyn Fn() -> ()>>,
}
unsafe impl Send for ButtonCallbacks {}

lazy_static::lazy_static! {
    pub static ref BUTTON_CALLBACKS: Mutex<ButtonCallbacks> = Mutex::new(ButtonCallbacks {
        left_cb: None,
        middle_cb: None,
        right_cb: None,
    });
}

pub fn register_button_callback(cb: impl Fn() -> () + 'static, button: Button) -> Result<(), ()> {
    extern "C" fn button_0_cb() {
        if let Some(cb) = &BUTTON_CALLBACKS.lock().left_cb {
            cb();
        }
    }

    extern "C" fn button_1_cb() {
        if let Some(cb) = &BUTTON_CALLBACKS.lock().middle_cb {
            cb();
        }
    }

    extern "C" fn button_2_cb() {
        if let Some(cb) = &BUTTON_CALLBACKS.lock().right_cb {
            cb();
        }
    }

    match button {
        Button::Left => {
            BUTTON_CALLBACKS.lock().left_cb = Some(Box::new(cb));
            unsafe {
                pros_sys::lcd_register_btn0_cb(Some(button_0_cb));
            }
        }
        Button::Middle => {
            BUTTON_CALLBACKS.lock().middle_cb = Some(Box::new(cb));
            unsafe {
                pros_sys::lcd_register_btn1_cb(Some(button_1_cb));
            }
        }
        Button::Right => {
            BUTTON_CALLBACKS.lock().right_cb = Some(Box::new(cb));
            unsafe {
                pros_sys::lcd_register_btn2_cb(Some(button_2_cb));
            }
        }
    }

    Ok(())
}
