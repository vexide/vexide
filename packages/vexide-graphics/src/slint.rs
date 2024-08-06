//! Slint platform implementation for the V5 Brain screen.

extern crate alloc;
use alloc::{boxed::Box, rc::Rc};
use vex_sdk::vexSystemHighResTimeGet;
use core::{cell::RefCell, time::Duration};

use slint::{
    platform::{
        software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
        Platform, PointerEventButton, WindowEvent,
    },
    LogicalPosition, PhysicalPosition, PhysicalSize, Rgb8Pixel,
};
use vexide_core::time::Instant;
use vexide_devices::display::{Display, Rect};

/// A Slint platform implementation for the V5 Brain screen.
pub struct V5Platform {
    start: u64,
    window: Rc<MinimalSoftwareWindow>,
    display: RefCell<Display>,
    display_pressed: RefCell<bool>,

    buffer: RefCell<
        [Rgb8Pixel;
            Display::HORIZONTAL_RESOLUTION as usize * Display::VERTICAL_RESOLUTION as usize],
    >,
}
impl V5Platform {
    /// Create a new [`V5Platform`] from a [`Display`].
    #[must_use]
    pub fn new(display: Display) -> Self {
        let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
        window.set_size(PhysicalSize::new(
            Display::HORIZONTAL_RESOLUTION as _,
            Display::VERTICAL_RESOLUTION as _,
        ));
        Self {
            start: unsafe { vexSystemHighResTimeGet() },
            window,
            display: RefCell::new(display),
            display_pressed: RefCell::new(false),
            #[allow(clippy::large_stack_arrays)] // we got plenty
            buffer: RefCell::new(
                [Rgb8Pixel::new(0, 0, 0);
                    Display::HORIZONTAL_RESOLUTION as usize * Display::VERTICAL_RESOLUTION as usize],
            ),
        }
    }

    fn get_touch_event(&self) -> WindowEvent {
        let event = self.display.borrow().touch_status();
        let physical_pos = PhysicalPosition::new(event.x.into(), event.y.into());
        let position = LogicalPosition::from_physical(physical_pos, 1.0);
        match event.state {
            vexide_devices::display::TouchState::Released => {
                *self.display_pressed.borrow_mut() = false;
                WindowEvent::PointerReleased {
                    position,
                    button: PointerEventButton::Left,
                }
            }
            vexide_devices::display::TouchState::Pressed => {
                if self.display_pressed.replace(true) {
                    WindowEvent::PointerMoved { position }
                } else {
                    WindowEvent::PointerPressed {
                        position,
                        button: PointerEventButton::Left,
                    }
                }
            }
            vexide_devices::display::TouchState::Held => WindowEvent::PointerMoved { position },
        }
    }
}

impl Platform for V5Platform {
    fn create_window_adapter(
        &self,
    ) -> Result<alloc::rc::Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }
    fn duration_since_start(&self) -> core::time::Duration {
        Duration::from_micros(self.start)
    }
    fn run_event_loop(&self) -> Result<(), slint::PlatformError> {
        loop {
            slint::platform::update_timers_and_animations();

            self.window.draw_if_needed(|renderer| {
                let mut buf = *self.buffer.borrow_mut();
                renderer.render(&mut buf, Display::HORIZONTAL_RESOLUTION as _);
                // Unwrap because the buffer is guaranteed to be the correct size
                self.display
                    .borrow_mut()
                    .draw_buffer(
                        Rect::from_dimensions(
                            [0, 0],
                            Display::HORIZONTAL_RESOLUTION as _,
                            Display::VERTICAL_RESOLUTION as _,
                        ),
                        buf,
                        Display::HORIZONTAL_RESOLUTION.into(),
                    )
                    .unwrap();
            });

            self.window.dispatch_event(self.get_touch_event());

            if !self.window.has_active_animations() {
                vexide_runtime::block_on(vexide_runtime::time::sleep(Duration::from_millis(1)));
            }
        }
    }
}

/// Sets the Slint platform to [`V5Platform`].
/// # Panics
/// Panics if the Slint platform is already set.
pub fn initialize_slint_platform(display: Display) {
    slint::platform::set_platform(Box::new(V5Platform::new(display)))
        .expect("Slint platform already set!");
}
