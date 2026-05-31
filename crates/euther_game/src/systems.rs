pub mod apothecary;
pub mod combat;
pub mod contamination;
pub mod pickups;
pub mod ui;

pub use apothecary::{aim_apothecary, move_apothecary, quit_on_escape};
pub use combat::{fire_syringe_round, move_projectiles, resolve_projectile_hits};
pub use contamination::{move_contaminants, resolve_contaminant_contact, spawn_contaminants};
pub use pickups::{collect_pickups, report_exit_overlap, unlock_doors};
pub use ui::update_status_text;
