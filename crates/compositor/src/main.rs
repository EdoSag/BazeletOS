//! The Wayland compositor.
//!
//! For now it runs nested, as a window on a desktop. It starts locked, starts
//! the shell, and shows the shell's lock screen until the shell unlocks.

mod backend;
mod handlers;
mod input;
mod session;
mod shell_process;
mod state;

use std::{ffi::OsString, path::PathBuf, time::Duration};

use smithay::{
    reexports::{
        calloop::{
            EventLoop, Interest, Mode, PostAction,
            generic::Generic,
            timer::{TimeoutAction, Timer},
        },
        wayland_server::Display,
    },
    wayland::socket::ListeningSocketSource,
};

use crate::{shell_process::ShellProcess, state::State};

/// What the event loop hands to every callback.
pub struct Loop {
    state: State,
    display: Display<State>,
    shell: ShellProcess,
}

/// How often the compositor checks that the shell is still running.
const SHELL_CHECK: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn,compositor=info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let shell_program = shell_program()?;

    let mut event_loop: EventLoop<'static, Loop> = EventLoop::try_new()?;
    let handle = event_loop.handle();
    let mut display: Display<State> = Display::new()?;

    // The public socket. Apps connect here and get no special rights.
    let socket = ListeningSocketSource::new_auto()?;
    let socket_name = socket.socket_name().to_os_string();
    handle.insert_source(socket, |stream, _, data| {
        data.state.insert_app_client(stream)
    })?;

    // Wake up when a client sent something.
    let poll_fd = display.backend().poll_fd().try_clone_to_owned()?;
    handle.insert_source(
        Generic::new(poll_fd, Interest::READ, Mode::Level),
        |_, _, data| {
            data.display
                .dispatch_clients(&mut data.state)
                .map(|_| PostAction::Continue)
        },
    )?;

    let mut state = State::new(display.handle(), event_loop.get_signal(), socket_name);
    backend::init(&handle, &mut state)?;
    tracing::info!(socket = ?state.socket_name, "{} is running", branding::PRODUCT_NAME);

    handle.insert_source(Timer::immediate(), |_, _, data| {
        data.shell
            .ensure_running(&mut data.state.display_handle, &data.state.socket_name);
        TimeoutAction::ToDuration(SHELL_CHECK)
    })?;

    let mut data = Loop {
        state,
        display,
        shell: ShellProcess::new(shell_program),
    };
    event_loop.run(None, &mut data, |data| {
        let _ = data.display.flush_clients();
    })?;

    data.shell.stop();
    Ok(())
}

/// Reads the arguments after the program name: nothing, or `--shell <path>`
/// to run a different shell program.
fn parse_args<I: IntoIterator<Item = OsString>>(args: I) -> Result<Option<PathBuf>, String> {
    let mut args = args.into_iter();
    let shell = match args.next() {
        None => return Ok(None),
        Some(flag) if flag == "--shell" => args.next().ok_or("--shell needs a path")?,
        Some(other) => return Err(format!("unknown argument: {}", other.display())),
    };
    match args.next() {
        Some(extra) => Err(format!("unexpected argument: {}", extra.display())),
        None => Ok(Some(shell.into())),
    }
}

/// Finds the shell program: the one named on the command line, or the
/// `shell` binary next to this one.
fn shell_program() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(path) = parse_args(std::env::args_os().skip(1))? {
        return Ok(path);
    }
    let mut path = std::env::current_exe()?;
    path.set_file_name("shell");
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<OsString> {
        list.iter().map(OsString::from).collect()
    }

    #[test]
    fn no_arguments_means_the_default_shell() {
        assert_eq!(parse_args(args(&[])), Ok(None));
    }

    #[test]
    fn a_shell_can_be_named() {
        assert_eq!(
            parse_args(args(&["--shell", "/opt/shell"])),
            Ok(Some(PathBuf::from("/opt/shell")))
        );
    }

    #[test]
    fn bad_arguments_are_errors() {
        assert!(parse_args(args(&["--shell"])).is_err());
        assert!(parse_args(args(&["--other"])).is_err());
        assert!(parse_args(args(&["--shell", "a", "b"])).is_err());
    }
}
