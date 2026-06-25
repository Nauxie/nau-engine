use super::*;
use image::Rgb;

fn paint_player_and_route_markers(image: &mut RgbImage) {
    let width = image.width();
    let height = image.height();
    let center_x = width / 2;

    for y in height * 54 / 100..height * 72 / 100 {
        for x in center_x - width / 80..=center_x + width / 80 {
            image.put_pixel(x, y, Rgb([24, 28, 26]));
        }
    }
    for y in height * 48 / 100..height * 56 / 100 {
        for x in center_x - width / 22..=center_x + width / 22 {
            image.put_pixel(x, y, Rgb([188, 84, 34]));
        }
    }
    for y in height * 48 / 100..height * 72 / 100 {
        let x = width * 68 / 100 + (y % 3);
        image.put_pixel(x, y, Rgb([246, 58, 142]));
        image.put_pixel(x + 8, y, Rgb([64, 226, 92]));
    }
}

fn paint_distant_scene_signals(image: &mut RgbImage) {
    let width = image.width();
    let height = image.height();

    for y in height * 19 / 100..height * 30 / 100 {
        for x in width * 42 / 100..width * 54 / 100 {
            let cap = y < height * 24 / 100;
            let r = if cap { 68 + (x % 34) } else { 50 + (x % 46) };
            let g = if cap { 112 + (y % 48) } else { 54 + (y % 42) };
            let b = if cap {
                48 + ((x + y) % 24)
            } else {
                32 + ((x + y) % 22)
            };
            image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
        }
    }

    for y in height * 22 / 100..height * 32 / 100 {
        for x in width * 72 / 100..width * 86 / 100 {
            let cap = y < height * 27 / 100;
            let r = if cap { 96 + (x % 38) } else { 58 + (x % 52) };
            let g = if cap { 118 + (y % 44) } else { 50 + (y % 38) };
            let b = if cap {
                54 + ((x + y) % 28)
            } else {
                34 + ((x + y) % 26)
            };
            image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
        }
    }
}

fn paint_high_sky_textured_scene(image: &mut RgbImage) {
    let width = image.width();
    let height = image.height();
    for y in 0..height {
        for x in 0..width {
            let checker = (x + y) % 2 == 0;
            let (r, g, b) = if y < height * 64 / 100 {
                (104 + (x % 18), 136 + (y % 22), 198 + ((x + y) % 24))
            } else if checker {
                (48 + (x % 92), 110 + (y % 80), 58 + ((x + y) % 50))
            } else {
                (112 + (x % 70), 82 + (y % 60), 54 + ((x + y) % 44))
            };
            image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
        }
    }
}

fn paint_high_sky_foliage_scene(image: &mut RgbImage) {
    let width = image.width();
    let height = image.height();
    for y in 0..height {
        for x in 0..width {
            let (r, g, b) = if y < height * 64 / 100 {
                (112 + (x % 84), 142 + (y % 78), 178 + ((x + y) % 72))
            } else {
                (36 + (x % 84), 124 + (y % 76), 18 + ((x + y) % 22))
            };
            image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
        }
    }
}

fn paint_high_sky_non_foliage_scene(image: &mut RgbImage) {
    let width = image.width();
    let height = image.height();
    for y in 0..height {
        for x in 0..width {
            let checker = (x + y) % 3;
            let (r, g, b) = if y < height * 64 / 100 {
                (104 + (x % 18), 136 + (y % 22), 198 + ((x + y) % 24))
            } else if checker == 0 {
                (34 + (x % 28), 86 + (y % 38), 118 + ((x + y) % 42))
            } else if checker == 1 {
                (126 + (x % 50), 62 + (y % 24), 46 + ((x + y) % 26))
            } else {
                (80 + (x % 32), 46 + (y % 10), 60 + ((x + y) % 16))
            };
            image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
        }
    }
}

fn paint_foliage_distant_scene_signals(image: &mut RgbImage) {
    let width = image.width();
    let height = image.height();
    for y in height * 19 / 100..height * 30 / 100 {
        for x in width * 42 / 100..width * 54 / 100 {
            image.put_pixel(
                x,
                y,
                Rgb([
                    48 + (x % 22) as u8,
                    112 + (y % 36) as u8,
                    26 + ((x + y) % 12) as u8,
                ]),
            );
        }
    }
    for y in height * 22 / 100..height * 31 / 100 {
        for x in width * 72 / 100..width * 86 / 100 {
            image.put_pixel(
                x,
                y,
                Rgb([
                    52 + (x % 24) as u8,
                    118 + (y % 34) as u8,
                    24 + ((x + y) % 10) as u8,
                ]),
            );
        }
    }
}

fn paint_flat_distant_scene_signals(image: &mut RgbImage) {
    let width = image.width();
    let height = image.height();
    for y in height * 20 / 100..height * 29 / 100 {
        for x in width * 42 / 100..width * 54 / 100 {
            image.put_pixel(x, y, Rgb([132, 92, 52]));
        }
    }
    for y in height * 22 / 100..height * 31 / 100 {
        for x in width * 72 / 100..width * 86 / 100 {
            image.put_pixel(x, y, Rgb([132, 92, 52]));
        }
    }
}

fn paint_cloud_layer_signals(image: &mut RgbImage) {
    let width = image.width();
    let height = image.height();
    for y in height * 7 / 100..height * 15 / 100 {
        for x in width * 45 / 100..width * 66 / 100 {
            let dx = x as i64 - (width * 55 / 100) as i64;
            let dy = y as i64 - (height * 11 / 100) as i64;
            if dx * dx * 36 + dy * dy * 240 <= (width as i64 * width as i64) / 3 {
                image.put_pixel(
                    x,
                    y,
                    Rgb([
                        164 + (x % 18) as u8,
                        174 + (y % 16) as u8,
                        184 + ((x + y) % 18) as u8,
                    ]),
                );
            }
        }
    }
    for y in height * 15 / 100..height * 23 / 100 {
        for x in width * 74 / 100..width * 92 / 100 {
            let dx = x as i64 - (width * 83 / 100) as i64;
            let dy = y as i64 - (height * 19 / 100) as i64;
            if dx * dx * 44 + dy * dy * 280 <= (width as i64 * width as i64) / 4 {
                image.put_pixel(
                    x,
                    y,
                    Rgb([
                        148 + (x % 20) as u8,
                        158 + (y % 18) as u8,
                        168 + ((x + y) % 20) as u8,
                    ]),
                );
            }
        }
    }
}

fn paint_readability_signals(image: &mut RgbImage) {
    paint_player_and_route_markers(image);
    paint_distant_scene_signals(image);
    paint_cloud_layer_signals(image);
}

#[test]
fn audit_passes_textured_color_image() {
    let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
    for y in 0..MIN_HEIGHT {
        for x in 0..MIN_WIDTH {
            let checker = (x + y) % 2 == 0;
            let (r, g, b) = if y < MIN_HEIGHT / 3 {
                (120 + (x % 48), 150 + (y % 48), 185 + ((x + y) % 48))
            } else if checker {
                (36 + (x % 120), 90 + (y % 120), 45 + ((x + y) % 80))
            } else {
                (100 + (x % 120), 70 + (y % 80), 42 + ((x + y) % 70))
            };
            image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
        }
    }
    paint_readability_signals(&mut image);

    let audit = audit_image("synthetic.png".to_string(), image).expect("audit should load");

    assert!(audit.passed, "{audit:?}");
}

#[test]
fn audit_rejects_flat_image() {
    let image = RgbImage::from_pixel(MIN_WIDTH, MIN_HEIGHT, Rgb([16, 16, 16]));

    let audit = audit_image("flat.png".to_string(), image).expect("audit should load");

    assert!(!audit.passed);
    assert!(
        audit
            .checks
            .iter()
            .any(|check| check.name == "luma_stddev" && !check.passed)
    );
    assert!(
        audit
            .checks
            .iter()
            .any(|check| check.name == "edge_density" && !check.passed)
    );
}

#[test]
fn audit_rejects_non_opaque_image() {
    let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
    for y in 0..MIN_HEIGHT {
        for x in 0..MIN_WIDTH {
            let checker = (x + y) % 2 == 0;
            let (r, g, b) = if y < MIN_HEIGHT / 3 {
                (120 + (x % 48), 150 + (y % 48), 185 + ((x + y) % 48))
            } else if checker {
                (36 + (x % 120), 90 + (y % 120), 45 + ((x + y) % 80))
            } else {
                (100 + (x % 120), 70 + (y % 80), 42 + ((x + y) % 70))
            };
            image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
        }
    }
    paint_readability_signals(&mut image);

    let audit = audit_image_with_alpha("transparent.png".to_string(), image, 0.01)
        .expect("audit should load");

    assert!(!audit.passed);
    assert!(
        audit
            .checks
            .iter()
            .any(|check| check.name == "transparent_pixel_fraction" && !check.passed)
    );
}

#[test]
fn audit_rejects_large_foreign_canvas_regions() {
    let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
    for y in 0..MIN_HEIGHT {
        for x in 0..MIN_WIDTH {
            let checker = (x + y) % 2 == 0;
            let (r, g, b) = if y < MIN_HEIGHT / 3 {
                (120 + (x % 48), 150 + (y % 48), 185 + ((x + y) % 48))
            } else if checker {
                (36 + (x % 120), 90 + (y % 120), 45 + ((x + y) % 80))
            } else {
                (100 + (x % 120), 70 + (y % 80), 42 + ((x + y) % 70))
            };
            image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
        }
    }
    for y in 0..MIN_HEIGHT * 86 / 100 {
        for x in MIN_WIDTH * 42 / 100..MIN_WIDTH {
            image.put_pixel(x, y, Rgb([246, 242, 232]));
        }
    }
    paint_readability_signals(&mut image);

    let audit = audit_image("foreign_canvas.png".to_string(), image).expect("audit should load");

    assert!(!audit.passed);
    assert!(audit.foreign_canvas_fraction > MAX_FOREIGN_CANVAS_FRACTION);
    assert!(
        audit
            .checks
            .iter()
            .any(|check| check.name == "foreign_canvas_fraction" && !check.passed)
    );
}

#[test]
fn audit_rejects_large_low_detail_scene_surface() {
    let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
    for y in 0..MIN_HEIGHT {
        for x in 0..MIN_WIDTH {
            let (r, g, b) = if y < MIN_HEIGHT / 3 {
                (126, 158, 196)
            } else {
                (92, 74, 52)
            };
            image.put_pixel(x, y, Rgb([r, g, b]));
        }
    }
    paint_readability_signals(&mut image);

    let audit = audit_image("low_detail_scene.png".to_string(), image).expect("audit should load");

    assert!(!audit.passed);
    assert!(audit.scene_candidate_tile_count > 0);
    assert!(
        audit
            .checks
            .iter()
            .any(|check| check.name == "scene_detail_tile_fraction" && !check.passed)
    );
}

#[test]
fn report_rejects_readable_sequence_without_sky() {
    let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
    for y in 0..MIN_HEIGHT {
        for x in 0..MIN_WIDTH {
            let checker = (x + y) % 2 == 0;
            let (r, g, b) = if checker {
                (46 + (x % 90), 104 + (y % 86), 56 + ((x + y) % 54))
            } else {
                (108 + (x % 76), 78 + (y % 62), 50 + ((x + y) % 48))
            };
            image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
        }
    }
    paint_readability_signals(&mut image);

    let audit = audit_image("no_sky.png".to_string(), image).expect("audit should load");
    let checks = report_checks(std::slice::from_ref(&audit));

    assert!(audit.passed, "{audit:?}");
    assert!(!report_passed(std::slice::from_ref(&audit), &checks));
    assert!(
        checks
            .iter()
            .any(|check| check.name == "max_top_sky_fraction" && !check.passed)
    );
}

#[test]
fn report_allows_low_sky_close_frame_when_checkpoint_has_sky() {
    let mut close_image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
    for y in 0..MIN_HEIGHT {
        for x in 0..MIN_WIDTH {
            let checker = (x + y) % 2 == 0;
            let (r, g, b) = if y < MIN_HEIGHT / 18 {
                (124 + (x % 32), 154 + (y % 32), 190 + ((x + y) % 32))
            } else if y < MIN_HEIGHT / 3 {
                (72 + (x % 74), 60 + (y % 58), 42 + ((x + y) % 42))
            } else if checker {
                (42 + (x % 95), 105 + (y % 90), 54 + ((x + y) % 56))
            } else {
                (112 + (x % 82), 82 + (y % 64), 50 + ((x + y) % 52))
            };
            close_image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
        }
    }
    paint_readability_signals(&mut close_image);

    let close_audit = audit_image("close.png".to_string(), close_image).expect("audit should load");
    assert!(close_audit.passed, "{close_audit:?}");
    assert!(close_audit.top_sky_fraction < MIN_SEQUENCE_TOP_SKY_FRACTION);

    let close_only_checks = report_checks(std::slice::from_ref(&close_audit));
    assert!(!report_passed(
        std::slice::from_ref(&close_audit),
        &close_only_checks
    ));

    let mut checkpoint_image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
    for y in 0..MIN_HEIGHT {
        for x in 0..MIN_WIDTH {
            let checker = (x + y) % 2 == 0;
            let (r, g, b) = if y < MIN_HEIGHT / 3 {
                (132 + (x % 32), 162 + (y % 32), 198 + ((x + y) % 32))
            } else if checker {
                (44 + (x % 90), 110 + (y % 90), 58 + ((x + y) % 52))
            } else {
                (118 + (x % 78), 84 + (y % 58), 52 + ((x + y) % 48))
            };
            checkpoint_image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
        }
    }
    paint_readability_signals(&mut checkpoint_image);

    let checkpoint_audit =
        audit_image("checkpoint.png".to_string(), checkpoint_image).expect("audit should load");
    let audits = vec![close_audit, checkpoint_audit];
    let checks = report_checks(&audits);

    assert!(report_passed(&audits, &checks), "{checks:?}");
}

#[test]
fn report_rejects_sequence_without_distant_scene_components() {
    let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
    paint_high_sky_textured_scene(&mut image);
    paint_player_and_route_markers(&mut image);

    let audit =
        audit_image("missing_distant_scene.png".to_string(), image).expect("audit should load");
    let checks = report_checks(std::slice::from_ref(&audit));

    assert!(audit.passed, "{audit:?}");
    assert!(!report_passed(std::slice::from_ref(&audit), &checks));
    assert!(
        checks
            .iter()
            .any(|check| check.name == "max_distant_scene_component_count" && !check.passed)
    );
}

#[test]
fn report_rejects_flat_distant_scene_identity() {
    let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
    paint_high_sky_textured_scene(&mut image);
    paint_player_and_route_markers(&mut image);
    paint_flat_distant_scene_signals(&mut image);

    let audit =
        audit_image("flat_distant_scene.png".to_string(), image).expect("audit should load");
    let checks = report_checks(std::slice::from_ref(&audit));

    assert!(audit.passed, "{audit:?}");
    assert!(audit.distant_scene_component_count >= MIN_SEQUENCE_DISTANT_SCENE_COMPONENTS);
    assert!(!report_passed(std::slice::from_ref(&audit), &checks));
    assert!(
        checks
            .iter()
            .any(|check| { check.name == "max_distant_scene_color_bucket_count" && !check.passed })
    );
}

#[test]
fn report_rejects_single_family_scene_materials() {
    let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
    paint_high_sky_foliage_scene(&mut image);
    paint_player_and_route_markers(&mut image);
    paint_foliage_distant_scene_signals(&mut image);
    paint_cloud_layer_signals(&mut image);

    let audit =
        audit_image("single_family_materials.png".to_string(), image).expect("audit should load");
    let checks = report_checks(std::slice::from_ref(&audit));

    assert!(audit.passed, "{audit:?}");
    assert!(audit.scene_material_family_count < MIN_SEQUENCE_SCENE_MATERIAL_FAMILIES);
    assert!(!report_passed(std::slice::from_ref(&audit), &checks));
    assert!(
        checks
            .iter()
            .any(|check| check.name == "max_scene_material_family_count" && !check.passed)
    );
}

#[test]
fn report_rejects_sequence_without_foliage_readability() {
    let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
    paint_high_sky_non_foliage_scene(&mut image);
    paint_player_and_route_markers(&mut image);
    paint_flat_distant_scene_signals(&mut image);
    paint_cloud_layer_signals(&mut image);

    let audit = audit_image("missing_foliage.png".to_string(), image).expect("audit should load");
    let checks = report_checks(std::slice::from_ref(&audit));

    assert!(audit.passed, "{audit:?}");
    assert!(audit.scene_material_family_count >= MIN_SEQUENCE_SCENE_MATERIAL_FAMILIES);
    assert!(
        audit.foliage_scene_fraction < MIN_SEQUENCE_FOLIAGE_SCENE_FRACTION,
        "foliage_scene_fraction {} should stay below {}",
        audit.foliage_scene_fraction,
        MIN_SEQUENCE_FOLIAGE_SCENE_FRACTION
    );
    assert!(!report_passed(std::slice::from_ref(&audit), &checks));
    assert!(
        checks
            .iter()
            .any(|check| check.name == "max_foliage_scene_fraction" && !check.passed)
    );
}

#[test]
fn report_rejects_sequence_without_cloud_layer_components() {
    let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
    paint_high_sky_textured_scene(&mut image);
    paint_player_and_route_markers(&mut image);
    paint_distant_scene_signals(&mut image);

    let audit =
        audit_image("missing_cloud_layer.png".to_string(), image).expect("audit should load");
    let checks = report_checks(std::slice::from_ref(&audit));

    assert!(audit.passed, "{audit:?}");
    assert!(
        audit.cloud_layer_fraction < MIN_SEQUENCE_CLOUD_LAYER_FRACTION
            || audit.cloud_layer_component_count < MIN_SEQUENCE_CLOUD_LAYER_COMPONENTS
    );
    assert!(!report_passed(std::slice::from_ref(&audit), &checks));
    assert!(checks.iter().any(|check| {
        (check.name == "max_cloud_layer_fraction"
            || check.name == "max_cloud_layer_component_count")
            && !check.passed
    }));
}

#[test]
fn audit_rejects_sky_only_frame() {
    let image = RgbImage::from_pixel(MIN_WIDTH, MIN_HEIGHT, Rgb([136, 170, 208]));

    let audit = audit_image("sky_only.png".to_string(), image).expect("audit should load");

    assert!(!audit.passed);
    assert!(
        audit
            .checks
            .iter()
            .any(|check| check.name == "lower_scene_fraction" && !check.passed)
    );
}

#[test]
fn audit_passes_water_heavy_scene() {
    let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
    for y in 0..MIN_HEIGHT {
        for x in 0..MIN_WIDTH {
            let (r, g, b) = if y < MIN_HEIGHT / 3 {
                (125 + (x % 38), 154 + (y % 46), 190 + ((x + y) % 46))
            } else {
                let wave = ((x / 5 + y / 3) % 2) * 42;
                (34 + (x % 28), 88 + wave + (y % 40), 112 + wave + (x % 52))
            };
            image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
        }
    }
    paint_readability_signals(&mut image);

    let audit = audit_image("water.png".to_string(), image).expect("audit should load");

    assert!(audit.passed, "{audit:?}");
    assert!(audit.lower_scene_fraction >= MIN_LOWER_SCENE_FRACTION);
    assert!(audit.center_scene_fraction >= MIN_CENTER_SCENE_FRACTION);
}

#[test]
fn audit_allows_dark_player_without_warm_focus_pixels() {
    let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
    for y in 0..MIN_HEIGHT {
        for x in 0..MIN_WIDTH {
            let checker = (x + y) % 2 == 0;
            let (r, g, b) = if y < MIN_HEIGHT / 3 {
                (126 + (x % 36), 158 + (y % 36), 196 + ((x + y) % 36))
            } else if checker {
                (42 + (x % 70), 96 + (y % 60), 58 + ((x + y) % 48))
            } else {
                (52 + (x % 52), 64 + (y % 48), 72 + ((x + y) % 36))
            };
            image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
        }
    }

    let center_x = MIN_WIDTH / 2;
    for y in MIN_HEIGHT * 50 / 100..MIN_HEIGHT * 72 / 100 {
        for x in center_x - MIN_WIDTH / 70..=center_x + MIN_WIDTH / 70 {
            image.put_pixel(x, y, Rgb([24, 28, 26]));
        }
    }

    let audit = audit_image("dark_player.png".to_string(), image).expect("audit should load");

    assert!(audit.passed, "{audit:?}");
    assert!(audit.player_focus_fraction >= MIN_PLAYER_FOCUS_FRACTION);
    assert_eq!(audit.player_warm_focus_fraction, 0.0);
}

#[test]
fn audit_does_not_count_bright_clouds_outside_hud_regions_as_hud_text() {
    let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
    for y in 0..MIN_HEIGHT {
        for x in 0..MIN_WIDTH {
            let checker = (x + y) % 2 == 0;
            let (r, g, b) = if y < MIN_HEIGHT / 3 {
                if x > MIN_WIDTH / 2 && y < MIN_HEIGHT / 5 {
                    (232, 235, 238)
                } else {
                    (124 + (x % 42), 156 + (y % 44), 194 + ((x + y) % 44))
                }
            } else if checker {
                (48 + (x % 95), 102 + (y % 95), 55 + ((x + y) % 60))
            } else {
                (105 + (x % 85), 76 + (y % 66), 48 + ((x + y) % 54))
            };
            image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
        }
    }
    paint_readability_signals(&mut image);

    let audit = audit_image("clouds.png".to_string(), image).expect("audit should load");

    assert!(audit.passed, "{audit:?}");
    assert!(audit.hud_text_fraction <= MAX_HUD_TEXT_FRACTION);
}

#[test]
fn audit_rejects_missing_player_focus() {
    let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
    for y in 0..MIN_HEIGHT {
        for x in 0..MIN_WIDTH {
            let (r, g, b) = if y < MIN_HEIGHT / 3 {
                (126 + (x % 36), 158 + (y % 36), 196 + ((x + y) % 36))
            } else {
                (54 + (x % 90), 112 + (y % 78), 58 + ((x + y) % 48))
            };
            image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
        }
    }

    let audit = audit_image("missing_player.png".to_string(), image).expect("audit should load");

    assert!(!audit.passed);
    assert!(
        audit
            .checks
            .iter()
            .any(|check| check.name == "player_focus_fraction" && !check.passed)
    );
}

#[test]
fn audit_rejects_severe_border_clipping() {
    let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
    for y in 0..MIN_HEIGHT {
        for x in 0..MIN_WIDTH {
            let checker = (x + y) % 2 == 0;
            let (r, g, b) = if y < MIN_HEIGHT / 3 {
                (126 + (x % 36), 158 + (y % 36), 196 + ((x + y) % 36))
            } else if checker {
                (48 + (x % 92), 110 + (y % 80), 58 + ((x + y) % 50))
            } else {
                (112 + (x % 70), 82 + (y % 60), 54 + ((x + y) % 44))
            };
            image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
        }
    }
    paint_readability_signals(&mut image);

    let top_band = MIN_HEIGHT * 8 / 100;
    for y in 0..top_band {
        for x in MIN_WIDTH * 36 / 100..MIN_WIDTH {
            image.put_pixel(x, y, Rgb([74, 62, 46]));
        }
    }

    let audit = audit_image("clipped.png".to_string(), image).expect("audit should load");

    assert!(!audit.passed);
    assert!(audit.severe_clipping_fraction > MAX_SEVERE_CLIPPING_FRACTION);
    assert!(
        audit
            .checks
            .iter()
            .any(|check| check.name == "severe_clipping_fraction" && !check.passed)
    );
}

#[test]
fn audit_allows_continuous_scene_surface_at_border() {
    let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
    for y in 0..MIN_HEIGHT {
        for x in 0..MIN_WIDTH {
            let checker = (x + y) % 2 == 0;
            let (r, g, b) = if y < MIN_HEIGHT / 3 {
                (126 + (x % 36), 158 + (y % 36), 196 + ((x + y) % 36))
            } else if checker {
                (48 + (x % 92), 110 + (y % 80), 58 + ((x + y) % 50))
            } else {
                (112 + (x % 70), 82 + (y % 60), 54 + ((x + y) % 44))
            };
            image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
        }
    }
    paint_readability_signals(&mut image);

    let top_inner_band = MIN_HEIGHT * 16 / 100;
    for y in 0..top_inner_band {
        for x in MIN_WIDTH * 36 / 100..MIN_WIDTH {
            image.put_pixel(x, y, Rgb([74, 62, 46]));
        }
    }

    let audit =
        audit_image("continuous_border_surface.png".to_string(), image).expect("audit should load");

    assert!(audit.passed, "{audit:?}");
    assert!(audit.severe_clipping_fraction <= MAX_SEVERE_CLIPPING_FRACTION);
}

#[test]
fn report_rejects_missing_route_markers() {
    let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
    for y in 0..MIN_HEIGHT {
        for x in 0..MIN_WIDTH {
            let checker = (x + y) % 2 == 0;
            let (r, g, b) = if y < MIN_HEIGHT / 3 {
                (126 + (x % 34), 158 + (y % 34), 196 + ((x + y) % 34))
            } else if checker {
                (48 + (x % 92), 110 + (y % 80), 58 + ((x + y) % 50))
            } else {
                (112 + (x % 70), 82 + (y % 60), 54 + ((x + y) % 44))
            };
            image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
        }
    }
    let center_x = MIN_WIDTH / 2;
    for y in MIN_HEIGHT * 54 / 100..MIN_HEIGHT * 72 / 100 {
        for x in center_x - MIN_WIDTH / 80..=center_x + MIN_WIDTH / 80 {
            image.put_pixel(x, y, Rgb([24, 28, 26]));
        }
    }
    for y in MIN_HEIGHT * 48 / 100..MIN_HEIGHT * 56 / 100 {
        for x in center_x - MIN_WIDTH / 22..=center_x + MIN_WIDTH / 22 {
            image.put_pixel(x, y, Rgb([188, 84, 34]));
        }
    }
    paint_distant_scene_signals(&mut image);

    let audit =
        audit_image("missing_route_marker.png".to_string(), image).expect("audit should load");
    assert!(audit.passed, "{audit:?}");
    let checks = report_checks(std::slice::from_ref(&audit));

    assert!(!report_passed(std::slice::from_ref(&audit), &checks));
    assert!(
        checks
            .iter()
            .any(|check| check.name == "max_route_marker_fraction" && !check.passed)
    );
}

#[test]
fn report_rejects_single_blob_route_marker_identity() {
    let mut image = RgbImage::new(MIN_WIDTH, MIN_HEIGHT);
    for y in 0..MIN_HEIGHT {
        for x in 0..MIN_WIDTH {
            let checker = (x + y) % 2 == 0;
            let (r, g, b) = if y < MIN_HEIGHT / 3 {
                (126 + (x % 34), 158 + (y % 34), 196 + ((x + y) % 34))
            } else if checker {
                (48 + (x % 92), 110 + (y % 80), 58 + ((x + y) % 50))
            } else {
                (112 + (x % 70), 82 + (y % 60), 54 + ((x + y) % 44))
            };
            image.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
        }
    }

    let center_x = MIN_WIDTH / 2;
    for y in MIN_HEIGHT * 54 / 100..MIN_HEIGHT * 72 / 100 {
        for x in center_x - MIN_WIDTH / 80..=center_x + MIN_WIDTH / 80 {
            image.put_pixel(x, y, Rgb([24, 28, 26]));
        }
    }
    for y in MIN_HEIGHT * 48 / 100..MIN_HEIGHT * 56 / 100 {
        for x in center_x - MIN_WIDTH / 22..=center_x + MIN_WIDTH / 22 {
            image.put_pixel(x, y, Rgb([188, 84, 34]));
        }
    }
    for y in MIN_HEIGHT * 50 / 100..MIN_HEIGHT * 62 / 100 {
        for x in MIN_WIDTH * 66 / 100..MIN_WIDTH * 70 / 100 {
            image.put_pixel(x, y, Rgb([246, 184, 48]));
        }
    }
    paint_distant_scene_signals(&mut image);

    let audit =
        audit_image("single_route_marker.png".to_string(), image).expect("audit should load");
    assert!(audit.passed, "{audit:?}");
    assert!(audit.route_marker_fraction >= MIN_SEQUENCE_ROUTE_MARKER_FRACTION);
    assert_eq!(audit.route_marker_component_count, 1);
    assert_eq!(audit.route_marker_hue_family_count, 1);

    let checks = report_checks(std::slice::from_ref(&audit));
    assert!(!report_passed(std::slice::from_ref(&audit), &checks));
    assert!(
        checks
            .iter()
            .any(|check| check.name == "max_route_marker_component_count" && !check.passed)
    );
    assert!(
        checks
            .iter()
            .any(|check| check.name == "max_route_marker_hue_family_count" && !check.passed)
    );
}

#[test]
fn json_string_escapes_control_characters() {
    let escaped = json_string("quote\" slash\\ newline\n carriage\r tab\t unit\u{1f}");

    assert_eq!(
        escaped,
        "\"quote\\\" slash\\\\ newline\\n carriage\\r tab\\t unit\\u001f\""
    );
}
