//! Panic handler implementation for [`vexide`](https://crates.io/crates/vexide).
//! Supports printing a backtrace when running in the simulator.
//! If the `display_panics` feature is enabled, it will also display the panic message on the V5 Brain display.

#![no_std]

extern crate alloc;

use alloc::{format, string::String};

use vexide_core::eprintln;
#[cfg(feature = "display_panics")]
use vexide_devices::Screen;

#[cfg(target_arch = "wasm32")]
extern "C" {
    /// Prints a backtrace to the debug console
    fn sim_log_backtrace();
}

/// Draw an error box to the screen.
///
/// This function is internally used by the vexide panic handler for displaying
/// panic messages graphically before exiting.
#[cfg(feature = "display_panics")]
fn draw_error(
    screen: &mut vexide_devices::screen::Screen,
    msg: &str,
) -> Result<(), vexide_devices::screen::ScreenError> {
    const ERROR_BOX_MARGIN: i16 = 16;
    const ERROR_BOX_PADDING: i16 = 16;
    const LINE_MAX_WIDTH: usize = 52;

    let error_box_rect = vexide_devices::screen::Rect::new(
        ERROR_BOX_MARGIN,
        ERROR_BOX_MARGIN,
        Screen::HORIZONTAL_RESOLUTION - ERROR_BOX_MARGIN,
        Screen::VERTICAL_RESOLUTION - ERROR_BOX_MARGIN,
    );

    screen.fill(&error_box_rect, vexide_devices::color::Rgb::RED)?;
    screen.stroke(&error_box_rect, vexide_devices::color::Rgb::WHITE)?;

    let mut buffer = String::new();
    let mut line: i16 = 0;

    for (i, character) in msg.char_indices() {
        if !character.is_ascii_control() {
            buffer.push(character);
        }

        if character == '\n' || ((buffer.len() % LINE_MAX_WIDTH == 0) && (i > 0)) {
            screen.fill(
                &vexide_devices::screen::Text::new(
                    buffer.as_str(),
                    vexide_devices::screen::TextPosition::Point(
                        ERROR_BOX_MARGIN + ERROR_BOX_PADDING,
                        ERROR_BOX_MARGIN + ERROR_BOX_PADDING + (line * Screen::LINE_HEIGHT),
                    ),
                    vexide_devices::screen::TextFormat::Small,
                ),
                vexide_devices::color::Rgb::WHITE,
            )?;

            line += 1;
            buffer.clear();
        }
    }

    screen.fill(
        &vexide_devices::screen::Text::new(
            buffer.as_str(),
            vexide_devices::screen::TextPosition::Point(
                ERROR_BOX_MARGIN + ERROR_BOX_PADDING,
                ERROR_BOX_MARGIN + ERROR_BOX_PADDING + (line * Screen::LINE_HEIGHT),
            ),
            vexide_devices::screen::TextFormat::Small,
        ),
        vexide_devices::color::Rgb::WHITE,
    )?;

    Ok(())
}

#[panic_handler]
/// The panic handler for vexide.
pub fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    let current_task = vexide_core::task::current();

    let task_name = current_task.name().unwrap_or_else(|_| "<unknown>".into());

    // task 'User Initialization (PROS)' panicked at src/lib.rs:22:1:
    // panic message here
    let msg = format!("task '{task_name}' {info}");

    eprintln!("{msg}");

    unsafe {
        #[cfg(feature = "display_panics")]
        draw_error(&mut Screen::new(), &msg).unwrap_or_else(|err| {
            eprintln!("Failed to draw error message to screen: {err}");
        });

        #[cfg(target_arch = "wasm32")]
        sim_log_backtrace();

        pros_sys::exit(1);
    }
}
