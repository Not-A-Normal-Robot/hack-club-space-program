use bevy::{math::DVec2, prelude::*};

use crate::{
    assets::fonts::{URI_FONT_JETBRAINS_MONO, URI_FONT_JETBRAINS_MONO_ITALIC},
    components::main_game::{
        frames::{RootSpaceLinearVelocity, RootSpacePosition},
        ui::speedometer::{
            HorizontalSpeedometerText, SpeedometerUnitText, TotalSpeedometerText,
            VerticalSpeedometerText,
        },
    },
    consts::{
        colors::{
            SPEEDOMETER_BACKGROUND, SPEEDOMETER_BORDER, SPEEDOMETER_DOTS, SPEEDOMETER_HSPD,
            SPEEDOMETER_TSPD, SPEEDOMETER_VSPD,
        },
        si::SIPrefix,
        ui::speedometer::{
            DIRECTIONAL_FONT_SIZE, SpeedometerFormat, SpeedometerUnit, TSPD_FONT_SIZE,
            UNIT_FONT_SIZE,
        },
    },
    fl,
    resources::{scene::GameScene, simulation::ActiveVessel},
};

#[must_use]
fn directional_display(
    tag: impl Component,
    text: String,
    color: Color,
    label_font: TextFont,
    value_font: TextFont,
    commands: &mut Commands,
) -> Entity {
    let label = (Text(text), TextColor(color), label_font);
    let value = (Text("0.000".into()), TextColor(color), value_font, tag);

    commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                ..Default::default()
            },
            children![label, value],
        ))
        .id()
}

#[must_use]
fn hspd_display(label_font: TextFont, value_font: TextFont, commands: &mut Commands) -> Entity {
    directional_display(
        HorizontalSpeedometerText,
        fl!("speedometer__horizontalSpeed__label"),
        SPEEDOMETER_HSPD,
        label_font,
        value_font,
        commands,
    )
}

#[must_use]
fn vspd_display(label_font: TextFont, value_font: TextFont, commands: &mut Commands) -> Entity {
    directional_display(
        VerticalSpeedometerText,
        fl!("speedometer__verticalSpeed__label"),
        SPEEDOMETER_VSPD,
        label_font,
        value_font,
        commands,
    )
}

#[must_use]
fn tspd_display(font: TextFont, commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Node {
                margin: UiRect::vertical(Val::Px(4.0)),
                ..Default::default()
            },
            font,
            Text("0.000".into()),
            TextColor(SPEEDOMETER_TSPD),
            TotalSpeedometerText,
        ))
        .id()
}

#[must_use]
fn unit_dots(commands: &mut Commands) -> Entity {
    const DOT_AMOUNT: u8 = 8;

    let children: [Entity; DOT_AMOUNT as usize] = core::array::from_fn(|i| {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap,
            reason = "`i` has a cap of u8::MAX, so this can never overflow"
        )]
        let grid_placement = GridPlacement::start((i * 2 + 1) as i16);
        commands
            .spawn((
                Node {
                    height: Val::Px(2.5),
                    grid_column: grid_placement,
                    ..Default::default()
                },
                BackgroundColor(SPEEDOMETER_DOTS),
            ))
            .id()
    });

    commands
        .spawn((Node {
            display: Display::Grid,
            grid_template_columns: vec![RepeatedGridTrack::flex(
                const { DOT_AMOUNT as u16 * 2 - 1 },
                1.0,
            )],
            flex_grow: 1.0,
            align_items: AlignItems::Center,
            ..Default::default()
        },))
        .add_children(&children)
        .id()
}

#[must_use]
fn unit_display(font: TextFont, commands: &mut Commands) -> Entity {
    let dots = unit_dots(commands);
    let unit = commands
        .spawn((
            Text(SpeedometerUnit::from(SIPrefix::Unit).to_string()),
            font,
            SpeedometerUnitText,
        ))
        .id();

    commands
        .spawn((Node {
            display: Display::Flex,
            ..Default::default()
        },))
        .add_children(&[dots, unit])
        .id()
}

fn root(children: &[Entity], commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_self: AlignSelf::End,
                justify_self: JustifySelf::End,
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                border: UiRect::left(Val::Px(1.0)).with_top(Val::Px(1.0)),
                border_radius: BorderRadius::top_left(Val::Px(4.0)),
                ..Default::default()
            },
            BackgroundColor(SPEEDOMETER_BACKGROUND),
            BorderColor::all(SPEEDOMETER_BORDER),
            ZIndex(5),
            DespawnOnExit(GameScene::InGame),
        ))
        .add_children(children)
        .id()
}

pub(crate) fn init_speedometer(mut commands: Commands, server: Res<AssetServer>) {
    let jetbrains_mono = server.load::<Font>(URI_FONT_JETBRAINS_MONO);
    let jetbrains_mono_italic = server.load::<Font>(URI_FONT_JETBRAINS_MONO_ITALIC);

    let dir_label_font =
        TextFont::from(jetbrains_mono_italic.clone()).with_font_size(DIRECTIONAL_FONT_SIZE);
    let dir_value_font =
        TextFont::from(jetbrains_mono.clone()).with_font_size(DIRECTIONAL_FONT_SIZE);

    let hspd = hspd_display(
        dir_label_font.clone(),
        dir_value_font.clone(),
        &mut commands,
    );
    let vspd = vspd_display(dir_label_font, dir_value_font, &mut commands);
    let tspd = tspd_display(
        TextFont::from(jetbrains_mono).with_font_size(TSPD_FONT_SIZE),
        &mut commands,
    );
    let unit_display = unit_display(
        TextFont::from(jetbrains_mono_italic).with_font_size(UNIT_FONT_SIZE),
        &mut commands,
    );

    root(&[hspd, vspd, tspd, unit_display], &mut commands);
}

pub(crate) fn calculate_speedometer_format(
    sv_query: Query<(&RootSpacePosition, &RootSpaceLinearVelocity)>,
    active_vessel: Res<ActiveVessel>,
) -> Option<SpeedometerFormat> {
    let vessel_sv = sv_query
        .get(active_vessel.entity)
        .map(|(&x, &y)| (x, y))
        .unwrap_or((
            active_vessel.prev_tick_position,
            active_vessel.prev_tick_velocity,
        ));

    let Ok(parent_sv) = sv_query
        .get(active_vessel.prev_tick_parent)
        .map(|(&x, &y)| (x, y))
    else {
        return None;
    };

    let (rel_pos, rel_vel) = (vessel_sv.0.0 - parent_sv.0.0, vessel_sv.1.0 - parent_sv.1.0);

    let up = rel_pos.normalize_or(DVec2::new(1.0, 0.0));
    let perp_up = up.perp();
    let hspd = rel_vel.dot(perp_up).abs();
    let vspd = rel_vel.dot(up);
    let tspd = rel_vel.length();

    Some(SpeedometerFormat::from_speeds(hspd, vspd, tspd))
}

type SpeedometerUpdateQuery<'w, 's, 'qw, 'qs> = ParamSet<
    'w,
    's,
    (
        Query<'qw, 'qs, &'static mut Text, With<HorizontalSpeedometerText>>,
        Query<'qw, 'qs, &'static mut Text, With<VerticalSpeedometerText>>,
        Query<'qw, 'qs, &'static mut Text, With<TotalSpeedometerText>>,
        Query<'qw, 'qs, &'static mut Text, With<SpeedometerUnitText>>,
    ),
>;

pub(crate) fn apply_speedometer_format(
    In(format): In<Option<SpeedometerFormat>>,
    mut query: SpeedometerUpdateQuery,
) {
    let Some(format) = format else { return };

    for mut hspd in query.p0() {
        hspd.0.clear();

        for ch in format.hspd {
            hspd.0.push(ch);
        }
    }

    for mut vspd in query.p1() {
        vspd.0.clear();

        for ch in format.vspd {
            vspd.0.push(ch);
        }
    }

    for mut tspd in query.p2() {
        tspd.0.clear();

        for ch in format.tspd {
            tspd.0.push(ch);
        }
    }

    for mut unit in query.p3() {
        unit.0.clear();

        for ch in format.unit.to_text() {
            unit.0.push(ch);
        }
    }
}
