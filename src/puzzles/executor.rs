//! Deterministic, turn-stepped executor. No host runtime or callbacks.
//! Quiescence retains sleeping errands; panic contains the authored fault
//! and releases loans by the same cleanup path as cancellation (D-1262).

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Outcome {
    Pending,
    /// Authored busy poll: remain runnable without sleeping.
    Continue,
    Ready,
    Panic(u8),
    Cancel,
}

/// Each poll consumes one authored state. Pending sleeps until a signal;
/// a state with no next entry retains its last state at quiescence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct State {
    pub name: &'static str,
    pub outcome: Outcome,
    pub heart: Option<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Timer {
    pub tick: u32,
    /// Zero means the generation armed by this sleep; nonzero is an
    /// explicit signal generation, allowing authored stale witnesses.
    pub generation: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErrandSpec {
    pub seal: &'static str,
    pub states: Vec<State>,
    /// State-table indices in poll order. Finite authored drive, no
    /// implicit turn cap. Indices outside the table retain the errand.
    pub drive: Vec<usize>,
    pub timers: Vec<Timer>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FinalState {
    Completed,
    Cancelled,
    Panicked,
    Running(&'static str),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    Poll {
        seal: &'static str,
        generation: u32,
        state: &'static str,
    },
    Sleep {
        seal: &'static str,
        generation: u32,
    },
    Wake {
        seal: &'static str,
        generation: u32,
    },
    TimerFired {
        seal: &'static str,
        generation: u32,
        tick: u32,
    },
    Completed {
        seal: &'static str,
        generation: u32,
    },
    CancelledWithCleanup {
        seal: &'static str,
        generation: u32,
        released: Vec<usize>,
    },
    Panicked {
        seal: &'static str,
        generation: u32,
        code: u8,
        released: Vec<usize>,
    },
    StaleRefused {
        seal: &'static str,
        generation: u32,
        current: u32,
    },
    Accounting {
        seal: &'static str,
        generation: u32,
        state: FinalState,
    },
}

struct Live {
    cursor: usize,
    generation: u32,
    runnable: bool,
    terminal: Option<FinalState>,
    name: &'static str,
    loans: Vec<usize>,
}

fn cleanup(
    task: &mut Live,
    hearts: &mut std::collections::BTreeMap<usize, super::FlagState>,
) -> Vec<usize> {
    let released = std::mem::take(&mut task.loans);
    for heart in &released {
        if let Some(state) = hearts.get_mut(heart)
            && let Some(next) = super::flag_step(*state, super::FlagOp::CloseExclusive)
        {
            *state = next;
        }
    }
    released
}

/// Run in roster order until no runnable errand or live wake source
/// remains. Timer ties are ordered by (tick, roster index, authored index).
/// An errand is accounted exactly once, including never-waking errands.
pub fn run(roster: &[ErrandSpec]) -> Vec<Event> {
    let mut events = Vec::new();
    let mut hearts = std::collections::BTreeMap::new();
    let mut live: Vec<Live> = roster
        .iter()
        .map(|_| Live {
            cursor: 0,
            generation: 0,
            runnable: true,
            terminal: None,
            name: "unpolled",
            loans: Vec::new(),
        })
        .collect();
    let mut timers: Vec<_> = roster
        .iter()
        .enumerate()
        .flat_map(|(i, spec)| {
            spec.timers
                .iter()
                .enumerate()
                .map(move |(j, timer)| (timer.tick, i, j, timer.generation))
        })
        .collect();
    timers.sort_by_key(|&(tick, i, j, _)| (tick, i, j));
    let mut timer_cursor = 0;
    loop {
        let mut progressed = false;
        for (i, spec) in roster.iter().enumerate() {
            let task = &mut live[i];
            if !task.runnable || task.terminal.is_some() {
                continue;
            }
            task.runnable = false;
            let Some(state) = spec
                .drive
                .get(task.cursor)
                .and_then(|&index| spec.states.get(index))
            else {
                continue;
            };
            // Use Floor 6's real flag transition, not a second rule table.
            // Blocked borrowers wait for release without polling or spinning.
            if let Some(heart) = state.heart
                && !task.loans.contains(&heart)
            {
                let flag = hearts.entry(heart).or_insert(super::FlagState::Free);
                let Some(next) = super::flag_step(*flag, super::FlagOp::OpenExclusive) else {
                    task.runnable = true;
                    continue;
                };
                *flag = next;
                task.loans.push(heart);
            }
            progressed = true;
            task.name = state.name;
            task.cursor += 1;
            events.push(Event::Poll {
                seal: spec.seal,
                generation: task.generation,
                state: state.name,
            });
            match state.outcome {
                Outcome::Continue => task.runnable = true,
                Outcome::Pending => {
                    task.generation += 1;
                    events.push(Event::Sleep {
                        seal: spec.seal,
                        generation: task.generation,
                    });
                }
                Outcome::Ready => {
                    cleanup(task, &mut hearts);
                    task.terminal = Some(FinalState::Completed);
                    events.push(Event::Completed {
                        seal: spec.seal,
                        generation: task.generation,
                    });
                }
                Outcome::Cancel | Outcome::Panic(_) => {
                    let released = cleanup(task, &mut hearts);
                    match state.outcome {
                        Outcome::Panic(code) => {
                            task.terminal = Some(FinalState::Panicked);
                            events.push(Event::Panicked {
                                seal: spec.seal,
                                generation: task.generation,
                                code,
                                released,
                            });
                        }
                        _ => {
                            task.terminal = Some(FinalState::Cancelled);
                            events.push(Event::CancelledWithCleanup {
                                seal: spec.seal,
                                generation: task.generation,
                                released,
                            });
                        }
                    }
                }
            }
        }
        if progressed && live.iter().any(|task| task.runnable) {
            continue;
        }
        // Finished errands have no live timer sources. Already-authored
        // stale signals to living errands remain observable evidence.
        let next = timers[timer_cursor..]
            .iter()
            .position(|&(_, i, _, _)| live[i].terminal.is_none());
        let Some(offset) = next else {
            break;
        };
        timer_cursor += offset;
        let (tick, i, _, signal) = timers[timer_cursor];
        timer_cursor += 1;
        let task = &mut live[i];
        let generation = if signal == 0 { task.generation } else { signal };
        events.push(Event::TimerFired {
            seal: roster[i].seal,
            generation,
            tick,
        });
        if generation != task.generation {
            events.push(Event::StaleRefused {
                seal: roster[i].seal,
                generation,
                current: task.generation,
            });
        } else {
            events.push(Event::Wake {
                seal: roster[i].seal,
                generation,
            });
            task.runnable = true;
        }
    }
    for (spec, task) in roster.iter().zip(live) {
        events.push(Event::Accounting {
            seal: spec.seal,
            generation: task.generation,
            state: task.terminal.unwrap_or(FinalState::Running(task.name)),
        });
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    fn errand(seal: &'static str, outcomes: &[Outcome], timers: Vec<Timer>) -> ErrandSpec {
        ErrandSpec {
            seal,
            states: outcomes
                .iter()
                .map(|&outcome| State {
                    name: "state 1",
                    outcome,
                    heart: Some(0),
                })
                .collect(),
            drive: (0..outcomes.len()).collect(),
            timers,
        }
    }
    #[test]
    fn completed_is_accounted_once() {
        let events = run(&[errand("A", &[Outcome::Ready], vec![])]);
        assert_eq!(events.len(), 3);
        assert!(matches!(
            events.last(),
            Some(Event::Accounting {
                state: FinalState::Completed,
                ..
            })
        ));
    }
    #[test]
    fn sleeping_never_waking_is_retained_by_name() {
        let events = run(&[errand("A", &[Outcome::Pending], vec![])]);
        assert!(matches!(
            events.last(),
            Some(Event::Accounting {
                generation: 1,
                state: FinalState::Running("state 1"),
                ..
            })
        ));
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::CancelledWithCleanup { .. }))
        );
    }
    #[test]
    fn panic_releases_heart_and_other_errands_continue() {
        let events = run(&[
            errand("fault", &[Outcome::Panic(7)], vec![]),
            errand("survivor", &[Outcome::Ready], vec![]),
        ]);
        assert!(
            events.iter().any(
                |e| matches!(e, Event::Panicked { code: 7, released, .. } if released == &[0])
            )
        );
        assert!(events.iter().any(|e| matches!(
            e,
            Event::Completed {
                seal: "survivor",
                ..
            }
        )));
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::Accounting { .. }))
                .count(),
            2
        );
    }
    #[test]
    fn stale_generation_is_evidence_and_never_wakes() {
        let events = run(&[errand(
            "A",
            &[Outcome::Pending, Outcome::Ready],
            vec![Timer {
                tick: 1,
                generation: 9,
            }],
        )]);
        assert!(events.iter().any(|e| matches!(
            e,
            Event::StaleRefused {
                generation: 9,
                current: 1,
                ..
            }
        )));
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::Wake { .. } | Event::Completed { .. }))
        );
    }
    #[test]
    fn sleep_twice_generations_increase_and_timer_order_is_stable() {
        let spec = errand(
            "A",
            &[Outcome::Pending, Outcome::Pending, Outcome::Ready],
            vec![
                Timer {
                    tick: 3,
                    generation: 0,
                },
                Timer {
                    tick: 4,
                    generation: 0,
                },
            ],
        );
        let first = run(std::slice::from_ref(&spec));
        assert_eq!(first, run(&[spec]));
        assert_eq!(
            first
                .iter()
                .filter_map(|e| if let Event::Wake { generation, .. } = e {
                    Some(*generation)
                } else {
                    None
                })
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
    }
    #[test]
    fn equal_tick_timers_follow_roster_order() {
        let mut a = errand(
            "A",
            &[Outcome::Pending, Outcome::Ready],
            vec![Timer {
                tick: 5,
                generation: 0,
            }],
        );
        let mut b = a.clone();
        b.seal = "B";
        for state in &mut a.states {
            state.heart = None;
        }
        for state in &mut b.states {
            state.heart = None;
        }
        let events = run(&[a, b]);
        let fired: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let Event::TimerFired { seal, tick, .. } = e {
                    Some((*tick, *seal))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(fired, vec![(5, "A"), (5, "B")]);
    }

    #[test]
    fn previous_watch_is_stale_after_rearming() {
        let events = run(&[errand(
            "A",
            &[Outcome::Pending, Outcome::Pending, Outcome::Ready],
            vec![
                Timer {
                    tick: 1,
                    generation: 0,
                },
                Timer {
                    tick: 2,
                    generation: 1,
                },
                Timer {
                    tick: 3,
                    generation: 0,
                },
            ],
        )]);
        assert!(events.iter().any(|e| matches!(
            e,
            Event::StaleRefused {
                generation: 1,
                current: 2,
                ..
            }
        )));
        let wakes: Vec<_> = events
            .iter()
            .filter_map(|e| {
                if let Event::Wake { generation, .. } = e {
                    Some(*generation)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(wakes, vec![1, 2]);
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::Poll { .. }))
                .count(),
            3
        );
    }

    #[test]
    fn cancellation_cleans_borrow_but_does_not_complete() {
        let events = run(&[errand("A", &[Outcome::Cancel], vec![])]);
        assert!(
            events.iter().any(
                |e| matches!(e, Event::CancelledWithCleanup {released, ..} if released == &[0])
            )
        );
        assert!(!events.iter().any(|e| matches!(e, Event::Completed { .. })));
    }
}
