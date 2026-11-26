//! Protocol handlers: what the compositor does when clients ask for things.

use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    delegate_compositor, delegate_data_device, delegate_layer_shell, delegate_output,
    delegate_seat, delegate_session_lock, delegate_shm, delegate_xdg_shell,
    desktop::{
        LayerSurface, PopupKind, Window, WindowSurfaceType, find_popup_root_surface,
        get_popup_toplevel_coords, layer_map_for_output,
    },
    input::{Seat, SeatHandler, SeatState, pointer::CursorImageStatus},
    output::Output,
    reexports::wayland_server::{
        Client, Resource,
        protocol::{
            wl_buffer::WlBuffer, wl_output::WlOutput, wl_seat::WlSeat, wl_surface::WlSurface,
        },
    },
    utils::Serial,
    wayland::{
        buffer::BufferHandler,
        compositor::{
            CompositorClientState, CompositorHandler, CompositorState, get_parent,
            is_sync_subsurface, with_states,
        },
        output::OutputHandler,
        selection::{
            SelectionHandler,
            data_device::{
                ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler,
                set_data_device_focus,
            },
        },
        session_lock::{LockSurface, SessionLockHandler, SessionLockManagerState, SessionLocker},
        shell::{
            wlr_layer::{
                Layer, LayerSurface as WlrLayerSurface, LayerSurfaceData, WlrLayerShellHandler,
                WlrLayerShellState,
            },
            xdg::{
                PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
                XdgToplevelSurfaceData,
            },
        },
        shm::{ShmHandler, ShmState},
    },
};

use crate::state::{ClientState, State};

// Surfaces and buffers

impl CompositorHandler for State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client
            .get_data::<ClientState>()
            .expect("every client is inserted with ClientState")
            .compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);

        if !is_sync_subsurface(surface) {
            let mut root = surface.clone();
            while let Some(parent) = get_parent(&root) {
                root = parent;
            }
            if let Some(window) = self.window_for(&root) {
                window.on_commit();
                // The window may have drawn its first frame just now.
                self.refresh_focus();
            }
        }

        self.commit_toplevel(surface);
        self.commit_popup(surface);
        self.commit_layer(surface);
    }
}

impl State {
    fn window_for(&self, surface: &WlSurface) -> Option<Window> {
        self.space
            .elements()
            .find(|window| {
                window
                    .toplevel()
                    .is_some_and(|toplevel| toplevel.wl_surface() == surface)
            })
            .cloned()
    }

    /// Sends the first configure to a new app window.
    fn commit_toplevel(&mut self, surface: &WlSurface) {
        let Some(window) = self.window_for(surface) else {
            return;
        };
        let sent = with_states(surface, |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .and_then(|data| data.lock().ok().map(|data| data.initial_configure_sent))
                .unwrap_or(true)
        });
        if !sent && let Some(toplevel) = window.toplevel() {
            toplevel.send_configure();
        }
    }

    fn commit_popup(&mut self, surface: &WlSurface) {
        self.popups.commit(surface);
        if let Some(PopupKind::Xdg(popup)) = self.popups.find_popup(surface)
            && !popup.is_initial_configure_sent()
        {
            // The first configure is always allowed, so an error here would
            // mean the popup is already gone.
            let _ = popup.send_configure();
        }
    }

    /// Arranges layer surfaces and sends the first configure to a new one.
    fn commit_layer(&mut self, surface: &WlSurface) {
        let Some(output) = self.output() else { return };
        let mut layers = layer_map_for_output(&output);
        let Some(layer) = layers
            .layer_for_surface(surface, WindowSurfaceType::TOPLEVEL)
            .cloned()
        else {
            return;
        };

        // Arrange before the first configure so it carries the right size.
        let changed = layers.arrange();
        let sent = with_states(surface, |states| {
            states
                .data_map
                .get::<LayerSurfaceData>()
                .and_then(|data| data.lock().ok().map(|data| data.initial_configure_sent))
                .unwrap_or(true)
        });
        if !sent {
            layer.layer_surface().send_configure();
        }
        drop(layers);

        if changed {
            // A layer surface reserved or released space, so apps get a new size.
            for window in self.space.elements().cloned().collect::<Vec<_>>() {
                self.fit_window(&window, false);
            }
        }
    }
}

impl BufferHandler for State {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {}
}

impl ShmHandler for State {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

delegate_compositor!(State);
delegate_shm!(State);

// Seat, clipboard, output

impl SeatHandler for State {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: CursorImageStatus) {}

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
        let client = focused.and_then(|surface| self.display_handle.get_client(surface.id()).ok());
        set_data_device_focus(&self.display_handle, seat, client);
    }
}

impl SelectionHandler for State {
    type SelectionUserData = ();
}

impl DataDeviceHandler for State {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

impl ClientDndGrabHandler for State {}
impl ServerDndGrabHandler for State {}
impl OutputHandler for State {}

delegate_seat!(State);
delegate_data_device!(State);
delegate_output!(State);

// App windows

impl XdgShellHandler for State {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = Window::new_wayland_window(surface);
        // The window gets the keyboard once it has drawn its first frame.
        self.fit_window(&window, true);
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        if let Some(window) = self.window_for(surface.wl_surface()) {
            self.space.unmap_elem(&window);
        }
        self.refresh_focus();
        self.refresh_pointer_focus();
    }

    fn new_popup(&mut self, surface: PopupSurface, _positioner: PositionerState) {
        self.unconstrain_popup(&surface);
        let _ = self.popups.track_popup(PopupKind::Xdg(surface));
    }

    fn reposition_request(
        &mut self,
        surface: PopupSurface,
        positioner: PositionerState,
        token: u32,
    ) {
        surface.with_pending_state(|state| {
            state.geometry = positioner.get_geometry();
            state.positioner = positioner;
        });
        self.unconstrain_popup(&surface);
        surface.send_repositioned(token);
    }

    // Popup grabs are not implemented yet. Menus still open and close on click.
    fn grab(&mut self, _surface: PopupSurface, _seat: WlSeat, _serial: Serial) {}
}

impl State {
    /// Keeps a popup inside the output.
    fn unconstrain_popup(&self, popup: &PopupSurface) {
        let kind = PopupKind::Xdg(popup.clone());
        let Ok(root) = find_popup_root_surface(&kind) else {
            return;
        };
        let (Some(window), Some(output)) = (self.window_for(&root), self.output()) else {
            return;
        };
        let (Some(mut target), Some(window_geometry)) = (
            self.space.output_geometry(&output),
            self.space.element_geometry(&window),
        ) else {
            return;
        };
        target.loc -= get_popup_toplevel_coords(&kind);
        target.loc -= window_geometry.loc;
        popup.with_pending_state(|state| {
            state.geometry = state.positioner.get_unconstrained_geometry(target);
        });
    }
}

delegate_xdg_shell!(State);

// The home screen and other shell surfaces

impl WlrLayerShellHandler for State {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.layer_shell_state
    }

    fn new_layer_surface(
        &mut self,
        surface: WlrLayerSurface,
        output: Option<WlOutput>,
        _layer: Layer,
        namespace: String,
    ) {
        let Some(output) = output
            .as_ref()
            .and_then(Output::from_resource)
            .or_else(|| self.output())
        else {
            return;
        };
        let mut layers = layer_map_for_output(&output);
        if let Err(error) = layers.map_layer(&LayerSurface::new(surface, namespace)) {
            tracing::warn!(%error, "could not map a layer surface");
        }
    }

    fn layer_destroyed(&mut self, surface: WlrLayerSurface) {
        let Some(output) = self.output() else { return };
        let mut layers = layer_map_for_output(&output);
        let layer = layers
            .layers()
            .find(|layer| layer.layer_surface() == &surface)
            .cloned();
        if let Some(layer) = layer {
            layers.unmap_layer(&layer);
        }
    }
}

delegate_layer_shell!(State);

// The lock screen

impl SessionLockHandler for State {
    fn lock_state(&mut self) -> &mut SessionLockManagerState {
        &mut self.session_lock_state
    }

    fn lock(&mut self, confirmation: SessionLocker) {
        self.check_lock_holder();
        if !self.session.request_lock() {
            // Dropping the confirmation tells the client it was refused.
            tracing::warn!("refused a second lock screen");
            return;
        }
        self.lock_holder = Some(confirmation.ext_session_lock().clone());
        self.lock_surface = None;
        // From here on only the lock surface is drawn or gets input, so the
        // lock can be confirmed at once.
        confirmation.lock();
        self.refresh_focus();
        // Move pointer focus now, so no app keeps receiving input.
        self.refresh_pointer_focus();
    }

    fn unlock(&mut self) {
        self.session.unlock();
        self.lock_holder = None;
        self.lock_surface = None;
        self.refresh_focus();
        self.refresh_pointer_focus();
    }

    fn new_surface(&mut self, surface: LockSurface, _output: WlOutput) {
        self.lock_surface = Some(surface);
        self.configure_lock_surface();
        self.refresh_focus();
    }
}

delegate_session_lock!(State);
