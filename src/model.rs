use bevy::prelude::IVec2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TileKind {
    Empty,
    Road,
    Building,
    Industrial,
    ControlCenter,
    River,
    Park,
    Utility,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AssetKind {
    Bridge,
    Sensor,
    Plc,
    Gateway,
    Substation,
    PumpStation,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tool {
    Inspect,
    Bridge,
    Sensor,
    Plc,
    Gateway,
    Substation,
    PumpStation,
}

impl Tool {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Inspect => "Inspect",
            Self::Bridge => "Bridge",
            Self::Sensor => "Sensor",
            Self::Plc => "PLC",
            Self::Gateway => "Gateway",
            Self::Substation => "Substation",
            Self::PumpStation => "Pump",
        }
    }

    pub const fn asset_kind(self) -> Option<AssetKind> {
        match self {
            Self::Inspect => None,
            Self::Bridge => Some(AssetKind::Bridge),
            Self::Sensor => Some(AssetKind::Sensor),
            Self::Plc => Some(AssetKind::Plc),
            Self::Gateway => Some(AssetKind::Gateway),
            Self::Substation => Some(AssetKind::Substation),
            Self::PumpStation => Some(AssetKind::PumpStation),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BridgeProfile {
    Standard,
    Heavy,
    Cheap,
}

impl BridgeProfile {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::Heavy => "Heavy",
            Self::Cheap => "Cheap",
        }
    }

    pub const fn next(self) -> Self {
        match self {
            Self::Standard => Self::Heavy,
            Self::Heavy => Self::Cheap,
            Self::Cheap => Self::Standard,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogicProfile {
    Manual,
    Timed,
    Responsive,
}

impl LogicProfile {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Manual => "Manual",
            Self::Timed => "Timed",
            Self::Responsive => "Responsive",
        }
    }

    pub const fn next(self) -> Self {
        match self {
            Self::Manual => Self::Timed,
            Self::Timed => Self::Responsive,
            Self::Responsive => Self::Manual,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ZoneKind {
    Corporate,
    Municipal,
    OT,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IncidentKind {
    Brownout,
    LowPressure,
    BridgeStress,
    OtIntrusion,
}

#[derive(Clone, Debug)]
pub struct Asset {
    pub kind: AssetKind,
    pub pos: IVec2,
    pub compliant: bool,
    pub note: String,
    pub bridge_profile: Option<BridgeProfile>,
    pub logic_profile: Option<LogicProfile>,
    pub zone: ZoneKind,
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub kind: TileKind,
    pub asset: Option<usize>,
}

impl Tile {
    pub const fn empty() -> Self {
        Self {
            kind: TileKind::Empty,
            asset: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CityScore {
    pub smartness: i32,
    pub engineering: i32,
    pub ot_security: i32,
    pub citizen_trust: i32,
}

impl CityScore {
    pub const fn baseline() -> Self {
        Self {
            smartness: 18,
            engineering: 58,
            ot_security: 42,
            citizen_trust: 56,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Coverage {
    pub powered: i32,
    pub watered: i32,
    pub total_demand: i32,
}

impl Coverage {
    pub const fn empty() -> Self {
        Self {
            powered: 0,
            watered: 0,
            total_demand: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Incident {
    pub kind: IncidentKind,
    pub ttl: f32,
    pub note: String,
}
