//! A switch is a structure that represents two states: `on` and `off`.
//!
//! This is to solve the problem that systems created from `Reactors`
//! cannot run except on the main thread.
//!
//! Resource
//!
//! - [`Switch`]
//!
//! run conditions
//!
//! - [`switch_is_on`]
//! - [`switch_is_off`]
//! - [`switch_just_turned_on`]
//! - [`switch_just_turned_off`]
//!
//! actions 
//!
//! - [`once::switch::on`](crate::prelude::once::switch::on)
//! - [`once::switch::off`](crate::prelude::once::switch::off)
//! - [`wait::switch::on`](crate::prelude::wait::switch::on)
//! - [`wait::switch::off`](crate::prelude::wait::switch::off)


use std::marker::PhantomData;

use bevy::prelude::{Local, Mut, Res, Resource, World};

/// A Condition-satisfying system that returns true if the switch has been turned on.
#[inline]
pub fn switch_is_on<M>(switch: Option<Res<Switch<M>>>) -> bool
    where M: Send + Sync + 'static
{
    switch.is_some_and(|s| s.is_on())
}


/// A Condition-satisfying system that returns true if the switch has been turned off.
#[inline]
pub fn switch_is_off<M>(switch: Option<Res<Switch<M>>>) -> bool
    where M: Send + Sync + 'static
{
    switch.is_some_and(|s| {
        s.is_off()
    })
}

/// A Condition-satisfying system that returns true if the switch has just been turned on. 
#[inline]
pub fn switch_just_turned_on<M>(
    switch: Option<Res<Switch<M>>>,
    mut is_on: Local<bool>,
) -> bool
    where M: Send + Sync + 'static
{
    if switch.is_some_and(|s| s.is_on()) {
        if *is_on {
            false
        } else {
            *is_on = true;
            true
        }
    } else {
        *is_on = false;
        false
    }
}

/// A Condition-satisfying system that returns true if the switch has just been turned off. 
#[inline]
pub fn switch_just_turned_off<M>(
    switch: Option<Res<Switch<M>>>,
    mut is_off: Local<bool>,
) -> bool
    where M: Send + Sync + 'static
{
    if switch.is_some_and(|s| s.is_off()) {
        if *is_off {
            false
        } else {
            *is_off = true;
            true
        }
    } else {
        *is_off = false;
        false
    }
}

/// A switch is a structure that represents two states: `on` and `off`.
///
/// This is to solve the problem that systems created from `Reactors`
/// cannot run except on the main thread.
///
/// ## Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_flurx::prelude::*;
///
/// struct HeavyTask;
///
/// App::new()
///     .add_systems(Update, (|mut switch: ResMut<Switch<HeavyTask>>|{
///         // heavy task
///         //..
///         
///         switch.off();
///     }).run_if(switch_is_on::<HeavyTask>))
///     .add_systems(Update, |mut commands: Commands|{
///         commands.spawn(Reactor::schedule(|task| async move{
///             task.will(Update, once::switch::on::<HeavyTask>()).await;
///             task.will(Update, wait::switch::off::<HeavyTask>()).await;
///         })); 
///     });
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct Switch<M> {
    just_change: bool,
    turned_on: bool,
    checked: bool,
    _m: PhantomData<M>,
}

impl<M> Resource for Switch<M>
    where M: Send + Sync + 'static
{}

impl<M> Switch<M>
    where M: Send + Sync + 'static
{
    /// Create new Switch with initial status.
    #[inline(always)]
    const fn new(turn_on: bool) -> Switch<M> {
        Self {
            just_change: true,
            turned_on: turn_on,
            checked: false,
            _m: PhantomData,
        }
    }

    /// Returns true if switch is on.
    #[inline(always)]
    pub const fn is_on(&self) -> bool {
        self.turned_on
    }

    /// Returns true if switch is off.
    #[inline(always)]
    pub const fn is_off(&self) -> bool {
        !self.turned_on
    }

    /// Sets turn on or off.
    pub fn set(&mut self, turn_on: bool) {
        if turn_on {
            self.on();
        } else {
            self.off();
        }
    }

    /// Turn on the switch.
    #[inline(always)]
    pub fn on(&mut self) {
        if self.is_off() {
            self.just_change = true;
            self.turned_on = true;
        }
    }

    /// Turn off the switch.
    #[inline(always)]
    pub fn off(&mut self) {
        if self.turned_on {
            self.just_change = true;
            self.turned_on = false;
        }
    }

    pub(crate) fn setup(world: &mut World, turn_on: bool) -> Mut<Switch<M>> {
        world.insert_resource(Self::new(turn_on));
        world.resource_mut::<Switch<M>>()
    }
}

impl<M> Default for Switch<M>
    where M: Send + Sync + 'static
{
    fn default() -> Self {
        Self::new(false)
    }
}


#[cfg(test)]
mod tests {
    use crate::prelude::Switch;

    struct T;

    #[test]
    fn off() {
        let mut s = Switch::<T>::new(true);
        assert!(s.just_change);
        assert!(s.turned_on);
        s.just_change = false;
        s.off();
        assert!(s.just_change);
        assert!(s.is_off());
    }

    #[test]
    fn on() {
        let mut s = Switch::<T>::new(false);
        assert!(s.just_change);
        assert!(s.is_off());
        s.just_change = false;
        s.on();
        assert!(s.just_change);
        assert!(s.is_on());
    }
}