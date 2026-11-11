//! Floor 8 projections: every event and accounting fact comes from run().
use super::executor::{self, Event, FinalState};

pub fn ledger() -> Vec<(String, u8)> {
    executor::run(&crate::content::hour_ledger_roster())
        .into_iter()
        .filter_map(|e| match e {
            Event::Accounting { seal, state, .. } => Some((
                seal.into(),
                match state {
                    FinalState::Completed => 1,
                    FinalState::Cancelled => 2,
                    FinalState::Running(_) => 3,
                    FinalState::Panicked => 4,
                },
            )),
            _ => None,
        })
        .collect()
}

pub fn ledger_holds(lines: &[u8; 6]) -> bool {
    lines.as_slice() == ledger().iter().map(|(_, state)| *state).collect::<Vec<_>>()
}

pub fn roster(index: usize) -> Vec<Event> {
    executor::run(&crate::content::hour_foe_roster(index))
}

/// Presentation permutation only: canonical executor output is unchanged.
pub fn presentation(index: usize) -> Vec<Event> {
    let mut events = roster(index);
    if crate::content::HOUR_ROSTER_REVERSED
        .get(index)
        .copied()
        .unwrap_or(false)
    {
        events.reverse();
    }
    events
}

pub fn stale_trace() -> Vec<Event> {
    executor::run(&crate::content::hour_stale_roster())
}
pub fn stale_signals() -> Vec<Event> {
    stale_trace()
        .into_iter()
        .filter(|e| matches!(e, Event::StaleRefused { .. }))
        .collect()
}

pub fn living_signal() -> (&'static str, u32) {
    stale_trace()
        .into_iter()
        .find_map(|e| match e {
            Event::Accounting {
                seal,
                generation,
                state: FinalState::Running(_),
                ..
            } => Some((seal, generation)),
            _ => None,
        })
        .expect("authored stuck errand is retained")
}

pub fn accepts_signal(seal: &str, generation: u32) -> bool {
    let living = living_signal();
    (seal, generation) == living
}

pub fn policy_trace() -> Vec<Event> {
    executor::run(&crate::content::hour_policy_roster())
}
pub fn policy_holds() -> bool {
    let events = policy_trace();
    events
        .iter()
        .any(|e| matches!(e, Event::Panicked { released, .. } if !released.is_empty()))
        && events.iter().any(|e| {
            matches!(
                e,
                Event::Accounting {
                    seal: "survivor",
                    state: FinalState::Completed,
                    ..
                }
            )
        })
        && events.iter().any(|e| {
            matches!(
                e,
                Event::Accounting {
                    seal: "never waking",
                    state: FinalState::Running(_),
                    ..
                }
            )
        })
        && !events.iter().any(|e| {
            matches!(
                e,
                Event::CancelledWithCleanup {
                    seal: "never waking",
                    ..
                }
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ledger_has_six_started_once_and_real_cleanup() {
        let events = executor::run(&crate::content::hour_ledger_roster());
        assert_eq!(ledger().len(), 6);
        for (seal, _) in ledger() {
            assert!(
                events
                    .iter()
                    .any(|e| matches!(e, Event::Poll { seal: found, .. } if *found == seal))
            );
            assert_eq!(
                events
                    .iter()
                    .filter(
                        |e| matches!(e, Event::Accounting { seal: found, .. } if *found == seal)
                    )
                    .count(),
                1
            );
        }
        assert_eq!(events.iter().filter(|e| matches!(e, Event::CancelledWithCleanup { released, .. } if !released.is_empty())).count(), 2);
        assert!(ledger_holds(
            &ledger()
                .iter()
                .map(|(_, s)| *s)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap()
        ));
    }
    #[test]
    fn duplicated_or_missing_ledger_outcomes_fail() {
        assert!(!ledger_holds(&[1; 6]));
        assert!(!ledger_holds(&[0; 6]));
        let mut lines: [u8; 6] = ledger()
            .iter()
            .map(|(_, s)| *s)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        lines.swap(0, 1);
        assert!(!ledger_holds(&lines));
    }
    #[test]
    fn canonical_timers_and_unsequenced_presentation_are_distinct() {
        let i = crate::content::HOUR_UNSEQUENCED;
        let canonical = roster(i);
        let mut reversed = canonical.clone();
        reversed.reverse();
        assert_eq!(presentation(i), reversed);
        assert_eq!(roster(i), canonical);
        let timers: Vec<_> = canonical
            .iter()
            .filter_map(|e| match e {
                Event::TimerFired { tick, .. } => Some(*tick),
                _ => None,
            })
            .collect();
        assert!(timers.windows(2).all(|w| w[0] <= w[1]));
        assert!(policy_holds());
    }
    #[test]
    fn stale_or_wrong_room_never_publishes_completion() {
        let (seal, generation) = living_signal();
        assert!(accepts_signal(seal, generation));
        assert!(!accepts_signal("previous watch", generation));
        assert!(!accepts_signal(seal, generation - 1));
        assert_eq!(stale_signals().len(), 3);
        assert!(
            !stale_trace()
                .iter()
                .any(|e| matches!(e, Event::Completed { .. }))
        );
    }
}
