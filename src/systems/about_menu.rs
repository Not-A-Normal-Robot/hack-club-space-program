use core::f32::consts::PI;

use bevy::{
    input_focus::tab_navigation::{TabGroup, TabIndex},
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};

use crate::{
    assets::fonts::{URI_FONT_DOTO_ROUNDED_BOLD, URI_FONT_WDXL_LUBRIFONT_SC},
    builders::button::ButtonBuilder,
    checked_assign,
    consts::colors::shades::{
        NEUTRAL_50, PRIMARY_15, PRIMARY_50, PRIMARY_60, PRIMARY_80, PRIMARY_98, SECONDARY_80,
        TERTIARY_30,
    },
    fl,
    resources::scene::GameScene,
    systems::general::ui_activation::ActivationEvent,
};

#[derive(Component)]
#[require(DespawnOnExit::<GameScene>(GameScene::AboutMenu), TabGroup)]
pub(crate) struct AboutMenuRootNode;

type ResponsiveQuery<'w, 's, 'qw, 'qs> = ParamSet<
    'w,
    's,
    (
        Query<'qw, 'qs, &'static mut Node, With<AboutMenuTitle>>,
        Query<'qw, 'qs, &'static mut Node, With<AboutMenuBackButton>>,
        Query<'qw, 'qs, &'static mut Node, With<MainAsideWrapper>>,
    ),
>;

#[derive(Component)]
pub(crate) struct AboutMenuBackButton;

#[derive(Component)]
pub(crate) struct AboutMenuTitle;

#[derive(Component)]
pub(crate) struct MainAsideWrapper;

#[derive(Clone, Copy, Debug, PartialEq)]
struct ResponsiveData {
    title_display: Display,
    back_button_flex_grow: f32,
    main_aside_wrapper_direction: FlexDirection,
}

impl ResponsiveData {
    const SHOW_TITLE_THRESHOLD: f32 = 450.0;
    const MAIN_ASIDE_DIRECTION_THRESHOLD: f32 = 500.0;

    fn from_resolution(window_size: Vec2) -> Self {
        let show_title = window_size.x > Self::SHOW_TITLE_THRESHOLD;

        Self {
            title_display: if show_title {
                Display::Flex
            } else {
                Display::None
            },
            back_button_flex_grow: if show_title { 0.0 } else { 1.0 },
            main_aside_wrapper_direction: if window_size.x > Self::MAIN_ASIDE_DIRECTION_THRESHOLD {
                FlexDirection::Row
            } else {
                FlexDirection::ColumnReverse
            },
        }
    }

    fn apply(self, mut query: ResponsiveQuery) {
        for mut title in &mut query.p0() {
            checked_assign!(title.display, self.title_display);
        }

        for mut back_button in &mut query.p1() {
            checked_assign!(back_button.flex_grow, self.back_button_flex_grow);
        }

        for mut main_aside in &mut query.p2() {
            checked_assign!(main_aside.flex_direction, self.main_aside_wrapper_direction);
        }
    }
}

fn back_button(
    font: &TextFont,
    responsive_data: ResponsiveData,
    commands: &mut Commands,
) -> Entity {
    let bundle = ButtonBuilder {
        extra: (
            AboutMenuBackButton,
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                min_width: Val::Px(96.0),
                min_height: Val::Px(48.0),
                flex_grow: responsive_data.back_button_flex_grow,
                ..Node::DEFAULT
            },
            TextLayout {
                justify: Justify::Center,
                linebreak: LineBreak::WordOrCharacter,
            },
            TabIndex(0),
        ),
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

fn title(font: &TextFont, responsive_data: ResponsiveData, commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Node {
                display: responsive_data.title_display,
                flex_grow: 1.0,
                justify_content: JustifyContent::End,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            AboutMenuTitle,
            children![(
                Text::new(fl!("aboutMenu__title__text")),
                font.clone(),
                TextColor(PRIMARY_98),
            )],
        ))
        .id()
}

fn header(children: &[Entity], commands: &mut Commands) -> Entity {
    commands
        .spawn((Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            padding: UiRect::all(Val::VMin(2.0)).with_bottom(Val::ZERO),
            ..Default::default()
        },))
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

fn aside_node(children: &[Entity], commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                min_width: Val::Px(48.0),
                min_height: Val::Px(48.0),
                ..Default::default()
            },
            BackgroundColor(TERTIARY_30), // DEBUG
        ))
        .add_children(children)
        .id()
}

fn main_aside_separator(responsive_data: ResponsiveData, commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Node {
                min_width: Val::Px(2.0),
                min_height: Val::Px(2.0),
                ..Default::default()
            },
            BackgroundColor(NEUTRAL_50),
        ))
        .id()
}

fn main_node(children: &[Entity], commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                ..Default::default()
            },
            BackgroundColor(PRIMARY_15), // DEBUG
        ))
        .add_children(children)
        .id()
}

fn main_aside_wrapper(
    responsive_data: ResponsiveData,
    children: &[Entity],
    commands: &mut Commands,
) -> Entity {
    commands
        .spawn((
            MainAsideWrapper,
            Node {
                display: Display::Flex,
                flex_direction: responsive_data.main_aside_wrapper_direction,
                align_items: AlignItems::Stretch,
                flex_grow: 1.0,
                ..Default::default()
            },
        ))
        .add_children(children)
        .id()
}

fn root_node(children: &[Entity], commands: &mut Commands) -> Entity {
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
        .add_children(children)
        .id()
}

pub(crate) fn init_about_menu(
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    mut commands: Commands,
    server: Res<AssetServer>,
) {
    let doto_font = server.load::<Font>(URI_FONT_DOTO_ROUNDED_BOLD);
    let wdxl_font = server.load::<Font>(URI_FONT_WDXL_LUBRIFONT_SC);

    let responsive_data =
        ResponsiveData::from_resolution(window.map(|w| w.size()).unwrap_or_default());

    let doto_font = TextFont::from(doto_font).with_font_size(32.0);

    let back_button = back_button(&doto_font, responsive_data, &mut commands);
    let title = title(&doto_font, responsive_data, &mut commands);
    let header = header(&[back_button, title], &mut commands);

    let header_separator = top_separator(&mut commands);

    let main = main_node(&[], &mut commands);
    let main_aside_separator = main_aside_separator(responsive_data, &mut commands);
    let aside = aside_node(&[], &mut commands);
    let main_aside_wrapper = main_aside_wrapper(
        responsive_data,
        &[aside, main_aside_separator, main],
        &mut commands,
    );

    root_node(
        &[header, header_separator, main_aside_wrapper],
        &mut commands,
    );

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

pub(crate) fn handle_resize(
    query: ResponsiveQuery,
    mut resize_reader: MessageReader<WindowResized>,
) {
    let Some(resize) = resize_reader.read().last() else {
        return;
    };

    ResponsiveData::from_resolution(Vec2::new(resize.width, resize.height)).apply(query);
}
