pub(super) fn is_sky_like(r: f64, g: f64, b: f64, luma: f64) -> bool {
    let blue_haze = b >= 105.0 && g >= 95.0 && b >= r + 8.0 && luma >= 80.0;
    let pale_cloud_haze = r >= 130.0 && g >= 140.0 && b >= 145.0 && b >= r - 4.0 && g >= r - 12.0;
    blue_haze || pale_cloud_haze
}

pub(super) fn is_scene_like(r: f64, g: f64, b: f64, luma: f64, sky_like: bool) -> bool {
    if luma <= 8.0 || luma >= 245.0 {
        return false;
    }

    let water = luma <= 170.0
        && r <= 115.0
        && g >= 45.0
        && b >= 40.0
        && r <= g + 25.0
        && (g >= r + 8.0 || b >= r + 8.0);
    if water {
        return true;
    }
    if sky_like {
        return false;
    }

    let foliage = g >= 60.0 && g >= r * 0.75 && g >= b * 0.65;
    let earth = r >= 55.0 && g >= 40.0 && r >= b + 10.0 && g >= b * 0.75;
    let rock_or_shadow = (18.0..=150.0).contains(&luma) && (r - g).abs() <= 45.0;

    foliage || earth || rock_or_shadow
}

pub(super) fn is_wind_effect_like(r: f64, g: f64, b: f64, luma: f64) -> bool {
    if !(55.0..=230.0).contains(&luma) {
        return false;
    }

    let saturated_cyan =
        g >= 92.0 && b >= 112.0 && r <= 135.0 && g >= r + 28.0 && b >= r + 34.0 && b + 46.0 >= g;
    let translucent_teal = g >= 78.0
        && b >= 68.0
        && r <= 120.0
        && g >= r + 24.0
        && b >= r + 12.0
        && (g - b).abs() <= 64.0;

    saturated_cyan || translucent_teal
}

pub(super) fn scene_material_family(
    r: f64,
    g: f64,
    b: f64,
    luma: f64,
    sky_like: bool,
) -> Option<usize> {
    if sky_like || luma <= 8.0 || luma >= 235.0 {
        return None;
    }

    let water = luma <= 170.0
        && r <= 115.0
        && g >= 45.0
        && b >= 40.0
        && r <= g + 25.0
        && (g >= r + 8.0 || b >= r + 8.0);
    if water {
        return Some(0);
    }

    let foliage = g >= 60.0 && g >= r * 0.75 && g >= b * 0.65;
    if foliage {
        return Some(1);
    }

    let earth = r >= 55.0 && g >= 40.0 && r >= b + 10.0 && g >= b * 0.75;
    if earth {
        return Some(2);
    }

    let rock_or_shadow = (18.0..=150.0).contains(&luma) && (r - g).abs() <= 45.0;
    rock_or_shadow.then_some(3)
}

pub(super) fn is_hud_region(x: usize, y: usize, width: usize, height: usize) -> bool {
    let left_panel = x < width * 36 / 100 && y < height * 88 / 100;
    let bottom_help = y >= height * 88 / 100;
    left_panel || bottom_help
}

pub(super) fn is_player_focus_region(x: usize, y: usize, width: usize, height: usize) -> bool {
    let center_x = width / 2;
    let x_radius = width * 18 / 100;
    let y_start = height * 36 / 100;
    let y_end = height * 82 / 100;
    x >= center_x.saturating_sub(x_radius) && x <= center_x + x_radius && y >= y_start && y <= y_end
}

pub(super) fn is_player_focus_like(r: f64, g: f64, b: f64, luma: f64) -> bool {
    let dark_body = (10.0..=85.0).contains(&luma) && r <= 95.0 && g <= 95.0 && b <= 105.0;
    is_player_warm_like(r, g, b) || dark_body
}

pub(super) fn is_player_warm_like(r: f64, g: f64, b: f64) -> bool {
    r >= 115.0 && (35.0..=125.0).contains(&g) && b <= 95.0 && r >= g + 35.0
}

pub(super) fn is_distant_scene_region(x: usize, y: usize, width: usize, height: usize) -> bool {
    let y_start = height * 16 / 100;
    let y_end = height * 64 / 100;
    let x_margin = width * 4 / 100;
    x >= x_margin && x < width.saturating_sub(x_margin) && y >= y_start && y < y_end
}

pub(super) fn is_cloud_layer_region(x: usize, y: usize, width: usize, height: usize) -> bool {
    let y_start = height * 4 / 100;
    let y_end = height * 55 / 100;
    let x_margin = width * 4 / 100;
    x >= x_margin && x < width.saturating_sub(x_margin) && y >= y_start && y < y_end
}

pub(super) fn is_cloud_layer_like(r: f64, g: f64, b: f64, luma: f64, sky_like: bool) -> bool {
    if !(72.0..=238.0).contains(&luma) {
        return false;
    }

    let max_channel = r.max(g).max(b);
    let min_channel = r.min(g).min(b);
    let saturation = max_channel - min_channel;
    let blue_sky = sky_like && b >= r + 22.0 && b >= g + 10.0 && saturation >= 40.0;
    if blue_sky {
        return false;
    }

    let pale_cloud = sky_like && luma >= 118.0 && saturation <= 72.0 && b <= r + 28.0;
    let gray_bank =
        saturation <= 44.0 && r >= 68.0 && g >= 68.0 && b >= 68.0 && b + 20.0 >= r && b + 20.0 >= g;
    let warm_haze_bank = saturation <= 54.0 && r >= 86.0 && g >= 78.0 && b >= 68.0 && r + 16.0 >= b;

    pale_cloud || gray_bank || warm_haze_bank
}

pub(super) fn is_distant_scene_like(r: f64, g: f64, b: f64, luma: f64, sky_like: bool) -> bool {
    if sky_like || !(12.0..=210.0).contains(&luma) {
        return false;
    }

    let water_like =
        r <= 115.0 && g >= 45.0 && b >= 40.0 && r <= g + 25.0 && (g >= r + 8.0 || b >= r + 8.0);
    if water_like {
        return false;
    }

    let foliage = g >= 58.0 && g >= r * 0.72 && g >= b * 0.58;
    let earth = r >= 50.0 && g >= 38.0 && r >= b + 8.0 && g >= b * 0.68;
    let rock_or_shadow =
        (18.0..=155.0).contains(&luma) && (r - g).abs() <= 50.0 && b <= r.max(g) + 20.0;

    foliage || earth || rock_or_shadow
}

pub(super) fn is_route_marker_like(r: f64, g: f64, b: f64, luma: f64) -> bool {
    if luma < 90.0 {
        return false;
    }

    let max_channel = r.max(g).max(b);
    let min_channel = r.min(g).min(b);
    let saturation = max_channel - min_channel;
    max_channel >= 190.0
        && saturation >= 90.0
        && (r >= g + 60.0 || g >= r + 60.0 || b >= g + 50.0 || b >= r + 50.0)
}

pub(super) fn route_marker_hue_family(r: f64, g: f64, b: f64) -> Option<usize> {
    if b >= r + 45.0 && b >= g + 18.0 {
        Some(0)
    } else if g >= r + 50.0 && g >= b + 8.0 {
        Some(1)
    } else if r >= 180.0 && g >= 130.0 && b <= 130.0 {
        Some(2)
    } else if r >= g + 45.0 && b >= g + 25.0 {
        Some(3)
    } else {
        None
    }
}

pub(super) fn is_clipping_occluder_like(
    r: f64,
    g: f64,
    b: f64,
    luma: f64,
    sky_like: bool,
    scene_like: bool,
) -> bool {
    if sky_like || is_route_marker_like(r, g, b, luma) || luma <= 8.0 || luma >= 230.0 {
        return false;
    }

    let max_channel = r.max(g).max(b);
    let min_channel = r.min(g).min(b);
    let dark_close_surface = luma <= 70.0 && max_channel - min_channel <= 70.0;
    let earth_or_rock = r >= 48.0 && g >= 36.0 && r >= b + 8.0 && g >= b * 0.7 && luma <= 180.0;
    let foliage = g >= 55.0 && g >= r * 0.75 && g >= b * 0.65 && luma <= 170.0;

    scene_like && (dark_close_surface || earth_or_rock || foliage)
}

pub(super) fn is_hud_text_like(r: f64, g: f64, b: f64) -> bool {
    let max_channel = r.max(g).max(b);
    let min_channel = r.min(g).min(b);
    max_channel >= 220.0 && max_channel - min_channel <= 24.0
}

pub(super) fn is_foreign_canvas_like(r: f64, g: f64, b: f64, luma: f64, sky_like: bool) -> bool {
    let max_channel = r.max(g).max(b);
    let min_channel = r.min(g).min(b);
    !sky_like && luma >= 210.0 && max_channel - min_channel <= 36.0
}
