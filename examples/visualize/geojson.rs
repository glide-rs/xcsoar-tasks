use geo::{Bearing, Destination, Haversine, Point as GeoPoint};
use serde_json::{Value, json};
use std::f64::consts::PI;
use xcsoar_tasks::{Location, ObservationZone, Point, PointType, Task};

const CIRCLE_POINTS: usize = 64;
const FAI_SECTOR_RADIUS: f64 = 20000.0; // 20km for visualization (technically infinite)

pub fn task_to_geojson(task: &Task) -> Value {
    let mut features = Vec::new();

    // Generate course line
    if let Some(line) = generate_course_line(task) {
        features.push(line);
    }

    // Calculate bearings for sector orientation
    let bearings = calculate_leg_bearings(task);

    // Generate observation zones and waypoint markers
    let mut turnpoint_number = 0usize;
    for (i, point) in task.points.iter().enumerate() {
        let (bearing_in, bearing_out) = bearings[i];

        // Track turnpoint numbers (Turn and Area points only)
        let label_number = match point.point_type {
            PointType::Turn | PointType::Area => {
                turnpoint_number += 1;
                Some(turnpoint_number)
            }
            _ => None,
        };

        // Observation zone
        if let Some(zone) = generate_observation_zone_feature(point, bearing_in, bearing_out) {
            features.push(zone);
        }

        // Waypoint marker
        features.push(generate_waypoint_feature(point, label_number));
    }

    json!({
        "type": "FeatureCollection",
        "features": features
    })
}

fn generate_course_line(task: &Task) -> Option<Value> {
    if task.points.len() < 2 {
        return None;
    }

    let coordinates: Vec<[f64; 2]> = task
        .points
        .iter()
        .map(|p| [p.waypoint.location.longitude, p.waypoint.location.latitude])
        .collect();

    Some(json!({
        "type": "Feature",
        "properties": {
            "feature_type": "course_line"
        },
        "geometry": {
            "type": "LineString",
            "coordinates": coordinates
        }
    }))
}

fn generate_waypoint_feature(point: &Point, turnpoint_number: Option<usize>) -> Value {
    let loc = &point.waypoint.location;
    let label = match turnpoint_number {
        Some(n) => format!("{}. {}", n, point.waypoint.name),
        None => point.waypoint.name.clone(),
    };
    json!({
        "type": "Feature",
        "properties": {
            "feature_type": "waypoint",
            "name": label,
            "point_type": point_type_to_string(point.point_type)
        },
        "geometry": {
            "type": "Point",
            "coordinates": [loc.longitude, loc.latitude]
        }
    })
}

fn generate_observation_zone_feature(
    point: &Point,
    bearing_in: Option<f64>,
    bearing_out: Option<f64>,
) -> Option<Value> {
    let loc = point.waypoint.location;
    let bisector = calculate_bisector(bearing_in, bearing_out);

    let geometry = generate_zone_geometry(&point.observation_zone, loc, bisector)?;

    Some(json!({
        "type": "Feature",
        "properties": {
            "feature_type": "observation_zone",
            "name": point.waypoint.name,
            "point_type": point_type_to_string(point.point_type)
        },
        "geometry": geometry
    }))
}

fn generate_zone_geometry(
    zone: &ObservationZone,
    center: Location,
    bisector: f64,
) -> Option<Value> {
    match zone {
        ObservationZone::Cylinder { radius } => Some(generate_circle_geometry(center, *radius)),

        ObservationZone::MatCylinder => Some(generate_circle_geometry(center, 1609.34)), // 1 mile

        ObservationZone::Line { length } => Some(generate_line_geometry(center, *length, bisector)),

        ObservationZone::FAISector => Some(generate_sector_geometry(
            center,
            FAI_SECTOR_RADIUS,
            bisector,
            90.0,
            None,
        )),

        ObservationZone::Sector {
            radius,
            start_radial,
            end_radial,
            inner_radius,
        } => Some(generate_sector_from_radials(
            center,
            *radius,
            *start_radial,
            *end_radial,
            *inner_radius,
        )),

        ObservationZone::SymmetricQuadrant { radius, angle } => {
            let radius = radius.unwrap_or(10000.0);
            let angle = angle.unwrap_or(90.0);
            Some(generate_sector_geometry(
                center, radius, bisector, angle, None,
            ))
        }

        ObservationZone::Keyhole => Some(generate_keyhole_geometry(
            center, 10000.0, 500.0, 90.0, bisector,
        )),

        ObservationZone::CustomKeyhole {
            radius,
            angle,
            inner_radius,
        } => {
            let radius = radius.unwrap_or(10000.0);
            let angle = angle.unwrap_or(90.0);
            let inner_radius = inner_radius.unwrap_or(500.0);
            Some(generate_keyhole_geometry(
                center,
                radius,
                inner_radius,
                angle,
                bisector,
            ))
        }

        ObservationZone::BGAStartSector => Some(generate_sector_geometry(
            center, 5000.0, bisector, 180.0, None,
        )),

        ObservationZone::BGAFixedCourse => Some(generate_keyhole_geometry(
            center, 20000.0, 500.0, 90.0, bisector,
        )),

        ObservationZone::BGAEnhancedOption => Some(generate_keyhole_geometry(
            center, 10000.0, 500.0, 180.0, bisector,
        )),
    }
}

fn generate_circle_geometry(center: Location, radius: f64) -> Value {
    let coords = generate_circle_coords(center, radius);
    json!({
        "type": "Polygon",
        "coordinates": [coords]
    })
}

fn generate_circle_coords(center: Location, radius: f64) -> Vec<[f64; 2]> {
    let mut coords = Vec::with_capacity(CIRCLE_POINTS + 1);
    for i in 0..=CIRCLE_POINTS {
        let angle = (i as f64 / CIRCLE_POINTS as f64) * 2.0 * PI;
        let bearing = angle.to_degrees();
        let point = destination_point(center, bearing, radius);
        coords.push([point.longitude, point.latitude]);
    }
    coords
}

fn generate_line_geometry(center: Location, length: f64, bisector: f64) -> Value {
    let half_length = length / 2.0;
    let perpendicular_left = normalize_angle(bisector + 90.0);
    let perpendicular_right = normalize_angle(bisector - 90.0);

    let left = destination_point(center, perpendicular_left, half_length);
    let right = destination_point(center, perpendicular_right, half_length);

    json!({
        "type": "LineString",
        "coordinates": [
            [left.longitude, left.latitude],
            [right.longitude, right.latitude]
        ]
    })
}

fn generate_sector_geometry(
    center: Location,
    radius: f64,
    bisector: f64,
    angle: f64,
    inner_radius: Option<f64>,
) -> Value {
    let half_angle = angle / 2.0;
    let start_angle = normalize_angle(bisector - half_angle);
    let end_angle = normalize_angle(bisector + half_angle);

    let coords = generate_sector_coords(center, radius, start_angle, end_angle, inner_radius);

    json!({
        "type": "Polygon",
        "coordinates": [coords]
    })
}

fn generate_sector_from_radials(
    center: Location,
    radius: f64,
    start_radial: f64,
    end_radial: f64,
    inner_radius: Option<f64>,
) -> Value {
    let coords = generate_sector_coords(center, radius, start_radial, end_radial, inner_radius);

    json!({
        "type": "Polygon",
        "coordinates": [coords]
    })
}

fn generate_sector_coords(
    center: Location,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
    inner_radius: Option<f64>,
) -> Vec<[f64; 2]> {
    let arc_points = CIRCLE_POINTS / 2;
    let mut coords = Vec::new();

    // Normalize angles and calculate sweep
    let start = start_angle.to_radians();
    let end = end_angle.to_radians();
    let sweep = normalize_sweep(start, end);

    // Outer arc
    for i in 0..=arc_points {
        let t = i as f64 / arc_points as f64;
        let angle = start + t * sweep;
        let bearing = angle.to_degrees();
        let point = destination_point(center, bearing, radius);
        coords.push([point.longitude, point.latitude]);
    }

    if let Some(inner_r) = inner_radius {
        // Inner arc (reverse direction)
        for i in (0..=arc_points).rev() {
            let t = i as f64 / arc_points as f64;
            let angle = start + t * sweep;
            let bearing = angle.to_degrees();
            let point = destination_point(center, bearing, inner_r);
            coords.push([point.longitude, point.latitude]);
        }
    } else {
        // Close to center
        coords.push([center.longitude, center.latitude]);
    }

    // Close the polygon
    coords.push(coords[0]);
    coords
}

fn generate_keyhole_geometry(
    center: Location,
    outer_radius: f64,
    inner_radius: f64,
    angle: f64,
    bisector: f64,
) -> Value {
    let half_angle = angle / 2.0;
    let start_angle = normalize_angle(bisector - half_angle);
    let end_angle = normalize_angle(bisector + half_angle);

    let arc_points = CIRCLE_POINTS / 2;
    let mut coords = Vec::new();

    let start_rad = start_angle.to_radians();
    let end_rad = end_angle.to_radians();
    let sweep = normalize_sweep(start_rad, end_rad);

    // Outer arc of the sector
    for i in 0..=arc_points {
        let t = i as f64 / arc_points as f64;
        let angle = start_rad + t * sweep;
        let bearing = angle.to_degrees();
        let point = destination_point(center, bearing, outer_radius);
        coords.push([point.longitude, point.latitude]);
    }

    // Connect to inner circle at end_angle
    // Then trace the inner circle back around (the long way)
    let inner_sweep = 2.0 * PI - sweep;
    for i in 0..=CIRCLE_POINTS {
        let t = i as f64 / CIRCLE_POINTS as f64;
        let angle = end_rad + t * inner_sweep;
        let bearing = angle.to_degrees();
        let point = destination_point(center, bearing, inner_radius);
        coords.push([point.longitude, point.latitude]);
    }

    // Close the polygon
    coords.push(coords[0]);

    json!({
        "type": "Polygon",
        "coordinates": [coords]
    })
}

fn calculate_leg_bearings(task: &Task) -> Vec<(Option<f64>, Option<f64>)> {
    let points = &task.points;
    let n = points.len();

    if n == 0 {
        return vec![];
    }

    let mut bearings = Vec::with_capacity(n);

    for i in 0..n {
        let bearing_in = if i > 0 {
            Some(calculate_bearing(
                points[i - 1].waypoint.location,
                points[i].waypoint.location,
            ))
        } else {
            None
        };

        let bearing_out = if i < n - 1 {
            Some(calculate_bearing(
                points[i].waypoint.location,
                points[i + 1].waypoint.location,
            ))
        } else {
            None
        };

        bearings.push((bearing_in, bearing_out));
    }

    bearings
}

fn calculate_bisector(bearing_in: Option<f64>, bearing_out: Option<f64>) -> f64 {
    match (bearing_in, bearing_out) {
        (Some(inc), Some(out)) => {
            let outgoing_reversed = normalize_angle(out + 180.0);
            bisect_angles(inc, outgoing_reversed)
        }
        (Some(inc), None) => inc,
        (None, Some(out)) => normalize_angle(out + 180.0),
        (None, None) => panic!("cannot calculate bisector without at least one bearing"),
    }
}

fn calculate_bearing(from: Location, to: Location) -> f64 {
    let bearing = Haversine.bearing(
        GeoPoint::new(from.longitude, from.latitude),
        GeoPoint::new(to.longitude, to.latitude),
    );
    normalize_angle(bearing)
}

fn destination_point(from: Location, bearing: f64, distance: f64) -> Location {
    let result = Haversine.destination(
        GeoPoint::new(from.longitude, from.latitude),
        bearing,
        distance,
    );
    Location {
        latitude: result.y(),
        longitude: result.x(),
    }
}

fn normalize_angle(angle: f64) -> f64 {
    let mut a = angle % 360.0;
    if a < 0.0 {
        a += 360.0;
    }
    a
}

fn normalize_sweep(start: f64, end: f64) -> f64 {
    let mut sweep = end - start;
    while sweep < 0.0 {
        sweep += 2.0 * PI;
    }
    while sweep > 2.0 * PI {
        sweep -= 2.0 * PI;
    }
    sweep
}

fn bisect_angles(a: f64, b: f64) -> f64 {
    let a_rad = a.to_radians();
    let b_rad = b.to_radians();

    // Average using unit vectors
    let x = a_rad.cos() + b_rad.cos();
    let y = a_rad.sin() + b_rad.sin();

    normalize_angle(y.atan2(x).to_degrees())
}

fn point_type_to_string(pt: PointType) -> &'static str {
    match pt {
        PointType::Start => "Start",
        PointType::Turn => "Turn",
        PointType::Area => "Area",
        PointType::Finish => "Finish",
        PointType::OptionalStart => "OptionalStart",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn geojson_aat_task() {
        let xml = include_str!("../../fixtures/aat-task.tsk");
        let task = xcsoar_tasks::from_str(xml).unwrap();
        let geojson = task_to_geojson(&task);
        assert_json_snapshot!(geojson, {
            ".features[].geometry.coordinates.*" => insta::rounded_redaction(8),
            ".features[].geometry.coordinates[].*" => insta::rounded_redaction(8),
            ".features[].geometry.coordinates[][].*" => insta::rounded_redaction(8),
        });
    }

    #[test]
    fn geojson_racing_task() {
        let xml = include_str!("../../fixtures/racing-task.tsk");
        let task = xcsoar_tasks::from_str(xml).unwrap();
        let geojson = task_to_geojson(&task);
        assert_json_snapshot!(geojson, {
            ".features[].geometry.coordinates.*" => insta::rounded_redaction(8),
            ".features[].geometry.coordinates[].*" => insta::rounded_redaction(8),
            ".features[].geometry.coordinates[][].*" => insta::rounded_redaction(8),
        });
    }

    #[test]
    fn geojson_fai_task() {
        let xml = include_str!("../../fixtures/fai-task.tsk");
        let task = xcsoar_tasks::from_str(xml).unwrap();
        let geojson = task_to_geojson(&task);
        assert_json_snapshot!(geojson, {
            ".features[].geometry.coordinates.*" => insta::rounded_redaction(8),
            ".features[].geometry.coordinates[].*" => insta::rounded_redaction(8),
            ".features[].geometry.coordinates[][].*" => insta::rounded_redaction(8),
        });
    }
}
