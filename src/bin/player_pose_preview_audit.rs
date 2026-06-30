use image::{ImageReader, RgbImage};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

const MAX_BRIGHT_NEUTRAL_FRACTION: f64 = 0.08;
const MIN_DARK_PREVIEW_FRACTION: f64 = 0.42;
const MIN_COLORED_DETAIL_FRACTION: f64 = 0.0015;
const MIN_CONTENT_FRACTION: f64 = 0.004;
const MIN_QUANTIZED_COLORS: usize = 12;

const SHEETS: &[&str] = &[
    "player_pose_sheet",
    "player_anatomy_review_sheet",
    "player_rig_stress_review_sheet",
    "player_motion_integrity_review_sheet",
    "player_transition_pose_sheet",
    "glider_pose_sheet",
    "player_glider_attachment_sheet",
];

fn main() {
    let Some(preview_dir) = env::args().nth(1).map(PathBuf::from) else {
        eprintln!("Usage: cargo run --bin player_pose_preview_audit -- <preview_dir>");
        process::exit(2);
    };

    match audit_preview_dir(&preview_dir) {
        Ok(report) => {
            println!("{}", report_json(&report));
            if !report.passed {
                process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("failed to audit {}: {error}", preview_dir.display());
            process::exit(2);
        }
    }
}

#[derive(Clone, Debug)]
struct Check {
    name: &'static str,
    value: f64,
    threshold: f64,
    passed: bool,
}

#[derive(Clone, Debug)]
struct SheetAudit {
    name: &'static str,
    svg_path: String,
    png_path: String,
    width: u32,
    height: u32,
    expected_width: u32,
    expected_height: u32,
    bright_neutral_fraction: f64,
    dark_preview_fraction: f64,
    colored_detail_fraction: f64,
    content_fraction: f64,
    quantized_colors: usize,
    passed: bool,
    checks: Vec<Check>,
}

#[derive(Clone, Debug)]
struct PreviewAuditReport {
    preview_dir: String,
    passed: bool,
    sheet_count: usize,
    checks: Vec<Check>,
    sheets: Vec<SheetAudit>,
}

fn audit_preview_dir(preview_dir: &Path) -> Result<PreviewAuditReport, String> {
    let sheets = SHEETS
        .iter()
        .map(|name| audit_sheet(preview_dir, name))
        .collect::<Result<Vec<_>, _>>()?;
    let all_sheets_passed = sheets.iter().all(|sheet| sheet.passed);
    let sheet_count = sheets.len();
    let checks = vec![
        Check {
            name: "sheet_count",
            value: sheet_count as f64,
            threshold: SHEETS.len() as f64,
            passed: sheet_count == SHEETS.len(),
        },
        Check {
            name: "all_sheets_passed",
            value: if all_sheets_passed { 1.0 } else { 0.0 },
            threshold: 1.0,
            passed: all_sheets_passed,
        },
    ];
    let passed = checks.iter().all(|check| check.passed);

    Ok(PreviewAuditReport {
        preview_dir: preview_dir.to_string_lossy().into_owned(),
        passed,
        sheet_count,
        checks,
        sheets,
    })
}

fn audit_sheet(preview_dir: &Path, name: &'static str) -> Result<SheetAudit, String> {
    let svg_path = preview_dir.join(format!("{name}.svg"));
    let png_path = preview_dir.join(format!("{name}.png"));
    let svg = fs::read_to_string(&svg_path).map_err(|error| error.to_string())?;
    let expected_width = svg_dimension(&svg, "width")?;
    let expected_height = svg_dimension(&svg, "height")?;
    let image = ImageReader::open(&png_path)
        .map_err(|error| error.to_string())?
        .decode()
        .map_err(|error| error.to_string())?
        .to_rgb8();
    let metrics = sheet_metrics(&image);
    let (width, height) = image.dimensions();

    let checks = vec![
        Check {
            name: "width_matches_svg",
            value: width as f64,
            threshold: expected_width as f64,
            passed: width == expected_width,
        },
        Check {
            name: "height_matches_svg",
            value: height as f64,
            threshold: expected_height as f64,
            passed: height == expected_height,
        },
        Check {
            name: "bright_neutral_fraction_max",
            value: metrics.bright_neutral_fraction,
            threshold: MAX_BRIGHT_NEUTRAL_FRACTION,
            passed: metrics.bright_neutral_fraction <= MAX_BRIGHT_NEUTRAL_FRACTION,
        },
        Check {
            name: "dark_preview_fraction_min",
            value: metrics.dark_preview_fraction,
            threshold: MIN_DARK_PREVIEW_FRACTION,
            passed: metrics.dark_preview_fraction >= MIN_DARK_PREVIEW_FRACTION,
        },
        Check {
            name: "colored_detail_fraction_min",
            value: metrics.colored_detail_fraction,
            threshold: MIN_COLORED_DETAIL_FRACTION,
            passed: metrics.colored_detail_fraction >= MIN_COLORED_DETAIL_FRACTION,
        },
        Check {
            name: "content_fraction_min",
            value: metrics.content_fraction,
            threshold: MIN_CONTENT_FRACTION,
            passed: metrics.content_fraction >= MIN_CONTENT_FRACTION,
        },
        Check {
            name: "quantized_colors_min",
            value: metrics.quantized_colors as f64,
            threshold: MIN_QUANTIZED_COLORS as f64,
            passed: metrics.quantized_colors >= MIN_QUANTIZED_COLORS,
        },
    ];
    let passed = checks.iter().all(|check| check.passed);

    Ok(SheetAudit {
        name,
        svg_path: svg_path.to_string_lossy().into_owned(),
        png_path: png_path.to_string_lossy().into_owned(),
        width,
        height,
        expected_width,
        expected_height,
        bright_neutral_fraction: metrics.bright_neutral_fraction,
        dark_preview_fraction: metrics.dark_preview_fraction,
        colored_detail_fraction: metrics.colored_detail_fraction,
        content_fraction: metrics.content_fraction,
        quantized_colors: metrics.quantized_colors,
        passed,
        checks,
    })
}

#[derive(Clone, Debug)]
struct SheetMetrics {
    bright_neutral_fraction: f64,
    dark_preview_fraction: f64,
    colored_detail_fraction: f64,
    content_fraction: f64,
    quantized_colors: usize,
}

fn sheet_metrics(image: &RgbImage) -> SheetMetrics {
    let mut bright_neutral_pixels = 0usize;
    let mut dark_preview_pixels = 0usize;
    let mut colored_detail_pixels = 0usize;
    let mut content_pixels = 0usize;
    let mut quantized_colors = std::collections::HashSet::new();
    let total_pixels = image.width() as usize * image.height() as usize;

    for pixel in image.pixels() {
        let [r, g, b] = pixel.0;
        let r_u16 = r as u16;
        let g_u16 = g as u16;
        let b_u16 = b as u16;
        let max_channel = r_u16.max(g_u16).max(b_u16);
        let min_channel = r_u16.min(g_u16).min(b_u16);
        let chroma = max_channel - min_channel;
        let luma = 0.2126 * r as f64 + 0.7152 * g as f64 + 0.0722 * b as f64;
        let key = ((r as u32 / 32) << 6) | ((g as u32 / 32) << 3) | (b as u32 / 32);
        quantized_colors.insert(key);

        if luma >= 220.0 && chroma <= 18 {
            bright_neutral_pixels += 1;
        }
        if luma <= 58.0 {
            dark_preview_pixels += 1;
        }
        if chroma >= 36 && (34.0..=225.0).contains(&luma) {
            colored_detail_pixels += 1;
        }
        if luma > 64.0 && !(luma >= 220.0 && chroma <= 18) {
            content_pixels += 1;
        }
    }

    let total = total_pixels.max(1) as f64;
    SheetMetrics {
        bright_neutral_fraction: bright_neutral_pixels as f64 / total,
        dark_preview_fraction: dark_preview_pixels as f64 / total,
        colored_detail_fraction: colored_detail_pixels as f64 / total,
        content_fraction: content_pixels as f64 / total,
        quantized_colors: quantized_colors.len(),
    }
}

fn svg_dimension(svg: &str, attribute: &str) -> Result<u32, String> {
    let needle = format!("{attribute}=\"");
    let Some(start) = svg.find(&needle).map(|index| index + needle.len()) else {
        return Err(format!("missing SVG {attribute} attribute"));
    };
    let value = svg[start..]
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>();
    value
        .parse::<u32>()
        .map_err(|_| format!("invalid SVG {attribute} attribute"))
}

fn report_json(report: &PreviewAuditReport) -> String {
    let checks = report
        .checks
        .iter()
        .map(check_json)
        .collect::<Vec<_>>()
        .join(",\n    ");
    let sheets = report
        .sheets
        .iter()
        .map(sheet_json)
        .collect::<Vec<_>>()
        .join(",\n    ");
    format!(
        "{{\n  \"passed\": {},\n  \"preview_dir\": {},\n  \"sheet_count\": {},\n  \"checks\": [\n    {}\n  ],\n  \"sheets\": [\n    {}\n  ]\n}}",
        report.passed,
        json_string(&report.preview_dir),
        report.sheet_count,
        checks,
        sheets
    )
}

fn sheet_json(sheet: &SheetAudit) -> String {
    let checks = sheet
        .checks
        .iter()
        .map(check_json)
        .collect::<Vec<_>>()
        .join(",\n      ");
    format!(
        "{{\n      \"name\": {},\n      \"passed\": {},\n      \"svg_path\": {},\n      \"png_path\": {},\n      \"width\": {},\n      \"height\": {},\n      \"expected_width\": {},\n      \"expected_height\": {},\n      \"bright_neutral_fraction\": {},\n      \"dark_preview_fraction\": {},\n      \"colored_detail_fraction\": {},\n      \"content_fraction\": {},\n      \"quantized_colors\": {},\n      \"checks\": [\n      {}\n      ]\n    }}",
        json_string(sheet.name),
        sheet.passed,
        json_string(&sheet.svg_path),
        json_string(&sheet.png_path),
        sheet.width,
        sheet.height,
        sheet.expected_width,
        sheet.expected_height,
        json_number(sheet.bright_neutral_fraction),
        json_number(sheet.dark_preview_fraction),
        json_number(sheet.colored_detail_fraction),
        json_number(sheet.content_fraction),
        sheet.quantized_colors,
        checks
    )
}

fn check_json(check: &Check) -> String {
    format!(
        "{{\"name\": {}, \"value\": {}, \"threshold\": {}, \"passed\": {}}}",
        json_string(check.name),
        json_number(check.value),
        json_number(check.threshold),
        check.passed
    )
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).expect("string serialization should not fail")
}

fn json_number(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.6}")
    } else {
        "null".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgb;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn accepts_dark_pose_preview_with_colored_detail() {
        let preview_dir = test_preview_dir("accepts_dark_pose_preview_with_colored_detail");
        write_preview_set(&preview_dir, |image| {
            for x in 32..180 {
                for y in 36..120 {
                    image.put_pixel(x, y, Rgb([18, 28, 42]));
                }
            }
            for x in 72..118 {
                for y in 58..104 {
                    image.put_pixel(
                        x,
                        y,
                        Rgb([54 + (x % 6) as u8 * 8, 118, 154 + (y % 5) as u8 * 7]),
                    );
                }
            }
            for x in 122..142 {
                for y in 70..116 {
                    image.put_pixel(
                        x,
                        y,
                        Rgb([172 + (x % 4) as u8 * 9, 88 + (y % 4) as u8 * 7, 52]),
                    );
                }
            }
            let accent_colors = [
                Rgb([58, 96, 168]),
                Rgb([76, 142, 188]),
                Rgb([98, 156, 112]),
                Rgb([134, 116, 188]),
                Rgb([178, 102, 54]),
                Rgb([206, 142, 72]),
                Rgb([158, 78, 94]),
                Rgb([92, 178, 162]),
            ];
            for (index, color) in accent_colors.iter().enumerate() {
                let x_start = 24 + index as u32 * 12;
                for x in x_start..x_start + 8 {
                    for y in 128..138 {
                        image.put_pixel(x, y, *color);
                    }
                }
            }
        });

        let report = audit_preview_dir(&preview_dir).expect("audit");
        fs::remove_dir_all(&preview_dir).ok();

        assert!(report.passed, "{}", report_json(&report));
        assert_eq!(report.sheet_count, SHEETS.len());
    }

    #[test]
    fn rejects_foreign_bright_canvas_preview() {
        let preview_dir = test_preview_dir("rejects_foreign_bright_canvas_preview");
        write_preview_set(&preview_dir, |image| {
            for x in 0..image.width() {
                for y in 0..image.height() {
                    image.put_pixel(x, y, Rgb([246, 245, 242]));
                }
            }
            for x in 80..130 {
                for y in 80..128 {
                    image.put_pixel(x, y, Rgb([42, 122, 165]));
                }
            }
        });

        let report = audit_preview_dir(&preview_dir).expect("audit");
        fs::remove_dir_all(&preview_dir).ok();

        assert!(!report.passed);
        assert!(report.sheets.iter().all(|sheet| {
            sheet
                .checks
                .iter()
                .any(|check| check.name == "bright_neutral_fraction_max" && !check.passed)
        }));
    }

    fn test_preview_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let dir = env::temp_dir().join(format!("nau_{name}_{}_{}", process::id(), nanos));
        fs::create_dir_all(&dir).expect("create test preview dir");
        dir
    }

    fn write_preview_set(preview_dir: &Path, paint: impl Fn(&mut RgbImage)) {
        for sheet in SHEETS {
            let svg_path = preview_dir.join(format!("{sheet}.svg"));
            let png_path = preview_dir.join(format!("{sheet}.png"));
            fs::write(
                &svg_path,
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="240" height="160"></svg>"#,
            )
            .expect("write svg");
            let mut image = RgbImage::from_pixel(240, 160, Rgb([15, 23, 35]));
            paint(&mut image);
            image.save(&png_path).expect("write png");
        }
    }
}
