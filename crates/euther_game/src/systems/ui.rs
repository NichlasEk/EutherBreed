use bevy::prelude::*;

use crate::components::{NoticeText, SectionText, StatusText};
use crate::resources::{ApothecaryVitals, CampaignRuntime, GameNotice, LevelRuntime};

pub fn update_status_text(
    vitals: Res<ApothecaryVitals>,
    mut text_query: Query<&mut Text, With<StatusText>>,
) {
    if !vitals.is_changed() {
        return;
    }

    for mut text in &mut text_query {
        **text = format!(
            "Health {} | Reagent rounds {} | Bio-samples {}",
            vitals.0.health, vitals.0.ammo, vitals.0.bio_samples
        );
    }
}

pub fn update_section_text(
    runtime: Res<CampaignRuntime>,
    level_runtime: Res<LevelRuntime>,
    mut text_query: Query<&mut Text, With<SectionText>>,
) {
    if !runtime.is_changed() && !level_runtime.is_changed() {
        return;
    }

    let exits = if level_runtime.available_exits.is_empty() {
        "none".to_string()
    } else {
        level_runtime.available_exits.join(", ")
    };

    for mut text in &mut text_query {
        **text = format!(
            "Section {} | Exits {}",
            runtime.progress.current_level(),
            exits
        );
    }
}

pub fn update_notice_text(
    time: Res<Time>,
    mut notice: ResMut<GameNotice>,
    mut text_query: Query<&mut Text, With<NoticeText>>,
) {
    if notice.is_visible() {
        notice.timer.tick(time.delta());

        if notice.timer.is_finished() {
            notice.clear();
        }
    }

    if !notice.is_changed() {
        return;
    }

    for mut text in &mut text_query {
        **text = notice.text.clone();
    }
}
