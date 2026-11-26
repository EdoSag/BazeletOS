//! Starting the system shell and keeping it running.
//!
//! The shell draws the lock screen and the home screen. The compositor starts
//! it itself and hands it an already connected socket. That connection is the
//! only one allowed to lock the session or to use layer shell, so an app that
//! connects to the public socket can do neither.

use std::{
    ffi::OsString,
    io,
    os::{
        fd::{AsRawFd, OwnedFd},
        unix::net::UnixStream,
    },
    path::PathBuf,
    process::{Child, Command},
    sync::Arc,
};

use rustix::io::{FdFlags, fcntl_getfd, fcntl_setfd};
use smithay::reexports::wayland_server::DisplayHandle;

use crate::state::ClientState;

/// The running shell and how to start it again.
pub struct ShellProcess {
    program: PathBuf,
    child: Option<Child>,
}

impl ShellProcess {
    /// Remembers which program to run. Nothing is started yet.
    pub fn new(program: PathBuf) -> Self {
        Self {
            program,
            child: None,
        }
    }

    /// Starts the shell if it is not running.
    ///
    /// If the shell crashed while the session was unlocked, the new one locks
    /// the session again when it starts.
    pub fn ensure_running(&mut self, display: &mut DisplayHandle, socket_name: &OsString) {
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(None) => return,
                Ok(Some(status)) => tracing::warn!(%status, "the shell stopped; starting it again"),
                Err(error) => tracing::warn!(%error, "could not check on the shell"),
            }
        }
        match spawn(&self.program, display, socket_name) {
            Ok(child) => self.child = Some(child),
            Err(error) => {
                tracing::error!(%error, program = %self.program.display(), "could not start the shell");
                self.child = None;
            }
        }
    }

    /// Stops the shell. Called when the compositor exits.
    pub fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn spawn(
    program: &PathBuf,
    display: &mut DisplayHandle,
    socket_name: &OsString,
) -> io::Result<Child> {
    let (ours, theirs) = UnixStream::pair()?;
    let theirs = OwnedFd::from(theirs);

    // The child must inherit its end of the socket, so close-on-exec is
    // switched off for the moment of the spawn. The compositor is single
    // threaded, so no other child can be started in between.
    let flags = fcntl_getfd(&theirs)?;
    fcntl_setfd(&theirs, flags.difference(FdFlags::CLOEXEC))?;
    let child = Command::new(program)
        .env("WAYLAND_SOCKET", theirs.as_raw_fd().to_string())
        // Apps the shell launches connect here, without the shell's rights.
        .env("WAYLAND_DISPLAY", socket_name)
        .spawn();
    drop(theirs);
    let child = child?;

    display
        .insert_client(
            ours,
            Arc::new(ClientState {
                is_shell: true,
                ..ClientState::default()
            }),
        )
        .map_err(io::Error::other)?;
    Ok(child)
}
