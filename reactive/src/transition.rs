use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    rc::Rc,
};

use crate::{
    create_effect, create_signal, spawn::queue_microtask, system::System, ObserverLink, ReadSignal,
    Scope, ScopeId, SignalId, WriteSignal,
};

pub fn use_transition(cx: Scope) -> Transition {
    let (pending, set_pending) = create_signal(cx, false);
    Transition {
        system: cx.system,
        scope: cx,
        pending,
        set_pending,
    }
}

#[derive(Copy, Clone)]
pub struct Transition {
    system: &'static System,
    scope: Scope,
    pending: ReadSignal<bool>,
    set_pending: WriteSignal<bool>,
}

impl Transition {
    pub fn start(&self, f: impl FnOnce()) {
        /* if self.system.running_transition().is_some() {
            f();
        } else {
            {
                self.set_pending.update(|n| *n = true);
                *self.system.transition.borrow_mut() = Some(Rc::new(TransitionState {
                    running: Cell::new(true),
                    resources: Default::default(),
                    signals: Default::default(),
                    effects: Default::default(),
                }));
            }

            f();

            if let Some(running_transition) = self.system.running_transition() {
                running_transition.running.set(false);

                let system = self.system;
                let scope = self.scope;
                let resources = running_transition.resources.clone();
                let signals = running_transition.signals.clone();
                let effects = running_transition.effects.clone();
                let set_pending = self.set_pending;
                // place this at end of task queue so it doesn't start at 0
                queue_microtask(move || {
                    create_effect(scope, move |_| {
                        let pending = resources.borrow().iter().map(|p| p.get()).sum::<usize>();

                        if pending == 0 {
                            for signal in signals.borrow().iter() {
                                system.any_signal(*signal, |signal| {
                                    signal.end_transition(system);
                                });
                            }
                            for effect in effects.borrow().iter() {
                                system.any_effect(*effect, |any_effect| {
                                    any_effect.run(*effect);
                                });
                            }
                            set_pending.update(|n| *n = false);
                        }
                    });
                });
            }
        } */
        todo!()
    }

    pub fn pending(&self) -> bool {
        self.pending.get()
    }
}

#[derive(Debug)]
pub(crate) struct TransitionState {
    pub running: Cell<bool>,
    pub resources: Rc<RefCell<HashSet<ReadSignal<usize>>>>,
    pub signals: Rc<RefCell<HashSet<(ScopeId, SignalId)>>>,
    pub computation: Rc<RefCell<Vec<ObserverLink>>>,
}
