use crate::thresholds::TERRAIN_MATERIAL_REGION_COUNT;
use std::{collections::HashSet, fs, path::Path};

#[derive(Debug, PartialEq)]
pub(crate) struct ObjAudit {
    pub(crate) vertex_count: u64,
    pub(crate) face_count: u64,
    pub(crate) colored_vertex_count: u64,
    pub(crate) vertical_range_m: f64,
    pub(crate) vertical_band_count: u64,
    pub(crate) normal_slope_band_count: u64,
    pub(crate) horizontal_radius_bands: u64,
    pub(crate) silhouette_radius_bands: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct WeightCsvAudit {
    pub(crate) row_count: u64,
    pub(crate) region_row_count: u64,
    pub(crate) material_weight_bands: u64,
    pub(crate) material_channels: u64,
    pub(crate) material_regions: u64,
    pub(crate) region_promille: [u64; TERRAIN_MATERIAL_REGION_COUNT],
}

pub(crate) fn audit_obj_path(path: &Path) -> Result<ObjAudit, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(audit_obj_text(&text))
}

pub(crate) fn audit_obj_text(text: &str) -> ObjAudit {
    let mut vertex_count = 0;
    let mut face_count = 0;
    let mut colored_vertex_count = 0;
    let mut positions = Vec::new();
    let mut normals = Vec::new();

    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("v ") {
            vertex_count += 1;
            let columns = rest.split_whitespace().collect::<Vec<_>>();
            if columns.len() >= 6 {
                colored_vertex_count += 1;
            }
            if columns.len() >= 3 {
                let x = columns[0].parse::<f64>().unwrap_or(0.0);
                let y = columns[1].parse::<f64>().unwrap_or(0.0);
                let z = columns[2].parse::<f64>().unwrap_or(0.0);
                positions.push([x, y, z]);
            }
        } else if let Some(rest) = line.strip_prefix("vn ") {
            let columns = rest.split_whitespace().collect::<Vec<_>>();
            if columns.len() >= 3 {
                let x = columns[0].parse::<f64>().unwrap_or(0.0);
                let y = columns[1].parse::<f64>().unwrap_or(1.0);
                let z = columns[2].parse::<f64>().unwrap_or(0.0);
                normals.push([x, y, z]);
            }
        } else if line.starts_with("f ") {
            face_count += 1;
        }
    }
    let vertical_range_m = vertical_position_range_m(&positions);
    let vertical_band_count = vertical_position_band_count(&positions);
    let normal_slope_band_count = normal_slope_band_count(&normals);
    let horizontal_radius_bands = horizontal_radius_band_count(&positions);
    let silhouette_radius_bands = silhouette_radius_band_count(&positions);

    ObjAudit {
        vertex_count,
        face_count,
        colored_vertex_count,
        vertical_range_m,
        vertical_band_count,
        normal_slope_band_count,
        horizontal_radius_bands,
        silhouette_radius_bands,
    }
}

fn vertical_position_range_m(positions: &[[f64; 3]]) -> f64 {
    if positions.is_empty() {
        return 0.0;
    }
    let min_y = positions
        .iter()
        .map(|position| position[1])
        .fold(f64::INFINITY, f64::min);
    let max_y = positions
        .iter()
        .map(|position| position[1])
        .fold(f64::NEG_INFINITY, f64::max);

    (max_y - min_y).max(0.0)
}

fn vertical_position_band_count(positions: &[[f64; 3]]) -> u64 {
    if positions.is_empty() {
        return 0;
    }
    let min_y = positions
        .iter()
        .map(|position| position[1])
        .fold(f64::INFINITY, f64::min);
    if !min_y.is_finite() {
        return 0;
    }

    positions
        .iter()
        .map(|position| ((position[1] - min_y) / 0.05).round() as i64)
        .collect::<HashSet<_>>()
        .len() as u64
}

fn normal_slope_band_count(normals: &[[f64; 3]]) -> u64 {
    normals
        .iter()
        .filter(|normal| normal[1] > 0.0)
        .map(|normal| {
            let horizontal = (normal[0] * normal[0] + normal[2] * normal[2]).sqrt();
            let slope_degrees = horizontal.atan2(normal[1].max(0.0001)).to_degrees();
            (slope_degrees * 2.0).round() as i64
        })
        .collect::<HashSet<_>>()
        .len() as u64
}

fn horizontal_radius_band_count(positions: &[[f64; 3]]) -> u64 {
    if positions.len() < 3 {
        return 0;
    }
    let inv_count = 1.0 / positions.len() as f64;
    let center_x = positions.iter().map(|position| position[0]).sum::<f64>() * inv_count;
    let center_z = positions.iter().map(|position| position[2]).sum::<f64>() * inv_count;
    let radii = positions
        .iter()
        .map(|position| {
            let x = position[0] - center_x;
            let z = position[2] - center_z;
            (x * x + z * z).sqrt()
        })
        .collect::<Vec<_>>();
    let max_radius = radii.iter().copied().fold(0.0, f64::max);
    if max_radius <= f64::EPSILON {
        return 0;
    }

    radii
        .iter()
        .copied()
        .filter(|radius| *radius > max_radius * 0.08)
        .map(|radius| ((radius / max_radius) * 24.0).round() as u64)
        .collect::<HashSet<_>>()
        .len() as u64
}

fn silhouette_radius_band_count(positions: &[[f64; 3]]) -> u64 {
    const ANGULAR_BINS: usize = 32;
    if positions.len() < 3 {
        return 0;
    }
    let inv_count = 1.0 / positions.len() as f64;
    let center_x = positions.iter().map(|position| position[0]).sum::<f64>() * inv_count;
    let center_z = positions.iter().map(|position| position[2]).sum::<f64>() * inv_count;
    let mut bins = [0.0_f64; ANGULAR_BINS];
    for position in positions {
        let x = position[0] - center_x;
        let z = position[2] - center_z;
        let radius = (x * x + z * z).sqrt();
        if radius <= f64::EPSILON {
            continue;
        }
        let mut angle = z.atan2(x);
        if angle < 0.0 {
            angle += std::f64::consts::TAU;
        }
        let bin = ((angle / std::f64::consts::TAU) * ANGULAR_BINS as f64).floor() as usize;
        let bin = bin.min(ANGULAR_BINS - 1);
        bins[bin] = bins[bin].max(radius);
    }
    let max_radius = bins.iter().copied().fold(0.0, f64::max);
    if max_radius <= f64::EPSILON {
        return 0;
    }

    bins.iter()
        .copied()
        .filter(|radius| *radius > max_radius * 0.5)
        .map(|radius| ((radius / max_radius) * 24.0).round() as u64)
        .collect::<HashSet<_>>()
        .len() as u64
}

pub(crate) fn audit_weight_csv_path(
    path: &Path,
    region_row_limit: Option<u64>,
) -> Result<WeightCsvAudit, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    audit_weight_csv_text(&text, region_row_limit)
}

pub(crate) fn audit_weight_csv_text(
    text: &str,
    region_row_limit: Option<u64>,
) -> Result<WeightCsvAudit, String> {
    let mut lines = text.lines();
    let header = lines
        .next()
        .ok_or_else(|| "empty material weights csv".to_string())?;
    if header != "vertex,lush_highland,exposed_edge" {
        return Err(format!("unexpected material weights csv header: {header}"));
    }

    let mut row_count = 0;
    let mut bands = HashSet::new();
    let mut base = false;
    let mut lush = false;
    let mut exposed = false;
    let mut regions = HashSet::new();
    let mut region_counts = [0u64; TERRAIN_MATERIAL_REGION_COUNT];
    let mut region_row_count = 0;
    let region_row_limit = region_row_limit.unwrap_or(u64::MAX);

    for line in lines {
        let columns = line.split(',').collect::<Vec<_>>();
        if columns.len() != 3 {
            return Err(format!("invalid material weights csv row: {line}"));
        }
        let lush_highland = columns[1]
            .parse::<f32>()
            .map_err(|error| format!("invalid lush/highland weight: {error}"))?
            .clamp(0.0, 1.0);
        let exposed_edge = columns[2]
            .parse::<f32>()
            .map_err(|error| format!("invalid exposed-edge weight: {error}"))?
            .clamp(0.0, 1.0);

        bands.insert([
            (lush_highland * 15.0).round() as u8,
            (exposed_edge * 15.0).round() as u8,
        ]);
        let region = terrain_material_region_id(lush_highland, exposed_edge);
        regions.insert(region);
        if region_row_count < region_row_limit {
            region_counts[region as usize] += 1;
            region_row_count += 1;
        }
        base |= lush_highland < 0.18 && exposed_edge < 0.18;
        lush |= lush_highland > 0.18;
        exposed |= exposed_edge > 0.18;
        row_count += 1;
    }

    let region_promille = if region_row_count == 0 {
        [0; TERRAIN_MATERIAL_REGION_COUNT]
    } else {
        region_counts.map(|count| count * 1000 / region_row_count)
    };

    Ok(WeightCsvAudit {
        row_count,
        region_row_count,
        material_weight_bands: bands.len() as u64,
        material_channels: u64::from(base) + u64::from(lush) + u64::from(exposed),
        material_regions: regions.len() as u64,
        region_promille,
    })
}

fn terrain_material_region_id(lush_highland: f32, exposed_edge: f32) -> u8 {
    if exposed_edge >= 0.48 {
        3
    } else if lush_highland >= 0.42 {
        2
    } else if lush_highland >= 0.24 || exposed_edge >= 0.10 {
        1
    } else {
        0
    }
}
