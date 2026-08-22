use std::fmt;
use std::io;

#[derive(Debug)]
pub(crate) enum ParentLifecycleError {
    InvalidParent(libc::pid_t),
    Arm(io::Error),
    ParentChanged {
        before: libc::pid_t,
        after: libc::pid_t,
    },
}

impl fmt::Display for ParentLifecycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidParent(pid) => {
                write!(
                    formatter,
                    "IBus parent PID must be greater than 1, got {pid}"
                )
            }
            Self::Arm(error) => write!(formatter, "failed to arm IBus parent lifecycle: {error}"),
            Self::ParentChanged { before, after } => write!(
                formatter,
                "IBus parent changed while arming lifecycle: {before} -> {after}"
            ),
        }
    }
}

impl std::error::Error for ParentLifecycleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Arm(error) => Some(error),
            Self::InvalidParent(_) | Self::ParentChanged { .. } => None,
        }
    }
}

pub(crate) fn arm_ibus_parent_death(ibus_owned: bool) -> Result<(), ParentLifecycleError> {
    arm_ibus_parent_death_with(
        ibus_owned,
        || unsafe { libc::getppid() },
        || {
            let result = unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM) };
            if result == 0 {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        },
    )
}

fn arm_ibus_parent_death_with<P, A>(
    ibus_owned: bool,
    mut parent_pid: P,
    mut arm: A,
) -> Result<(), ParentLifecycleError>
where
    P: FnMut() -> libc::pid_t,
    A: FnMut() -> io::Result<()>,
{
    if !ibus_owned {
        return Ok(());
    }

    let before = parent_pid();
    if before <= 1 {
        return Err(ParentLifecycleError::InvalidParent(before));
    }

    arm().map_err(ParentLifecycleError::Arm)?;

    let after = parent_pid();
    if after != before {
        return Err(ParentLifecycleError::ParentChanged { before, after });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::collections::VecDeque;

    use super::*;

    #[test]
    fn manual_mode_does_not_observe_or_arm_parent_lifecycle() {
        arm_ibus_parent_death_with(
            false,
            || panic!("manual mode must not observe the parent"),
            || panic!("manual mode must not arm a parent-death signal"),
        )
        .unwrap();
    }

    #[test]
    fn ibus_mode_accepts_the_same_positive_parent_across_arm() {
        let arm_calls = Cell::new(0);
        arm_ibus_parent_death_with(
            true,
            || 42,
            || {
                arm_calls.set(arm_calls.get() + 1);
                Ok(())
            },
        )
        .unwrap();

        assert_eq!(arm_calls.get(), 1);
    }

    #[test]
    fn ibus_mode_rejects_init_before_arm() {
        let error = arm_ibus_parent_death_with(true, || 1, || Ok(())).unwrap_err();
        assert!(matches!(error, ParentLifecycleError::InvalidParent(1)));
    }

    #[test]
    fn ibus_mode_rejects_a_parent_change_during_arm() {
        let mut parents = VecDeque::from([42, 7]);
        let error = arm_ibus_parent_death_with(true, || parents.pop_front().unwrap(), || Ok(()))
            .unwrap_err();

        assert!(matches!(
            error,
            ParentLifecycleError::ParentChanged {
                before: 42,
                after: 7
            }
        ));
    }

    #[test]
    fn ibus_mode_propagates_prctl_failure() {
        let error = arm_ibus_parent_death_with(
            true,
            || 42,
            || Err(io::Error::from_raw_os_error(libc::EPERM)),
        )
        .unwrap_err();

        assert!(matches!(error, ParentLifecycleError::Arm(_)));
    }
}
