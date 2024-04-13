use alloc::{rc::Rc, vec::Vec};
use core::{cell::RefCell, time::Duration};

use slint::{
    platform::{
        software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
        Platform, PointerEventButton, WindowEvent,
    },
    LogicalPosition, PhysicalPosition, Rgb8Pixel,
};
use vexide_core::time::Instant;
use vexide_devices::{color::Rgb, Screen};

pub struct V5Platform {
    start: Instant,
    window: Rc<MinimalSoftwareWindow>,
    screen: vexide_devices::Screen,

    buffer: RefCell<
        [Rgb8Pixel; Screen::HORIZONTAL_RESOLUTION as usize * Screen::VERTICAL_RESOLUTION as usize],
    >,
}
impl V5Platform {
    pub fn new(screen: vexide_devices::Screen, window: Rc<MinimalSoftwareWindow>) -> Self {
        Self {
            start: Instant::now(),
            window,
            screen,
            buffer: RefCell::new(
                [Rgb8Pixel::new(0, 0, 0);
                    Screen::HORIZONTAL_RESOLUTION as usize * Screen::VERTICAL_RESOLUTION as usize],
            ),
        }
    }

    fn get_touch_event(&self) -> WindowEvent {
        let event = self.screen.touch_status();
        let physical_pos = PhysicalPosition::new(event.x as _, event.y as _);
        let position = LogicalPosition::from_physical(physical_pos, 1.0);
        match event.state {
            vexide_devices::screen::TouchState::Released => WindowEvent::PointerReleased {
                position,
                button: PointerEventButton::Left,
            },
            vexide_devices::screen::TouchState::Pressed => WindowEvent::PointerPressed {
                position,
                button: PointerEventButton::Left,
            },
            vexide_devices::screen::TouchState::Held => WindowEvent::PointerMoved { position },
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
        self.start.elapsed()
    }
    fn run_event_loop(&self) -> Result<(), slint::PlatformError> {
        loop {
            slint::platform::update_timers_and_animations();

            self.window.draw_if_needed(|renderer| {
                let mut buf = *self.buffer.borrow_mut();
                renderer.render(&mut buf, Screen::HORIZONTAL_RESOLUTION as _);

                let mut buf: Vec<u32> = buf
                    .into_iter()
                    .map(|p| Rgb::new(p.r, p.g, p.b).into())
                    .collect();
                unsafe {
                    vex_sdk::vexDisplayCopyRect(
                        0,
                        0x20,
                        Screen::HORIZONTAL_RESOLUTION as _,
                        Screen::VERTICAL_RESOLUTION as i32 + 0x20,
                        buf.as_mut_ptr(),
                        Screen::HORIZONTAL_RESOLUTION as _,
                    )
                }
            });

            self.window.dispatch_event(self.get_touch_event());

            if !self.window.has_active_animations() {
                vexide_async::block_on(vexide_async::sleep(Duration::from_millis(1)));
            }
        }
    }
}
