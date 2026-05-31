use bevy::prelude::*;

use crate::components::StatusText;
use crate::resources::ApothecaryVitals;

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
