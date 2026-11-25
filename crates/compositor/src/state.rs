//! The compositor's state and the rules that tie its parts together.

use std::{ffi::OsString, sync::Arc, time::Instant};

use smithay::{
    backend::renderer::utils::with_renderer_surface_state,
    desktop::{
        LayerSurface, PopupManager, Space, Window, WindowSurfaceType, layer_map_for_output,
        utils::under_from_surface_tree,
    },
    input::{Seat, SeatState},
    output::Output,
    reexports::{
        calloop::LoopSignal,
        wayland_protocols::{
            ext::session_lock::v1::server::ext_session_lock_v1::ExtSessionLockV1,
            xdg::shell::server::xdg_toplevel,
        },
        wayland_server::{
            Client, DisplayHandle, Resource,
            backend::{ClientData, ClientId, DisconnectReason},
            protocol::wl_surface::WlSurface,
        },
    },
    utils::{Logical, Point, Rectangle, SERIAL_COUNTER},
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        output::OutputManagerState,
        selection::data_device::DataDeviceState,
        session_lock::{LockSurface, SessionLockManagerState},
        shell::{
            wlr_layer::{Layer, WlrLayerShellState},
            xdg::XdgShellState,
        },
        shm::ShmState,
    },
};

use crate::session::Session;

/// Everything the compositor keeps between events.
pub struct State {
    /// When the compositor started. Frame times are measured from here.
    pub start_time: Instant,
    /// The name of the socket apps connect to.
    pub socket_name: OsString,
    /// Handle to the Wayland display.
    pub display_handle: DisplayHandle,
    /// Stops the event loop.
    pub loop_signal: LoopSignal,

    /// App windows, bottom to top.
    pub space: Space<Window>,
    /// Open popups, such as menus.
    pub popups: PopupManager,

    /// Whether the session is locked.
    pub session: Session,
    /// The lock object of the client that holds the lock.
    pub lock_holder: Option<ExtSessionLockV1>,
    /// The surface the lock screen draws on.
    pub lock_surface: Option<LockSurface>,

    /// Protocol state: `wl_compositor`.
    pub compositor_state: CompositorState,
    /// Protocol state: `xdg_wm_base`, used by apps.
    pub xdg_shell_state: XdgShellState,
    /// Protocol state: layer shell, used by the home screen.
    pub layer_shell_state: WlrLayerShellState,
    /// Protocol state: session lock, used by the lock screen.
    pub session_lock_state: SessionLockManagerState,
    /// Protocol state: shared memory buffers.
    pub shm_state: ShmState,
    /// Protocol state: outputs. Kept alive, never read.
    pub _output_manager_state: OutputManagerState,
    /// Protocol state: seats.
    pub seat_state: SeatState<State>,
    /// Protocol state: clipboard and drag and drop.
    pub data_device_state: DataDeviceState,
    /// The one seat: keyboard and pointer.
    pub seat: Seat<State>,
}

/// What the compositor knows about each connected client.
#[derive(Default)]
pub struct ClientState {
    /// Per-client state of `wl_compositor`.
    pub compositor_state: CompositorClientState,
    /// Whether this client is the system shell, which the compositor started
    /// itself. Only the shell may lock the session or use layer shell.
    pub is_shell: bool,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

/// Whether a client is the system shell.
pub fn is_shell(client: &Client) -> bool {
    client
        .get_data::<ClientState>()
        .is_some_and(|data| data.is_shell)
}

/// Whether a surface has a buffer attached, that is, has drawn something.
fn is_mapped(surface: &WlSurface) -> bool {
    with_renderer_surface_state(surface, |state| state.buffer().is_some()).unwrap_or(false)
}

impl State {
    /// Creates the state and the protocol globals.
    pub fn new(
        display_handle: DisplayHandle,
        loop_signal: LoopSignal,
        socket_name: OsString,
    ) -> Self {
        let dh = &display_handle;

        let compositor_state = CompositorState::new::<Self>(dh);
        let xdg_shell_state = XdgShellState::new::<Self>(dh);
        // Layer shell and session lock let a client cover the screen and
        // take input, so only the shell may see them.
        let layer_shell_state = WlrLayerShellState::new_with_filter::<Self, _>(dh, is_shell);
        let session_lock_state = SessionLockManagerState::new::<Self, _>(dh, is_shell);
        let shm_state = ShmState::new::<Self>(dh, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(dh);
        let data_device_state = DataDeviceState::new::<Self>(dh);

        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_wl_seat(dh, "seat0");
        seat.add_keyboard(Default::default(), 200, 25)
            .expect("the default keymap loads");
        seat.add_pointer();

        Self {
            start_time: Instant::now(),
            socket_name,
            display_handle,
            loop_signal,
            space: Space::default(),
            popups: PopupManager::default(),
            session: Session::new(),
            lock_holder: None,
            lock_surface: None,
            compositor_state,
            xdg_shell_state,
            layer_shell_state,
            session_lock_state,
            shm_state,
            _output_manager_state: output_manager_state,
            seat_state,
            data_device_state,
            seat,
        }
    }

    /// The one output. The nested backend creates it before any client runs.
    pub fn output(&self) -> Option<Output> {
        self.space.outputs().next().cloned()
    }

    /// The area apps may use: the output minus what layer surfaces reserve.
    pub fn app_area(&self) -> Option<Rectangle<i32, Logical>> {
        let output = self.output()?;
        let geometry = self.space.output_geometry(&output)?;
        let zone = layer_map_for_output(&output).non_exclusive_zone();
        Some(Rectangle::new(geometry.loc + zone.loc, zone.size))
    }

    /// Makes a window fill the app area, the way apps do on a phone.
    pub fn fit_window(&mut self, window: &Window, activate: bool) {
        let Some(area) = self.app_area() else { return };
        if let Some(toplevel) = window.toplevel() {
            toplevel.with_pending_state(|state| {
                state.size = Some(area.size);
                state.bounds = Some(area.size);
                state.states.set(xdg_toplevel::State::Maximized);
            });
            if toplevel.is_initial_configure_sent() {
                toplevel.send_pending_configure();
            }
        }
        self.space.map_element(window.clone(), area.loc, activate);
    }

    /// Re-applies sizes after the output changed size.
    pub fn output_resized(&mut self) {
        let Some(output) = self.output() else { return };
        layer_map_for_output(&output).arrange();
        for window in self.space.elements().cloned().collect::<Vec<_>>() {
            self.fit_window(&window, false);
        }
        self.configure_lock_surface();
    }

    /// Tells the lock screen how large its surface must be.
    pub fn configure_lock_surface(&self) {
        let (Some(surface), Some(output)) = (&self.lock_surface, self.output()) else {
            return;
        };
        let Some(size) = self.space.output_geometry(&output).map(|g| g.size) else {
            return;
        };
        surface.with_pending_state(|state| {
            state.size = Some((size.w.max(0) as u32, size.h.max(0) as u32).into());
        });
        surface.send_configure();
    }

    /// Notices when the lock screen went away without unlocking.
    pub fn check_lock_holder(&mut self) {
        if self
            .lock_holder
            .as_ref()
            .is_some_and(|lock| !lock.is_alive())
        {
            tracing::warn!("the lock screen went away; the session stays locked");
            self.lock_holder = None;
            self.lock_surface = None;
            self.session.holder_gone();
            self.refresh_focus();
        }
        if self
            .lock_surface
            .as_ref()
            .is_some_and(|surface| !surface.alive())
        {
            self.lock_surface = None;
        }
    }

    /// The surface that should have the keyboard right now.
    fn focus_target(&self) -> Option<WlSurface> {
        if self.session.is_locked() {
            return self
                .lock_surface
                .as_ref()
                .filter(|surface| surface.alive())
                .map(|surface| surface.wl_surface().clone());
        }
        // Only a window that has drawn something can take the keyboard. Some
        // clients crash if they get the keyboard before their first frame.
        self.space
            .elements()
            .rev()
            .filter_map(|window| window.toplevel())
            .map(|toplevel| toplevel.wl_surface())
            .find(|surface| is_mapped(surface))
            .cloned()
    }

    /// Gives the keyboard to the lock screen while locked, otherwise to the
    /// app on top.
    pub fn refresh_focus(&mut self) {
        let target = self.focus_target();
        let Some(keyboard) = self.seat.get_keyboard() else {
            return;
        };
        if keyboard.current_focus() != target {
            keyboard.set_focus(self, target, SERIAL_COUNTER.next_serial());
        }
    }

    /// The surface under a point, and where that surface is.
    ///
    /// While the session is locked this only ever returns the lock screen.
    pub fn surface_under(
        &self,
        point: Point<f64, Logical>,
    ) -> Option<(WlSurface, Point<f64, Logical>)> {
        if self.session.is_locked() {
            let surface = self.lock_surface.as_ref().filter(|s| s.alive())?;
            return under_from_surface_tree(
                surface.wl_surface(),
                point,
                (0, 0),
                WindowSurfaceType::ALL,
            )
            .map(|(surface, location)| (surface, location.to_f64()));
        }

        let output = self.output()?;
        let output_location = self.space.output_geometry(&output)?.loc;
        let layers = layer_map_for_output(&output);
        let layer_under = |layer: Layer| {
            let found: &LayerSurface = layers.layer_under(layer, point)?;
            let location = layers.layer_geometry(found)?.loc + output_location;
            found
                .surface_under(point - location.to_f64(), WindowSurfaceType::ALL)
                .map(|(surface, offset)| (surface, (offset + location).to_f64()))
        };

        layer_under(Layer::Overlay)
            .or_else(|| layer_under(Layer::Top))
            .or_else(|| {
                let (window, location) = self.space.element_under(point)?;
                window
                    .surface_under(point - location.to_f64(), WindowSurfaceType::ALL)
                    .map(|(surface, offset)| (surface, (offset + location).to_f64()))
            })
            .or_else(|| layer_under(Layer::Bottom))
            .or_else(|| layer_under(Layer::Background))
    }

    /// Asks the app on top to close.
    pub fn close_top_window(&mut self) {
        if let Some(toplevel) = self.space.elements().last().and_then(|w| w.toplevel()) {
            toplevel.send_close();
        }
    }

    /// Inserts a client that connected to the public socket.
    pub fn insert_app_client(&mut self, stream: std::os::unix::net::UnixStream) {
        if let Err(error) = self
            .display_handle
            .insert_client(stream, Arc::new(ClientState::default()))
        {
            tracing::warn!(%error, "could not add a client");
        }
    }
}
