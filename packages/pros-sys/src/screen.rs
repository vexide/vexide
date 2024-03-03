//! Brain screen display and touch functions.
//!
//! Contains user calls to the v5 screen for touching and displaying graphics.

use core::ffi::{c_char, c_int};

/// Struct representing screen touch status, screen last x, screen last y, press count, release count.
#[repr(C)]
pub struct screen_touch_status_s_t {
    /// Represents if the screen is being held, released, or pressed.
    pub touch_status: last_touch_e_t,

    /// Represents the x value of the location of the touch.
    pub x: i16,

    /// Represents the y value of the location of the touch.
    pub y: i16,

    /// Represents how many times the screen has be pressed.
    pub press_count: i32,

    /// Represents how many times the user released after a touch on the screen.
    pub release_count: i32,
}

pub const E_TEXT_SMALL: c_int = 0;
pub const E_TEXT_MEDIUM: c_int = 1;
pub const E_TEXT_LARGE: c_int = 2;
pub const E_TEXT_MEDIUM_CENTER: c_int = 3;
pub const E_TEXT_LARGE_CENTER: c_int = 4;
pub type text_format_e_t = c_int;

pub const E_TOUCH_RELEASED: c_int = 0;
pub const E_TOUCH_PRESSED: c_int = 1;
pub const E_TOUCH_HELD: c_int = 2;
pub const E_TOUCH_ERROR: c_int = 3;
pub type last_touch_e_t = c_int;

pub type touch_event_cb_fn_t = unsafe extern "C" fn();

extern "C" {
    /// Set the pen color for subsequent graphics operations
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// EACCESS - Another resource is currently trying to access the screen mutex.
    ///
    /// \param color    The pen color to set (it is recommended to use values
    ///          from the enum defined in colors.h)
    ///
    /// \return Returns 1 if the mutex was successfully returned, or PROS_ERR if
    ///         there was an error either taking or returning the screen mutex.
    ///
    /// \b Example
    /// \code
    /// void initialize() {
    ///   screen_set_pen(COLOR_RED);
    /// }
    ///
    /// void opcontrol() {
    ///   int iter = 0;
    ///   while(1){
    ///     // This should print in red.
    ///     screen_print(TEXT_MEDIUM, 1, "%d", iter++);
    ///   }
    /// }
    /// \endcode
    pub fn screen_set_pen(color: u32) -> u32;

    /// Set the eraser color for erasing and the current background.
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// EACCESS - Another resource is currently trying to access the screen mutex.
    ///
    /// \param color    The background color to set (it is recommended to use values
    ///                     from the enum defined in colors.h)
    ///
    /// \return Returns 1 if the mutex was successfully returned, or
    /// PROS_ERR if there was an error either taking or returning the screen mutex.
    ///
    /// \b Example
    /// \code
    /// void initialize() {
    ///   screen_set_eraser(COLOR_RED);
    /// }
    ///
    /// void opcontrol() {
    ///   while(1){
    ///     // This should turn the screen red.
    ///     screen_erase();
    ///   }
    /// }
    /// \endcode
    pub fn screen_set_eraser(color: u32) -> u32;

    ///  Get the current pen color.
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// EACCESS - Another resource is currently trying to access the screen mutex.
    ///
    /// \return The current pen color in the form of a value from the enum defined
    ///         in colors.h, or PROS_ERR if there was an error taking or returning
    ///         the screen mutex.
    pub fn screen_get_pen() -> u32;

    /// Get the current eraser color.
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// EACCESS - Another resource is currently trying to access the screen mutex.
    ///
    /// \return The current eraser color in the form of a value from the enum
    ///         defined in colors.h, or PROS_ERR if there was an error taking or
    ///         returning the screen mutex.
    pub fn screen_get_eraser() -> u32;

    /// Clear display with eraser color
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// EACCESS - Another resource is currently trying to access the screen mutex.
    ///
    /// \return 1 if there were no errors, or PROS_ERR if an error occured
    ///         taking or returning the screen mutex.
    pub fn screen_erase() -> u32;

    /// Scroll lines on the display upwards.
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// EACCESS - Another resource is currently trying to access the screen mutex.
    ///
    ///
    /// \param start_line    The line from which scrolling will start
    /// \param lines            The number of lines to scroll up
    ///
    /// \return 1 if there were no errors, or PROS_ERR if an error occured
    ///         taking or returning the screen mutex.
    pub fn screen_scroll(start_line: i16, lines: i16) -> u32;

    /// Scroll lines within a region on the display
    ///
    /// This function behaves in the same way as `screen_scroll`, except that you
    /// specify a rectangular region within which to scroll lines instead of a start
    /// line.
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// EACCESS - Another resource is currently trying to access the screen mutex.
    ///
    /// \param x0, y0    The (x,y) coordinates of the first corner of the
    ///                         rectangular region
    /// \param x1, y1    The (x,y) coordinates of the second corner of the
    ///                         rectangular region
    /// \param lines     The number of lines to scroll upwards
    ///
    /// \return 1 if there were no errors, or PROS_ERR if an error occured
    ///           taking or returning the screen mutex.
    pub fn screen_scroll_area(x0: i16, y0: i16, x1: i16, y1: i16, lines: i16) -> u32;

    /// Copy a screen region (designated by a rectangle) from an off-screen buffer
    /// to the screen
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// EACCESS - Another resource is currently trying to access the screen mutex.
    ///
    /// \param x0, y0     The (x,y) coordinates of the first corner of the
    ///                         rectangular region of the screen
    /// \param x1, y1    The (x,y) coordinates of the second corner of the
    ///                         rectangular region of the screen
    /// \param buf        Off-screen buffer containing screen data
    /// \param stride    Off-screen buffer width in pixels, such that image size
    ///                         is stride-padding
    ///
    /// \return 1 if there were no errors, or PROS_ERR if an error occured
    ///         taking or returning the screen mutex.
    pub fn screen_copy_area(
        x0: i16,
        y0: i16,
        x1: i16,
        y1: i16,
        buf: *const u32,
        stride: i32,
    ) -> u32;

    /// Draw a single pixel on the screen using the current pen color
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// EACCESS - Another resource is currently trying to access the screen mutex.
    ///
    /// \param x, y     The (x,y) coordinates of the pixel
    ///
    /// \return 1 if there were no errors, or PROS_ERR if an error occured
    ///         taking or returning the screen mutex.
    pub fn screen_draw_pixel(x: i16, y: i16) -> u32;

    /// Erase a pixel from the screen (Sets the location)
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// EACCESS - Another resource is currently trying to access the screen mutex.
    ///
    /// \param x, y     The (x,y) coordinates of the erased
    ///
    /// \return 1 if there were no errors, or PROS_ERR if an error occured
    ///         taking or returning the screen mutex.
    pub fn screen_erase_pixel(x: i16, y: i16) -> u32;

    /// Draw a line on the screen using the current pen color
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// EACCESS - Another resource is currently trying to access the screen mutex.
    ///
    /// \param x0, y0    The (x, y) coordinates of the first point of the line
    /// \param x1, y1     The (x, y) coordinates of the second point of the line
    ///
    /// \return 1 if there were no errors, or PROS_ERR if an error occured
    ///         taking or returning the screen mutex.
    pub fn screen_draw_line(x0: i16, y0: i16, x1: i16, y1: i16) -> u32;

    /// Erase a line on the screen using the current eraser color
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// EACCESS - Another resource is currently trying to access the screen mutex.
    ///
    /// \param x0, y0    The (x, y) coordinates of the first point of the line
    /// \param x1, y1     The (x, y) coordinates of the second point of the line
    ///
    /// \return 1 if there were no errors, or PROS_ERR if an error occured
    ///         taking or returning the screen mutex.
    pub fn screen_erase_line(x0: i16, y0: i16, x1: i16, y1: i16) -> u32;

    /// Draw a rectangle on the screen using the current pen color
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// EACCESS - Another resource is currently trying to access the screen mutex.
    ///
    /// \param x0, y0     The (x,y) coordinates of the first point of the rectangle
    /// \param x1, y1     The (x,y) coordinates of the second point of the rectangle
    ///
    /// \return 1 if there were no errors, or PROS_ERR if an error occured
    ///         taking or returning the screen mutex.
    pub fn screen_draw_rect(x0: i16, y0: i16, x1: i16, y1: i16) -> u32;

    /// Erase a rectangle on the screen using the current eraser color
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// EACCESS - Another resource is currently trying to access the screen mutex.
    ///
    /// \param x0, y0     The (x,y) coordinates of the first point of the rectangle
    /// \param x1, y1     The (x,y) coordinates of the second point of the rectangle
    ///
    /// \return 1 if there were no errors, or PROS_ERR if an error occured
    ///         taking or returning the screen mutex.
    pub fn screen_erase_rect(x0: i16, y0: i16, x1: i16, y1: i16) -> u32;

    /// Fill a rectangular region of the screen using the current pen
    ///           color
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// EACCESS - Another resource is currently trying to access the screen mutex.
    ///
    /// \param x0, y0     The (x,y) coordinates of the first point of the rectangle
    /// \param x1, y1     The (x,y) coordinates of the second point of the rectangle
    ///
    /// \return 1 if there were no errors, or PROS_ERR if an error occured
    ///         taking or returning the screen mutex.
    pub fn screen_fill_rect(x0: i16, y0: i16, x1: i16, y1: i16) -> u32;

    /// Draw a circle on the screen using the current pen color
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// EACCESS - Another resource is currently trying to access the screen mutex.
    ///
    /// \param x, y     The (x,y) coordinates of the center of the circle
    /// \param r     The radius of the circle
    ///
    /// \return 1 if there were no errors, or PROS_ERR if an error occured
    ///         taking or returning the screen mutex.
    pub fn screen_draw_circle(x: i16, y: i16, radius: i16) -> u32;

    /// Erase a circle on the screen using the current eraser color
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// EACCESS - Another resource is currently trying to access the screen mutex.
    ///
    /// \param x, y     The (x,y) coordinates of the center of the circle
    /// \param r     The radius of the circle
    ///
    /// \return 1 if there were no errors, or PROS_ERR if an error occured
    ///         taking or returning the screen mutex.
    pub fn screen_erase_circle(x: i16, y: i16, radius: i16) -> u32;

    /// Fill a circular region of the screen using the current pen
    ///           color
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// EACCESS - Another resource is currently trying to access the screen mutex.
    ///
    /// \param x, y     The (x,y) coordinates of the center of the circle
    /// \param r     The radius of the circle
    ///
    /// \return 1 if there were no errors, or PROS_ERR if an error occured
    ///         taking or returning the screen mutex.
    pub fn screen_fill_circle(x: i16, y: i16, radius: i16) -> u32;

    /// Print a formatted string to the screen on the specified line
    ///
    /// Will default to a medium sized font by default if invalid txt_fmt is given.
    ///
    /// \param txt_fmt Text format enum that determines if the text is medium, large, medium_center, or large_center. (DOES
    /// NOT SUPPORT SMALL) \param line The line number on which to print \param text  Format string \param ...  Optional list
    /// of arguments for the format string
    ///
    /// \return 1 if there were no errors, or PROS_ERR if an error occured
    ///          taking or returning the screen mutex.
    pub fn screen_print(
        txt_fmt: text_format_e_t,
        line: i16,
        text: *const core::ffi::c_char,
        ...
    ) -> u32;

    /// Print a formatted string to the screen at the specified point
    ///
    /// Will default to a medium sized font by default if invalid txt_fmt is given.
    ///
    /// Text formats medium_center and large_center will default to medium and large respectively.
    ///
    /// \param txt_fmt Text format enum that determines if the text is small, medium, or large.
    /// \param x The y coordinate of the top left corner of the string
    /// \param y The x coordinate of the top left corner of the string
    /// \param text  Format string
    /// \param ...  Optional list of arguments for the format string
    ///
    ///  \return 1 if there were no errors, or PROS_ERR if an error occured
    ///          taking or returning the screen mutex.
    pub fn screen_print_at(
        txt_fmt: text_format_e_t,
        x: i16,
        y: i16,
        text: *const core::ffi::c_char,
        ...
    ) -> u32;

    /// Print a formatted string to the screen on the specified line
    ///
    /// Same as `display_printf` except that this uses a `va_list` instead of the
    /// ellipsis operator so this can be used by other functions.
    ///
    /// Will default to a medium sized font by default if invalid txt_fmt is given.
    /// Exposed mostly for writing libraries and custom functions.
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// EACCESS - Another resource is currently trying to access the screen mutex.
    ///
    /// \param txt_fmt Text format enum that determines if the text is medium, large, medium_center, or large_center. (DOES
    /// NOT SUPPORT SMALL) \param line The line number on which to print \param text  Format string \param args List of
    /// arguments for the format string
    ///
    /// \return 1 if there were no errors, or PROS_ERR if an error occured
    ///          while taking or returning the screen mutex.
    pub fn screen_vprintf(
        txt_fmt: text_format_e_t,
        line: i16,
        text: *const core::ffi::c_char,
        ...
    ) -> u32;

    /// Gets the touch status of the last touch of the screen.
    ///
    /// \return The last_touch_e_t enum specifier that indicates the last touch status of the screen (E_TOUCH_EVENT_RELEASE,
    /// E_TOUCH_EVENT_PRESS, or E_TOUCH_EVENT_PRESS_AND_HOLD). This will be released by default if no action was taken. If an
    /// error occured, the screen_touch_status_s_t will have its last_touch_e_t enum specifier set to E_TOUCH_ERR, and other
    /// values set to -1.
    pub fn screen_touch_status() -> screen_touch_status_s_t;

    /// Assigns a callback function to be called when a certain touch event happens.
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// EACCESS - Another resource is currently trying to access the screen mutex.
    ///
    /// \param cb Function pointer to callback when event type happens
    /// \param event_type Touch event that will trigger the callback.
    ///
    /// \return 1 if there were no errors, or PROS_ERR if an error occured
    ///          while taking or returning the screen mutex.
    pub fn screen_touch_callback(cb: touch_event_cb_fn_t, event_type: last_touch_e_t) -> u32;

    /// Display a fatal error to the built-in LCD/touch screen.
    ///
    /// This function is intended to be used when the integrity of the RTOS cannot be
    /// trusted. No thread-safety mechanisms are used and this function only relies
    /// on the use of the libv5rts.
    pub fn display_fatal_error(text: *const c_char);
}
