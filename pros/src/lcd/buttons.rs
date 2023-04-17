use pros_sys;

pub struct Buttons {
    pub left_pressed: bool,
    pub middle_pressed: bool,
    pub right_pressed: bool,
}

pub fn read_buttons() -> Buttons {
    let bit_mask = unsafe { pros_sys::lcd_read_buttons() };
    Buttons {
        left_pressed: bit_mask & 0b001 == bit_mask,
        middle_pressed: bit_mask & 0b010 == bit_mask,
        right_pressed: bit_mask & 0b100 == bit_mask
    }
}