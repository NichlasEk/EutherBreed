pub mod campaign;
pub mod collision;
pub mod level;
pub mod progression;
pub mod rules;
pub mod save;
pub mod state;
pub mod vitals;

pub use campaign::{
    CampaignDefinition, CampaignLevel, CampaignProgress, CampaignTravelError,
    CampaignValidationError,
};
pub use collision::{AxisAlignedBox, circle_intersects_aabb};
pub use level::{
    DecorDefinition, DecorKind, DoorDefinition, DoorKind, LevelDefinition, LevelExit,
    ObjectiveDefinition, PickupKind, PrototypeEntity, TerminalDefinition, TerminalKind,
};
pub use progression::{ExitReadiness, ObjectiveProgress};
pub use rules::{RuleContext, RuleGate, RuleGateStatus, RuleRequirement};
pub use save::{SAVE_GAME_VERSION, SaveGame, SaveLoadError};
pub use state::{LevelState, RunState};
pub use vitals::{ApothecaryVitals, DamageOutcome};
