//! Geometry-level progression checks: actual wall cuts, player clearance and staged locks.
use super::*;
use game_core::{LevelEvent, LevelState, RuleContext, RuleGate};
use std::collections::{HashSet, VecDeque};

struct Route {
    level: LevelDefinition,
    state: LevelState,
    vitals: game_core::ApothecaryVitals,
    position: Vec2,
}
impl Route {
    fn new(name: &str) -> Self {
        let level =
            LevelDefinition::from_ron_file(format!("../../assets/levels/{name}.ron")).unwrap();
        level.validate().unwrap();
        Self {
            position: level.apothecary_start,
            level,
            state: LevelState::default(),
            vitals: game_core::ApothecaryVitals::new(100, 48, 0),
        }
    }
    fn reachable(&self) -> Vec<Vec2> {
        let mut blockers = split_walls_around_doors(&self.level.walls, &self.level.doors);
        for door in &self.level.doors {
            if !self.state.has_unlocked_door(&door.id)
                && !RuleGate::for_door(&door.clearance_id, &door.required_objectives).is_open(
                    RuleContext {
                        level_state: &self.state,
                        vitals: &self.vitals,
                    },
                )
            {
                blockers.push(AxisAlignedBox::new(door.position, door.half_extents));
            }
        }
        for decor in self.level.decor.iter().filter(|d| d.blocking) {
            blockers.push(AxisAlignedBox::new(
                decor.position,
                decor_half_extents(decor),
            ));
        }
        let valid = |p: Vec2| {
            let e = (p - self.level.bounds.center).abs() + Vec2::splat(22.0);
            e.x <= self.level.bounds.half_extents.x
                && e.y <= self.level.bounds.half_extents.y
                && !blockers
                    .iter()
                    .any(|b| game_core::circle_intersects_aabb(p, 22.0, *b))
        };
        let start = (
            (self.position.x / 8.0).round() as i32,
            (self.position.y / 8.0).round() as i32,
        );
        assert!(
            valid(Vec2::new(start.0 as f32 * 8.0, start.1 as f32 * 8.0)),
            "invalid player start in {}",
            self.level.name
        );
        let mut seen = HashSet::from([start]);
        let mut queue = VecDeque::from([start]);
        while let Some((x, y)) = queue.pop_front() {
            for p in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                if !seen.contains(&p) && valid(Vec2::new(p.0 as f32 * 8.0, p.1 as f32 * 8.0)) {
                    seen.insert(p);
                    queue.push_back(p);
                }
            }
        }
        seen.into_iter()
            .map(|(x, y)| Vec2::new(x as f32 * 8.0, y as f32 * 8.0))
            .collect()
    }
    fn can_reach(&self, position: Vec2) -> bool {
        self.reachable().iter().any(|p| p.distance(position) < 20.0)
    }
    fn walk(&mut self, position: Vec2) {
        assert!(
            self.can_reach(position),
            "{}: unreachable {position:?}",
            self.level.name
        );
        self.position = position;
    }
    fn pickup(&mut self, id: &str) {
        let p = self
            .level
            .pickups
            .iter()
            .find(|p| p.id == id)
            .unwrap()
            .clone();
        self.walk(p.position);
        match p.kind {
            PickupKind::SecurityKeycard(c) => {
                self.state.grant_clearance(c);
            }
            PickupKind::BioSample => self.vitals.bio_samples += 1,
            _ => (),
        }
    }
    fn activate(&mut self, id: &str) {
        let t = self
            .level
            .terminals
            .iter()
            .find(|t| t.id == id)
            .unwrap()
            .clone();
        self.walk(t.position);
        assert!(self.vitals.bio_samples >= t.required_bio_samples);
        assert!(self.state.activate_terminal(id));
        for action in t.actions {
            match action {
                LevelEvent::CompleteObjective(o) => {
                    self.state.complete_objective(o);
                }
                LevelEvent::GrantClearance(c) => {
                    self.state.grant_clearance(c);
                }
                LevelEvent::UnlockDoor(d) => {
                    self.state.unlock_door(d);
                }
                _ => (),
            }
        }
    }
    fn terminal_pos(&self, id: &str) -> Vec2 {
        self.level
            .terminals
            .iter()
            .find(|t| t.id == id)
            .unwrap()
            .position
    }
    fn all_required_done(&self) {
        assert!(
            self.level
                .objectives
                .iter()
                .filter(|o| o.required)
                .all(|o| self.state.objectives.is_complete(&o.id))
        );
    }
}

#[test]
fn coolant_key_pump_sample_core_and_return_route() {
    let mut r = Route::new("coolant_cathedral");
    assert!(!r.can_reach(r.terminal_pos("pump_control")));
    assert!(!r.can_reach(r.terminal_pos("coolant_analyzer")));
    r.pickup("service_card");
    r.pickup("coolant_sample");
    r.activate("pump_control");
    r.activate("coolant_analyzer");
    r.all_required_done();
    for position in r
        .level
        .exits
        .iter()
        .map(|e| e.position)
        .chain(r.level.transitions.iter().map(|t| t.position))
        .collect::<Vec<_>>()
    {
        r.walk(position);
    }
}

#[test]
fn archive_detour_returns_freely_and_reserve_needs_its_own_card() {
    let mut r = Route::new("specimen_archive");
    r.walk(r.level.exits[0].position);
    assert!(!r.can_reach(r.terminal_pos("catalog_console")));
    assert!(!r.can_reach(r.terminal_pos("reserve_console")));
    r.pickup("index_card");
    r.activate("catalog_console");
    r.pickup("archive_sample");
    r.activate("specimen_analyzer");
    r.all_required_done();
    assert!(!r.can_reach(r.terminal_pos("reserve_console")));
    r.pickup("reserve_card");
    r.activate("reserve_console");
    r.walk(r.level.exits[0].position);
}

#[test]
fn relay_wings_work_in_either_order_and_both_are_needed() {
    for reverse in [false, true] {
        let mut r = Route::new("choir_relay");
        let mut wings = [
            ("west_sample", "ground_console"),
            ("east_sample", "phase_console"),
        ];
        if reverse {
            wings.reverse();
        }
        for (sample, terminal) in wings {
            assert!(!r.can_reach(r.terminal_pos("signal_analyzer")));
            r.pickup(sample);
            r.activate(terminal);
        }
        r.activate("signal_analyzer");
        r.all_required_done();
        r.walk(r.level.transitions[0].position);
        r.walk(r.level.exits[0].position);
    }
}

#[test]
fn revisiting_and_loading_reconstruct_pressure_without_replaying_rewards() {
    let mut route = Route::new("choir_relay");
    assert_eq!(restored_spawn_interval(&route.level, &route.state), 8.0);
    route.state.activate_terminal("ground_console");
    assert_eq!(restored_spawn_interval(&route.level, &route.state), 5.0);
    route.state.activate_terminal("signal_analyzer");
    let saved = ron::to_string(&route.state).unwrap();
    let loaded = ron::from_str(&saved).unwrap();
    let mut runtime = crate::initial_level_runtime();
    let mut map = CurrentLevelMap::default();
    let mut timer = ContaminantSpawnTimer(Timer::from_seconds(8.0, TimerMode::Repeating));
    update_level_runtime(&mut runtime, &mut map, &route.level, &loaded, &mut timer);
    assert_eq!(runtime.dynamic_spawn_interval_seconds, 2.8);
    assert!((timer.0.duration().as_secs_f32() - 2.8).abs() < 0.001);
    assert_eq!(
        restored_spawn_interval(&route.level, &game_core::LevelState::default()),
        8.0
    );
}

#[test]
fn research_has_a_reachable_gated_connection_to_the_new_sectors() {
    let mut route = Route::new("research_spine");
    let lift = route
        .level
        .transitions
        .iter()
        .find(|t| t.id == "spine_coolant_lift")
        .unwrap()
        .clone();
    assert!(
        !RuleGate::for_objectives(&lift.required_objectives).is_open(RuleContext {
            level_state: &route.state,
            vitals: &route.vitals
        })
    );
    for o in &route.level.objectives {
        route.state.complete_objective(&o.id);
    }
    for p in &route.level.pickups {
        if let PickupKind::SecurityKeycard(c) = &p.kind {
            route.state.grant_clearance(c);
        }
    }
    route.walk(lift.position);
    assert!(
        RuleGate::for_objectives(&lift.required_objectives).is_open(RuleContext {
            level_state: &route.state,
            vitals: &route.vitals
        })
    );
    for e in route
        .level
        .entry_points
        .clone()
        .iter()
        .filter(|e| e.id == "from_coolant_cathedral" || e.id == "from_choir_relay")
    {
        route.walk(e.position);
    }
}
