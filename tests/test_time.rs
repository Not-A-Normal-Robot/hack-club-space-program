//! Test for testing that time passes in tests.

use core::sync::atomic::{AtomicU8, Ordering};

use bevy::prelude::*;

mod common;

static TICKS: AtomicU8 = AtomicU8::new(0);

fn test_time(time: Res<Time<Fixed>>) {
    let old_ticks = TICKS.fetch_add(1, Ordering::SeqCst);
    assert_eq!(time.delta(), time.timestep());
    assert_eq!(
        time.elapsed(),
        time.timestep().saturating_mul(u32::from(old_ticks) + 1)
    );
}

#[test]
fn time_works() {
    let mut app = common::setup_default();

    app.add_systems(FixedUpdate, test_time);

    assert_eq!(TICKS.load(Ordering::SeqCst), 0);

    app.update();

    assert_eq!(TICKS.load(Ordering::SeqCst), 1);

    app.update();

    assert_eq!(TICKS.load(Ordering::SeqCst), 2);
}
