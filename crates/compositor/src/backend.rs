//! The nested backend: the compositor runs as a window on a desktop.
//!
//! This is how the system is developed and tested before it runs on a phone.

use std::time::Duration;

use smithay::{
    backend::{
        renderer::{
            damage::OutputDamageTracker,
            element::{Kind, surface::WaylandSurfaceRenderElement},
            gles::GlesRenderer,
        },
        winit::{self, WinitEvent},
    },
    desktop::{layer_map_for_output, space::render_output, utils::send_frames_surface_tree},
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{
        calloop::LoopHandle,
        winit::{dpi::LogicalSize, window::Window as HostWindow},
    },
    utils::{Rectangle, Transform},
};

use crate::{Loop, state::State};

/// The size of the window, close to a phone screen in portrait.
const WINDOW_SIZE: (f64, f64) = (400.0, 800.0);
/// What is shown where nothing is drawn, and while the session is locked
/// without a lock screen.
const BACKGROUND: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

/// Opens the window, creates the output and starts drawing.
pub fn init(
    handle: &LoopHandle<'static, Loop>,
    state: &mut State,
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut backend, events) = winit::init_from_attributes::<GlesRenderer>(
        HostWindow::default_attributes()
            .with_inner_size(LogicalSize::new(WINDOW_SIZE.0, WINDOW_SIZE.1))
            .with_title(branding::PRODUCT_NAME),
    )?;

    let mode = Mode {
        size: backend.window_size(),
        refresh: 60_000,
    };
    let output = Output::new(
        "nested".to_string(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: branding::PRODUCT_NAME.into(),
            model: "Nested".into(),
        },
    );
    let _global = output.create_global::<State>(&state.display_handle);
    // The window's framebuffer is upside down compared to what clients draw.
    output.change_current_state(
        Some(mode),
        Some(Transform::Flipped180),
        None,
        Some((0, 0).into()),
    );
    output.set_preferred(mode);
    state.space.map_output(&output, (0, 0));

    let mut damage_tracker = OutputDamageTracker::from_output(&output);

    handle.insert_source(events, move |event, _, data| {
        let state = &mut data.state;
        match event {
            WinitEvent::Resized { size, .. } => {
                output.change_current_state(
                    Some(Mode {
                        size,
                        refresh: 60_000,
                    }),
                    None,
                    None,
                    None,
                );
                state.output_resized();
            }
            WinitEvent::Input(event) => state.process_input_event(event),
            WinitEvent::Redraw => {
                state.check_lock_holder();
                let size = backend.window_size();
                let rendered = match backend.bind() {
                    Ok((renderer, mut framebuffer)) => draw(
                        state,
                        &output,
                        renderer,
                        &mut framebuffer,
                        &mut damage_tracker,
                    ),
                    Err(error) => Err(error.to_string()),
                };
                match rendered {
                    Ok(()) => {
                        if let Err(error) = backend.submit(Some(&[Rectangle::from_size(size)])) {
                            tracing::warn!(%error, "could not show a frame");
                        }
                    }
                    Err(error) => tracing::warn!(%error, "could not draw a frame"),
                }

                send_frame_callbacks(state, &output);
                state.space.refresh();
                state.popups.cleanup();
                layer_map_for_output(&output).cleanup();
                let _ = state.display_handle.flush_clients();

                backend.window().request_redraw();
            }
            WinitEvent::CloseRequested => state.loop_signal.stop(),
            _ => {}
        }
    })?;

    Ok(())
}

/// Draws one frame.
///
/// While the session is locked, the lock screen is the only thing drawn.
/// Apps and the home screen are not part of the frame at all.
fn draw(
    state: &State,
    output: &Output,
    renderer: &mut GlesRenderer,
    framebuffer: &mut <GlesRenderer as smithay::backend::renderer::RendererSuper>::Framebuffer<'_>,
    damage_tracker: &mut OutputDamageTracker,
) -> Result<(), String> {
    if state.session.is_locked() {
        let elements: Vec<WaylandSurfaceRenderElement<GlesRenderer>> = state
            .lock_surface
            .as_ref()
            .filter(|surface| surface.alive())
            .map(|surface| {
                smithay::backend::renderer::element::surface::render_elements_from_surface_tree(
                    renderer,
                    surface.wl_surface(),
                    (0, 0),
                    1.0,
                    1.0,
                    Kind::Unspecified,
                )
            })
            .unwrap_or_default();
        damage_tracker
            .render_output(renderer, framebuffer, 0, &elements, BACKGROUND)
            .map(|_| ())
            .map_err(|error| error.to_string())
    } else {
        render_output::<_, WaylandSurfaceRenderElement<GlesRenderer>, _, _>(
            output,
            renderer,
            framebuffer,
            1.0,
            0,
            [&state.space],
            &[],
            damage_tracker,
            BACKGROUND,
        )
        .map(|_| ())
        .map_err(|error| error.to_string())
    }
}

/// Tells clients a frame was shown, so they draw the next one.
fn send_frame_callbacks(state: &State, output: &Output) {
    let time = state.start_time.elapsed();
    let throttle = Some(Duration::ZERO);

    if let Some(surface) = state
        .lock_surface
        .as_ref()
        .filter(|surface| surface.alive())
    {
        send_frames_surface_tree(surface.wl_surface(), output, time, throttle, |_, _| {
            Some(output.clone())
        });
    }
    // Apps and the home screen are not drawn while locked, so they are not
    // asked for frames either.
    if state.session.is_locked() {
        return;
    }
    for window in state.space.elements() {
        window.send_frame(output, time, throttle, |_, _| Some(output.clone()));
    }
    for layer in layer_map_for_output(output).layers() {
        layer.send_frame(output, time, throttle, |_, _| Some(output.clone()));
    }
}
