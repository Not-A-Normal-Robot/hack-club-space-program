use crate::{
    fl,
    storage::{self, SaveInitError},
    systems::general::popup::{Popup, spawn_popup},
};
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task, futures::check_ready},
};
type StorageInitTaskResult = Result<(), SaveInitError>;

#[derive(Resource)]
#[doc(hidden)]
pub(crate) struct StorageInitTask(Task<StorageInitTaskResult>);

pub(crate) fn setup_storage(mut commands: Commands) {
    commands.insert_resource(StorageInitTask(
        IoTaskPool::get().spawn(storage::get_storage().init_saves()),
    ));
}

pub(crate) fn poll_storage_setup(mut commands: Commands, task: Option<ResMut<StorageInitTask>>) {
    let Some(mut task) = task else { return };

    let error = match check_ready(&mut task.0) {
        Some(Ok(())) => {
            info!("Save storage initialized");
            commands.remove_resource::<StorageInitTask>();
            return;
        }
        Some(Err(e)) => {
            commands.remove_resource::<StorageInitTask>();
            e
        }
        None => {
            // Not finished, we poll again later
            return;
        }
    };

    error!("{error}");

    commands.run_system_cached_with(
        spawn_popup,
        Popup {
            title: fl!("popup__saveInitError__title"),
            body: fl!("popup__saveInitError__body", inner = error.to_string()),
        },
    );
}
