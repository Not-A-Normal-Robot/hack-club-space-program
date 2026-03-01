use core::f32::consts::PI;

use bevy::{
    ecs::query::QueryData,
    input::mouse::MouseScrollUnit,
    input_focus::tab_navigation::{TabGroup, TabIndex},
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};

use crate::{
    assets::fonts::{URI_FONT_DOTO_ROUNDED_BOLD, URI_FONT_WDXL_LUBRIFONT_SC},
    builders::button::ButtonBuilder,
    checked_assign,
    components::ui::{ActiveTextColor, HoverTextColor, InactiveTextColor},
    consts::{
        about::{ABOUT_ENTRY_COUNT, load_article, load_article_title},
        colors::{
            scheme::{ON_TERTIARY, SURFACE, TERTIARY},
            shades::{
                NEUTRAL_50, PRIMARY_15, PRIMARY_50, PRIMARY_60, PRIMARY_80, PRIMARY_98,
                TERTIARY_40, TERTIARY_95,
            },
        },
        controls::MOUSE_WHEEL_ALT_DIR,
    },
    fl,
    resources::scene::GameScene,
    systems::general::ui_activation::ActivationEvent,
};

type ResponsiveQuery<'w, 's, 'qw, 'qs> = ParamSet<
    'w,
    's,
    (
        Query<'qw, 'qs, &'static mut Node, With<RootNode>>,
        Query<'qw, 'qs, &'static mut Node, With<HeaderTitle>>,
        Query<'qw, 'qs, &'static mut Node, With<BackButton>>,
        Query<'qw, 'qs, &'static mut Node, With<MainElement>>,
        Query<'qw, 'qs, &'static mut Node, With<AsideElement>>,
        Query<'qw, 'qs, &'static mut Node, With<MainAsideSeparator>>,
        Query<'qw, 'qs, &'static mut TextLayout, With<TabText>>,
    ),
>;

const ASIDE_FONT_SIZE: f32 = 24.0;
const MAIN_FONT_SIZE: f32 = 21.0;

#[derive(Component)]
#[require(DespawnOnExit::<GameScene>(GameScene::AboutMenu), TabGroup)]
pub(crate) struct RootNode;

#[derive(Component)]
pub(crate) struct BackButton;

#[derive(Component)]
pub(crate) struct HeaderTitle;

#[derive(Component)]
pub(crate) struct MainAsideSeparator;

#[derive(Component)]
pub(crate) struct MainElement;

#[derive(Component)]
pub(crate) struct ArticleElement;

#[derive(Component)]
pub(crate) struct AsideElement;

#[derive(Component)]
pub(crate) struct TabElement(usize);

#[derive(Component)]
pub(crate) struct TabText;

#[derive(Clone, Copy, Debug, Default, SubStates, PartialEq, Eq, Hash)]
#[source(GameScene = GameScene::AboutMenu)]
pub(crate) struct AboutTab(usize);

#[derive(Clone, Copy, Debug, PartialEq)]
struct TabStyle {
    bg_color: BackgroundColor,
    color: InactiveTextColor,
    hover_color: HoverTextColor,
    active_color: ActiveTextColor,
}

impl TabStyle {
    const UNSELECTED: Self = Self {
        bg_color: BackgroundColor(Color::NONE),
        color: InactiveTextColor(TERTIARY),
        hover_color: HoverTextColor(TERTIARY_95),
        active_color: ActiveTextColor(TERTIARY_40),
    };

    const SELECTED: Self = Self {
        bg_color: BackgroundColor(TERTIARY),
        color: InactiveTextColor(ON_TERTIARY),
        hover_color: HoverTextColor(ON_TERTIARY),
        active_color: ActiveTextColor(ON_TERTIARY),
    };
}

#[derive(Clone, Debug)]
struct ResponsiveData {
    root_template_cols: Vec<RepeatedGridTrack>,
    root_template_rows: Vec<RepeatedGridTrack>,
    title_display: Display,
    back_button_flex_grow: f32,
    main_rows: GridPlacement,
    main_cols: GridPlacement,
    main_padding: UiRect,
    aside_rows: GridPlacement,
    aside_cols: GridPlacement,
    aside_padding: UiRect,
    aside_direction: FlexDirection,
    aside_overflow: Overflow,
    main_aside_separator_rows: GridPlacement,
    main_aside_separator_cols: GridPlacement,
    tab_text_layout: TextLayout,
}

impl ResponsiveData {
    const SHOW_TITLE_THRESHOLD: f32 = 450.0;
    const MAIN_ASIDE_BOTTOM_THRESHOLD: f32 = 800.0;
    const ASIDE_BIG_WIDTH: f32 = 240.0;

    fn from_resolution(window_size: Vec2) -> Self {
        let show_title = window_size.x > Self::SHOW_TITLE_THRESHOLD;
        let aside_at_left = window_size.x > Self::MAIN_ASIDE_BOTTOM_THRESHOLD;

        Self {
            root_template_rows: if aside_at_left {
                vec![
                    RepeatedGridTrack::auto(1),
                    RepeatedGridTrack::px(1, 2.0),
                    RepeatedGridTrack::flex(1, 1.0),
                ]
            } else {
                vec![
                    RepeatedGridTrack::auto(1),
                    RepeatedGridTrack::px(1, 2.0),
                    RepeatedGridTrack::flex(1, 1.0),
                    RepeatedGridTrack::px(1, 2.0),
                    RepeatedGridTrack::auto(1),
                ]
            },
            root_template_cols: if aside_at_left {
                vec![
                    RepeatedGridTrack::px(1, Self::ASIDE_BIG_WIDTH),
                    RepeatedGridTrack::px(1, 2.0),
                    RepeatedGridTrack::flex(1, 1.0),
                ]
            } else {
                vec![RepeatedGridTrack::fr(1, 1.0)]
            },
            title_display: if show_title {
                Display::Flex
            } else {
                Display::None
            },
            back_button_flex_grow: if show_title { 0.0 } else { 1.0 },
            main_padding: UiRect::horizontal(Val::VMin(2.0)),
            aside_rows: if aside_at_left {
                GridPlacement::start(3)
            } else {
                GridPlacement::start(5)
            },
            aside_cols: GridPlacement::start(1),
            aside_padding: if aside_at_left {
                UiRect::horizontal(Val::VMin(2.0))
            } else {
                UiRect::vertical(Val::VMin(2.0))
            },
            aside_direction: if aside_at_left {
                FlexDirection::Column
            } else {
                FlexDirection::Row
            },
            aside_overflow: if aside_at_left {
                Overflow {
                    x: OverflowAxis::Hidden,
                    y: OverflowAxis::Scroll,
                }
            } else {
                Overflow {
                    x: OverflowAxis::Scroll,
                    y: OverflowAxis::Hidden,
                }
            },
            main_rows: GridPlacement::start(3),
            main_cols: if aside_at_left {
                GridPlacement::start(3)
            } else {
                GridPlacement::start(1)
            },
            main_aside_separator_rows: if aside_at_left {
                GridPlacement::start(3)
            } else {
                GridPlacement::start(4)
            },
            main_aside_separator_cols: if aside_at_left {
                GridPlacement::start(2)
            } else {
                GridPlacement::start(1)
            },
            tab_text_layout: if aside_at_left {
                TextLayout::new_with_linebreak(LineBreak::WordOrCharacter)
            } else {
                TextLayout::new_with_no_wrap()
            },
        }
    }

    fn apply(self, mut query: ResponsiveQuery) {
        for mut root in query.p0() {
            checked_assign!(
                root.grid_template_columns,
                self.root_template_cols,
                self.root_template_cols.clone(),
            );
            checked_assign!(
                root.grid_template_rows,
                self.root_template_rows,
                self.root_template_rows.clone(),
            );
        }

        for mut title in query.p1() {
            checked_assign!(title.display, self.title_display);
        }

        for mut back_button in query.p2() {
            checked_assign!(back_button.flex_grow, self.back_button_flex_grow);
        }

        for mut main in query.p3() {
            checked_assign!(main.grid_row, self.main_rows);
            checked_assign!(main.grid_column, self.main_cols);
            checked_assign!(main.padding, self.main_padding);
        }

        for mut aside in query.p4() {
            checked_assign!(aside.grid_row, self.aside_rows);
            checked_assign!(aside.grid_column, self.aside_cols);
            checked_assign!(aside.padding, self.aside_padding);
            checked_assign!(aside.flex_direction, self.aside_direction);
            checked_assign!(aside.overflow, self.aside_overflow);
        }

        for mut main_aside_separator in query.p5() {
            checked_assign!(
                main_aside_separator.grid_row,
                self.main_aside_separator_rows
            );
            checked_assign!(
                main_aside_separator.grid_column,
                self.main_aside_separator_cols
            );
        }

        for mut tab_text_layout in query.p6() {
            *tab_text_layout = self.tab_text_layout;
        }
    }
}

fn back_button(
    font: &TextFont,
    responsive_data: &ResponsiveData,
    commands: &mut Commands,
) -> Entity {
    let bundle = ButtonBuilder {
        extra: (
            BackButton,
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

fn title(font: &TextFont, responsive_data: &ResponsiveData, commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Node {
                display: responsive_data.title_display,
                flex_grow: 1.0,
                justify_content: JustifyContent::End,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            HeaderTitle,
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
            grid_column: GridPlacement::start_span(1, 3),
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
                grid_column: GridPlacement::start_span(1, 3),
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

fn main_aside_separator(responsive_data: &ResponsiveData, commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Node {
                min_width: Val::Px(2.0),
                min_height: Val::Px(2.0),
                grid_row: responsive_data.main_aside_separator_rows,
                grid_column: responsive_data.main_aside_separator_cols,
                ..Default::default()
            },
            BackgroundColor(NEUTRAL_50),
            MainAsideSeparator,
        ))
        .id()
}

fn article(index: usize, font: &TextFont, commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Text(load_article(index).into()),
            font.clone(),
            ArticleElement,
        ))
        .id()
}

fn main_node(
    responsive_data: &ResponsiveData,
    children: &[Entity],
    commands: &mut Commands,
) -> Entity {
    commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                padding: responsive_data.main_padding,
                grid_row: responsive_data.main_rows,
                grid_column: responsive_data.main_cols,
                overflow: Overflow {
                    x: OverflowAxis::Hidden,
                    y: OverflowAxis::Scroll,
                },
                ..Default::default()
            },
            MainElement,
        ))
        .observe(pointer_scroll_observer_system(MAIN_FONT_SIZE))
        .add_children(children)
        .id()
}

fn article_tab(
    responsive_data: &ResponsiveData,
    index: usize,
    selected_index: usize,
    font: &TextFont,
    commands: &mut Commands,
) -> Entity {
    let style = if index == selected_index {
        TabStyle::SELECTED
    } else {
        TabStyle::UNSELECTED
    };

    let bundle = ButtonBuilder {
        extra: (
            style.bg_color,
            TabIndex(0),
            TabElement(index),
            Node {
                padding: UiRect::axes(Val::Px(12.0), Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..Default::default()
            },
        ),
        text_extra: (TabText, responsive_data.tab_text_layout),
        text: load_article_title(index),
        font,
        color: style.color.0,
        hover_color: style.hover_color.0,
        active_color: style.active_color.0,
    }
    .build();

    commands
        .spawn(bundle)
        .observe(
            move |_: On<ActivationEvent>, mut cur_idx: ResMut<NextState<AboutTab>>| {
                cur_idx.set(AboutTab(index));
            },
        )
        .id()
}

fn article_tabs(
    responsive_data: &ResponsiveData,
    selected_index: usize,
    font: &TextFont,
    commands: &mut Commands,
) -> [Entity; ABOUT_ENTRY_COUNT] {
    core::array::from_fn(|i| article_tab(responsive_data, i, selected_index, font, commands))
}

fn aside_node(
    responsive_data: &ResponsiveData,
    children: &[Entity],
    commands: &mut Commands,
) -> Entity {
    commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: responsive_data.aside_direction,
                grid_row: responsive_data.aside_rows,
                grid_column: responsive_data.aside_cols,
                padding: responsive_data.aside_padding,
                min_width: Val::Px(48.0),
                min_height: Val::Px(48.0),
                max_height: Val::Vh(100.0),
                overflow: responsive_data.aside_overflow,
                ..Default::default()
            },
            AsideElement,
        ))
        .observe(pointer_scroll_observer_system(ASIDE_FONT_SIZE))
        .add_children(children)
        .id()
}

fn root_node(
    responsive_data: &ResponsiveData,
    children: &[Entity],
    commands: &mut Commands,
) -> Entity {
    commands
        .spawn((
            RootNode,
            Node {
                display: Display::Grid,
                grid_template_rows: responsive_data.root_template_rows.clone(),
                grid_template_columns: responsive_data.root_template_cols.clone(),
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
    selected_tab: Res<State<AboutTab>>,
    server: Res<AssetServer>,
) {
    let doto_font = server.load::<Font>(URI_FONT_DOTO_ROUNDED_BOLD);
    let wdxl_font = server.load::<Font>(URI_FONT_WDXL_LUBRIFONT_SC);

    let main_font = TextFont::from(wdxl_font.clone()).with_font_size(MAIN_FONT_SIZE);
    let tab_font = TextFont::from(wdxl_font).with_font_size(ASIDE_FONT_SIZE);

    let responsive_data =
        ResponsiveData::from_resolution(window.map(|w| w.size()).unwrap_or_default());

    let doto_font = TextFont::from(doto_font).with_font_size(32.0);

    let back_button = back_button(&doto_font, &responsive_data, &mut commands);
    let title = title(&doto_font, &responsive_data, &mut commands);
    let header = header(&[back_button, title], &mut commands);

    let header_separator = top_separator(&mut commands);

    let main = main_node(
        &responsive_data,
        &[article(selected_tab.get().0, &main_font, &mut commands)],
        &mut commands,
    );
    let main_aside_separator = main_aside_separator(&responsive_data, &mut commands);
    let aside = aside_node(
        &responsive_data,
        &article_tabs(
            &responsive_data,
            selected_tab.get().0,
            &tab_font,
            &mut commands,
        ),
        &mut commands,
    );

    root_node(
        &responsive_data,
        &[header, header_separator, main, main_aside_separator, aside],
        &mut commands,
    );

    commands.spawn((
        DespawnOnExit(GameScene::AboutMenu),
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(SURFACE),
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

fn handle_scroll(
    mut scroll_position: Mut<ScrollPosition>,
    node: &Node,
    computed: &ComputedNode,
    delta: Vec2,
) {
    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();

    if node.overflow.x == OverflowAxis::Scroll && delta.x != 0.0 {
        let at_scroll_limit = if delta.x > 0.0 {
            scroll_position.x >= max_offset.x
        } else {
            scroll_position.x <= 0.0
        };

        if !at_scroll_limit {
            scroll_position.x += delta.x;
        }
    }

    if node.overflow.y == OverflowAxis::Scroll && delta.y != 0.0 {
        let at_scroll_limit = if delta.y > 0.0 {
            scroll_position.y >= max_offset.y
        } else {
            scroll_position.y <= 0.0
        };

        if !at_scroll_limit {
            scroll_position.y += delta.y;
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct PointerScrollQueryData {
    scroll_position: &'static mut ScrollPosition,
    node: &'static Node,
    computed: &'static ComputedNode,
}

type ScrollQuery<'w, 's> = Query<'w, 's, PointerScrollQueryData>;

fn handle_pointer_scroll(
    scroll_data: PointerScrollQueryDataItem,
    keyboard_input: &ButtonInput<KeyCode>,
    scroll: &Scroll,
    font_size: f32,
) {
    let mut delta = -Vec2::new(scroll.x, scroll.y);

    match scroll.unit {
        MouseScrollUnit::Line => delta *= font_size,
        MouseScrollUnit::Pixel => (),
    }

    if keyboard_input.any_pressed(MOUSE_WHEEL_ALT_DIR) {
        core::mem::swap(&mut delta.x, &mut delta.y);
    }

    handle_scroll(
        scroll_data.scroll_position,
        scroll_data.node,
        scroll_data.computed,
        delta,
    );
}

fn pointer_scroll_observer_system(
    font_size: f32,
) -> impl Fn(On<Pointer<Scroll>>, ScrollQuery, Res<ButtonInput<KeyCode>>) {
    move |event: On<Pointer<Scroll>>,
          mut query: ScrollQuery,
          keyboard_input: Res<ButtonInput<KeyCode>>| {
        if let Ok(data) = query.get_mut(event.entity) {
            handle_pointer_scroll(data, &keyboard_input, event.event(), font_size);
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct TabComponents {
    bg_color: &'static mut BackgroundColor,
    tab_index: &'static TabElement,
    children: &'static Children,
    inactive_color: &'static mut InactiveTextColor,
    hover_color: &'static mut HoverTextColor,
    active_color: &'static mut ActiveTextColor,
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct TabTextColorComponents {
    current: &'static mut TextColor,
}

pub(crate) fn handle_tab_switch(
    mut main: Query<&mut ScrollPosition, With<MainElement>>,
    mut article: Query<&mut Text, With<ArticleElement>>,
    mut tabs: Query<TabComponents>,
    mut tab_texts: Query<TabTextColorComponents>,
    cur_tab: Res<State<AboutTab>>,
) {
    for mut pos in &mut main {
        pos.0 = Vec2::ZERO;
    }

    for mut text in &mut article {
        text.0 = load_article(cur_tab.get().0).to_string();
    }

    for mut tab in &mut tabs {
        let selected = tab.tab_index.0 == cur_tab.get().0;
        let style = if selected {
            TabStyle::SELECTED
        } else {
            TabStyle::UNSELECTED
        };

        checked_assign!(*tab.bg_color, style.bg_color);
        checked_assign!(*tab.inactive_color, style.color);
        checked_assign!(*tab.hover_color, style.hover_color);
        checked_assign!(*tab.active_color, style.active_color);

        for entity in tab.children {
            let Ok(mut text) = tab_texts.get_mut(*entity) else {
                continue;
            };

            checked_assign!(text.current.0, style.color.0);
        }
    }
}
