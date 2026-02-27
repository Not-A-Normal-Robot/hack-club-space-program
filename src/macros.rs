/// Observes for activation on a given entity.
///
/// # Activation
/// An activation is an action that is normally expected to trigger a button.\
/// Currently, this applies to both pointer clicks and Enter keypresses
///
/// # Mutability
/// Note that due to `macro_rules!` quirks, you should prepend your immutable
/// closure arguments with `final`, and your mutable ones with `mut`.
///
/// # Examples
/// ```
/// # use bevy::prelude::*;
/// # use hack_club_space_program::observe_activation;
/// # fn f(get_entity_commands: impl FnOnce() -> EntityCommands<'static>) {
/// let entity_commands: EntityCommands = get_entity_commands();
///
/// observe_activation!(
///     entity_commands,
///     || { eprintln!("Activated!") }
/// );
/// # }
/// ```
///
/// ```
/// # use bevy::prelude::*;
/// # use hack_club_space_program::observe_activation;
/// # fn f(get_entity_commands: impl FnOnce() -> EntityCommands<'static>) {
/// # let entity_commands: EntityCommands = get_entity_commands();
///
/// observe_activation!(
///     entity_commands,
///     |final name: Single<&Name, With<Camera>>| {
///         eprintln!("{}", name.as_str());
///     }
/// );
/// # }
/// ```
#[macro_export]
macro_rules! observe_activation {
    ($entity_commands:expr, |$( $mutability:ident $var:ident $(: $vartype:ty)?),* $(,)?| $inner:block) => {
        {
            type OnPointerClick<'a, 'b> = ::bevy::prelude::On<
                'a,
                'b,
                ::bevy::picking::events::Pointer<
                    ::bevy::picking::events::Click
                >,
            >;
            type OnFocusedKeyboardInput<'a, 'b> = ::bevy::prelude::On<
                'a,
                'b,
                ::bevy::input_focus::FocusedInput<
                    ::bevy::input::keyboard::KeyboardInput
                >,
            >;

            let mut entity_commands = $entity_commands;
            entity_commands.observe(
                |_: OnPointerClick, $(
                    observe_activation!(@mutability $mutability $var)
                    $(
                        : $vartype
                    )?),* | {
                    $inner
                });
            entity_commands.observe(
                | _macro_var_input: OnFocusedKeyboardInput, $(
                    observe_activation!(@mutability $mutability $var)
                    $(
                        : $vartype
                    )?
                ),* | {
                    if _macro_var_input.input.repeat
                        || !_macro_var_input.input.state.is_pressed()
                        || !matches!(
                            _macro_var_input.input.key_code,
                            ::bevy::input::keyboard::KeyCode::Enter |
                            ::bevy::input::keyboard::KeyCode::NumpadEnter |
                            ::bevy::input::keyboard::KeyCode::Space
                        )
                    {
                        return;
                    }

                    $inner
                });
            entity_commands
        }
    };
    ($entity_commands:expr, || $inner:block) => {
        observe_activation!($entity_commands, |,| $inner)
    };
    (@mutability final $var:ident) => {
        $var
    };
    (@mutability mut $var:ident) => {
        mut $var
    };
}

#[macro_export]
macro_rules! fl {
    ($($inner:tt)*) => {
        ::i18n_embed_fl::fl!(*$crate::consts::FLUENT_LANGUAGE_LOADER, $($inner)*)
    };
}

#[cfg(test)]
mod tests {
    //! All tests here are only for type checks and macro validation.

    use crate::{components::frames::RootSpacePosition, resources::simulation::ActiveVessel};
    use bevy::{
        ecs::{
            name::NameOrEntity,
            system::{EntityCommands, ResMut, Single},
        },
        math::DVec2,
    };

    /// This only checks for type safety
    fn _test_observe_activation(mut entity_commands: EntityCommands) {
        observe_activation!(entity_commands.reborrow(), || {
            eprintln!("Activated!");
        });
        observe_activation!(entity_commands.reborrow(), |final query: Single<NameOrEntity>| {
            let query = query.into_inner();
            eprintln!("{query}");
        });
        observe_activation!(entity_commands.reborrow(), |mut vessel: ResMut<
            ActiveVessel,
        >| {
            vessel.prev_tick_position = RootSpacePosition(DVec2::ZERO);
        });
    }

    #[test]
    fn test_fl() {
        let x = ::i18n_embed_fl::fl!(
            crate::consts::FLUENT_LANGUAGE_LOADER,
            "mainMenu__playButton__text"
        );
        let y = fl!("mainMenu__playButton__text");
        assert_eq!(x, y);
    }
}
