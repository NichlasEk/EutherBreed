use bevy::prelude::*;

use crate::resources::{CampaignRuntime, CampaignSignal};

pub fn update_campaign_progress(
    mut signal: ResMut<CampaignSignal>,
    mut runtime: ResMut<CampaignRuntime>,
) {
    let Some(target) = signal.pending_exit_target.take() else {
        return;
    };

    let is_known_level = runtime.definition.contains_level(&target);

    match runtime
        .progress
        .travel_to_known_level(is_known_level, &target)
    {
        Ok(true) => info!(
            "campaign traveled to level {}",
            runtime.progress.current_level()
        ),
        Ok(false) => debug!("campaign already at level {}", target),
        Err(error) => warn!("campaign travel to {} failed: {:?}", target, error),
    }
}
