#![doc = include_str!("../README.md")]

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Write as FmtWrite;
use std::io::BufRead;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename = "Task")]
pub struct Task {
    #[serde(rename = "@type")]
    pub task_type: TaskType,

    #[serde(
        rename = "@aat_min_time",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub aat_min_time: Option<u32>,

    #[serde(
        rename = "@start_requires_arm",
        default,
        deserialize_with = "de_opt_bool",
        serialize_with = "ser_opt_bool",
        skip_serializing_if = "Option::is_none"
    )]
    pub start_requires_arm: Option<bool>,

    #[serde(
        rename = "@start_score_exit",
        default,
        deserialize_with = "de_opt_bool",
        serialize_with = "ser_opt_bool",
        skip_serializing_if = "Option::is_none"
    )]
    pub start_score_exit: Option<bool>,

    #[serde(
        rename = "@start_max_speed",
        default,
        deserialize_with = "de_opt_f32",
        serialize_with = "ser_opt_f32",
        skip_serializing_if = "Option::is_none"
    )]
    pub start_max_speed: Option<f32>,

    #[serde(
        rename = "@start_max_height",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub start_max_height: Option<u32>,

    #[serde(
        rename = "@start_max_height_ref",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub start_max_height_ref: Option<AltitudeReference>,

    #[serde(
        rename = "@start_open_time",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub start_open_time: Option<u32>,

    #[serde(
        rename = "@start_close_time",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub start_close_time: Option<u32>,

    #[serde(
        rename = "@finish_min_height",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub finish_min_height: Option<u32>,

    #[serde(
        rename = "@finish_min_height_ref",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub finish_min_height_ref: Option<AltitudeReference>,

    #[serde(
        rename = "@fai_finish",
        default,
        deserialize_with = "de_opt_bool",
        serialize_with = "ser_opt_bool",
        skip_serializing_if = "Option::is_none"
    )]
    pub fai_finish: Option<bool>,

    #[serde(
        rename = "@pev_start_wait_time",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub pev_start_wait_time: Option<u32>,

    #[serde(
        rename = "@pev_start_window",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub pev_start_window: Option<u32>,

    #[serde(rename = "Point", default)]
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum TaskType {
    AAT,
    RT,
    FAIGeneral,
    FAITriangle,
    FAIOR,
    FAIGoal,
    MAT,
    Mixed,
    Touring,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AltitudeReference {
    AGL,
    MSL,
}

impl<'de> Deserialize<'de> for AltitudeReference {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s: String = Deserialize::deserialize(deserializer)?;
        // XCSoar treats anything that's not "MSL" as AGL
        if s == "MSL" {
            Ok(AltitudeReference::MSL)
        } else {
            Ok(AltitudeReference::AGL)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Point {
    #[serde(rename = "@type")]
    pub point_type: PointType,

    #[serde(
        rename = "@score_exit",
        default,
        deserialize_with = "de_opt_bool",
        serialize_with = "ser_opt_bool",
        skip_serializing_if = "Option::is_none"
    )]
    pub score_exit: Option<bool>,

    #[serde(rename = "Waypoint")]
    pub waypoint: Waypoint,

    #[serde(rename = "ObservationZone")]
    pub observation_zone: ObservationZone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum PointType {
    Start,
    Turn,
    Area,
    Finish,
    OptionalStart,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Waypoint {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(
        rename = "@altitude",
        default,
        deserialize_with = "de_opt_f32",
        serialize_with = "ser_opt_f32",
        skip_serializing_if = "Option::is_none"
    )]
    pub altitude: Option<f32>,

    #[serde(rename = "@id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "@comment", default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,

    #[serde(rename = "Location")]
    pub location: Location,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct Location {
    #[serde(
        rename = "@longitude",
        deserialize_with = "de_f32",
        serialize_with = "ser_f32"
    )]
    pub longitude: f32,

    #[serde(
        rename = "@latitude",
        deserialize_with = "de_f32",
        serialize_with = "ser_f32"
    )]
    pub latitude: f32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "@type")]
pub enum ObservationZone {
    /// A cylinder with configurable radius. Scored from center.
    Cylinder {
        #[serde(
            rename = "@radius",
            deserialize_with = "de_f32",
            serialize_with = "ser_f32"
        )]
        radius: f32,
    },

    /// A straight line gate, typically used for start/finish.
    Line {
        #[serde(
            rename = "@length",
            deserialize_with = "de_f32",
            serialize_with = "ser_f32"
        )]
        length: f32,
    },

    /// DAeC keyhole: 500m cylinder or 10km 90° sector. Scored from center.
    Keyhole,

    /// FAI 90° sector with infinite length sides. Scored from corner.
    FAISector,

    /// A sector with configurable radius and radial angles.
    ///
    /// If `inner_radius` is set, creates an annular sector.
    Sector {
        #[serde(
            rename = "@radius",
            deserialize_with = "de_f32",
            serialize_with = "ser_f32"
        )]
        radius: f32,
        #[serde(
            rename = "@start_radial",
            deserialize_with = "de_f32",
            serialize_with = "ser_f32"
        )]
        start_radial: f32,
        #[serde(
            rename = "@end_radial",
            deserialize_with = "de_f32",
            serialize_with = "ser_f32"
        )]
        end_radial: f32,
        #[serde(
            rename = "@inner_radius",
            default,
            deserialize_with = "de_opt_f32",
            serialize_with = "ser_opt_f32",
            skip_serializing_if = "Option::is_none"
        )]
        inner_radius: Option<f32>,
    },

    /// A symmetric quadrant with configurable radius and angle.
    ///
    /// Defaults: `radius = 10000m`.
    SymmetricQuadrant {
        #[serde(
            rename = "@radius",
            default,
            deserialize_with = "de_opt_f32",
            serialize_with = "ser_opt_f32",
            skip_serializing_if = "Option::is_none"
        )]
        radius: Option<f32>,
        #[serde(
            rename = "@angle",
            default,
            deserialize_with = "de_opt_f32",
            serialize_with = "ser_opt_f32",
            skip_serializing_if = "Option::is_none"
        )]
        angle: Option<f32>,
    },

    /// A keyhole with configurable outer radius, inner radius, and sector angle.
    ///
    /// Defaults: `radius=10000m, inner_radius=500m, angle=90°`.
    CustomKeyhole {
        #[serde(
            rename = "@radius",
            default,
            deserialize_with = "de_opt_f32",
            serialize_with = "ser_opt_f32",
            skip_serializing_if = "Option::is_none"
        )]
        radius: Option<f32>,
        #[serde(
            rename = "@angle",
            default,
            deserialize_with = "de_opt_f32",
            serialize_with = "ser_opt_f32",
            skip_serializing_if = "Option::is_none"
        )]
        angle: Option<f32>,
        #[serde(
            rename = "@inner_radius",
            default,
            deserialize_with = "de_opt_f32",
            serialize_with = "ser_opt_f32",
            skip_serializing_if = "Option::is_none"
        )]
        inner_radius: Option<f32>,
    },

    /// Fixed 1-mile radius cylinder for Modified Area Tasks.
    MatCylinder,

    /// BGA start sector: 5km 180° sector.
    BGAStartSector,

    /// BGA fixed course: 500m cylinder or 20km 90° sector.
    BGAFixedCourse,

    /// BGA enhanced option: 500m cylinder or 10km 180° sector.
    BGAEnhancedOption,
}

fn de_f32<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

fn de_opt_f32<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<f32>, D::Error> {
    let s: Option<String> = Deserialize::deserialize(deserializer)?;
    match s {
        Some(s) => s.parse().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

fn de_opt_bool<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<bool>, D::Error> {
    let s: Option<String> = Deserialize::deserialize(deserializer)?;
    match s {
        Some(s) => match s.as_str() {
            "1" | "true" => Ok(Some(true)),
            "0" | "false" => Ok(Some(false)),
            _ => Err(serde::de::Error::custom(format!("invalid bool: {s}"))),
        },
        None => Ok(None),
    }
}

fn ser_f32<S: Serializer>(value: &f32, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&value.to_string())
}

fn ser_opt_f32<S: Serializer>(value: &Option<f32>, serializer: S) -> Result<S::Ok, S::Error> {
    match value {
        Some(v) => serializer.serialize_str(&v.to_string()),
        None => serializer.serialize_none(),
    }
}

fn ser_opt_bool<S: Serializer>(value: &Option<bool>, serializer: S) -> Result<S::Ok, S::Error> {
    match value {
        Some(true) => serializer.serialize_str("1"),
        Some(false) => serializer.serialize_str("0"),
        None => serializer.serialize_none(),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("XML parsing failed: {0}")]
    Xml(#[from] quick_xml::DeError),
}

#[derive(Debug, thiserror::Error)]
pub enum SerializeError {
    #[error("XML serialization failed: {0}")]
    Xml(#[from] quick_xml::SeError),
}

pub fn from_str(xml: &str) -> Result<Task, ParseError> {
    Ok(quick_xml::de::from_str(xml)?)
}

pub fn from_reader(reader: impl BufRead) -> Result<Task, ParseError> {
    Ok(quick_xml::de::from_reader(reader)?)
}

pub fn to_writer(mut writer: impl FmtWrite, task: &Task) -> Result<(), SerializeError> {
    let serializer = quick_xml::se::Serializer::new(&mut writer);
    task.serialize(serializer)?;
    Ok(())
}

pub fn to_writer_pretty(mut writer: impl FmtWrite, task: &Task) -> Result<(), SerializeError> {
    let mut serializer = quick_xml::se::Serializer::new(&mut writer);
    serializer.indent(' ', 4);
    task.serialize(serializer)?;
    Ok(())
}

pub fn to_string(task: &Task) -> Result<String, SerializeError> {
    let mut buffer = String::new();
    to_writer(&mut buffer, task)?;
    Ok(buffer)
}

pub fn to_string_pretty(task: &Task) -> Result<String, SerializeError> {
    let mut buffer = String::new();
    to_writer_pretty(&mut buffer, task)?;
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::{assert_debug_snapshot, assert_snapshot};

    #[test]
    fn parse_aat_task() {
        let xml = include_str!("../fixtures/aat-task.tsk");
        let task = from_str(xml).unwrap();
        assert_debug_snapshot!(task);
    }

    #[test]
    fn parse_racing_task() {
        let xml = include_str!("../fixtures/racing-task.tsk");
        let task = from_str(xml).unwrap();
        assert_debug_snapshot!(task);
    }

    #[test]
    fn parse_fai_task() {
        let xml = include_str!("../fixtures/fai-task.tsk");
        let task = from_str(xml).unwrap();
        assert_debug_snapshot!(task);
    }

    #[test]
    fn parse_all_oz_types() {
        let xml = include_str!("../fixtures/all-oz-types.tsk");
        let task = from_str(xml).unwrap();
        assert_debug_snapshot!(task);
    }

    #[test]
    fn roundtrip_aat_task() {
        let xml = include_str!("../fixtures/aat-task.tsk");
        let task = from_str(xml).unwrap();

        let serialized = to_string_pretty(&task).unwrap();
        assert_snapshot!(serialized);

        let roundtripped = from_str(&serialized).unwrap();
        assert_eq!(task, roundtripped);
    }

    #[test]
    fn roundtrip_racing_task() {
        let xml = include_str!("../fixtures/racing-task.tsk");
        let task = from_str(xml).unwrap();

        let serialized = to_string_pretty(&task).unwrap();
        assert_snapshot!(serialized);

        let roundtripped = from_str(&serialized).unwrap();
        assert_eq!(task, roundtripped);
    }

    #[test]
    fn roundtrip_fai_task() {
        let xml = include_str!("../fixtures/fai-task.tsk");
        let task = from_str(xml).unwrap();

        let serialized = to_string_pretty(&task).unwrap();
        assert_snapshot!(serialized);

        let roundtripped = from_str(&serialized).unwrap();
        assert_eq!(task, roundtripped);
    }

    #[test]
    fn roundtrip_all_oz_types() {
        let xml = include_str!("../fixtures/all-oz-types.tsk");
        let task = from_str(xml).unwrap();

        let serialized = to_string_pretty(&task).unwrap();
        assert_snapshot!(serialized);

        let roundtripped = from_str(&serialized).unwrap();
        assert_eq!(task, roundtripped);
    }

    #[test]
    fn serialize_ugly() {
        let xml = include_str!("../fixtures/aat-task.tsk");
        let task = from_str(xml).unwrap();

        let serialized = to_string(&task).unwrap();
        assert_snapshot!(serialized);
    }
}
