pub mod campaign;
pub mod collision;
pub mod level;
pub mod progression;
pub mod vitals;

pub use campaign::{
    CampaignDefinition, CampaignLevel, CampaignProgress, CampaignTravelError,
    CampaignValidationError,
};
pub use collision::{AxisAlignedBox, circle_intersects_aabb};
pub use level::{
    DoorDefinition, LevelDefinition, LevelExit, ObjectiveDefinition, PickupKind, PrototypeEntity,
    TerminalDefinition, TerminalKind,
};
pub use progression::{ExitReadiness, ObjectiveProgress};
pub use vitals::{ApothecaryVitals, DamageOutcome};
