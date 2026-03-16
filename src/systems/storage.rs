use crate::storage::{self, SaveInitError};
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task, futures::check_ready},
};
use std::time::Duration;
type StorageInitTaskResult = Result<(), SaveInitError>;
type StorageInitTask = Task<StorageInitTaskResult>;

pub(crate) fn setup_storage(mut local: Local<Option<StorageInitTask>>) {
    let task = IoTaskPool::get().spawn(storage::init_saves());
    *local = Some(task);
}

pub(crate) fn poll_storage_setup(mut local: Local<Option<StorageInitTask>>) {
    let Some(mut task) = local.take() else { return };

    let error = match check_ready(&mut task) {
        Some(Ok(())) => {
            drop(task);
            return;
        }
        Some(Err(e)) => {
            drop(task);
            e
        }
        None => {
            // Not finished, we poll again later
            *local = Some(task);
            return;
        }
    };

    error!("{error}");

    todo!("proper error handling");
}
