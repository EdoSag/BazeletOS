//! Input from the backend, routed to the right surface.

use smithay::{
    backend::input::{
        AbsolutePositionEvent, Axis, ButtonState, Event, InputBackend, InputEvent, KeyState,
        KeyboardKeyEvent, PointerAxisEvent, PointerButtonEvent,
    },
    input::{
        keyboard::{FilterResult, Keysym},
        pointer::{AxisFrame, ButtonEvent, MotionEvent},
    },
    utils::SERIAL_COUNTER,
};

use crate::state::State;

/// Something the compositor does itself instead of passing a key on.
enum Shortcut {
    /// Close the app on top: Ctrl+Alt+Backspace.
    CloseApp,
}

impl State {
    /// Handles one input event from the backend.
    pub fn process_input_event<I: InputBackend>(&mut self, event: InputEvent<I>) {
        match event {
            InputEvent::Keyboard { event } => self.on_key::<I>(event),
            InputEvent::PointerMotionAbsolute { event } => {
                let Some(output) = self.output() else { return };
                let Some(geometry) = self.space.output_geometry(&output) else {
                    return;
                };
                let location = event.position_transformed(geometry.size) + geometry.loc.to_f64();
                let under = self.surface_under(location);
                let Some(pointer) = self.seat.get_pointer() else {
                    return;
                };
                pointer.motion(
                    self,
                    under,
                    &MotionEvent {
                        location,
                        serial: SERIAL_COUNTER.next_serial(),
                        time: event.time_msec(),
                    },
                );
                pointer.frame(self);
            }
            InputEvent::PointerButton { event } => {
                let Some(pointer) = self.seat.get_pointer() else {
                    return;
                };
                if event.state() == ButtonState::Pressed && !pointer.is_grabbed() {
                    // A window may have opened or closed under a pointer that
                    // did not move, so look again at what is under it. The
                    // click must go to the surface the user can see.
                    self.refresh_pointer_focus();
                    self.raise_window_under_pointer();
                }
                pointer.button(
                    self,
                    &ButtonEvent {
                        button: event.button_code(),
                        state: event.state(),
                        serial: SERIAL_COUNTER.next_serial(),
                        time: event.time_msec(),
                    },
                );
                pointer.frame(self);
            }
            InputEvent::PointerAxis { event } => {
                let Some(pointer) = self.seat.get_pointer() else {
                    return;
                };
                let mut frame = AxisFrame::new(event.time_msec()).source(event.source());
                for axis in [Axis::Horizontal, Axis::Vertical] {
                    let discrete = event.amount_v120(axis);
                    let amount = event
                        .amount(axis)
                        .unwrap_or_else(|| discrete.unwrap_or(0.0) * 15.0 / 120.0);
                    if amount != 0.0 {
                        frame = frame.value(axis, amount);
                        if let Some(discrete) = discrete {
                            frame = frame.v120(axis, discrete as i32);
                        }
                    }
                }
                pointer.axis(self, frame);
                pointer.frame(self);
            }
            _ => {}
        }
    }

    fn on_key<I: InputBackend>(&mut self, event: I::KeyboardKeyEvent) {
        let Some(keyboard) = self.seat.get_keyboard() else {
            return;
        };
        let locked = self.session.is_locked();
        let pressed = event.state() == KeyState::Pressed;
        let shortcut = keyboard.input(
            self,
            event.key_code(),
            event.state(),
            SERIAL_COUNTER.next_serial(),
            Event::time_msec(&event),
            |_, modifiers, key| {
                // No shortcut works while the session is locked.
                if !locked
                    && pressed
                    && modifiers.ctrl
                    && modifiers.alt
                    && key.modified_sym() == Keysym::BackSpace
                {
                    FilterResult::Intercept(Shortcut::CloseApp)
                } else {
                    FilterResult::Forward
                }
            },
        );
        match shortcut {
            Some(Shortcut::CloseApp) => self.close_top_window(),
            None => {}
        }
    }

    /// Brings the clicked app to the top and gives it the keyboard.
    fn raise_window_under_pointer(&mut self) {
        if self.session.is_locked() {
            return;
        }
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        if let Some((window, _)) = self.space.element_under(pointer.current_location()) {
            let window = window.clone();
            self.space.raise_element(&window, true);
            for window in self.space.elements() {
                if let Some(toplevel) = window.toplevel() {
                    toplevel.send_pending_configure();
                }
            }
        }
        self.refresh_focus();
    }

    /// Gives the pointer to whatever is under it now, under the current
    /// rules. Called when what is on screen changed without the pointer
    /// moving: on lock and unlock, when a window closes, and before a click.
    pub fn refresh_pointer_focus(&mut self) {
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        let location = pointer.current_location();
        let under = self.surface_under(location);
        pointer.motion(
            self,
            under,
            &MotionEvent {
                location,
                serial: SERIAL_COUNTER.next_serial(),
                time: self.start_time.elapsed().as_millis() as u32,
            },
        );
        pointer.frame(self);
    }
}
