use bevy::prelude::*;

use crate::components::{NoticeText, StatusText};
use crate::resources::{ApothecaryVitals, GameNotice};

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
