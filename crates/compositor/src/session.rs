//! The lock state of the session, kept apart from Wayland so it can be tested.
//!
//! Rules, from the security model in `docs/PLAN.md`:
//!
//! - The session starts locked. Nothing but the lock screen is drawn or
//!   receives input until an unlock is accepted.
//! - Only the lock screen client that holds the lock can unlock it.
//! - If that client goes away while the session is locked, the session
//!   stays locked. A crashed lock screen must never reveal the session.

/// Whether the session is locked, and who holds the lock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Session {
    /// Locked. `held` says whether a lock screen client currently holds the
    /// lock. Without one the screen stays blank.
    Locked {
        /// A lock screen client holds the lock.
        held: bool,
    },
    /// Unlocked. Apps and the home screen are visible.
    Unlocked,
}

impl Session {
    /// The state at start-up: locked, with no lock screen yet.
    pub const fn new() -> Self {
        Self::Locked { held: false }
    }

    /// Whether the session is locked.
    pub const fn is_locked(self) -> bool {
        matches!(self, Self::Locked { .. })
    }

    /// A lock screen client asked to take the lock.
    ///
    /// Returns `true` if the request is granted. A second lock screen cannot
    /// take a lock that another one already holds.
    pub fn request_lock(&mut self) -> bool {
        match *self {
            Self::Locked { held: true } => false,
            Self::Locked { held: false } | Self::Unlocked => {
                *self = Self::Locked { held: true };
                true
            }
        }
    }

    /// The client that holds the lock asked to unlock.
    pub fn unlock(&mut self) {
        if *self == (Self::Locked { held: true }) {
            *self = Self::Unlocked;
        }
    }

    /// The client that held the lock went away without unlocking.
    pub fn holder_gone(&mut self) {
        if self.is_locked() {
            *self = Self::Locked { held: false };
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Session;

    #[test]
    fn starts_locked() {
        assert!(Session::new().is_locked());
    }

    #[test]
    fn lock_then_unlock() {
        let mut session = Session::new();
        assert!(session.request_lock());
        session.unlock();
        assert_eq!(session, Session::Unlocked);
    }

    #[test]
    fn unlock_without_a_holder_is_ignored() {
        let mut session = Session::new();
        session.unlock();
        assert!(session.is_locked());
    }

    #[test]
    fn a_second_lock_screen_is_refused() {
        let mut session = Session::new();
        assert!(session.request_lock());
        assert!(!session.request_lock());
    }

    #[test]
    fn a_crashed_lock_screen_leaves_the_session_locked() {
        let mut session = Session::new();
        assert!(session.request_lock());
        session.holder_gone();
        assert_eq!(session, Session::Locked { held: false });
        session.unlock();
        assert!(session.is_locked());
    }

    #[test]
    fn a_new_lock_screen_can_take_over_after_a_crash() {
        let mut session = Session::new();
        assert!(session.request_lock());
        session.holder_gone();
        assert!(session.request_lock());
        session.unlock();
        assert_eq!(session, Session::Unlocked);
    }

    #[test]
    fn can_lock_again_after_unlocking() {
        let mut session = Session::Unlocked;
        assert!(session.request_lock());
        assert!(session.is_locked());
    }
}
