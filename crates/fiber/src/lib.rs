// A duel whose script functions run on a stack of their own, so that one waiting at a prompt resumes in place.
//
// Soundness: a waiting function's frames, and the processor's under them, hold `&mut Duel`. So a run that reaches a
// script function goes on in a coroutine to its end (Duel::run_to_script stops between units, where no frame holds the
// duel), and from then on the duel is reached only through that run: the coroutine's frames while it executes, and
// while a function waits, the pointer the innermost frame handed out, which [Fiber] stops using before it resumes the
// run. The duel is boxed, so moving the fiber moves no borrowed memory.
use std::cell::{Cell, RefCell};
use std::ops::Deref;
use std::ptr::NonNull;

use corosensei::stack::DefaultStack;
use corosensei::{Coroutine, CoroutineResult, Yielder};
use engine::{Action, ActionError, Duel, Resumed, Status};

// Reserved per stack; Windows commits its pages as they are touched.
const STACK_SIZE: usize = 2 << 20;
// Per thread: runs often finish on another thread than they started on, so pools drift.
const POOLED_STACKS: usize = 8;

type Run = Coroutine<Resumed, Waiting, Status, DefaultStack>;
type Resumer = NonNull<Yielder<Resumed, Waiting>>;

struct Waiting {
    duel: NonNull<Duel>,
    resumer: Resumer,
}

thread_local! {
    // The duel whose run executes on this thread, and the run's yielder.
    static RUNNING: Cell<Option<(NonNull<Duel>, Resumer)>> = const { Cell::new(None) };
    // The stacks of finished runs, reused by the next; reached with try_with, as a fiber may be dropped while the thread ends.
    static STACKS: RefCell<Vec<DefaultStack>> = const { RefCell::new(Vec::new()) };
}

pub struct Fiber {
    owned: NonNull<Duel>,
    // `owned`, or while a function waits, the pointer its innermost frame handed out.
    current: NonNull<Duel>,
    waiting: Option<(Run, Resumer)>,
}

// SAFETY: a waiting run's stack holds the engine's frames over this fiber's duel (`Send`), and nothing tied to a thread
// (the engine and the scripts keep no thread-local state across a prompt).
unsafe impl Send for Fiber {}

const _: fn() = || {
    fn send<T: Send>() {}
    send::<Duel>();
};

impl Fiber {
    pub fn new(duel: Duel) -> Self {
        engine::SUSPEND.get_or_init(|| suspend);
        let owned = NonNull::from(Box::leak(Box::new(duel)));
        Self {
            owned,
            current: owned,
            waiting: None,
        }
    }

    // [Duel::process].
    pub fn process(&mut self) -> Status {
        if self.prompt().is_some() {
            return Status::Awaiting;
        }
        self.run(Resumed::Answered)
    }

    // [Duel::apply], resuming the function that waits at the prompt.
    pub fn apply(&mut self, action: Action) -> Result<Status, ActionError> {
        Ok(match self.current_mut().answer(action)? {
            true => self.run(Resumed::Answered),
            false => Status::Awaiting,
        })
    }

    // A script function waits at the prompt, which a clone reruns from its snapshot.
    pub fn is_waiting(&self) -> bool {
        self.waiting.is_some()
    }

    pub fn take_trace(&mut self) -> Vec<u8> {
        self.current_mut().take_trace()
    }

    // The duel to change at will: a function waiting at the prompt is interrupted first, to be rerun as a clone's is.
    pub fn duel_mut(&mut self) -> &mut Duel {
        while self.waiting.is_some() {
            self.run(Resumed::Cancelled);
        }
        self.current_mut()
    }

    fn current_mut(&mut self) -> &mut Duel {
        // SAFETY: no run executes, so `current` is the only path to the duel in use.
        unsafe { self.current.as_mut() }
    }

    // Runs on the caller's stack up to the first script function, the rest on a stack of its own.
    fn run(&mut self, input: Resumed) -> Status {
        let (mut run, resumer) = match self.waiting.take() {
            Some((run, resumer)) => (run, Some(resumer)),
            None => {
                if let Some(status) = self.current_mut().run_to_script() {
                    return status;
                }
                match self.start() {
                    Some(run) => (run, None),
                    None => return self.current_mut().run(),
                }
            }
        };
        self.current = self.owned;
        let restore = Restore(RUNNING.replace(resumer.map(|resumer| (self.owned, resumer))));
        let result = run.resume(input);
        drop(restore);
        match result {
            CoroutineResult::Yield(Waiting { duel, resumer }) => {
                self.current = duel;
                self.waiting = Some((run, resumer));
                Status::Awaiting
            }
            CoroutineResult::Return(status) => {
                let _ = STACKS.try_with(|stacks| {
                    let mut stacks = stacks.borrow_mut();
                    if stacks.len() < POOLED_STACKS {
                        stacks.push(run.into_stack());
                    }
                });
                status
            }
        }
    }

    // None if no stack can be had, and the duel runs on the caller's.
    fn start(&self) -> Option<Run> {
        let reused = STACKS.try_with(|stacks| stacks.borrow_mut().pop()).ok().flatten();
        let stack = reused.or_else(|| DefaultStack::new(STACK_SIZE).ok())?;
        let duel = self.owned;
        Some(Coroutine::with_stack(stack, move |yielder: &Yielder<Resumed, Waiting>, _| {
            RUNNING.set(Some((duel, NonNull::from(yielder))));
            // SAFETY: the fiber leaves its duel alone while this run executes, and while it waits uses only what `suspend` handed out.
            unsafe { &mut *duel.as_ptr() }.run()
        }))
    }
}

// Called by the engine where a function meets a prompt with no answer: waits there if it is this thread's running duel.
fn suspend(duel: &mut Duel) -> Option<Resumed> {
    let (running, resumer) = RUNNING.get()?;
    if !std::ptr::eq(running.as_ptr(), duel) {
        return None;
    }
    // SAFETY: RUNNING holds the yielder of the run executing this call, which lives on that run's stack.
    let yielder = unsafe { resumer.as_ref() };
    Some(yielder.suspend(Waiting {
        duel: NonNull::from(duel),
        resumer,
    }))
}

struct Restore(Option<(NonNull<Duel>, Resumer)>);

impl Drop for Restore {
    fn drop(&mut self) {
        RUNNING.set(self.0);
    }
}

impl Deref for Fiber {
    type Target = Duel;

    fn deref(&self) -> &Duel {
        // SAFETY: no run executes while `&self` is held, so `current` is the only path to the duel in use.
        unsafe { self.current.as_ref() }
    }
}

impl Clone for Fiber {
    fn clone(&self) -> Self {
        Self::new(Duel::clone(self))
    }
}

impl Drop for Fiber {
    fn drop(&mut self) {
        self.duel_mut();
        // SAFETY: no run holds the duel any more, and `owned` came from Box::leak.
        drop(unsafe { Box::from_raw(self.owned.as_ptr()) });
    }
}
