use core::f32::consts::PI;

use bevy::{
    input_focus::tab_navigation::{TabGroup, TabIndex},
    prelude::*,
    window::WindowResized,
};

use crate::{
    assets::fonts::{URI_FONT_DOTO_ROUNDED_BOLD, URI_FONT_WDXL_LUBRIFONT_SC},
    builders::button::ButtonBuilder,
    checked_assign,
    consts::colors::shades::{
        PRIMARY_15, PRIMARY_50, PRIMARY_60, PRIMARY_80, PRIMARY_98, SECONDARY_20, TERTIARY_30,
    },
    fl,
    resources::scene::GameScene,
    systems::general::ui_activation::ActivationEvent,
};

#[derive(Component)]
#[require(DespawnOnExit::<GameScene>(GameScene::AboutMenu), TabGroup)]
pub(crate) struct AboutMenuRootNode;

type ResponsiveQuery<'w, 's> = ParamSet<
    'w,
    's,
    (
        Single<'w, 's, &'static mut Node, With<AboutMenuTitle>>,
        Single<'w, 's, &'static mut Node, With<AboutMenuBackButton>>,
    ),
>;

#[derive(Component)]
pub(crate) struct AboutMenuBackButton;

#[derive(Component)]
pub(crate) struct AboutMenuTitle;

struct ResponsiveData {
    title_display: Display,
    back_button_flex_grow: f32,
}

impl ResponsiveData {
    fn from_resolution(window_size: Vec2) -> Self {
        let show_title = window_size.x > 350.0;

        Self {
            title_display: if show_title {
                Display::Flex
            } else {
                Display::None
            },
            back_button_flex_grow: if show_title { 0.0 } else { 1.0 },
        }
    }

    fn apply(self, mut query: ResponsiveQuery) {
        let mut title = query.p0();
        checked_assign!(title.display, self.title_display);

        let mut back_button = query.p1();
        checked_assign!(back_button.flex_grow, self.back_button_flex_grow);
    }
}

#[inline]
fn button_common() -> impl Bundle {
    (
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            min_width: Val::Px(96.0),
            min_height: Val::Px(48.0),
            ..Node::DEFAULT
        },
        TextLayout {
            justify: Justify::Center,
            linebreak: LineBreak::WordOrCharacter,
        },
        TabIndex(0),
    )
}

fn back_button(font: &TextFont, commands: &mut Commands) -> Entity {
    let bundle = ButtonBuilder {
        extra: (AboutMenuBackButton, button_common()),
        text_extra: (),
        text: fl!("aboutMenu__backButton__text"),
        font,
        color: PRIMARY_60,
        hover_color: PRIMARY_80,
        active_color: PRIMARY_50,
    }
    .build();

    commands
        .spawn(bundle)
        .observe(
            |_: On<ActivationEvent>, mut scene: ResMut<NextState<GameScene>>| {
                scene.set(GameScene::MainMenu);
            },
        )
        .id()
}

fn title(font: &TextFont, commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Node {
                flex_grow: 1.0,
                justify_content: JustifyContent::End,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            children![(
                Text::new(fl!("aboutMenu__title__text")),
                font.clone(),
                TextColor(PRIMARY_98),
            )],
        ))
        .id()
}

fn top_row(children: &[Entity], commands: &mut Commands) -> Entity {
    commands
        .spawn((Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            padding: UiRect::all(Val::VMin(2.0)).with_bottom(Val::ZERO),
            ..Default::default()
        },))
        // .add_children([back_button, title].as_slice())
        .add_children(children)
        .id()
}

fn top_separator(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Node {
                height: Val::Px(2.0),
                margin: UiRect::horizontal(Val::VMin(2.0)),
                ..Default::default()
            },
            BackgroundGradient(vec![Gradient::Linear(LinearGradient {
                angle: const { PI / 2.0 },
                color_space: InterpolationColorSpace::Srgba,
                stops: vec![
                    ColorStop::new(PRIMARY_15, Val::Percent(0.0)),
                    ColorStop::new(PRIMARY_80, Val::Percent(100.0)),
                ],
            })]),
        ))
        .id()
}

pub(crate) fn init_about_menu(mut commands: Commands, server: Res<AssetServer>) {
    let doto_font = server.load::<Font>(URI_FONT_DOTO_ROUNDED_BOLD);
    let wdxl_font = server.load::<Font>(URI_FONT_WDXL_LUBRIFONT_SC);

    let doto_font = TextFont::from(doto_font).with_font_size(32.0);

    let back_button = back_button(&doto_font, &mut commands);
    let title = title(&doto_font, &mut commands);
    let top_row = top_row([back_button, title].as_slice(), &mut commands);

    let top_separator = top_separator(&mut commands);

    commands
        .spawn((
            AboutMenuRootNode,
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                width: Val::Vw(100.0),
                height: Val::Vh(100.0),
                ..Default::default()
            },
        ))
        .add_children([top_row, top_separator].as_slice());

    commands.spawn((
        DespawnOnExit(GameScene::AboutMenu),
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            is_active: true,
            ..Default::default()
        },
        IsDefaultUiCamera,
    ));
}

pub(crate) fn handle_resize(mut resize_reader: MessageReader<WindowResized>) {
    // TODO
}
