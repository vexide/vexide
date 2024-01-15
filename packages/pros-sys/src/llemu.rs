// #[cfg(feature = "xapi")]
// compile_error!("LVGL bindings (xapi) are a todo for now");

use cfg_if::cfg_if;

pub const LCD_BTN_LEFT: core::ffi::c_int = 4;
pub const LCD_BTN_CENTER: core::ffi::c_int = 2;
pub const LCD_BTN_RIGHT: core::ffi::c_int = 1;

pub type lcd_button_cb_fn_t = Option<unsafe extern "C" fn()>;

cfg_if! {
    if #[cfg(feature = "xapi")] {
        // #[repr(C)]
        // pub struct lcd_s_t {
        //     //TODO
        // }

        #[repr(C)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct lv_color_t {
            pub blue: u8,
            pub green: u8,
            pub red: u8,
            pub alpha: u8,
        }

        impl From<u32> for lv_color_t {
            fn from(color: u32) -> Self {
                Self {
                    blue: (color & 0xFF) as u8,
                    green: ((color >> 8) & 0xFF) as u8,
                    red: ((color >> 16) & 0xFF) as u8,
                    alpha: ((color >> 24) & 0xFF) as u8,
                }
            }
        }

        impl From<lv_color_t> for u32 {
            fn from(color: lv_color_t) -> Self {
                (color.blue as u32)
                    | ((color.green as u32) << 8)
                    | ((color.red as u32) << 16)
                    | ((color.alpha as u32) << 24)
            }
        }
    }
}

extern "C" {
    /** Checks whether the emulated three-button LCD has already been initialized.

    \return True if the LCD has been initialized or false if not.*/
    pub fn lcd_is_initialized() -> bool;
    /** Creates an emulation of the three-button, UART-based VEX LCD on the display.

    \return True if the LCD was successfully initialized, or false if it has
    already been initialized.*/
    pub fn lcd_initialize() -> bool;
    /** Turns off the Legacy LCD Emulator.

    Calling this function will clear the entire display, and you will not be able
    to call any further LLEMU functions until another call to lcd_initialize.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The LCD has not been initialized. Call lcd_initialize() first.

    \return True if the operation was successful, or false otherwise, setting
    errno values as specified above.*/
    pub fn lcd_shutdown() -> bool;
    /** Displays a formatted string on the emulated three-button LCD screen.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO  - The LCD has not been initialized. Call lcd_initialize() first.
    EINVAL - The line number specified is not in the range [0-7]

    \param line
     The line on which to display the text [0-7]
    \param fmt
     Format string
    \param ...
     Optional list of arguments for the format string

    \return True if the operation was successful, or false otherwise, setting
    errno values as specified above.*/
    pub fn lcd_print(line: i16, fmt: *const core::ffi::c_char, ...) -> bool;
    /** Displays a string on the emulated three-button LCD screen.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO  - The LCD has not been initialized. Call lcd_initialize() first.
    EINVAL - The line number specified is not in the range [0-7]

    \param line
     The line on which to display the text [0-7]
    \param text
     The text to display

    \return True if the operation was successful, or false otherwise, setting
    errno values as specified above.*/
    pub fn lcd_set_text(line: i16, text: *const core::ffi::c_char) -> bool;
    /** Clears the contents of the emulated three-button LCD screen.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO  - The LCD has not been initialized. Call lcd_initialize() first.
    EINVAL - The line number specified is not in the range [0-7]

    \return True if the operation was successful, or false otherwise, setting
    errno values as specified above.*/
    pub fn lcd_clear() -> bool;
    /** Clears the contents of a line of the emulated three-button LCD screen.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO  - The LCD has not been initialized. Call lcd_initialize() first.
    EINVAL - The line number specified is not in the range [0-7]

    \param line
     The line to clear

    \return True if the operation was successful, or false otherwise, setting
    errno values as specified above.*/
    pub fn lcd_clear_line(line: i16) -> bool;
    /** Registers a callback function for the leftmost button.

    When the leftmost button on the emulated three-button LCD is pressed, the
    user-provided callback function will be invoked.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO  - The LCD has not been initialized. Call lcd_initialize() first.

    \param cb
     A callback function of type lcd_btn_cb_fn_t (void (*cb)(void))

    \return True if the operation was successful, or false otherwise, setting
    errno values as specified above.*/
    pub fn lcd_register_btn0_cb(cb: lcd_button_cb_fn_t) -> bool;
    /** Registers a callback function for the center button.

    When the center button on the emulated three-button LCD is pressed, the
    user-provided callback function will be invoked.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO  - The LCD has not been initialized. Call lcd_initialize() first.

    \param cb
     A callback function of type lcd_btn_cb_fn_t (void (*cb)(void))

    \return True if the operation was successful, or false otherwise, setting
    errno values as specified above.*/
    pub fn lcd_register_btn1_cb(cb: lcd_button_cb_fn_t) -> bool;
    /** Registers a callback function for the rightmost button.

    When the rightmost button on the emulated three-button LCD is pressed, the
    user-provided callback function will be invoked.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO  - The LCD has not been initialized. Call lcd_initialize() first.

    \param cb
     A callback function of type lcd_btn_cb_fn_t (void (*cb)(void))

    \return True if the operation was successful, or false otherwise, setting
    errno values as specified above.*/
    pub fn lcd_register_btn2_cb(cb: lcd_button_cb_fn_t) -> bool;
    /** Gets the button status from the emulated three-button LCD.

    The value returned is a 3-bit integer where 1 0 0 indicates the left button
    is pressed, 0 1 0 indicates the center button is pressed, and 0 0 1
    indicates the right button is pressed. 0 is returned if no buttons are
    currently being pressed.

    Note that this function is provided for legacy API compatibility purposes,
    with the caveat that the V5 touch screen does not actually support pressing
    multiple points on the screen at the same time.

    \return The buttons pressed as a bit mask*/
    pub fn lcd_read_buttons() -> u8;

    cfg_if! {
        if #[cfg(feature = "xapi")] {
            /** Changes the color of the LCD background to a provided color expressed in
            type lv_color_t.

            \param color
                   A color of type lv_color_t

            \return void
            */
            pub fn lcd_set_background_color(color: lv_color_t);
            /** Changes the text color of the LCD to a provided color expressed in
            type lv_color_t.

            \param color
                   A color of type lv_color_t

            \return void
            */
            pub fn lcd_set_text_color(color: lv_color_t);
        }
    }
}
