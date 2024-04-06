//!
//!
//! - [`once`]
//! - [`wait`]
//! - [`delay`]

use std::marker::PhantomData;

use bevy::prelude::World;

use crate::runner::{CancellationToken, RunnerIntoAction, RunWithTaskOutput, TaskOutput, TaskRunner};

pub mod once;
pub mod wait;
pub mod delay;
pub mod sequence;
pub mod switch;
pub mod pipe;
pub mod seed;


/// Represents the system passed to [`ReactiveTask`](crate::task::ReactiveTask).
pub trait Action<In, Out> {
    /// Convert itself to [`TaskRunner`](crate::runner::TaskRunner).
    fn to_runner(self, token: CancellationToken, output: TaskOutput<Out>) -> impl TaskRunner + 'static;
}


/// Convert to the output of action to tuple. 
pub fn to_tuple<I, O>(action: impl Action<I, O> + 'static) -> impl Action<I, (O, )>
    where
        I: 'static,
        O: 'static
{
    struct Runner<I, O> {
        r: Box<dyn TaskRunner>,
        o: TaskOutput<O>,
        token: CancellationToken,
        _m: PhantomData<I>,
    }
    impl<I, O> RunWithTaskOutput<(O, )> for Runner<I, O> {
        type In = I;

        fn run_with_task_output(&mut self, token: &mut CancellationToken, output: &mut TaskOutput<(O, )>, world: &mut World) -> bool {
            if token.requested_cancel() {
                self.token.cancel();
                return true;
            }
            self.r.run(world);
            if let Some(o) = self.o.take() {
                output.replace((o, ));
                true
            } else {
                false
            }
        }
    }
    let token = CancellationToken::default();
    let o = TaskOutput::default();
    let r = action.to_runner(token.clone(), o.clone());
    RunnerIntoAction::new(Runner {
        r: Box::new(r),
        o,
        token,
        _m: PhantomData,
    })
}