//! Runs a program's screens as Wayland surfaces.
//!
//! The toolkit (iced) lays out and draws each screen in software, and this
//! module shows the result on a Wayland surface and feeds pointer input back.
//! It exists because the toolkit's own window code cannot create the two
//! kinds of surface the system shell needs: a session lock surface and a
//! layer surface.
//!
//! Limits, to be lifted as later milestones need them: one output, no
//! keyboard or touch input, and every frame is drawn in full.

use std::{borrow::Cow, time::Duration};

use iced_core::{
    Color, Event, Font, Pixels, Point, Size, clipboard, mouse,
    renderer::{Headless, Style},
    theme::{Base, Palette},
    time::Instant,
    window,
};
use iced_runtime::{UserInterface, user_interface::Cache};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState, FrameCallbackData},
    delegate_dispatch2, delegate_registry,
    output::{OutputHandler, OutputState},
    reexports::{
        calloop::{
            EventLoop,
            timer::{TimeoutAction, Timer},
        },
        calloop_wayland_source::WaylandSource,
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        Capability, SeatHandler, SeatState,
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
    },
    session_lock::{
        SessionLock, SessionLockHandler, SessionLockState, SessionLockSurface,
        SessionLockSurfaceConfigure,
    },
    shell::{
        WaylandSurface,
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        xdg::{
            XdgShell,
            window::{Window, WindowConfigure, WindowDecorations, WindowHandler},
        },
    },
    shm::{Shm, ShmHandler, slot::SlotPool},
};
use wayland_client::{
    Connection, QueueHandle,
    globals::registry_queue_init,
    protocol::{
        wl_output::WlOutput, wl_pointer::WlPointer, wl_seat::WlSeat, wl_shm, wl_surface::WlSurface,
    },
};

/// The renderer every screen is drawn with.
pub type Renderer = iced_tiny_skia::Renderer;
/// The theme type of every screen.
pub type Theme = iced_core::Theme;
/// A piece of user interface that produces messages of type `M`.
pub type Element<'a, M> = iced_core::Element<'a, M, Theme, Renderer>;

/// The name of the bundled font family.
pub const FONT_FAMILY: &str = "Heebo";
/// The regular weight of the bundled font.
pub const FONT: Font = Font::with_name(FONT_FAMILY);
/// The bold weight of the bundled font.
pub const FONT_BOLD: Font = Font {
    weight: iced_core::font::Weight::Bold,
    ..Font::with_name(FONT_FAMILY)
};

const FONT_REGULAR_BYTES: &[u8] = include_bytes!("../../../assets/fonts/Heebo-Regular.ttf");
const FONT_BOLD_BYTES: &[u8] = include_bytes!("../../../assets/fonts/Heebo-Bold.ttf");
const TEXT_SIZE: f32 = 16.0;
/// The size of an app window before the compositor says otherwise.
const DEFAULT_WINDOW_SIZE: (u32, u32) = (400, 800);

/// The system's dark theme.
pub fn theme() -> Theme {
    Theme::custom(
        "System dark",
        Palette {
            background: Color::from_rgb8(0x10, 0x13, 0x18),
            text: Color::from_rgb8(0xf2, 0xf4, 0xf7),
            primary: Color::from_rgb8(0x4f, 0x9c, 0xf9),
            success: Color::from_rgb8(0x4c, 0xc3, 0x6b),
            warning: Color::from_rgb8(0xf5, 0x9e, 0x0b),
            danger: Color::from_rgb8(0xff, 0x6b, 0x6b),
        },
    )
}

/// Loads the bundled font and creates a renderer that uses it.
pub fn renderer() -> Renderer {
    {
        let mut fonts = iced_graphics::text::font_system()
            .write()
            .expect("the font system lock is not poisoned");
        fonts.load_font(Cow::Borrowed(FONT_REGULAR_BYTES));
        fonts.load_font(Cow::Borrowed(FONT_BOLD_BYTES));
    }
    Renderer::new(FONT, Pixels(TEXT_SIZE))
}

/// Lays out and draws one screen into RGBA pixels, without any display.
///
/// This is what the runtime does for every frame, and what tests use to
/// check a screen.
pub fn render<M>(
    renderer: &mut Renderer,
    theme: &Theme,
    element: Element<'_, M>,
    size: (u32, u32),
    scale: u32,
) -> Vec<u8> {
    let mut interface = UserInterface::build(
        element,
        Size::new(size.0 as f32, size.1 as f32),
        Cache::default(),
        renderer,
    );
    draw(&mut interface, renderer, theme, mouse::Cursor::Unavailable);
    drop(interface);
    screenshot(renderer, theme, size, scale)
}

fn draw<M>(
    interface: &mut UserInterface<'_, M, Theme, Renderer>,
    renderer: &mut Renderer,
    theme: &Theme,
    cursor: mouse::Cursor,
) {
    let mut ignored = Vec::new();
    let _ = interface.update(
        &[Event::Window(
            window::Event::RedrawRequested(Instant::now()),
        )],
        cursor,
        renderer,
        &mut clipboard::Null,
        &mut ignored,
    );
    interface.draw(
        renderer,
        theme,
        &Style {
            text_color: theme.base().text_color,
        },
        cursor,
    );
}

fn screenshot(renderer: &mut Renderer, theme: &Theme, size: (u32, u32), scale: u32) -> Vec<u8> {
    renderer.screenshot(
        Size::new(size.0 * scale, size.1 * scale),
        scale as f32,
        theme.base().background_color,
    )
}

/// Which surface a screen is drawn on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// The lock screen. Covers everything while the session is locked.
    Lock,
    /// The home screen. Sits behind all apps.
    Home,
    /// An ordinary app window.
    Window,
}

/// Something a program asks the runtime to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Lock the session and show the lock screen. Shell only.
    Lock,
    /// Unlock the session. Shell only.
    Unlock,
    /// Stop the program.
    Exit,
}

/// A program with one or more screens.
pub trait Program {
    /// What the screens send back when the user does something.
    type Message;

    /// Builds the screen for a surface.
    fn view(&self, role: Role) -> Element<'_, Self::Message>;

    /// Reacts to a message.
    fn update(&mut self, message: Self::Message) -> Vec<Action>;

    /// Called at the interval set in [`Config::tick`], for clocks and the like.
    fn tick(&mut self) {}
}

/// What kind of program to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// An app with one window.
    App,
    /// The system shell: a lock screen and a home screen. The session is
    /// locked as soon as the shell starts.
    Shell,
}

/// How to run a program.
#[derive(Debug, Clone)]
pub struct Config {
    /// App or shell.
    pub kind: Kind,
    /// The app ID, from the `branding` crate.
    pub app_id: String,
    /// The window title.
    pub title: String,
    /// How often to call [`Program::tick`].
    pub tick: Option<Duration>,
}

/// Why a program could not run.
pub type Error = Box<dyn std::error::Error>;

/// Runs a program until it exits or the compositor goes away.
pub fn run<P: Program + 'static>(program: P, config: Config) -> Result<(), Error> {
    let connection = Connection::connect_to_env()?;
    let (globals, mut queue) = registry_queue_init(&connection)?;
    let qh: QueueHandle<Runtime<P>> = queue.handle();

    let shm = Shm::bind(&globals, &qh)?;
    let mut runtime = Runtime {
        program,
        theme: theme(),
        renderer: renderer(),
        registry: RegistryState::new(&globals),
        outputs: OutputState::new(&globals, &qh),
        seats: SeatState::new(&globals, &qh),
        compositor: CompositorState::bind(&globals, &qh)?,
        pool: SlotPool::new(
            (DEFAULT_WINDOW_SIZE.0 * DEFAULT_WINDOW_SIZE.1 * 4) as usize,
            &shm,
        )?,
        shm,
        lock_manager: SessionLockState::new(&globals, &qh),
        lock: None,
        pointer: None,
        views: Vec::new(),
        exit: false,
    };

    // Learn about the outputs before creating surfaces on them.
    queue.roundtrip(&mut runtime)?;

    match config.kind {
        Kind::App => {
            let shell = XdgShell::bind(&globals, &qh)?;
            let surface = runtime.compositor.create_surface(&qh);
            let window = shell.create_window(surface, WindowDecorations::RequestServer, &qh);
            window.set_title(config.title.clone());
            window.set_app_id(config.app_id.clone());
            window.commit();
            runtime
                .views
                .push(View::new(Role::Window, Keep::Window(window)));
        }
        Kind::Shell => {
            // Lock first, so the home screen is never visible before the
            // lock screen is up.
            runtime.lock_session(&qh)?;
            let output = runtime.first_output()?;
            let layers = LayerShell::bind(&globals, &qh)?;
            let surface = runtime.compositor.create_surface(&qh);
            let layer = layers.create_layer_surface(
                &qh,
                surface,
                Layer::Background,
                Some(config.app_id.clone()),
                Some(&output),
            );
            layer.set_anchor(Anchor::all());
            layer.set_size(0, 0);
            layer.set_exclusive_zone(-1);
            layer.set_keyboard_interactivity(KeyboardInteractivity::None);
            layer.commit();
            runtime
                .views
                .push(View::new(Role::Home, Keep::Layer(layer)));
        }
    }

    let mut event_loop: EventLoop<'static, Runtime<P>> = EventLoop::try_new()?;
    WaylandSource::new(connection, queue).insert(event_loop.handle())?;
    if let Some(interval) = config.tick {
        event_loop.handle().insert_source(
            Timer::from_duration(interval),
            move |_, _, runtime| {
                runtime.program.tick();
                runtime.mark_all_dirty();
                TimeoutAction::ToDuration(interval)
            },
        )?;
    }

    while !runtime.exit {
        event_loop.dispatch(None, &mut runtime)?;
        runtime.pump(&qh)?;
    }
    Ok(())
}

/// The Wayland object that keeps a surface alive.
enum Keep {
    Lock(SessionLockSurface),
    Layer(LayerSurface),
    Window(Window),
}

/// One surface and the screen drawn on it.
struct View {
    role: Role,
    keep: Keep,
    /// Size in logical pixels. Zero until the compositor configures it.
    size: (u32, u32),
    scale: u32,
    cache: Cache,
    cursor: mouse::Cursor,
    events: Vec<Event>,
    dirty: bool,
    /// A frame was shown and the compositor has not yet asked for the next.
    waiting_for_frame: bool,
}

impl View {
    fn new(role: Role, keep: Keep) -> Self {
        Self {
            role,
            keep,
            size: (0, 0),
            scale: 1,
            cache: Cache::default(),
            cursor: mouse::Cursor::Unavailable,
            events: Vec::new(),
            dirty: false,
            waiting_for_frame: false,
        }
    }

    fn surface(&self) -> &WlSurface {
        match &self.keep {
            Keep::Lock(surface) => surface.wl_surface(),
            Keep::Layer(surface) => surface.wl_surface(),
            Keep::Window(surface) => surface.wl_surface(),
        }
    }

    fn resize(&mut self, size: (u32, u32)) {
        if size.0 > 0 && size.1 > 0 {
            self.size = size;
            self.dirty = true;
        }
    }
}

struct Runtime<P: Program> {
    program: P,
    theme: Theme,
    renderer: Renderer,

    registry: RegistryState,
    outputs: OutputState,
    seats: SeatState,
    compositor: CompositorState,
    shm: Shm,
    pool: SlotPool,
    lock_manager: SessionLockState,
    /// The session lock, while this program holds it.
    lock: Option<SessionLock>,
    pointer: Option<WlPointer>,

    views: Vec<View>,
    exit: bool,
}

impl<P: Program + 'static> Runtime<P> {
    fn first_output(&self) -> Result<WlOutput, Error> {
        self.outputs
            .outputs()
            .next()
            .ok_or_else(|| "the compositor has no output".into())
    }

    fn view_mut(&mut self, surface: &WlSurface) -> Option<&mut View> {
        self.views.iter_mut().find(|view| view.surface() == surface)
    }

    fn mark_all_dirty(&mut self) {
        for view in &mut self.views {
            view.dirty = true;
        }
    }

    /// Takes the session lock and creates the lock screen surface.
    fn lock_session(&mut self, qh: &QueueHandle<Self>) -> Result<(), Error> {
        if self.lock.is_some() {
            return Ok(());
        }
        let output = self.first_output()?;
        let lock = self.lock_manager.lock(qh)?;
        let surface = self.compositor.create_surface(qh);
        let lock_surface = lock.create_lock_surface(surface, &output, qh);
        self.views
            .push(View::new(Role::Lock, Keep::Lock(lock_surface)));
        self.lock = Some(lock);
        Ok(())
    }

    fn unlock_session(&mut self) {
        if let Some(lock) = self.lock.take() {
            lock.unlock();
        }
        self.views.retain(|view| view.role != Role::Lock);
    }

    /// Handles queued input, lets the program react, and draws what changed.
    fn pump(&mut self, qh: &QueueHandle<Self>) -> Result<(), Error> {
        let mut messages = Vec::new();
        for view in &mut self.views {
            if view.events.is_empty() || view.size.0 == 0 {
                view.events.clear();
                continue;
            }
            let events = std::mem::take(&mut view.events);
            let mut interface = UserInterface::build(
                self.program.view(view.role),
                Size::new(view.size.0 as f32, view.size.1 as f32),
                std::mem::take(&mut view.cache),
                &mut self.renderer,
            );
            let _ = interface.update(
                &events,
                view.cursor,
                &mut self.renderer,
                &mut clipboard::Null,
                &mut messages,
            );
            view.cache = interface.into_cache();
            // Hover and press states change how widgets look.
            view.dirty = true;
        }

        if !messages.is_empty() {
            self.mark_all_dirty();
        }
        for message in messages {
            for action in self.program.update(message) {
                match action {
                    Action::Lock => self.lock_session(qh)?,
                    Action::Unlock => self.unlock_session(),
                    Action::Exit => self.exit = true,
                }
            }
        }

        for index in 0..self.views.len() {
            let view = &self.views[index];
            if view.dirty && !view.waiting_for_frame && view.size.0 > 0 {
                self.draw_view(index, qh)?;
            }
        }
        Ok(())
    }

    fn draw_view(&mut self, index: usize, qh: &QueueHandle<Self>) -> Result<(), Error> {
        let view = &mut self.views[index];
        let mut interface = UserInterface::build(
            self.program.view(view.role),
            Size::new(view.size.0 as f32, view.size.1 as f32),
            std::mem::take(&mut view.cache),
            &mut self.renderer,
        );
        draw(&mut interface, &mut self.renderer, &self.theme, view.cursor);
        view.cache = interface.into_cache();
        let rgba = screenshot(&mut self.renderer, &self.theme, view.size, view.scale);

        let width = (view.size.0 * view.scale) as i32;
        let height = (view.size.1 * view.scale) as i32;
        let (buffer, canvas) =
            self.pool
                .create_buffer(width, height, width * 4, wl_shm::Format::Argb8888)?;
        // The renderer gives RGBA bytes; the buffer wants BGRA.
        for (target, source) in canvas.chunks_exact_mut(4).zip(rgba.chunks_exact(4)) {
            target.copy_from_slice(&[source[2], source[1], source[0], source[3]]);
        }

        let surface = view.surface();
        surface.set_buffer_scale(view.scale as i32);
        surface.damage_buffer(0, 0, width, height);
        surface.frame(qh, FrameCallbackData(surface.clone()));
        buffer.attach_to(surface)?;
        surface.commit();

        view.dirty = false;
        view.waiting_for_frame = true;
        Ok(())
    }
}

impl<P: Program + 'static> CompositorHandler for Runtime<P> {
    fn scale_factor_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        surface: &WlSurface,
        factor: i32,
    ) {
        if let Some(view) = self.view_mut(surface) {
            view.scale = factor.max(1) as u32;
            view.dirty = true;
        }
    }

    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlSurface,
        _: wayland_client::protocol::wl_output::Transform,
    ) {
    }

    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>, surface: &WlSurface, _time: u32) {
        if let Some(view) = self.view_mut(surface) {
            view.waiting_for_frame = false;
        }
    }

    fn surface_enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlSurface,
        _: &WlOutput,
    ) {
    }
    fn surface_leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlSurface,
        _: &WlOutput,
    ) {
    }
}

impl<P: Program + 'static> OutputHandler for Runtime<P> {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.outputs
    }

    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlOutput) {}
}

impl<P: Program + 'static> SeatHandler for Runtime<P> {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seats
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlSeat) {}

    fn new_capability(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        seat: WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer && self.pointer.is_none() {
            self.pointer = self.seats.get_pointer(qh, &seat).ok();
        }
    }

    fn remove_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer
            && let Some(pointer) = self.pointer.take()
        {
            pointer.release();
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlSeat) {}
}

/// Linux input codes for the mouse buttons.
fn mouse_button(code: u32) -> mouse::Button {
    match code {
        0x110 => mouse::Button::Left,
        0x111 => mouse::Button::Right,
        0x112 => mouse::Button::Middle,
        other => mouse::Button::Other(other as u16),
    }
}

impl<P: Program + 'static> PointerHandler for Runtime<P> {
    fn pointer_frame(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlPointer,
        events: &[PointerEvent],
    ) {
        for event in events {
            let Some(view) = self.view_mut(&event.surface) else {
                continue;
            };
            let position = Point::new(event.position.0 as f32, event.position.1 as f32);
            match event.kind {
                PointerEventKind::Enter { .. } | PointerEventKind::Motion { .. } => {
                    view.cursor = mouse::Cursor::Available(position);
                    view.events
                        .push(Event::Mouse(mouse::Event::CursorMoved { position }));
                }
                PointerEventKind::Leave { .. } => {
                    view.cursor = mouse::Cursor::Unavailable;
                    view.events.push(Event::Mouse(mouse::Event::CursorLeft));
                }
                PointerEventKind::Press { button, .. } => {
                    view.cursor = mouse::Cursor::Available(position);
                    view.events
                        .push(Event::Mouse(mouse::Event::ButtonPressed(mouse_button(
                            button,
                        ))));
                }
                PointerEventKind::Release { button, .. } => {
                    view.events
                        .push(Event::Mouse(mouse::Event::ButtonReleased(mouse_button(
                            button,
                        ))));
                }
                PointerEventKind::Axis {
                    horizontal,
                    vertical,
                    ..
                } => {
                    view.events.push(Event::Mouse(mouse::Event::WheelScrolled {
                        delta: mouse::ScrollDelta::Pixels {
                            x: -horizontal.absolute as f32,
                            y: -vertical.absolute as f32,
                        },
                    }));
                }
            }
        }
    }
}

impl<P: Program + 'static> SessionLockHandler for Runtime<P> {
    fn locked(&mut self, _: &Connection, _: &QueueHandle<Self>, _: SessionLock) {}

    fn finished(&mut self, _: &Connection, _: &QueueHandle<Self>, _: SessionLock) {
        // The compositor refused or ended the lock. A shell that cannot lock
        // must not keep running as if it had.
        self.exit = true;
    }

    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        surface: SessionLockSurface,
        configure: SessionLockSurfaceConfigure,
        _serial: u32,
    ) {
        if let Some(view) = self.view_mut(surface.wl_surface()) {
            view.resize(configure.new_size);
        }
    }
}

impl<P: Program + 'static> LayerShellHandler for Runtime<P> {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if let Some(view) = self.view_mut(layer.wl_surface()) {
            view.resize(configure.new_size);
        }
    }
}

impl<P: Program + 'static> WindowHandler for Runtime<P> {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &Window) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        window: &Window,
        configure: WindowConfigure,
        _serial: u32,
    ) {
        if let Some(view) = self.view_mut(window.wl_surface()) {
            let width = configure
                .new_size
                .0
                .map_or(DEFAULT_WINDOW_SIZE.0, |w| w.get());
            let height = configure
                .new_size
                .1
                .map_or(DEFAULT_WINDOW_SIZE.1, |h| h.get());
            view.resize((width, height));
        }
    }
}

impl<P: Program + 'static> ShmHandler for Runtime<P> {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl<P: Program + 'static> ProvidesRegistryState for Runtime<P> {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry
    }
    registry_handlers![OutputState, SeatState];
}

delegate_registry!(@<P: Program + 'static> Runtime<P>);
delegate_dispatch2!(@<P: Program + 'static> Runtime<P>);
