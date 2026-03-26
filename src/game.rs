use crate::constants::{GRID_H, GRID_W, HOURS_PER_DAY};
use crate::model::{
    Asset, AssetKind, BridgeProfile, CityScore, Coverage, Incident, IncidentKind, LogicProfile,
    Tile, TileKind, Tool, ZoneKind,
};
use bevy::prelude::{IVec2, Resource};
use rand::{thread_rng, Rng};

#[derive(Clone, Debug, Resource)]
pub struct GameState {
    pub(crate) grid: Vec<Tile>,
    pub(crate) assets: Vec<Asset>,
    powered_tiles: Vec<bool>,
    watered_tiles: Vec<bool>,
    pub(crate) selected_tool: Tool,
    pub(crate) bridge_profile: BridgeProfile,
    pub(crate) logic_profile: LogicProfile,
    pub(crate) budget: i32,
    pub(crate) ops_budget: i32,
    pub(crate) selected_tile: Option<IVec2>,
    pub(crate) log: Vec<String>,
    pub(crate) score: CityScore,
    pub(crate) coverage: Coverage,
    pub(crate) incidents: Vec<Incident>,
    pub(crate) river_x: i32,
    pub(crate) research_points: i32,
    pub(crate) advanced_unlocked: bool,
    pub(crate) elapsed_hours: u32,
    visual_revision: u64,
}

impl GameState {
    pub fn new() -> Self {
        let mut state = Self {
            grid: blank_grid(),
            assets: Vec::new(),
            powered_tiles: blank_service_grid(),
            watered_tiles: blank_service_grid(),
            selected_tool: Tool::Inspect,
            bridge_profile: BridgeProfile::Standard,
            logic_profile: LogicProfile::Manual,
            budget: 760,
            ops_budget: 160,
            selected_tile: None,
            log: Vec::new(),
            score: CityScore::baseline(),
            coverage: Coverage::empty(),
            incidents: Vec::new(),
            river_x: 0,
            research_points: 0,
            advanced_unlocked: false,
            elapsed_hours: 0,
            visual_revision: 0,
        };
        state.regenerate();
        state
    }

    pub fn regenerate(&mut self) {
        self.grid = blank_grid();
        self.assets.clear();
        self.log.clear();
        self.incidents.clear();
        self.powered_tiles.fill(false);
        self.watered_tiles.fill(false);
        self.selected_tile = None;
        self.bridge_profile = BridgeProfile::Standard;
        self.logic_profile = LogicProfile::Manual;
        self.budget = 760;
        self.ops_budget = 160;
        self.research_points = 0;
        self.advanced_unlocked = false;
        self.elapsed_hours = 0;
        self.score = CityScore::baseline();
        self.coverage = Coverage::empty();

        let mut rng = thread_rng();
        let band_center = rng.gen_range(8..(GRID_W - 8));
        self.river_x = band_center;

        for y in 0..GRID_H {
            for x in 0..GRID_W {
                let idx = self.idx(x, y);
                let band_shift = if y % 4 == 0 { rng.gen_range(-1..=1) } else { 0 };
                let river_here = x == band_center + band_shift;
                self.grid[idx].kind = if river_here {
                    TileKind::River
                } else if y == GRID_H / 2 || y == 4 || y == GRID_H - 5 || x == 4 || x == GRID_W - 5
                {
                    TileKind::Road
                } else {
                    let roll = rng.gen_range(0..100);
                    match roll {
                        0..=38 => TileKind::Building,
                        39..=56 => TileKind::Industrial,
                        57..=64 => TileKind::Park,
                        65..=69 => TileKind::Utility,
                        _ => TileKind::Empty,
                    }
                };
            }
        }

        let cc_idx = self.idx(2, 2);
        self.grid[cc_idx].kind = TileKind::ControlCenter;

        for y in [4, GRID_H / 2, GRID_H - 5] {
            for x in 0..GRID_W {
                let idx = self.idx(x, y);
                if self.grid[idx].kind != TileKind::River {
                    self.grid[idx].kind = TileKind::Road;
                }
            }
        }

        self.push_log(
            "Legacy city generated. Utilities are inconsistent, OT is sloppy, and optimism is irrational."
                .to_string(),
        );
        self.push_log(
            "Phase 2 objective: raise smartness to 70+ while keeping engineering, OT security, and trust above 45."
                .to_string(),
        );
        self.recompute_coverage();
        self.bump_visual_revision();
    }

    fn idx(&self, x: i32, y: i32) -> usize {
        (y * GRID_W + x) as usize
    }

    fn idxv(&self, p: IVec2) -> usize {
        self.idx(p.x, p.y)
    }

    fn in_bounds(&self, p: IVec2) -> bool {
        p.x >= 0 && p.y >= 0 && p.x < GRID_W && p.y < GRID_H
    }

    pub(crate) fn tile(&self, p: IVec2) -> &Tile {
        &self.grid[self.idxv(p)]
    }

    fn tile_mut(&mut self, p: IVec2) -> &mut Tile {
        let idx = self.idxv(p);
        &mut self.grid[idx]
    }

    fn cost_for(&self, tool: Tool) -> i32 {
        match tool {
            Tool::Inspect => 0,
            Tool::Bridge => match self.bridge_profile {
                BridgeProfile::Standard => 120,
                BridgeProfile::Heavy => 170,
                BridgeProfile::Cheap => 80,
            },
            Tool::Sensor => 24,
            Tool::Plc => 70,
            Tool::Gateway => 48,
            Tool::Substation => 110,
            Tool::PumpStation => 95,
        }
    }

    pub fn handle_click(&mut self, p: IVec2) {
        if !self.in_bounds(p) {
            return;
        }
        self.selected_tile = Some(p);

        if self.selected_tool == Tool::Inspect {
            return;
        }

        let cost = self.cost_for(self.selected_tool);
        if self.budget < cost {
            self.push_log(format!("Budget denied. Need {cost}, have {}.", self.budget));
            self.score.citizen_trust -= 1;
            return;
        }
        if self.tile(p).asset.is_some() {
            self.push_log("Tile already occupied by an asset.".to_string());
            return;
        }

        let (allowed, compliant, note, zone) = self.validate_placement(self.selected_tool, p);
        if !allowed {
            self.push_log(format!("Placement rejected: {note}"));
            self.score.engineering -= 1;
            self.clamp_scores();
            return;
        }

        let kind = self
            .selected_tool
            .asset_kind()
            .expect("selected tool should map to an asset kind");
        let asset_index = self.assets.len();
        let bridge_profile = (kind == AssetKind::Bridge).then_some(self.bridge_profile);
        let logic_profile = (kind == AssetKind::Plc).then_some(self.logic_profile);

        self.assets.push(Asset {
            kind,
            pos: p,
            compliant,
            note: note.clone(),
            bridge_profile,
            logic_profile,
            zone,
        });
        self.tile_mut(p).asset = Some(asset_index);
        self.budget -= cost;

        match kind {
            AssetKind::Bridge => {
                self.score.smartness += 5;
                self.score.engineering += if compliant { 5 } else { -14 };
                self.score.citizen_trust += if compliant { 4 } else { -18 };
            }
            AssetKind::Sensor => {
                self.score.smartness += 4;
                self.score.ot_security += if self.nearby_asset_kind(p, AssetKind::Gateway, 4) {
                    1
                } else {
                    -1
                };
                self.research_points += 1;
            }
            AssetKind::Plc => {
                self.score.smartness += 6;
                self.score.ot_security += if compliant { 5 } else { -9 };
                self.ops_budget -= 4;
                self.research_points += 1;
            }
            AssetKind::Gateway => {
                self.score.ot_security += 8;
                self.score.smartness += 2;
                self.ops_budget -= 1;
            }
            AssetKind::Substation => {
                self.score.engineering += 4;
                self.score.smartness += 3;
                self.ops_budget -= 2;
                self.research_points += 1;
            }
            AssetKind::PumpStation => {
                self.score.engineering += if compliant { 4 } else { -8 };
                self.score.smartness += 3;
                self.ops_budget -= 2;
                self.research_points += 1;
            }
        }

        self.clamp_scores();
        self.recompute_coverage();
        self.bump_visual_revision();
        self.push_log(format!("Built {:?} at ({}, {}). {}", kind, p.x, p.y, note));
    }

    fn validate_placement(&self, tool: Tool, p: IVec2) -> (bool, bool, String, ZoneKind) {
        let tile_kind = self.tile(p).kind;
        match tool {
            Tool::Bridge => {
                if tile_kind != TileKind::River {
                    return (
                        false,
                        false,
                        "Bridge must be placed on a river tile.".to_string(),
                        ZoneKind::Municipal,
                    );
                }
                let left = p + IVec2::new(-1, 0);
                let right = p + IVec2::new(1, 0);
                if !(self.in_bounds(left) && self.in_bounds(right)) {
                    return (
                        false,
                        false,
                        "Bridge alignment invalid at map edge.".to_string(),
                        ZoneKind::Municipal,
                    );
                }
                let left_ok = self.tile(left).kind == TileKind::Road;
                let right_ok = self.tile(right).kind == TileKind::Road;
                if !(left_ok && right_ok) {
                    return (
                        false,
                        false,
                        "Bridge requires road approaches on both banks.".to_string(),
                        ZoneKind::Municipal,
                    );
                }

                let nearby_industry = self.count_nearby_kind(p, TileKind::Industrial, 3);
                let compliant = match self.bridge_profile {
                    BridgeProfile::Standard => true,
                    BridgeProfile::Heavy => nearby_industry > 0,
                    BridgeProfile::Cheap => nearby_industry == 0,
                };
                let note = match self.bridge_profile {
                    BridgeProfile::Standard => {
                        "Standard bridge approved. Boring, serviceable, and therefore respectable."
                            .to_string()
                    }
                    BridgeProfile::Heavy => {
                        if compliant {
                            "Heavy bridge approved. High load margin, high cost, lower embarrassment risk."
                                .to_string()
                        } else {
                            "Heavy bridge is overkill here. Expensive, maintenance-heavy, and politically annoying."
                                .to_string()
                        }
                    }
                    BridgeProfile::Cheap => {
                        if compliant {
                            "Cheap bridge approved for low-demand crossing. Pray traffic growth stays polite."
                                .to_string()
                        } else {
                            "Cheap bridge undersized for expected demand. Future headlines look grim."
                                .to_string()
                        }
                    }
                };
                (true, compliant, note, ZoneKind::Municipal)
            }
            Tool::Sensor => {
                if tile_kind != TileKind::Building
                    && tile_kind != TileKind::Industrial
                    && tile_kind != TileKind::Utility
                {
                    return (
                        false,
                        false,
                        "Sensor nodes belong on buildings, industry, or utility sites.".to_string(),
                        ZoneKind::Municipal,
                    );
                }
                let gateway = self.nearby_asset_kind(p, AssetKind::Gateway, 4);
                let note = if gateway {
                    "Telemetry node deployed with plausible network path.".to_string()
                } else {
                    "Telemetry node deployed, but upstream segmentation is weak. Naturally."
                        .to_string()
                };
                (true, true, note, ZoneKind::Municipal)
            }
            Tool::Plc => {
                if tile_kind != TileKind::Industrial && tile_kind != TileKind::Utility {
                    return (
                        false,
                        false,
                        "PLC cabinets can only be deployed at industrial or utility sites."
                            .to_string(),
                        ZoneKind::OT,
                    );
                }
                let has_gateway = self.nearby_asset_kind(p, AssetKind::Gateway, 4);
                let has_sensor = self.nearby_asset_kind(p, AssetKind::Sensor, 2);
                let compliant = match self.logic_profile {
                    LogicProfile::Manual => has_gateway,
                    LogicProfile::Timed => has_gateway && has_sensor,
                    LogicProfile::Responsive => has_gateway && has_sensor && self.advanced_unlocked,
                };
                let note = match self.logic_profile {
                    LogicProfile::Manual => {
                        if compliant {
                            "Manual PLC mode deployed behind a gateway. Limited, but unlikely to invent chaos on its own."
                                .to_string()
                        } else {
                            "Manual PLC lacks segmented access. Someone will eventually regret that."
                                .to_string()
                        }
                    }
                    LogicProfile::Timed => {
                        if compliant {
                            "Timed PLC deployed with telemetry and segmentation. A decent compromise between control and hubris."
                                .to_string()
                        } else {
                            "Timed PLC needs both telemetry and segmentation or it becomes a scheduled mistake."
                                .to_string()
                        }
                    }
                    LogicProfile::Responsive => {
                        if compliant {
                            "Responsive PLC deployed. Better process behavior, but only because advanced controls were actually unlocked."
                                .to_string()
                        } else if !self.advanced_unlocked {
                            "Responsive PLC requires advanced controls research.".to_string()
                        } else {
                            "Responsive PLC missing telemetry or segmentation. Smart failure is still failure."
                                .to_string()
                        }
                    }
                };
                (true, compliant, note, ZoneKind::OT)
            }
            Tool::Gateway => {
                if tile_kind != TileKind::Road && tile_kind != TileKind::ControlCenter {
                    return (
                        false,
                        false,
                        "Gateway should be placed on road-accessible or control-center tiles."
                            .to_string(),
                        ZoneKind::Corporate,
                    );
                }
                let compliant = self.count_nearby_kind(p, TileKind::Industrial, 5) > 0
                    || self.count_nearby_kind(p, TileKind::Utility, 5) > 0
                    || self.count_nearby_kind(p, TileKind::ControlCenter, 5) > 0;
                let note = if compliant {
                    "Gateway installed. Segmentation boundary improved.".to_string()
                } else {
                    "Gateway installed, but coverage is strategically mediocre.".to_string()
                };
                (true, compliant, note, ZoneKind::Corporate)
            }
            Tool::Substation => {
                if tile_kind != TileKind::Road && tile_kind != TileKind::Utility {
                    return (
                        false,
                        false,
                        "Substation needs utility or road-adjacent serviceable ground.".to_string(),
                        ZoneKind::Municipal,
                    );
                }
                let near_demand = self.count_nearby_kind(p, TileKind::Building, 4)
                    + self.count_nearby_kind(p, TileKind::Industrial, 4);
                let compliant = near_demand >= 3;
                let note = if compliant {
                    "Substation installed. Local feeder coverage improves.".to_string()
                } else {
                    "Substation installed in a low-demand area. Excellent way to waste capital."
                        .to_string()
                };
                (true, compliant, note, ZoneKind::Municipal)
            }
            Tool::PumpStation => {
                let near_river = self.count_nearby_kind(p, TileKind::River, 1) > 0;
                if tile_kind != TileKind::Road && tile_kind != TileKind::Utility && !near_river {
                    return (
                        false,
                        false,
                        "Pump station needs utility ground, road access, or river adjacency."
                            .to_string(),
                        ZoneKind::OT,
                    );
                }
                let has_plc = self.nearby_asset_kind(p, AssetKind::Plc, 2);
                let compliant = near_river || has_plc;
                let note = if compliant {
                    "Pump station installed with viable hydraulic context.".to_string()
                } else {
                    "Pump station lacks river proximity or nearby control integration. Water, apparently, dislikes fantasy."
                        .to_string()
                };
                (true, compliant, note, ZoneKind::OT)
            }
            Tool::Inspect => (false, false, String::new(), ZoneKind::Municipal),
        }
    }

    fn count_nearby_kind(&self, center: IVec2, kind: TileKind, radius: i32) -> i32 {
        let mut count = 0;
        for y in center.y - radius..=center.y + radius {
            for x in center.x - radius..=center.x + radius {
                let p = IVec2::new(x, y);
                if self.in_bounds(p) && self.tile(p).kind == kind {
                    count += 1;
                }
            }
        }
        count
    }

    fn nearby_asset_kind(&self, center: IVec2, kind: AssetKind, radius: i32) -> bool {
        self.assets.iter().any(|asset| {
            asset.kind == kind
                && (asset.pos.x - center.x).abs() <= radius
                && (asset.pos.y - center.y).abs() <= radius
        })
    }

    pub(crate) fn powered_at(&self, p: IVec2) -> bool {
        self.powered_tiles[self.idxv(p)]
    }

    pub(crate) fn watered_at(&self, p: IVec2) -> bool {
        self.watered_tiles[self.idxv(p)]
    }

    fn recompute_coverage(&mut self) {
        self.powered_tiles.fill(false);
        self.watered_tiles.fill(false);

        for asset in &self.assets {
            match asset.kind {
                AssetKind::Substation => mark_service_area(&mut self.powered_tiles, asset.pos, 5),
                AssetKind::PumpStation => mark_service_area(&mut self.watered_tiles, asset.pos, 5),
                _ => {}
            }
        }

        let mut powered = 0;
        let mut watered = 0;
        let mut total = 0;
        for y in 0..GRID_H {
            for x in 0..GRID_W {
                let p = IVec2::new(x, y);
                let kind = self.tile(p).kind;
                if kind == TileKind::Building
                    || kind == TileKind::Industrial
                    || kind == TileKind::ControlCenter
                {
                    total += 1;
                    if self.powered_at(p) {
                        powered += 1;
                    }
                    if self.watered_at(p) || kind == TileKind::ControlCenter {
                        watered += 1;
                    }
                }
            }
        }
        self.coverage.powered = powered;
        self.coverage.watered = watered;
        self.coverage.total_demand = total;
    }

    pub(crate) fn push_log(&mut self, msg: String) {
        self.log.push(msg);
        if self.log.len() > 12 {
            self.log.remove(0);
        }
    }

    fn clamp_scores(&mut self) {
        self.score.smartness = self.score.smartness.clamp(0, 100);
        self.score.engineering = self.score.engineering.clamp(0, 100);
        self.score.ot_security = self.score.ot_security.clamp(0, 100);
        self.score.citizen_trust = self.score.citizen_trust.clamp(0, 100);
    }

    pub fn spend_research(&mut self) {
        if self.advanced_unlocked {
            self.push_log(
                "Advanced controls already unlocked. Humans do enjoy buying the same thing twice."
                    .to_string(),
            );
            return;
        }
        if self.research_points < 3 {
            self.push_log(format!(
                "Need 3 research points to unlock advanced controls. Have {}.",
                self.research_points
            ));
            return;
        }
        self.research_points -= 3;
        self.advanced_unlocked = true;
        self.score.smartness += 4;
        self.push_log(
            "Advanced controls unlocked. Responsive PLC mode is now permitted to cause sophisticated trouble."
                .to_string(),
        );
        self.clamp_scores();
    }

    pub fn cycle_bridge_profile(&mut self) {
        self.bridge_profile = self.bridge_profile.next();
        self.push_log(format!(
            "Bridge profile set to {}.",
            self.bridge_profile.label()
        ));
    }

    pub fn cycle_logic_profile(&mut self) {
        self.logic_profile = self.logic_profile.next();
        self.push_log(format!(
            "PLC logic profile set to {}.",
            self.logic_profile.label()
        ));
    }

    fn bridge_count(&self) -> i32 {
        self.assets
            .iter()
            .filter(|asset| asset.kind == AssetKind::Bridge)
            .count() as i32
    }

    fn gateway_count(&self) -> i32 {
        self.assets
            .iter()
            .filter(|asset| asset.kind == AssetKind::Gateway)
            .count() as i32
    }

    fn plc_count(&self) -> i32 {
        self.assets
            .iter()
            .filter(|asset| asset.kind == AssetKind::Plc)
            .count() as i32
    }

    fn substation_count(&self) -> i32 {
        self.assets
            .iter()
            .filter(|asset| asset.kind == AssetKind::Substation)
            .count() as i32
    }

    fn pump_count(&self) -> i32 {
        self.assets
            .iter()
            .filter(|asset| asset.kind == AssetKind::PumpStation)
            .count() as i32
    }

    fn count_noncompliant(&self, kind: AssetKind) -> i32 {
        self.assets
            .iter()
            .filter(|asset| asset.kind == kind && !asset.compliant)
            .count() as i32
    }

    fn bridge_connectivity_bonus(&self) -> i32 {
        self.assets
            .iter()
            .filter(|asset| asset.kind == AssetKind::Bridge)
            .map(|bridge| {
                self.count_nearby_kind(bridge.pos, TileKind::Industrial, 3)
                    + self.count_nearby_kind(bridge.pos, TileKind::Building, 2)
            })
            .sum()
    }

    pub fn advance_hour(&mut self) {
        self.elapsed_hours += 1;
        self.recompute_coverage();
        self.incidents.retain_mut(|incident| {
            incident.ttl -= 1.0;
            incident.ttl > 0.0
        });

        if self.elapsed_hours % HOURS_PER_DAY == 0 {
            self.run_daily_review();
        }
    }

    fn run_daily_review(&mut self) {
        let total = self.coverage.total_demand.max(1);
        let power_ratio = self.coverage.powered as f32 / total as f32;
        let water_ratio = self.coverage.watered as f32 / total as f32;
        let bridge_count = self.bridge_count();
        let connectivity_bonus = self.bridge_connectivity_bonus();
        let noncompliant_bridges = self.count_noncompliant(AssetKind::Bridge);
        let plc_count = self.plc_count();
        let gateway_count = self.gateway_count();
        let noncompliant_plcs = self.count_noncompliant(AssetKind::Plc);
        let pumps = self.pump_count();
        let substations = self.substation_count();

        self.ops_budget -= plc_count + pumps + substations;

        if power_ratio < 0.65 {
            self.trigger_incident(
                IncidentKind::Brownout,
                "Brownout risk rising. Too much demand, not enough feeder coverage.",
            );
            self.score.citizen_trust -= 2;
            self.score.engineering -= 1;
        } else {
            self.score.smartness += 1;
        }

        if water_ratio < 0.60 {
            self.trigger_incident(
                IncidentKind::LowPressure,
                "Water pressure complaints increasing. The city remains unconvinced by dry taps.",
            );
            self.score.citizen_trust -= 2;
            self.score.engineering -= 1;
        }

        if bridge_count == 0 {
            self.score.citizen_trust -= 1;
        } else {
            self.score.smartness += (connectivity_bonus / 8).clamp(0, 2);
        }

        if noncompliant_bridges > 0 {
            self.trigger_incident(
                IncidentKind::BridgeStress,
                "Bridge stress alert. Someone saved money in a way gravity intends to discuss.",
            );
            self.score.engineering -= noncompliant_bridges * 2;
            self.score.citizen_trust -= noncompliant_bridges;
        }

        if plc_count > 0 && gateway_count == 0 {
            self.trigger_incident(
                IncidentKind::OtIntrusion,
                "OT intrusion pathway detected. Flat trust model, shocking outcome.",
            );
            self.score.ot_security -= 3;
        } else if noncompliant_plcs > 0 {
            self.trigger_incident(
                IncidentKind::OtIntrusion,
                "Poorly integrated PLC logic increases process and cyber risk.",
            );
            self.score.ot_security -= 2;
        } else if plc_count > 0 && gateway_count > 0 {
            self.score.ot_security += 1;
        }

        if self.advanced_unlocked && plc_count > 0 {
            self.score.smartness += 1;
        }

        if self.ops_budget < 0 {
            self.score.engineering -= 1;
            self.score.citizen_trust -= 1;
        }

        if self.coverage.powered + self.coverage.watered > total {
            self.research_points += 1;
        }

        self.score.smartness += ((power_ratio + water_ratio) * 2.0) as i32 - 1;

        self.clamp_scores();
    }

    fn trigger_incident(&mut self, kind: IncidentKind, note: &str) {
        let already_active = self.incidents.iter().any(|incident| incident.kind == kind);
        if !already_active {
            self.incidents.push(Incident {
                kind,
                ttl: HOURS_PER_DAY as f32,
                note: note.to_string(),
            });
            self.push_log(format!("Incident: {note}"));
        }
    }

    fn bump_visual_revision(&mut self) {
        self.visual_revision = self.visual_revision.wrapping_add(1);
    }

    pub fn visual_revision(&self) -> u64 {
        self.visual_revision
    }

    pub fn current_day(&self) -> u32 {
        self.elapsed_hours / HOURS_PER_DAY + 1
    }

    pub fn hour_of_day(&self) -> u32 {
        self.elapsed_hours % HOURS_PER_DAY
    }

    pub fn clock_label(&self) -> String {
        format!("Day {:02} {:02}:00", self.current_day(), self.hour_of_day())
    }
}

fn blank_grid() -> Vec<Tile> {
    vec![Tile::empty(); (GRID_W * GRID_H) as usize]
}

fn blank_service_grid() -> Vec<bool> {
    vec![false; (GRID_W * GRID_H) as usize]
}

fn mark_service_area(service_map: &mut [bool], center: IVec2, radius: i32) {
    let min_y = (center.y - radius).max(0);
    let max_y = (center.y + radius).min(GRID_H - 1);
    let min_x = (center.x - radius).max(0);
    let max_x = (center.x + radius).min(GRID_W - 1);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let pos = IVec2::new(x, y);
            if manhattan(center, pos) <= radius {
                let idx = (y * GRID_W + x) as usize;
                service_map[idx] = true;
            }
        }
    }
}

fn manhattan(a: IVec2, b: IVec2) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}
