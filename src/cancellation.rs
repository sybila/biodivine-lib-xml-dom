//! Periodic, cheap cancellation checks for the long-running operations.
//!
//! Consulting the thread-local trigger stack of [`cancel_this`] on every node of a large document
//! is measurable (it is a thread-local borrow and a dynamic call), so the walks check it once every
//! [`CHECK_INTERVAL`] steps instead. The very first step always checks, so an operation started
//! under a trigger that has already fired returns immediately, even for a tiny document.

use crate::error::XmlResult;

/// How many steps of a walk pass between two consultations of the cancellation triggers.
const CHECK_INTERVAL: u32 = 1024;

/// A countdown that consults the cancellation triggers of the current thread every
/// [`CHECK_INTERVAL`] calls of [`CancellationCheck::tick`].
pub(crate) struct CancellationCheck {
    /// `false` for operations that must not be cancelled (for example `Display`).
    enabled: bool,
    /// Steps left until the next consultation.
    remaining: u32,
}

impl CancellationCheck {
    /// A check that consults the triggers on the first step and then periodically.
    pub(crate) fn new() -> Self {
        Self {
            enabled: true,
            remaining: 0,
        }
    }

    /// A check that never fires, for the code paths that cannot report an error.
    pub(crate) fn disabled() -> Self {
        Self {
            enabled: false,
            remaining: 0,
        }
    }

    /// Accounts for one step of the walk.
    ///
    /// # Errors
    ///
    /// Returns [`crate::XmlError::Cancelled`] if this step consults the triggers and one of them
    /// has fired.
    #[inline]
    pub(crate) fn tick(&mut self) -> XmlResult<()> {
        if !self.enabled {
            return Ok(());
        }
        if self.remaining == 0 {
            self.remaining = CHECK_INTERVAL;
            cancel_this::is_cancelled!()?;
        } else {
            self.remaining -= 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_step_and_every_interval_consult_the_triggers() {
        let trigger = cancel_this::CancelAtomic::new();
        trigger.cancel();
        let result: XmlResult<()> = cancel_this::on_atomic(trigger, || {
            let mut check = CancellationCheck::new();
            assert!(check.tick().is_err(), "the first step checks");
            for _ in 0..CHECK_INTERVAL {
                assert!(check.tick().is_ok(), "the steps in between do not");
            }
            assert!(check.tick().is_err(), "the next interval checks again");

            let mut disabled = CancellationCheck::disabled();
            assert!(disabled.tick().is_ok());
            Ok(())
        });
        assert!(result.is_ok());
    }
}
