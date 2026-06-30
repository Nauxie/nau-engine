#![recursion_limit = "256"]

use base64::{Engine as _, engine::general_purpose::STANDARD};
use bevy::prelude::{Mat4, Quat, Vec2, Vec3};
use nau_engine::animation::{
    CharacterPart, CharacterPartRole, MIN_KEY_POSE_READABILITY_SCORE, PartPose, PlayerPoseContext,
    PlayerPoseIntent, Side, glider_deployment_for_context, glider_deployment_for_mode,
    glider_traversal_pose, part_pose_with_context, pose_readability_metrics,
};
use nau_engine::asset_pipeline::{
    PLAYER_ANIMATION_CLIP_NAMES, VISUAL_ASSET_SPECS, VisualAssetKind, VisualAssetResidency,
    VisualAssetSpec,
};
use nau_engine::movement::{FlightInput, FlightMode};
use serde_json::{Value, json};
use std::{env, fmt::Write as _, fs, path::Path, process};

const NAU_FIXTURE_SCHEMA: &str = "nau_visual_asset_fixture.v1";
const NAU_FIXTURE_LICENSE: &str = "self_authored_no_third_party";
const PLAYER_POSE_MAX_CONNECTED_LIMB_TRANSLATION_M: f64 = 0.02;
const PLAYER_REST_MAX_ARTICULATED_JOINT_GAP_M: f64 = 0.015;
const PLAYER_REST_MAX_NON_ADJACENT_MESH_OVERLAP_M: f64 = 0.005;
const PLAYER_REST_MAX_SHOULDER_MESH_OVERLAP_M: f64 = 0.015;
const PLAYER_POSE_MAX_ARTICULATED_JOINT_GAP_M: f64 = 0.018;
const PLAYER_POSE_MAX_JOINT_COVER_MESH_GAP_M: f64 = 0.035;
const PLAYER_POSE_MAX_JOINT_COVER_MESH_OVERLAP_M: f64 = 0.078;
const PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_GAP_M: f64 = 0.012;
const PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_OVERLAP_M: f64 = 0.072;
const PLAYER_POSE_MAX_JOINT_SEAM_MESH_GAP_M: f64 = 0.008;
const PLAYER_POSE_MIN_JOINT_SEAM_MESH_OVERLAP_M: f64 = 0.004;
const PLAYER_POSE_MAX_JOINT_SEAM_MESH_OVERLAP_M: f64 = 0.120;
const PLAYER_POSE_MAX_PROXIMAL_CONTACT_MESH_OVERLAP_M: f64 = 0.160;
const PLAYER_POSE_MAX_SURFACE_CONTACT_DISTANCE_M: f64 = 0.065;
const PLAYER_POSE_MAX_PROJECTED_CONTACT_GAP_M: f64 = 0.018;
const PLAYER_POSE_MAX_NON_ADJACENT_MESH_OVERLAP_M: f64 = 0.001;
const PLAYER_POSE_CONTACT_EXPECTED_POSE_COUNT: f64 = 10.0;
const PLAYER_POSE_CONTACT_EXPECTED_PHASE_COUNT: f64 = 12.0;
const PLAYER_JOINT_BRIDGE_EXPECTED_NODE_COUNT: f64 = 12.0;
const PLAYER_JOINT_BRIDGE_EXPECTED_PAIR_COUNT: f64 = 12.0;
const PLAYER_JOINT_SEAM_EXPECTED_NODE_COUNT: f64 = 12.0;
const PLAYER_JOINT_SEAM_EXPECTED_PAIR_COUNT: f64 = 26.0;
const PLAYER_PROXIMAL_CONTACT_EXPECTED_PAIR_COUNT: f64 = 94.0;
const PLAYER_SURFACE_CONTACT_EXPECTED_PAIR_COUNT: f64 = 145.0;
const PLAYER_POSE_TRANSITION_EXPECTED_TRANSITION_COUNT: f64 = 9.0;
const PLAYER_POSE_TRANSITION_EXPECTED_BLEND_COUNT: f64 = 4.0;
const PLAYER_MOTION_INTEGRITY_REVIEW_EXPECTED_PANEL_COUNT: f64 = 15.0;
const PLAYER_MOTION_INTEGRITY_OVERLAY_MAX_WARNING_COUNT: f64 = 0.0;
const PLAYER_MESH_SILHOUETTE_EXPECTED_POSE_COUNT: f64 = 10.0;
const PLAYER_MESH_SILHOUETTE_EXPECTED_SAMPLE_COUNT: f64 = 40.0;
const PLAYER_MESH_SILHOUETTE_MIN_PROJECTED_SPAN_M: f64 = 0.70;
const PLAYER_MESH_SILHOUETTE_MIN_FALL_TOP_WIDTH_M: f64 = 2.40;
const PLAYER_MESH_SILHOUETTE_MIN_FALL_TOP_DEPTH_M: f64 = 1.20;
const PLAYER_MESH_SILHOUETTE_MIN_GLIDE_FRONT_WIDTH_M: f64 = 2.50;
const PLAYER_MESH_SILHOUETTE_MAX_DIVE_FRONT_TO_FALL_FRONT_WIDTH_RATIO: f64 = 0.65;
const PLAYER_MESH_SILHOUETTE_MIN_DIVE_FRONT_HEIGHT_M: f64 = 2.10;
const PLAYER_MESH_SILHOUETTE_MAX_DIVE_FRONT_WIDTH_TO_HEIGHT_RATIO: f64 = 0.72;
const PLAYER_MESH_SILHOUETTE_MAX_DIVE_SIDE_WIDTH_TO_HEIGHT_RATIO: f64 = 0.68;
const PLAYER_POSE_MIN_FALLING_TORSO_PITCH_DEGREES: f64 = 72.0;
const PLAYER_POSE_MIN_FALLING_ARM_SPREAD_DEGREES: f64 = 150.0;
const PLAYER_POSE_MIN_LAUNCH_OVERHEAD_ARM_SCORE: f64 = 0.60;
const PLAYER_POSE_MIN_DIVE_TORSO_PITCH_DEGREES: f64 = 82.0;
const PLAYER_POSE_MAX_DIVE_ARM_SPREAD_DEGREES: f64 = 74.0;
const PLAYER_POSE_MIN_DIVE_LEG_TUCK_DEGREES: f64 = 68.0;
const PLAYER_POSE_MIN_FALLING_HIPS_PITCH_DEGREES: f64 = 58.0;
const PLAYER_POSE_MIN_DIVE_HIPS_PITCH_DEGREES: f64 = 108.0;
const PLAYER_POSE_MAX_FALLING_TORSO_LOCAL_PITCH_DEGREES: f64 = 22.0;
const PLAYER_POSE_MAX_DIVE_TORSO_LOCAL_PITCH_DEGREES: f64 = 42.0;
const PLAYER_MIN_FINGER_GRIP_LENGTH_M: f64 = 0.10;
const PLAYER_MAX_FINGER_GRIP_LENGTH_M: f64 = 0.22;
const PLAYER_MIN_BOOT_SOLE_LENGTH_M: f64 = 0.32;
const PLAYER_LIMB_ANATOMY_EXPECTED_NODE_COUNT: f64 = 82.0;
const PLAYER_MIN_LIMB_ANATOMY_MAJOR_EXTENT_M: f64 = 0.15;
const PLAYER_GLIDER_MIN_LAUNCH_DEPLOYMENT: f64 = 0.45;
const PLAYER_GLIDER_MAX_LAUNCH_DEPLOYMENT: f64 = 0.70;
const PLAYER_GLIDER_MIN_LAUNCH_RESPONSE_DEGREES: f64 = 8.0;
const PLAYER_GLIDER_MIN_LAUNCH_MOTION_M: f64 = 0.18;
const PLAYER_GLIDER_MIN_DIVE_RESPONSE_DEGREES: f64 = 4.0;
const PLAYER_GLIDER_MIN_DIVE_MOTION_M: f64 = 0.16;
const PLAYER_GLIDER_MIN_DIVE_RELEASE_DEPLOYMENT: f64 = 0.28;
const PLAYER_GLIDER_MAX_DIVE_RELEASE_DEPLOYMENT: f64 = 0.58;
const PLAYER_GLIDER_HAND_GRIP_EXPECTED_SAMPLE_COUNT: f64 = 6.0;
const PLAYER_GLIDER_MAX_HAND_GRIP_DISTANCE_M: f64 = 0.28;
const PLAYER_GLIDER_MAX_HAND_GRIP_PROJECTED_GAP_M: f64 = 0.10;
const PLAYER_REST_TORSO_BLOCKING_MESH_NODES: &[&str] = &[
    "Nau Suit Armored Torso Shell",
    "Nau Suit Tapered Hips Shell",
    "Nau Suit Shoulder Yoke Plate",
    "Nau Left Suit Collarbone Plate",
    "Nau Right Suit Collarbone Plate",
    "Nau Suit Pelvis Hip Yoke",
    "Nau Left Suit Pelvis Side Plate",
    "Nau Right Suit Pelvis Side Plate",
    "Nau Skin Rounded Head",
];
const PLAYER_REST_LEFT_ARM_MESH_NODES: &[&str] = &[
    "Nau Left Suit Upper Arm",
    "Nau Left Suit Deltoid Filler",
    "Nau Left Leather Forearm Wrap",
    "Nau Left Leather Hand Palm",
    "Nau Left Leather Palm Heel Pad",
    "Nau Left Leather Index Finger Grip",
    "Nau Left Leather Finger Grip",
    "Nau Left Leather Ring Finger Grip",
    "Nau Left Leather Pinky Finger Grip",
    "Nau Left Leather Thumb Grip",
    "Nau Left Leather Index Knuckle Pad",
    "Nau Left Leather Middle Knuckle Pad",
    "Nau Left Leather Ring Knuckle Pad",
    "Nau Left Leather Pinky Knuckle Pad",
];
const PLAYER_REST_RIGHT_ARM_MESH_NODES: &[&str] = &[
    "Nau Right Suit Upper Arm",
    "Nau Right Suit Deltoid Filler",
    "Nau Right Leather Forearm Wrap",
    "Nau Right Leather Hand Palm",
    "Nau Right Leather Palm Heel Pad",
    "Nau Right Leather Index Finger Grip",
    "Nau Right Leather Finger Grip",
    "Nau Right Leather Ring Finger Grip",
    "Nau Right Leather Pinky Finger Grip",
    "Nau Right Leather Thumb Grip",
    "Nau Right Leather Index Knuckle Pad",
    "Nau Right Leather Middle Knuckle Pad",
    "Nau Right Leather Ring Knuckle Pad",
    "Nau Right Leather Pinky Knuckle Pad",
];
const PLAYER_REST_LEFT_DISTAL_ARM_MESH_NODES: &[&str] = &[
    "Nau Left Leather Forearm Wrap",
    "Nau Left Leather Hand Palm",
    "Nau Left Leather Palm Heel Pad",
    "Nau Left Leather Index Finger Grip",
    "Nau Left Leather Finger Grip",
    "Nau Left Leather Ring Finger Grip",
    "Nau Left Leather Pinky Finger Grip",
    "Nau Left Leather Thumb Grip",
    "Nau Left Leather Index Knuckle Pad",
    "Nau Left Leather Middle Knuckle Pad",
    "Nau Left Leather Ring Knuckle Pad",
    "Nau Left Leather Pinky Knuckle Pad",
];
const PLAYER_REST_RIGHT_DISTAL_ARM_MESH_NODES: &[&str] = &[
    "Nau Right Leather Forearm Wrap",
    "Nau Right Leather Hand Palm",
    "Nau Right Leather Palm Heel Pad",
    "Nau Right Leather Index Finger Grip",
    "Nau Right Leather Finger Grip",
    "Nau Right Leather Ring Finger Grip",
    "Nau Right Leather Pinky Finger Grip",
    "Nau Right Leather Thumb Grip",
    "Nau Right Leather Index Knuckle Pad",
    "Nau Right Leather Middle Knuckle Pad",
    "Nau Right Leather Ring Knuckle Pad",
    "Nau Right Leather Pinky Knuckle Pad",
];
const PLAYER_REST_LEFT_LEG_MESH_NODES: &[&str] = &[
    "Nau Left Suit Thigh Guard",
    "Nau Left Suit Lower Leg Greave",
    "Nau Left Leather Boot Shell",
    "Nau Left Leather Boot Toe Cap",
    "Nau Left Leather Outer Toe Lug",
    "Nau Left Leather Inner Toe Lug",
    "Nau Left Leather Boot Sole",
    "Nau Left Leather Boot Heel",
];
const PLAYER_REST_RIGHT_LEG_MESH_NODES: &[&str] = &[
    "Nau Right Suit Thigh Guard",
    "Nau Right Suit Lower Leg Greave",
    "Nau Right Leather Boot Shell",
    "Nau Right Leather Boot Toe Cap",
    "Nau Right Leather Outer Toe Lug",
    "Nau Right Leather Inner Toe Lug",
    "Nau Right Leather Boot Sole",
    "Nau Right Leather Boot Heel",
];
const PLAYER_REST_LEFT_DISTAL_LEG_MESH_NODES: &[&str] = &[
    "Nau Left Suit Lower Leg Greave",
    "Nau Left Leather Boot Shell",
    "Nau Left Leather Boot Toe Cap",
    "Nau Left Leather Outer Toe Lug",
    "Nau Left Leather Inner Toe Lug",
    "Nau Left Leather Boot Sole",
    "Nau Left Leather Boot Heel",
];
const PLAYER_REST_RIGHT_DISTAL_LEG_MESH_NODES: &[&str] = &[
    "Nau Right Suit Lower Leg Greave",
    "Nau Right Leather Boot Shell",
    "Nau Right Leather Boot Toe Cap",
    "Nau Right Leather Outer Toe Lug",
    "Nau Right Leather Inner Toe Lug",
    "Nau Right Leather Boot Sole",
    "Nau Right Leather Boot Heel",
];
const PLAYER_RUNTIME_POSE_NODE_ROLES: &[(&str, CharacterPartRole)] = &[
    ("Nau Hips", CharacterPartRole::Hips),
    ("Nau Torso", CharacterPartRole::Torso),
    ("Nau Head", CharacterPartRole::Head),
    ("Nau Left Arm", CharacterPartRole::Arm(Side::Left)),
    ("Nau Right Arm", CharacterPartRole::Arm(Side::Right)),
    ("Nau Left Forearm", CharacterPartRole::Forearm(Side::Left)),
    ("Nau Right Forearm", CharacterPartRole::Forearm(Side::Right)),
    (
        "Nau Left Leather Hand Palm",
        CharacterPartRole::Hand(Side::Left),
    ),
    (
        "Nau Right Leather Hand Palm",
        CharacterPartRole::Hand(Side::Right),
    ),
    ("Nau Left Leg", CharacterPartRole::Leg(Side::Left)),
    ("Nau Right Leg", CharacterPartRole::Leg(Side::Right)),
    (
        "Nau Left Lower Leg",
        CharacterPartRole::LowerLeg(Side::Left),
    ),
    (
        "Nau Right Lower Leg",
        CharacterPartRole::LowerLeg(Side::Right),
    ),
    ("Nau Left Boot", CharacterPartRole::Foot(Side::Left)),
    ("Nau Right Boot", CharacterPartRole::Foot(Side::Right)),
];

#[derive(Clone, Copy)]
struct Requirement {
    kind: VisualAssetKind,
    min_nodes: u64,
    min_meshes: u64,
    min_materials: u64,
    min_vertices: u64,
    min_triangles: u64,
    required_name_fragments: &'static [&'static str],
    require_blend_material: bool,
    require_player_clips: bool,
}

const PLAYER_NAME_FRAGMENTS: &[&str] = &[
    "suit",
    "skin",
    "accent",
    "helmet",
    "shoulder",
    "scarf",
    "boot",
    "face",
    "eye",
    "belt",
    "gauntlet",
    "knee",
    "hand",
    "finger",
    "toe",
    "neck",
    "elbow",
    "ankle",
    "bridge",
    "sleeve",
    "yoke",
    "collarbone",
    "pelvis",
    "deltoid",
    "bicep",
    "tricep",
    "knuckle",
    "web",
    "tendon",
    "thigh",
    "calf",
    "shin",
    "instep",
    "lace",
    "sole",
    "heel",
    "seamless",
    "flex",
];
const GLIDER_NAME_FRAGMENTS: &[&str] = &["cloth panel", "spar", "rib", "tether", "grip"];
const TERRAIN_NAME_FRAGMENTS: &[&str] = &[
    "relief",
    "cliff",
    "underside",
    "landing",
    "terrace",
    "erosion",
    "path",
];
const FOLIAGE_NAME_FRAGMENTS: &[&str] = &[
    "trunk",
    "branch",
    "canopy",
    "grass",
    "detail card",
    "wildflower",
    "root",
    "fern",
    "moss",
];
const ROCK_NAME_FRAGMENTS: &[&str] = &[
    "boulder", "stone", "strata", "fracture", "quartz", "rust", "shale",
];
const WATER_NAME_FRAGMENTS: &[&str] = &[
    "pond", "rim", "ripple", "reed", "depth", "glint", "lily", "specular",
];
const ROUTE_MARKER_NAME_FRAGMENTS: &[&str] = &[
    "gate", "mast", "shard", "cairn", "pennant", "glyph", "pebble",
];
const WEATHER_NAME_FRAGMENTS: &[&str] = &[
    "cloud bank",
    "shadow belly",
    "cirrus",
    "wisp",
    "haze",
    "filament",
];
const IMPOSTOR_NAME_FRAGMENTS: &[&str] = &[
    "terrain",
    "underside",
    "rim",
    "tree silhouette",
    "waterfall",
    "broken",
];

const REQUIREMENTS: &[Requirement] = &[
    Requirement {
        kind: VisualAssetKind::PlayerCharacter,
        min_nodes: 190,
        min_meshes: 62,
        min_materials: 8,
        min_vertices: 15000,
        min_triangles: 28000,
        required_name_fragments: PLAYER_NAME_FRAGMENTS,
        require_blend_material: false,
        require_player_clips: true,
    },
    Requirement {
        kind: VisualAssetKind::Glider,
        min_nodes: 13,
        min_meshes: 12,
        min_materials: 5,
        min_vertices: 100,
        min_triangles: 120,
        required_name_fragments: GLIDER_NAME_FRAGMENTS,
        require_blend_material: false,
        require_player_clips: false,
    },
    Requirement {
        kind: VisualAssetKind::IslandTerrain,
        min_nodes: 11,
        min_meshes: 10,
        min_materials: 7,
        min_vertices: 400,
        min_triangles: 520,
        required_name_fragments: TERRAIN_NAME_FRAGMENTS,
        require_blend_material: false,
        require_player_clips: false,
    },
    Requirement {
        kind: VisualAssetKind::IslandFoliage,
        min_nodes: 18,
        min_meshes: 17,
        min_materials: 7,
        min_vertices: 280,
        min_triangles: 390,
        required_name_fragments: FOLIAGE_NAME_FRAGMENTS,
        require_blend_material: false,
        require_player_clips: false,
    },
    Requirement {
        kind: VisualAssetKind::IslandRock,
        min_nodes: 9,
        min_meshes: 9,
        min_materials: 6,
        min_vertices: 370,
        min_triangles: 530,
        required_name_fragments: ROCK_NAME_FRAGMENTS,
        require_blend_material: false,
        require_player_clips: false,
    },
    Requirement {
        kind: VisualAssetKind::IslandWater,
        min_nodes: 11,
        min_meshes: 10,
        min_materials: 7,
        min_vertices: 230,
        min_triangles: 220,
        required_name_fragments: WATER_NAME_FRAGMENTS,
        require_blend_material: true,
        require_player_clips: false,
    },
    Requirement {
        kind: VisualAssetKind::RouteMarker,
        min_nodes: 10,
        min_meshes: 10,
        min_materials: 6,
        min_vertices: 480,
        min_triangles: 870,
        required_name_fragments: ROUTE_MARKER_NAME_FRAGMENTS,
        require_blend_material: false,
        require_player_clips: false,
    },
    Requirement {
        kind: VisualAssetKind::WeatherLayer,
        min_nodes: 13,
        min_meshes: 13,
        min_materials: 5,
        min_vertices: 390,
        min_triangles: 560,
        required_name_fragments: WEATHER_NAME_FRAGMENTS,
        require_blend_material: true,
        require_player_clips: false,
    },
    Requirement {
        kind: VisualAssetKind::DistantImpostor,
        min_nodes: 9,
        min_meshes: 9,
        min_materials: 5,
        min_vertices: 220,
        min_triangles: 230,
        required_name_fragments: IMPOSTOR_NAME_FRAGMENTS,
        require_blend_material: true,
        require_player_clips: false,
    },
];

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if let Some(command) = args.first() {
        if command == "--export-player-pose-preview" {
            let output_dir = args
                .get(1)
                .map(Path::new)
                .unwrap_or_else(|| Path::new("target/player_pose_previews"));
            match export_player_pose_preview(output_dir) {
                Ok(report) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report)
                            .expect("preview report should serialize")
                    );
                }
                Err(error) => {
                    eprintln!("player pose preview export failed: {error}");
                    process::exit(2);
                }
            }
            return;
        }

        eprintln!("unknown asset fixture audit command: {command}");
        process::exit(2);
    }

    match audit_all_fixtures() {
        Ok(report) => {
            let passed = report
                .get("passed")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            println!(
                "{}",
                serde_json::to_string_pretty(&report).expect("audit report should serialize")
            );
            if !passed {
                process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("asset fixture audit failed: {error}");
            process::exit(2);
        }
    }
}

#[derive(Clone, Copy)]
struct PlayerPosePreviewSpec {
    label: &'static str,
    title: &'static str,
    context: PlayerPoseContext,
    phase: f32,
}

#[derive(Clone, Copy)]
struct PlayerTransitionPosePreviewSpec {
    label: &'static str,
    title: &'static str,
    transition: PlayerPoseTransition,
    phase: f32,
    blend: f32,
}

#[derive(Clone, Copy)]
struct PlayerAnatomyReviewSpec {
    label: &'static str,
    title: &'static str,
    context: PlayerPoseContext,
    phase: f32,
    view: PlayerPosePreviewView,
    nodes: &'static [&'static str],
}

#[derive(Clone, Copy)]
struct GliderPosePreviewSpec {
    label: &'static str,
    title: &'static str,
    context: PlayerPoseContext,
    phase: f32,
    deployment: f32,
}

#[derive(Clone, Debug)]
struct PlayerPosePreviewShape {
    node_name: String,
    vertices: Vec<Vec3>,
    surface_points: Vec<Vec3>,
    bounds: Aabb3,
    obb: Obb3,
    color: &'static str,
}

#[derive(Clone, Copy, Debug)]
struct PlayerSurfaceContactPair {
    category: &'static str,
    left: &'static str,
    right: &'static str,
}

#[derive(Clone, Copy, Debug)]
struct ClosestSurfacePoints {
    distance_m: f64,
    left: Vec3,
    right: Vec3,
}

#[derive(Clone, Debug)]
struct PlayerMeshSilhouetteSample {
    label: &'static str,
    title: &'static str,
    pose_intent: &'static str,
    view: PlayerPosePreviewView,
    width_m: f64,
    height_m: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlayerPosePreviewView {
    Front,
    Rear,
    Side,
    Top,
}

const PLAYER_POSE_PREVIEW_VIEWS: [PlayerPosePreviewView; 4] = [
    PlayerPosePreviewView::Front,
    PlayerPosePreviewView::Rear,
    PlayerPosePreviewView::Side,
    PlayerPosePreviewView::Top,
];

#[derive(Clone, Copy)]
struct PosePreviewPanel {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Clone, Copy)]
struct GliderAttachmentPreviewRow<'a> {
    player_shapes: &'a [PlayerPosePreviewShape],
    glider_shapes: &'a [PlayerPosePreviewShape],
    combined_shapes: &'a [PlayerPosePreviewShape],
    requires_hand_grip: bool,
}

fn export_player_pose_preview(output_dir: &Path) -> Result<Value, String> {
    let path = Path::new("assets/models/player/player.gltf");
    let text =
        fs::read_to_string(path).map_err(|error| format!("failed to read {path:?}: {error}"))?;
    let gltf = serde_json::from_str::<Value>(&text)
        .map_err(|error| format!("invalid glTF JSON: {error}"))?;
    let glider_path = Path::new("assets/models/player/glider.gltf");
    let glider_text = fs::read_to_string(glider_path)
        .map_err(|error| format!("failed to read {glider_path:?}: {error}"))?;
    let glider_gltf = serde_json::from_str::<Value>(&glider_text)
        .map_err(|error| format!("invalid glider glTF JSON: {error}"))?;
    let specs = player_pose_preview_specs();
    let svg = render_player_pose_preview_sheet(&gltf, &specs)?;
    let anatomy_specs = player_anatomy_review_specs();
    let anatomy_svg = render_player_anatomy_review_sheet(&gltf, &anatomy_specs)?;
    let stress_specs = player_rig_stress_review_specs();
    let stress_svg = render_player_rig_stress_review_sheet(&gltf, &stress_specs)?;
    let motion_specs = player_motion_integrity_review_specs();
    let motion_svg = render_player_motion_integrity_review_sheet(&gltf, &motion_specs)?;
    let motion_overlay_warning_audit = player_motion_integrity_overlay_warning_audit(&gltf);
    let transition_specs = player_transition_pose_preview_specs();
    let transition_svg = render_player_transition_pose_preview_sheet(&gltf, &transition_specs)?;
    let glider_specs = glider_pose_preview_specs();
    let glider_svg = render_glider_pose_preview_sheet(&glider_gltf, &glider_specs)?;
    let attachment_specs = player_glider_attachment_preview_specs();
    let attachment_svg =
        render_player_glider_attachment_preview_sheet(&gltf, &glider_gltf, &attachment_specs)?;
    fs::create_dir_all(output_dir)
        .map_err(|error| format!("failed to create {output_dir:?}: {error}"))?;
    let sheet_path = output_dir.join("player_pose_sheet.svg");
    fs::write(&sheet_path, svg)
        .map_err(|error| format!("failed to write {sheet_path:?}: {error}"))?;
    let anatomy_sheet_path = output_dir.join("player_anatomy_review_sheet.svg");
    fs::write(&anatomy_sheet_path, anatomy_svg)
        .map_err(|error| format!("failed to write {anatomy_sheet_path:?}: {error}"))?;
    let stress_sheet_path = output_dir.join("player_rig_stress_review_sheet.svg");
    fs::write(&stress_sheet_path, stress_svg)
        .map_err(|error| format!("failed to write {stress_sheet_path:?}: {error}"))?;
    let motion_sheet_path = output_dir.join("player_motion_integrity_review_sheet.svg");
    fs::write(&motion_sheet_path, motion_svg)
        .map_err(|error| format!("failed to write {motion_sheet_path:?}: {error}"))?;
    let transition_sheet_path = output_dir.join("player_transition_pose_sheet.svg");
    fs::write(&transition_sheet_path, transition_svg)
        .map_err(|error| format!("failed to write {transition_sheet_path:?}: {error}"))?;
    let glider_sheet_path = output_dir.join("glider_pose_sheet.svg");
    fs::write(&glider_sheet_path, glider_svg)
        .map_err(|error| format!("failed to write {glider_sheet_path:?}: {error}"))?;
    let attachment_sheet_path = output_dir.join("player_glider_attachment_sheet.svg");
    fs::write(&attachment_sheet_path, attachment_svg)
        .map_err(|error| format!("failed to write {attachment_sheet_path:?}: {error}"))?;

    let manifest = json!({
        "schema": "nau_player_pose_preview.v1",
        "renderer": "mesh_projected_convex_hull_surface_contact_overlay.v1",
        "source": path,
        "glider_source": glider_path,
        "source_bytes": text.len(),
        "source_hash_fnv1a64": fnv1a64_hex(text.as_bytes()),
        "glider_source_bytes": glider_text.len(),
        "glider_source_hash_fnv1a64": fnv1a64_hex(glider_text.as_bytes()),
        "pose_count": specs.len(),
        "anatomy_review_panel_count": anatomy_specs.len(),
        "stress_review_panel_count": stress_specs.len(),
        "motion_review_panel_count": motion_specs.len(),
        "transition_pose_count": transition_specs.len(),
        "glider_pose_count": glider_specs.len(),
        "attachment_pose_count": attachment_specs.len(),
        "views": PLAYER_POSE_PREVIEW_VIEWS
            .iter()
            .map(|view| view.key())
            .collect::<Vec<_>>(),
        "phase_samples": specs.iter().map(|spec| spec.phase).collect::<Vec<_>>(),
        "poses": specs.iter().map(|spec| json!({
            "label": spec.label,
            "title": spec.title,
            "phase": spec.phase,
            "pose_intent": spec.context.intent().label(),
        })).collect::<Vec<_>>(),
        "anatomy_review_panels": anatomy_specs.iter().map(|spec| json!({
            "label": spec.label,
            "title": spec.title,
            "phase": spec.phase,
            "pose_intent": spec.context.intent().label(),
            "view": spec.view.key(),
            "node_count": spec.nodes.len(),
        })).collect::<Vec<_>>(),
        "stress_review_panels": stress_specs.iter().map(|spec| json!({
            "label": spec.label,
            "title": spec.title,
            "phase": spec.phase,
            "pose_intent": spec.context.intent().label(),
            "view": spec.view.key(),
            "node_count": spec.nodes.len(),
        })).collect::<Vec<_>>(),
        "motion_review_panels": motion_specs.iter().map(|spec| json!({
            "label": spec.label,
            "title": spec.title,
            "phase": spec.phase,
            "pose_intent": spec.context.intent().label(),
            "view": spec.view.key(),
            "node_count": spec.nodes.len(),
        })).collect::<Vec<_>>(),
        "transition_poses": transition_specs.iter().map(|spec| json!({
            "label": spec.label,
            "title": spec.title,
            "transition": spec.transition.label,
            "from_pose_intent": spec.transition.from.intent().label(),
            "to_pose_intent": spec.transition.to.intent().label(),
            "phase": spec.phase,
            "blend": spec.blend,
        })).collect::<Vec<_>>(),
        "glider_poses": glider_specs.iter().map(|spec| json!({
            "label": spec.label,
            "title": spec.title,
            "phase": spec.phase,
            "deployment": spec.deployment,
            "pose_intent": spec.context.intent().label(),
            "response_degrees": glider_traversal_pose(spec.context, spec.phase).response_degrees(),
            "motion_m": glider_traversal_pose(spec.context, spec.phase).motion_m(),
        })).collect::<Vec<_>>(),
        "attachment_poses": attachment_specs.iter().map(|spec| json!({
            "label": spec.label,
            "title": spec.title,
            "phase": spec.phase,
            "deployment": spec.deployment,
            "pose_intent": spec.context.intent().label(),
        })).collect::<Vec<_>>(),
        "joint_seam_contact_audit": player_joint_seam_contact_audit(&gltf),
        "player_limb_anatomy_detail_audit": player_limb_anatomy_detail_audit(&gltf),
        "player_mesh_silhouette_audit": player_mesh_silhouette_audit(&gltf),
        "motion_integrity_overlay_warning_audit": motion_overlay_warning_audit,
        "surface_contact_audit": player_pose_surface_contact_audit(&gltf),
        "player_glider_attachment_audit": player_glider_attachment_audit(&gltf, &glider_gltf),
        "artifacts": {
            "pose_sheet_svg": sheet_path,
            "pose_sheet_png": output_dir.join("player_pose_sheet.png"),
            "anatomy_review_sheet_svg": anatomy_sheet_path,
            "anatomy_review_sheet_png": output_dir.join("player_anatomy_review_sheet.png"),
            "rig_stress_review_sheet_svg": stress_sheet_path,
            "rig_stress_review_sheet_png": output_dir.join("player_rig_stress_review_sheet.png"),
            "motion_integrity_review_sheet_svg": motion_sheet_path,
            "motion_integrity_review_sheet_png": output_dir.join("player_motion_integrity_review_sheet.png"),
            "transition_pose_sheet_svg": transition_sheet_path,
            "transition_pose_sheet_png": output_dir.join("player_transition_pose_sheet.png"),
            "glider_pose_sheet_svg": glider_sheet_path,
            "glider_pose_sheet_png": output_dir.join("glider_pose_sheet.png"),
            "player_glider_attachment_sheet_svg": attachment_sheet_path,
            "player_glider_attachment_sheet_png": output_dir.join("player_glider_attachment_sheet.png"),
        },
    });
    let manifest_path = output_dir.join("manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).expect("preview manifest should serialize"),
    )
    .map_err(|error| format!("failed to write {manifest_path:?}: {error}"))?;

    Ok(json!({
        "passed": true,
        "manifest": manifest_path,
        "pose_sheet_svg": sheet_path,
        "pose_sheet_png": output_dir.join("player_pose_sheet.png"),
        "anatomy_review_sheet_svg": anatomy_sheet_path,
        "anatomy_review_sheet_png": output_dir.join("player_anatomy_review_sheet.png"),
        "rig_stress_review_sheet_svg": stress_sheet_path,
        "rig_stress_review_sheet_png": output_dir.join("player_rig_stress_review_sheet.png"),
        "motion_integrity_review_sheet_svg": motion_sheet_path,
        "motion_integrity_review_sheet_png": output_dir.join("player_motion_integrity_review_sheet.png"),
        "transition_pose_sheet_svg": transition_sheet_path,
        "transition_pose_sheet_png": output_dir.join("player_transition_pose_sheet.png"),
        "glider_pose_sheet_svg": glider_sheet_path,
        "glider_pose_sheet_png": output_dir.join("glider_pose_sheet.png"),
        "player_glider_attachment_sheet_svg": attachment_sheet_path,
        "player_glider_attachment_sheet_png": output_dir.join("player_glider_attachment_sheet.png"),
        "pose_count": specs.len(),
        "anatomy_review_panel_count": anatomy_specs.len(),
        "stress_review_panel_count": stress_specs.len(),
        "motion_review_panel_count": motion_specs.len(),
        "transition_pose_count": transition_specs.len(),
        "glider_pose_count": glider_specs.len(),
        "attachment_pose_count": attachment_specs.len(),
    }))
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn player_pose_preview_specs() -> Vec<PlayerPosePreviewSpec> {
    vec![
        PlayerPosePreviewSpec {
            label: "grounded_idle",
            title: "Grounded Idle",
            context: PlayerPoseContext::new(
                FlightMode::Grounded,
                Vec3::ZERO,
                FlightInput::default(),
                0.0,
            )
            .with_resolved_intent(PlayerPoseIntent::GroundedIdle),
            phase: 0.75,
        },
        PlayerPosePreviewSpec {
            label: "grounded_walk",
            title: "Grounded Walk",
            context: PlayerPoseContext::new(
                FlightMode::Grounded,
                Vec3::new(0.0, 0.0, -4.5),
                FlightInput::default(),
                0.0,
            )
            .with_resolved_intent(PlayerPoseIntent::GroundedWalk),
            phase: 0.75,
        },
        PlayerPosePreviewSpec {
            label: "grounded_run",
            title: "Grounded Run",
            context: PlayerPoseContext::new(
                FlightMode::Grounded,
                Vec3::new(0.0, 0.0, -10.0),
                FlightInput::default(),
                0.0,
            )
            .with_resolved_intent(PlayerPoseIntent::GroundedRun),
            phase: 1.5,
        },
        PlayerPosePreviewSpec {
            label: "launch_takeout",
            title: "Launch Takeout",
            context: PlayerPoseContext::new(
                FlightMode::Launching,
                Vec3::new(0.0, 24.0, -18.0),
                FlightInput::default(),
                80.0,
            )
            .with_resolved_intent(PlayerPoseIntent::Launching),
            phase: 0.75,
        },
        PlayerPosePreviewSpec {
            label: "falling_belly_down",
            title: "Belly-Down Fall",
            context: PlayerPoseContext::new(
                FlightMode::Airborne,
                Vec3::new(0.0, -22.0, -24.0),
                FlightInput::default(),
                80.0,
            )
            .with_resolved_intent(PlayerPoseIntent::Falling),
            phase: 0.75,
        },
        PlayerPosePreviewSpec {
            label: "gliding",
            title: "Glide",
            context: PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -4.0, -30.0),
                FlightInput {
                    glide: true,
                    ..FlightInput::default()
                },
                80.0,
            )
            .with_resolved_intent(PlayerPoseIntent::Gliding),
            phase: 0.75,
        },
        PlayerPosePreviewSpec {
            label: "diving_head_down",
            title: "Head-Down Dive",
            context: PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -34.0, -36.0),
                FlightInput {
                    glide: true,
                    dive: true,
                    ..FlightInput::default()
                },
                120.0,
            )
            .with_resolved_intent(PlayerPoseIntent::Diving),
            phase: 0.75,
        },
        PlayerPosePreviewSpec {
            label: "air_brake",
            title: "Air Brake",
            context: PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -8.0, -24.0),
                FlightInput {
                    glide: true,
                    backward: true,
                    ..FlightInput::default()
                },
                80.0,
            )
            .with_resolved_intent(PlayerPoseIntent::AirBrake),
            phase: 0.75,
        },
        PlayerPosePreviewSpec {
            label: "landing_anticipation",
            title: "Landing Anticipation",
            context: PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(3.0, -20.0, -22.0),
                FlightInput {
                    glide: true,
                    ..FlightInput::default()
                },
                1.5,
            )
            .with_resolved_intent(PlayerPoseIntent::LandingAnticipation),
            phase: 0.75,
        },
        PlayerPosePreviewSpec {
            label: "landing_recovery",
            title: "Landing Recovery",
            context: PlayerPoseContext::new(
                FlightMode::Grounded,
                Vec3::new(2.0, 0.0, -7.0),
                FlightInput::default(),
                0.0,
            )
            .with_resolved_intent(PlayerPoseIntent::LandingRecovery),
            phase: 0.75,
        },
    ]
}

fn player_transition_pose_preview_specs() -> Vec<PlayerTransitionPosePreviewSpec> {
    player_pose_transition_contact_transitions()
        .into_iter()
        .flat_map(|transition| {
            player_pose_transition_contact_blends().map(move |blend| {
                PlayerTransitionPosePreviewSpec {
                    label: transition.label,
                    title: transition.label,
                    transition,
                    phase: 0.75,
                    blend,
                }
            })
        })
        .collect()
}

fn glider_pose_preview_specs() -> Vec<GliderPosePreviewSpec> {
    let launch_context = PlayerPoseContext::new(
        FlightMode::Launching,
        Vec3::new(0.0, 24.0, -18.0),
        FlightInput::default(),
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::Launching);
    let glide_context = PlayerPoseContext::new(
        FlightMode::Gliding,
        Vec3::new(0.0, -4.0, -38.0),
        FlightInput {
            glide: true,
            ..FlightInput::default()
        },
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::Gliding);
    let dive_context = PlayerPoseContext::new(
        FlightMode::Gliding,
        Vec3::new(0.0, -34.0, -44.0),
        FlightInput {
            glide: true,
            dive: true,
            ..FlightInput::default()
        },
        120.0,
    )
    .with_resolved_intent(PlayerPoseIntent::Diving);
    let air_brake_context = PlayerPoseContext::new(
        FlightMode::Gliding,
        Vec3::new(0.0, -8.0, -24.0),
        FlightInput {
            glide: true,
            backward: true,
            ..FlightInput::default()
        },
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::AirBrake);

    vec![
        GliderPosePreviewSpec {
            label: "stowed",
            title: "Stowed",
            context: PlayerPoseContext::new(
                FlightMode::Grounded,
                Vec3::ZERO,
                FlightInput::default(),
                0.0,
            )
            .with_resolved_intent(PlayerPoseIntent::GroundedIdle),
            phase: 0.0,
            deployment: 0.0,
        },
        GliderPosePreviewSpec {
            label: "takeout_early",
            title: "Takeout 35%",
            context: launch_context,
            phase: 0.35,
            deployment: 0.35,
        },
        GliderPosePreviewSpec {
            label: "takeout_ready",
            title: "Launch Takeout",
            context: launch_context,
            phase: 0.75,
            deployment: glider_deployment_for_mode(FlightMode::Launching),
        },
        GliderPosePreviewSpec {
            label: "glide_open",
            title: "Glide Open",
            context: glide_context,
            phase: 0.75,
            deployment: glider_deployment_for_mode(FlightMode::Gliding),
        },
        GliderPosePreviewSpec {
            label: "dive_release",
            title: "Dive Release",
            context: dive_context,
            phase: 0.75,
            deployment: glider_deployment_for_context(dive_context),
        },
        GliderPosePreviewSpec {
            label: "air_brake_cupped",
            title: "Air Brake Cup",
            context: air_brake_context,
            phase: 0.75,
            deployment: glider_deployment_for_mode(FlightMode::Gliding),
        },
    ]
}

fn player_glider_attachment_preview_specs() -> Vec<GliderPosePreviewSpec> {
    glider_pose_preview_specs()
        .into_iter()
        .filter(|spec| spec.deployment > 0.0)
        .collect()
}

fn player_glider_attachment_audit_specs() -> Vec<GliderPosePreviewSpec> {
    glider_pose_preview_specs()
        .into_iter()
        .filter(|spec| glider_pose_requires_hand_grip(spec.label))
        .collect()
}

fn glider_pose_requires_hand_grip(label: &str) -> bool {
    matches!(label, "takeout_ready" | "glide_open" | "air_brake_cupped")
}

const PLAYER_ANATOMY_FULL_BODY_REVIEW_NODES: &[&str] = &[];
const PLAYER_ANATOMY_CORE_REVIEW_NODES: &[&str] = &[
    "Nau Skin Rounded Head",
    "Nau Skin Neck Column",
    "Nau Neck Joint Cover",
    "Nau Suit Neck Collar Pad",
    "Nau Suit Shoulder Yoke Plate",
    "Nau Suit Armored Torso Shell",
    "Nau Suit Ribcage Soft Volume",
    "Nau Left Suit Pectoral Soft Volume",
    "Nau Right Suit Pectoral Soft Volume",
    "Nau Left Suit Scapula Soft Volume",
    "Nau Right Suit Scapula Soft Volume",
    "Nau Suit Lower Rib Flex Lip",
    "Nau Suit Abdominal Flex Gasket",
    "Nau Suit Waist Soft Volume",
    "Nau Suit Tapered Hips Shell",
    "Nau Suit Pelvis Hip Yoke",
    "Nau Left Suit Oblique Flex Connector",
    "Nau Right Suit Oblique Flex Connector",
];
const PLAYER_ANATOMY_SHOULDER_REVIEW_NODES: &[&str] = &[
    "Nau Suit Armored Torso Shell",
    "Nau Suit Ribcage Soft Volume",
    "Nau Suit Shoulder Yoke Plate",
    "Nau Left Suit Collarbone Plate",
    "Nau Right Suit Collarbone Plate",
    "Nau Left Suit Shoulder Chest Blend",
    "Nau Right Suit Shoulder Chest Blend",
    "Nau Left Suit Pectoral Soft Volume",
    "Nau Right Suit Pectoral Soft Volume",
    "Nau Left Suit Scapula Soft Volume",
    "Nau Right Suit Scapula Soft Volume",
    "Nau Left Suit Axilla Blend",
    "Nau Right Suit Axilla Blend",
    "Nau Left Suit Shoulder Web Capsule",
    "Nau Right Suit Shoulder Web Capsule",
    "Nau Left Suit Lat Shoulder Connector",
    "Nau Right Suit Lat Shoulder Connector",
    "Nau Left Shoulder Joint Cover",
    "Nau Right Shoulder Joint Cover",
    "Nau Left Suit Shoulder Root Blend",
    "Nau Right Suit Shoulder Root Blend",
    "Nau Left Shoulder Bridge Sleeve",
    "Nau Right Shoulder Bridge Sleeve",
    "Nau Left Seamless Shoulder Flex Cover",
    "Nau Right Seamless Shoulder Flex Cover",
    "Nau Left Suit Upper Arm",
    "Nau Right Suit Upper Arm",
    "Nau Left Suit Deltoid Filler",
    "Nau Right Suit Deltoid Filler",
];
const PLAYER_ANATOMY_HIP_REVIEW_NODES: &[&str] = &[
    "Nau Suit Tapered Hips Shell",
    "Nau Suit Waist Soft Volume",
    "Nau Suit Pelvis Hip Yoke",
    "Nau Left Suit Pelvis Side Plate",
    "Nau Right Suit Pelvis Side Plate",
    "Nau Left Suit Hip Inguinal Blend",
    "Nau Right Suit Hip Inguinal Blend",
    "Nau Left Suit Hip Web Capsule",
    "Nau Right Suit Hip Web Capsule",
    "Nau Left Suit Glute Hip Connector",
    "Nau Right Suit Glute Hip Connector",
    "Nau Left Suit Hip Thigh Fairing",
    "Nau Right Suit Hip Thigh Fairing",
    "Nau Left Hip Joint Cover",
    "Nau Right Hip Joint Cover",
    "Nau Left Hip Bridge Sleeve",
    "Nau Right Hip Bridge Sleeve",
    "Nau Left Seamless Hip Flex Cover",
    "Nau Right Seamless Hip Flex Cover",
    "Nau Left Suit Hip Root Blend",
    "Nau Right Suit Hip Root Blend",
    "Nau Left Suit Thigh Guard",
    "Nau Right Suit Thigh Guard",
];
const PLAYER_ANATOMY_HAND_REVIEW_NODES: &[&str] = &[
    "Nau Left Wrist Bridge Sleeve",
    "Nau Left Seamless Wrist Flex Cover",
    "Nau Left Leather Wrist Palm Gusset",
    "Nau Left Leather Hand Palm",
    "Nau Left Leather Outer Palm Edge Pad",
    "Nau Left Leather Inner Palm Edge Pad",
    "Nau Left Leather Thumb Web Pad",
    "Nau Left Leather Index Finger Grip",
    "Nau Left Leather Finger Grip",
    "Nau Left Leather Ring Finger Grip",
    "Nau Left Leather Pinky Finger Grip",
    "Nau Left Leather Thumb Grip",
    "Nau Left Leather Index Finger Tip Pad",
    "Nau Left Leather Middle Finger Tip Pad",
    "Nau Left Leather Ring Finger Tip Pad",
    "Nau Left Leather Pinky Finger Tip Pad",
    "Nau Left Leather Thumb Tip Pad",
    "Nau Left Leather Palm Heel Pad",
    "Nau Left Leather Finger Web Bridge",
    "Nau Left Leather Index Knuckle Pad",
    "Nau Left Leather Middle Knuckle Pad",
    "Nau Left Leather Ring Knuckle Pad",
    "Nau Left Leather Pinky Knuckle Pad",
];
const PLAYER_ANATOMY_BOOT_REVIEW_NODES: &[&str] = &[
    "Nau Left Leather Boot Shell",
    "Nau Left Ankle Bridge Sleeve",
    "Nau Left Seamless Ankle Flex Cover",
    "Nau Left Leather Ankle Wrap",
    "Nau Left Leather Heel Tendon Guard",
    "Nau Left Leather Boot Instep Plate",
    "Nau Left Leather Outer Boot Side Guard",
    "Nau Left Leather Inner Boot Side Guard",
    "Nau Left Leather Boot Arch Rib",
    "Nau Left Leather Ankle Boot Tongue",
    "Nau Left Leather Lace Cross Strap A",
    "Nau Left Leather Lace Cross Strap B",
    "Nau Left Leather Boot Toe Cap",
    "Nau Left Leather Outer Toe Lug",
    "Nau Left Leather Inner Toe Lug",
    "Nau Left Leather Boot Sole",
    "Nau Left Leather Boot Heel",
];
const PLAYER_ANATOMY_RIGHT_HAND_REVIEW_NODES: &[&str] = &[
    "Nau Right Wrist Bridge Sleeve",
    "Nau Right Seamless Wrist Flex Cover",
    "Nau Right Leather Wrist Palm Gusset",
    "Nau Right Leather Hand Palm",
    "Nau Right Leather Outer Palm Edge Pad",
    "Nau Right Leather Inner Palm Edge Pad",
    "Nau Right Leather Thumb Web Pad",
    "Nau Right Leather Index Finger Grip",
    "Nau Right Leather Finger Grip",
    "Nau Right Leather Ring Finger Grip",
    "Nau Right Leather Pinky Finger Grip",
    "Nau Right Leather Thumb Grip",
    "Nau Right Leather Index Finger Tip Pad",
    "Nau Right Leather Middle Finger Tip Pad",
    "Nau Right Leather Ring Finger Tip Pad",
    "Nau Right Leather Pinky Finger Tip Pad",
    "Nau Right Leather Thumb Tip Pad",
    "Nau Right Leather Palm Heel Pad",
    "Nau Right Leather Finger Web Bridge",
    "Nau Right Leather Index Knuckle Pad",
    "Nau Right Leather Middle Knuckle Pad",
    "Nau Right Leather Ring Knuckle Pad",
    "Nau Right Leather Pinky Knuckle Pad",
];
const PLAYER_ANATOMY_RIGHT_BOOT_REVIEW_NODES: &[&str] = &[
    "Nau Right Leather Boot Shell",
    "Nau Right Ankle Bridge Sleeve",
    "Nau Right Seamless Ankle Flex Cover",
    "Nau Right Leather Ankle Wrap",
    "Nau Right Leather Heel Tendon Guard",
    "Nau Right Leather Boot Instep Plate",
    "Nau Right Leather Outer Boot Side Guard",
    "Nau Right Leather Inner Boot Side Guard",
    "Nau Right Leather Boot Arch Rib",
    "Nau Right Leather Ankle Boot Tongue",
    "Nau Right Leather Lace Cross Strap A",
    "Nau Right Leather Lace Cross Strap B",
    "Nau Right Leather Boot Toe Cap",
    "Nau Right Leather Outer Toe Lug",
    "Nau Right Leather Inner Toe Lug",
    "Nau Right Leather Boot Sole",
    "Nau Right Leather Boot Heel",
];

fn player_anatomy_review_specs() -> Vec<PlayerAnatomyReviewSpec> {
    let grounded_context = PlayerPoseContext::new(
        FlightMode::Grounded,
        Vec3::ZERO,
        FlightInput::default(),
        0.0,
    )
    .with_resolved_intent(PlayerPoseIntent::GroundedIdle);
    let fall_context = PlayerPoseContext::new(
        FlightMode::Airborne,
        Vec3::new(0.0, -22.0, -24.0),
        FlightInput::default(),
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::Falling);
    let dive_context = PlayerPoseContext::new(
        FlightMode::Gliding,
        Vec3::new(0.0, -34.0, -36.0),
        FlightInput {
            glide: true,
            dive: true,
            ..FlightInput::default()
        },
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::Diving);

    vec![
        PlayerAnatomyReviewSpec {
            label: "full_front",
            title: "Full Body Front",
            context: grounded_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Front,
            nodes: PLAYER_ANATOMY_FULL_BODY_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "full_side",
            title: "Full Body Side",
            context: grounded_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Side,
            nodes: PLAYER_ANATOMY_FULL_BODY_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "core_front",
            title: "Core Connectors Front",
            context: grounded_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Front,
            nodes: PLAYER_ANATOMY_CORE_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "core_side",
            title: "Core Connectors Side",
            context: grounded_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Side,
            nodes: PLAYER_ANATOMY_CORE_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "fall_top",
            title: "Belly-Down Footprint",
            context: fall_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Top,
            nodes: PLAYER_ANATOMY_FULL_BODY_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "dive_side",
            title: "Dive Side Stack",
            context: dive_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Side,
            nodes: PLAYER_ANATOMY_FULL_BODY_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "shoulder_front",
            title: "Shoulders Front",
            context: grounded_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Front,
            nodes: PLAYER_ANATOMY_SHOULDER_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "shoulder_side",
            title: "Shoulders Side",
            context: grounded_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Side,
            nodes: PLAYER_ANATOMY_SHOULDER_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "hip_front",
            title: "Hips Front",
            context: grounded_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Front,
            nodes: PLAYER_ANATOMY_HIP_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "hip_side",
            title: "Hips Side",
            context: grounded_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Side,
            nodes: PLAYER_ANATOMY_HIP_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "hand_front",
            title: "Left Hand Front",
            context: grounded_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Front,
            nodes: PLAYER_ANATOMY_HAND_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "hand_top",
            title: "Left Hand Top",
            context: grounded_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Top,
            nodes: PLAYER_ANATOMY_HAND_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "right_hand_front",
            title: "Right Hand Front",
            context: grounded_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Front,
            nodes: PLAYER_ANATOMY_RIGHT_HAND_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "right_hand_top",
            title: "Right Hand Top",
            context: grounded_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Top,
            nodes: PLAYER_ANATOMY_RIGHT_HAND_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "boot_side",
            title: "Left Boot Side",
            context: grounded_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Side,
            nodes: PLAYER_ANATOMY_BOOT_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "boot_top",
            title: "Left Boot Top",
            context: grounded_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Top,
            nodes: PLAYER_ANATOMY_BOOT_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "right_boot_side",
            title: "Right Boot Side",
            context: grounded_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Side,
            nodes: PLAYER_ANATOMY_RIGHT_BOOT_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "right_boot_top",
            title: "Right Boot Top",
            context: grounded_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Top,
            nodes: PLAYER_ANATOMY_RIGHT_BOOT_REVIEW_NODES,
        },
    ]
}

fn player_rig_stress_review_specs() -> Vec<PlayerAnatomyReviewSpec> {
    let launch_context = PlayerPoseContext::new(
        FlightMode::Launching,
        Vec3::new(0.0, 24.0, -18.0),
        FlightInput::default(),
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::Launching);
    let fall_context = PlayerPoseContext::new(
        FlightMode::Airborne,
        Vec3::new(0.0, -22.0, -24.0),
        FlightInput::default(),
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::Falling);
    let dive_context = PlayerPoseContext::new(
        FlightMode::Gliding,
        Vec3::new(0.0, -34.0, -36.0),
        FlightInput {
            glide: true,
            dive: true,
            ..FlightInput::default()
        },
        120.0,
    )
    .with_resolved_intent(PlayerPoseIntent::Diving);
    let landing_context = PlayerPoseContext::new(
        FlightMode::Gliding,
        Vec3::new(3.0, -20.0, -22.0),
        FlightInput {
            glide: true,
            ..FlightInput::default()
        },
        1.5,
    )
    .with_resolved_intent(PlayerPoseIntent::LandingAnticipation);

    vec![
        PlayerAnatomyReviewSpec {
            label: "launch_shoulders_front",
            title: "Launch Shoulders Front",
            context: launch_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Front,
            nodes: PLAYER_ANATOMY_SHOULDER_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "launch_left_hand_front",
            title: "Launch Left Hand Front",
            context: launch_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Front,
            nodes: PLAYER_ANATOMY_HAND_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "launch_right_hand_front",
            title: "Launch Right Hand Front",
            context: launch_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Front,
            nodes: PLAYER_ANATOMY_RIGHT_HAND_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "fall_core_top",
            title: "Fall Core Top",
            context: fall_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Top,
            nodes: PLAYER_ANATOMY_CORE_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "fall_shoulders_top",
            title: "Fall Shoulders Top",
            context: fall_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Top,
            nodes: PLAYER_ANATOMY_SHOULDER_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "fall_hips_top",
            title: "Fall Hips Top",
            context: fall_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Top,
            nodes: PLAYER_ANATOMY_HIP_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "dive_core_side",
            title: "Dive Core Side",
            context: dive_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Side,
            nodes: PLAYER_ANATOMY_CORE_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "dive_shoulders_side",
            title: "Dive Shoulders Side",
            context: dive_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Side,
            nodes: PLAYER_ANATOMY_SHOULDER_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "dive_hips_side",
            title: "Dive Hips Side",
            context: dive_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Side,
            nodes: PLAYER_ANATOMY_HIP_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "landing_core_front",
            title: "Landing Core Front",
            context: landing_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Front,
            nodes: PLAYER_ANATOMY_CORE_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "landing_core_side",
            title: "Landing Core Side",
            context: landing_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Side,
            nodes: PLAYER_ANATOMY_CORE_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "landing_hips_front",
            title: "Landing Hips Front",
            context: landing_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Front,
            nodes: PLAYER_ANATOMY_HIP_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "landing_hips_side",
            title: "Landing Hips Side",
            context: landing_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Side,
            nodes: PLAYER_ANATOMY_HIP_REVIEW_NODES,
        },
    ]
}

fn player_motion_integrity_review_specs() -> Vec<PlayerAnatomyReviewSpec> {
    let launch_context = PlayerPoseContext::new(
        FlightMode::Launching,
        Vec3::new(0.0, 24.0, -18.0),
        FlightInput::default(),
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::Launching);
    let fall_context = PlayerPoseContext::new(
        FlightMode::Airborne,
        Vec3::new(0.0, -22.0, -24.0),
        FlightInput::default(),
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::Falling);
    let glide_context = PlayerPoseContext::new(
        FlightMode::Gliding,
        Vec3::new(0.0, -4.0, -38.0),
        FlightInput::default(),
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::Gliding);
    let dive_context = PlayerPoseContext::new(
        FlightMode::Gliding,
        Vec3::new(0.0, -34.0, -36.0),
        FlightInput {
            glide: true,
            dive: true,
            ..FlightInput::default()
        },
        120.0,
    )
    .with_resolved_intent(PlayerPoseIntent::Diving);
    let air_brake_context = PlayerPoseContext::new(
        FlightMode::Gliding,
        Vec3::new(0.0, -5.0, 16.0),
        FlightInput {
            backward: true,
            ..FlightInput::default()
        },
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::AirBrake);
    let landing_context = PlayerPoseContext::new(
        FlightMode::Gliding,
        Vec3::new(3.0, -20.0, -22.0),
        FlightInput {
            glide: true,
            ..FlightInput::default()
        },
        1.5,
    )
    .with_resolved_intent(PlayerPoseIntent::LandingAnticipation);
    let landing_recovery_context = PlayerPoseContext::new(
        FlightMode::Grounded,
        Vec3::new(0.0, 0.0, -8.0),
        FlightInput::default(),
        0.0,
    )
    .with_landing_recovery(0.36, 16.0)
    .with_resolved_intent(PlayerPoseIntent::LandingRecovery);

    vec![
        PlayerAnatomyReviewSpec {
            label: "launch_front_full",
            title: "Launch Front Full",
            context: launch_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Front,
            nodes: PLAYER_ANATOMY_FULL_BODY_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "launch_side_full",
            title: "Launch Side Full",
            context: launch_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Side,
            nodes: PLAYER_ANATOMY_FULL_BODY_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "launch_top_full",
            title: "Launch Top Full",
            context: launch_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Top,
            nodes: PLAYER_ANATOMY_FULL_BODY_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "fall_front_full",
            title: "Fall Front Full",
            context: fall_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Front,
            nodes: PLAYER_ANATOMY_FULL_BODY_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "fall_side_full",
            title: "Fall Side Full",
            context: fall_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Side,
            nodes: PLAYER_ANATOMY_FULL_BODY_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "fall_top_full",
            title: "Fall Top Full",
            context: fall_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Top,
            nodes: PLAYER_ANATOMY_FULL_BODY_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "glide_rear_full",
            title: "Glide Rear Full",
            context: glide_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Rear,
            nodes: PLAYER_ANATOMY_FULL_BODY_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "glide_side_full",
            title: "Glide Side Full",
            context: glide_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Side,
            nodes: PLAYER_ANATOMY_FULL_BODY_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "glide_top_full",
            title: "Glide Top Full",
            context: glide_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Top,
            nodes: PLAYER_ANATOMY_FULL_BODY_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "dive_front_full",
            title: "Dive Front Full",
            context: dive_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Front,
            nodes: PLAYER_ANATOMY_FULL_BODY_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "dive_side_full",
            title: "Dive Side Full",
            context: dive_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Side,
            nodes: PLAYER_ANATOMY_FULL_BODY_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "dive_top_full",
            title: "Dive Top Full",
            context: dive_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Top,
            nodes: PLAYER_ANATOMY_FULL_BODY_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "air_brake_front_full",
            title: "Air Brake Front Full",
            context: air_brake_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Front,
            nodes: PLAYER_ANATOMY_FULL_BODY_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "landing_side_full",
            title: "Landing Side Full",
            context: landing_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Side,
            nodes: PLAYER_ANATOMY_FULL_BODY_REVIEW_NODES,
        },
        PlayerAnatomyReviewSpec {
            label: "landing_recovery_front_full",
            title: "Landing Recovery Front Full",
            context: landing_recovery_context,
            phase: 0.75,
            view: PlayerPosePreviewView::Front,
            nodes: PLAYER_ANATOMY_FULL_BODY_REVIEW_NODES,
        },
    ]
}

fn render_player_anatomy_review_sheet(
    gltf: &Value,
    specs: &[PlayerAnatomyReviewSpec],
) -> Result<String, String> {
    const COLUMNS: usize = 4;
    const PANEL_WIDTH: f32 = 282.0;
    const PANEL_HEIGHT: f32 = 262.0;
    const HEADER_HEIGHT: f32 = 56.0;
    const PADDING: f32 = 18.0;
    const LABEL_HEIGHT: f32 = 38.0;

    let rows = specs.len().div_ceil(COLUMNS);
    let width = PANEL_WIDTH * COLUMNS as f32 + PADDING * 2.0;
    let height = HEADER_HEIGHT + (PANEL_HEIGHT + LABEL_HEIGHT) * rows as f32 + PADDING;
    let mut svg = String::new();
    writeln!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width:.0}\" height=\"{height:.0}\" viewBox=\"0 0 {width:.0} {height:.0}\">"
    )
    .expect("writing to string should not fail");
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#10151f\"/>\n");
    svg.push_str("<text x=\"18\" y=\"24\" fill=\"#dbe7f3\" font-family=\"Menlo, monospace\" font-size=\"16\">NAU player anatomy review sheet</text>\n");
    svg.push_str("<text x=\"18\" y=\"42\" fill=\"#7f95a7\" font-family=\"Menlo, monospace\" font-size=\"10\">large panels for human review of proportions, connectors, hands, and boots</text>\n");

    for (index, spec) in specs.iter().enumerate() {
        let column = index % COLUMNS;
        let row = index / COLUMNS;
        let x = PADDING + PANEL_WIDTH * column as f32;
        let y = HEADER_HEIGHT + (PANEL_HEIGHT + LABEL_HEIGHT) * row as f32;
        let overrides =
            player_pose_node_overrides(gltf, spec.context, spec.phase).ok_or_else(|| {
                format!(
                    "failed to compute anatomy pose overrides for {}",
                    spec.label
                )
            })?;
        let shapes = player_pose_preview_shapes(gltf, &overrides).ok_or_else(|| {
            format!(
                "failed to compute anatomy preview shapes for {}",
                spec.label
            )
        })?;
        let review_shapes = filter_player_pose_preview_shapes(&shapes, spec.nodes);
        if review_shapes.is_empty() {
            return Err(format!("anatomy review panel {} has no shapes", spec.label));
        }

        writeln!(
            svg,
            "<text x=\"{x:.1}\" y=\"{:.1}\" fill=\"#edf5ff\" font-family=\"Menlo, monospace\" font-size=\"13\">{}</text>",
            y + 16.0,
            escape_xml(spec.title)
        )
        .expect("writing to string should not fail");
        writeln!(
            svg,
            "<text x=\"{x:.1}\" y=\"{:.1}\" fill=\"#8fb1c9\" font-family=\"Menlo, monospace\" font-size=\"10\">{} / {} nodes</text>",
            y + 32.0,
            spec.view.key(),
            review_shapes.len()
        )
        .expect("writing to string should not fail");
        render_player_pose_preview_view(
            &mut svg,
            &review_shapes,
            spec.view,
            x,
            y + LABEL_HEIGHT,
            PANEL_WIDTH - 12.0,
            PANEL_HEIGHT - 12.0,
        );
    }

    svg.push_str("</svg>\n");
    Ok(svg)
}

fn render_player_rig_stress_review_sheet(
    gltf: &Value,
    specs: &[PlayerAnatomyReviewSpec],
) -> Result<String, String> {
    const COLUMNS: usize = 4;
    const PANEL_WIDTH: f32 = 282.0;
    const PANEL_HEIGHT: f32 = 262.0;
    const HEADER_HEIGHT: f32 = 56.0;
    const PADDING: f32 = 18.0;
    const LABEL_HEIGHT: f32 = 38.0;

    let rows = specs.len().div_ceil(COLUMNS);
    let width = PANEL_WIDTH * COLUMNS as f32 + PADDING * 2.0;
    let height = HEADER_HEIGHT + (PANEL_HEIGHT + LABEL_HEIGHT) * rows as f32 + PADDING;
    let mut svg = String::new();
    writeln!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width:.0}\" height=\"{height:.0}\" viewBox=\"0 0 {width:.0} {height:.0}\">"
    )
    .expect("writing to string should not fail");
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#10151f\"/>\n");
    svg.push_str("<text x=\"18\" y=\"24\" fill=\"#dbe7f3\" font-family=\"Menlo, monospace\" font-size=\"16\">NAU player rig stress review sheet</text>\n");
    svg.push_str("<text x=\"18\" y=\"42\" fill=\"#7f95a7\" font-family=\"Menlo, monospace\" font-size=\"10\">connector closeups in launch, fall, dive, and landing poses</text>\n");

    for (index, spec) in specs.iter().enumerate() {
        let column = index % COLUMNS;
        let row = index / COLUMNS;
        let x = PADDING + PANEL_WIDTH * column as f32;
        let y = HEADER_HEIGHT + (PANEL_HEIGHT + LABEL_HEIGHT) * row as f32;
        let overrides = player_pose_node_overrides(gltf, spec.context, spec.phase)
            .ok_or_else(|| format!("failed to compute stress pose overrides for {}", spec.label))?;
        let shapes = player_pose_preview_shapes(gltf, &overrides)
            .ok_or_else(|| format!("failed to compute stress preview shapes for {}", spec.label))?;
        let review_shapes = filter_player_pose_preview_shapes(&shapes, spec.nodes);
        if review_shapes.is_empty() {
            return Err(format!("stress review panel {} has no shapes", spec.label));
        }

        writeln!(
            svg,
            "<text x=\"{x:.1}\" y=\"{:.1}\" fill=\"#edf5ff\" font-family=\"Menlo, monospace\" font-size=\"13\">{}</text>",
            y + 16.0,
            escape_xml(spec.title)
        )
        .expect("writing to string should not fail");
        writeln!(
            svg,
            "<text x=\"{x:.1}\" y=\"{:.1}\" fill=\"#8fb1c9\" font-family=\"Menlo, monospace\" font-size=\"10\">{} / {} nodes</text>",
            y + 32.0,
            spec.view.key(),
            review_shapes.len()
        )
        .expect("writing to string should not fail");
        render_player_pose_preview_view(
            &mut svg,
            &review_shapes,
            spec.view,
            x,
            y + LABEL_HEIGHT,
            PANEL_WIDTH - 12.0,
            PANEL_HEIGHT - 12.0,
        );
    }

    svg.push_str("</svg>\n");
    Ok(svg)
}

fn render_player_motion_integrity_review_sheet(
    gltf: &Value,
    specs: &[PlayerAnatomyReviewSpec],
) -> Result<String, String> {
    const COLUMNS: usize = 3;
    const PANEL_WIDTH: f32 = 366.0;
    const PANEL_HEIGHT: f32 = 318.0;
    const HEADER_HEIGHT: f32 = 58.0;
    const PADDING: f32 = 18.0;
    const LABEL_HEIGHT: f32 = 38.0;

    let rows = specs.len().div_ceil(COLUMNS);
    let width = PANEL_WIDTH * COLUMNS as f32 + PADDING * 2.0;
    let height = HEADER_HEIGHT + (PANEL_HEIGHT + LABEL_HEIGHT) * rows as f32 + PADDING;
    let mut svg = String::new();
    writeln!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width:.0}\" height=\"{height:.0}\" viewBox=\"0 0 {width:.0} {height:.0}\">"
    )
    .expect("writing to string should not fail");
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#10151f\"/>\n");
    svg.push_str("<text x=\"18\" y=\"24\" fill=\"#dbe7f3\" font-family=\"Menlo, monospace\" font-size=\"16\">NAU player motion integrity review sheet</text>\n");
    svg.push_str("<text x=\"18\" y=\"42\" fill=\"#7f95a7\" font-family=\"Menlo, monospace\" font-size=\"10\">large full-body panels for human review of limb wiring across traversal poses</text>\n");

    for (index, spec) in specs.iter().enumerate() {
        let column = index % COLUMNS;
        let row = index / COLUMNS;
        let x = PADDING + PANEL_WIDTH * column as f32;
        let y = HEADER_HEIGHT + (PANEL_HEIGHT + LABEL_HEIGHT) * row as f32;
        let overrides = player_pose_node_overrides(gltf, spec.context, spec.phase)
            .ok_or_else(|| format!("failed to compute motion pose overrides for {}", spec.label))?;
        let shapes = player_pose_preview_shapes(gltf, &overrides)
            .ok_or_else(|| format!("failed to compute motion preview shapes for {}", spec.label))?;
        let review_shapes = filter_player_pose_preview_shapes(&shapes, spec.nodes);
        if review_shapes.is_empty() {
            return Err(format!("motion review panel {} has no shapes", spec.label));
        }

        writeln!(
            svg,
            "<text x=\"{x:.1}\" y=\"{:.1}\" fill=\"#edf5ff\" font-family=\"Menlo, monospace\" font-size=\"13\">{}</text>",
            y + 16.0,
            escape_xml(spec.title)
        )
        .expect("writing to string should not fail");
        writeln!(
            svg,
            "<text x=\"{x:.1}\" y=\"{:.1}\" fill=\"#8fb1c9\" font-family=\"Menlo, monospace\" font-size=\"10\">{} / {} nodes</text>",
            y + 32.0,
            spec.view.key(),
            review_shapes.len()
        )
        .expect("writing to string should not fail");
        render_player_pose_preview_view(
            &mut svg,
            &review_shapes,
            spec.view,
            x,
            y + LABEL_HEIGHT,
            PANEL_WIDTH - 12.0,
            PANEL_HEIGHT - 12.0,
        );
    }

    svg.push_str("</svg>\n");
    Ok(svg)
}

fn filter_player_pose_preview_shapes(
    shapes: &[PlayerPosePreviewShape],
    node_names: &[&str],
) -> Vec<PlayerPosePreviewShape> {
    if node_names.is_empty() {
        return shapes.to_vec();
    }

    shapes
        .iter()
        .filter(|shape| {
            node_names
                .iter()
                .any(|node_name| shape.node_name == *node_name)
        })
        .cloned()
        .collect()
}

fn render_player_pose_preview_sheet(
    gltf: &Value,
    specs: &[PlayerPosePreviewSpec],
) -> Result<String, String> {
    const ROW_HEIGHT: f32 = 210.0;
    const LABEL_WIDTH: f32 = 180.0;
    const VIEW_WIDTH: f32 = 250.0;
    const HEADER_HEIGHT: f32 = 58.0;
    const PADDING: f32 = 18.0;

    let width = LABEL_WIDTH + VIEW_WIDTH * PLAYER_POSE_PREVIEW_VIEWS.len() as f32 + PADDING * 2.0;
    let height = HEADER_HEIGHT + ROW_HEIGHT * specs.len() as f32 + PADDING;
    let mut svg = String::new();
    writeln!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width:.0}\" height=\"{height:.0}\" viewBox=\"0 0 {width:.0} {height:.0}\">"
    )
    .expect("writing to string should not fail");
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#10151f\"/>\n");
    svg.push_str("<text x=\"18\" y=\"24\" fill=\"#dbe7f3\" font-family=\"Menlo, monospace\" font-size=\"16\">NAU player fixture pose preview</text>\n");
    for (index, view) in PLAYER_POSE_PREVIEW_VIEWS.iter().enumerate() {
        let x = LABEL_WIDTH + PADDING + VIEW_WIDTH * index as f32 + VIEW_WIDTH * 0.5;
        writeln!(
            svg,
            "<text x=\"{x:.1}\" y=\"44\" fill=\"#8fb1c9\" text-anchor=\"middle\" font-family=\"Menlo, monospace\" font-size=\"12\">{}</text>",
            view.label()
        )
        .expect("writing to string should not fail");
    }

    for (row, spec) in specs.iter().enumerate() {
        let y = HEADER_HEIGHT + ROW_HEIGHT * row as f32;
        let overrides = player_pose_node_overrides(gltf, spec.context, spec.phase)
            .ok_or_else(|| format!("failed to compute pose overrides for {}", spec.label))?;
        let shapes = player_pose_preview_shapes(gltf, &overrides)
            .ok_or_else(|| format!("failed to compute preview shapes for {}", spec.label))?;
        writeln!(
            svg,
            "<text x=\"18\" y=\"{:.1}\" fill=\"#edf5ff\" font-family=\"Menlo, monospace\" font-size=\"13\">{}</text>",
            y + 28.0,
            escape_xml(spec.title)
        )
        .expect("writing to string should not fail");
        writeln!(
            svg,
            "<text x=\"18\" y=\"{:.1}\" fill=\"#7f95a7\" font-family=\"Menlo, monospace\" font-size=\"10\">intent: {}</text>",
            y + 46.0,
            spec.context.intent().label()
        )
        .expect("writing to string should not fail");

        for (column, view) in PLAYER_POSE_PREVIEW_VIEWS.iter().enumerate() {
            let x = LABEL_WIDTH + PADDING + VIEW_WIDTH * column as f32;
            render_player_pose_preview_view(
                &mut svg,
                &shapes,
                *view,
                x,
                y + 12.0,
                VIEW_WIDTH - 12.0,
                ROW_HEIGHT - 20.0,
            );
        }
    }

    svg.push_str("</svg>\n");
    Ok(svg)
}

fn render_player_transition_pose_preview_sheet(
    gltf: &Value,
    specs: &[PlayerTransitionPosePreviewSpec],
) -> Result<String, String> {
    const ROW_HEIGHT: f32 = 210.0;
    const LABEL_WIDTH: f32 = 235.0;
    const VIEW_WIDTH: f32 = 250.0;
    const HEADER_HEIGHT: f32 = 58.0;
    const PADDING: f32 = 18.0;

    let width = LABEL_WIDTH + VIEW_WIDTH * PLAYER_POSE_PREVIEW_VIEWS.len() as f32 + PADDING * 2.0;
    let height = HEADER_HEIGHT + ROW_HEIGHT * specs.len() as f32 + PADDING;
    let mut svg = String::new();
    writeln!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width:.0}\" height=\"{height:.0}\" viewBox=\"0 0 {width:.0} {height:.0}\">"
    )
    .expect("writing to string should not fail");
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#10151f\"/>\n");
    svg.push_str("<text x=\"18\" y=\"24\" fill=\"#dbe7f3\" font-family=\"Menlo, monospace\" font-size=\"16\">NAU player fixture transition pose preview</text>\n");
    svg.push_str("<desc>surface distance plus mesh overlap overlays for high-risk blends</desc>\n");
    for (index, view) in PLAYER_POSE_PREVIEW_VIEWS.iter().enumerate() {
        let x = LABEL_WIDTH + PADDING + VIEW_WIDTH * index as f32 + VIEW_WIDTH * 0.5;
        writeln!(
            svg,
            "<text x=\"{x:.1}\" y=\"44\" fill=\"#8fb1c9\" text-anchor=\"middle\" font-family=\"Menlo, monospace\" font-size=\"12\">{}</text>",
            view.label()
        )
        .expect("writing to string should not fail");
    }

    for (row, spec) in specs.iter().enumerate() {
        let y = HEADER_HEIGHT + ROW_HEIGHT * row as f32;
        let overrides = player_pose_transition_node_overrides(
            gltf,
            spec.transition.from,
            spec.transition.to,
            spec.phase,
            spec.blend,
        )
        .ok_or_else(|| format!("failed to compute transition overrides for {}", spec.label))?;
        let shapes = player_pose_preview_shapes(gltf, &overrides).ok_or_else(|| {
            format!(
                "failed to compute transition preview shapes for {}",
                spec.label
            )
        })?;
        writeln!(
            svg,
            "<text x=\"18\" y=\"{:.1}\" fill=\"#edf5ff\" font-family=\"Menlo, monospace\" font-size=\"12\">{}</text>",
            y + 28.0,
            escape_xml(spec.title)
        )
        .expect("writing to string should not fail");
        writeln!(
            svg,
            "<text x=\"18\" y=\"{:.1}\" fill=\"#7f95a7\" font-family=\"Menlo, monospace\" font-size=\"10\">{} -> {}</text>",
            y + 46.0,
            spec.transition.from.intent().label(),
            spec.transition.to.intent().label()
        )
        .expect("writing to string should not fail");
        writeln!(
            svg,
            "<text x=\"18\" y=\"{:.1}\" fill=\"#7f95a7\" font-family=\"Menlo, monospace\" font-size=\"10\">blend: {:.0}% phase: {:.2}</text>",
            y + 62.0,
            spec.blend * 100.0,
            spec.phase
        )
        .expect("writing to string should not fail");

        for (column, view) in PLAYER_POSE_PREVIEW_VIEWS.iter().enumerate() {
            let x = LABEL_WIDTH + PADDING + VIEW_WIDTH * column as f32;
            render_player_pose_preview_view(
                &mut svg,
                &shapes,
                *view,
                x,
                y + 12.0,
                VIEW_WIDTH - 12.0,
                ROW_HEIGHT - 20.0,
            );
        }
    }

    svg.push_str("</svg>\n");
    Ok(svg)
}

fn render_glider_pose_preview_sheet(
    gltf: &Value,
    specs: &[GliderPosePreviewSpec],
) -> Result<String, String> {
    const ROW_HEIGHT: f32 = 170.0;
    const LABEL_WIDTH: f32 = 180.0;
    const VIEW_WIDTH: f32 = 250.0;
    const HEADER_HEIGHT: f32 = 58.0;
    const PADDING: f32 = 18.0;

    let width = LABEL_WIDTH + VIEW_WIDTH * PLAYER_POSE_PREVIEW_VIEWS.len() as f32 + PADDING * 2.0;
    let height = HEADER_HEIGHT + ROW_HEIGHT * specs.len() as f32 + PADDING;
    let mut svg = String::new();
    writeln!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width:.0}\" height=\"{height:.0}\" viewBox=\"0 0 {width:.0} {height:.0}\">"
    )
    .expect("writing to string should not fail");
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#10151f\"/>\n");
    svg.push_str("<text x=\"18\" y=\"24\" fill=\"#dbe7f3\" font-family=\"Menlo, monospace\" font-size=\"16\">NAU glider deployment pose preview</text>\n");
    let views = PLAYER_POSE_PREVIEW_VIEWS;
    for (index, view) in views.iter().enumerate() {
        let x = LABEL_WIDTH + PADDING + VIEW_WIDTH * index as f32 + VIEW_WIDTH * 0.5;
        writeln!(
            svg,
            "<text x=\"{x:.1}\" y=\"44\" fill=\"#8fb1c9\" text-anchor=\"middle\" font-family=\"Menlo, monospace\" font-size=\"12\">{}</text>",
            view.label()
        )
        .expect("writing to string should not fail");
    }

    let shape_rows = specs
        .iter()
        .map(|spec| {
            glider_pose_preview_shapes(gltf, *spec).ok_or_else(|| {
                format!("failed to compute glider preview shapes for {}", spec.label)
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let view_extents = views.map(|view| preview_shape_rows_projected_extent(&shape_rows, view));

    for (row, (spec, shapes)) in specs.iter().zip(shape_rows.iter()).enumerate() {
        let y = HEADER_HEIGHT + ROW_HEIGHT * row as f32;
        writeln!(
            svg,
            "<text x=\"18\" y=\"{:.1}\" fill=\"#edf5ff\" font-family=\"Menlo, monospace\" font-size=\"13\">{}</text>",
            y + 28.0,
            escape_xml(spec.title)
        )
        .expect("writing to string should not fail");
        writeln!(
            svg,
            "<text x=\"18\" y=\"{:.1}\" fill=\"#7f95a7\" font-family=\"Menlo, monospace\" font-size=\"10\">deploy: {:.0}%</text>",
            y + 46.0,
            spec.deployment * 100.0
        )
        .expect("writing to string should not fail");

        for (column, view) in views.iter().enumerate() {
            let x = LABEL_WIDTH + PADDING + VIEW_WIDTH * column as f32;
            render_glider_pose_preview_view(
                &mut svg,
                shapes,
                *view,
                view_extents[column],
                PosePreviewPanel {
                    x,
                    y: y + 12.0,
                    width: VIEW_WIDTH - 12.0,
                    height: ROW_HEIGHT - 20.0,
                },
            );
        }
    }

    svg.push_str("</svg>\n");
    Ok(svg)
}

fn render_player_glider_attachment_preview_sheet(
    player_gltf: &Value,
    glider_gltf: &Value,
    specs: &[GliderPosePreviewSpec],
) -> Result<String, String> {
    const ROW_HEIGHT: f32 = 230.0;
    const LABEL_WIDTH: f32 = 190.0;
    const VIEW_WIDTH: f32 = 270.0;
    const HEADER_HEIGHT: f32 = 58.0;
    const PADDING: f32 = 18.0;

    let width = LABEL_WIDTH + VIEW_WIDTH * PLAYER_POSE_PREVIEW_VIEWS.len() as f32 + PADDING * 2.0;
    let height = HEADER_HEIGHT + ROW_HEIGHT * specs.len() as f32 + PADDING;
    let mut svg = String::new();
    writeln!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width:.0}\" height=\"{height:.0}\" viewBox=\"0 0 {width:.0} {height:.0}\">"
    )
    .expect("writing to string should not fail");
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#10151f\"/>\n");
    svg.push_str("<text x=\"18\" y=\"24\" fill=\"#dbe7f3\" font-family=\"Menlo, monospace\" font-size=\"16\">NAU player/glider attachment preview</text>\n");
    for (index, view) in PLAYER_POSE_PREVIEW_VIEWS.iter().enumerate() {
        let x = LABEL_WIDTH + PADDING + VIEW_WIDTH * index as f32 + VIEW_WIDTH * 0.5;
        writeln!(
            svg,
            "<text x=\"{x:.1}\" y=\"44\" fill=\"#8fb1c9\" text-anchor=\"middle\" font-family=\"Menlo, monospace\" font-size=\"12\">{}</text>",
            view.label()
        )
        .expect("writing to string should not fail");
    }

    let shape_rows = specs
        .iter()
        .map(|spec| {
            let (player_shapes, glider_shapes) =
                player_glider_attachment_preview_shapes(player_gltf, glider_gltf, *spec)
                    .ok_or_else(|| {
                        format!(
                            "failed to compute player/glider attachment preview shapes for {}",
                            spec.label
                        )
                    })?;
            let combined = player_shapes
                .iter()
                .chain(glider_shapes.iter())
                .cloned()
                .collect::<Vec<_>>();
            Ok((player_shapes, glider_shapes, combined))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let combined_rows = shape_rows
        .iter()
        .map(|(_player_shapes, _glider_shapes, combined)| combined.clone())
        .collect::<Vec<_>>();
    let view_extents = PLAYER_POSE_PREVIEW_VIEWS
        .map(|view| preview_shape_rows_projected_extent(&combined_rows, view));

    for (row, (spec, (player_shapes, glider_shapes, combined_shapes))) in
        specs.iter().zip(shape_rows.iter()).enumerate()
    {
        let y = HEADER_HEIGHT + ROW_HEIGHT * row as f32;
        writeln!(
            svg,
            "<text x=\"18\" y=\"{:.1}\" fill=\"#edf5ff\" font-family=\"Menlo, monospace\" font-size=\"13\">{}</text>",
            y + 28.0,
            escape_xml(spec.title)
        )
        .expect("writing to string should not fail");
        writeln!(
            svg,
            "<text x=\"18\" y=\"{:.1}\" fill=\"#7f95a7\" font-family=\"Menlo, monospace\" font-size=\"10\">deploy: {:.0}%</text>",
            y + 46.0,
            spec.deployment * 100.0
        )
        .expect("writing to string should not fail");

        for (column, view) in PLAYER_POSE_PREVIEW_VIEWS.iter().enumerate() {
            let x = LABEL_WIDTH + PADDING + VIEW_WIDTH * column as f32;
            let preview_row = GliderAttachmentPreviewRow {
                player_shapes,
                glider_shapes,
                combined_shapes,
                requires_hand_grip: glider_pose_requires_hand_grip(spec.label),
            };
            render_player_glider_attachment_preview_view(
                &mut svg,
                preview_row,
                *view,
                view_extents[column],
                PosePreviewPanel {
                    x,
                    y: y + 12.0,
                    width: VIEW_WIDTH - 12.0,
                    height: ROW_HEIGHT - 20.0,
                },
            );
        }
    }

    svg.push_str("</svg>\n");
    Ok(svg)
}

fn player_pose_preview_shapes(
    gltf: &Value,
    overrides: &[PoseNodeOverride],
) -> Option<Vec<PlayerPosePreviewShape>> {
    let nodes = gltf.get("nodes")?.as_array()?;
    let buffers = embedded_gltf_buffers(gltf)?;
    let mut shapes = Vec::new();
    for node in nodes {
        let node_name = node.get("name").and_then(Value::as_str)?;
        let Some(mesh_index) = node
            .get("mesh")
            .and_then(Value::as_u64)
            .map(|mesh| mesh as usize)
        else {
            continue;
        };
        let mesh = gltf.get("meshes")?.as_array()?.get(mesh_index)?;
        let transform = world_node_transform_with_pose(gltf, node_name, overrides)?;
        let (vertices, surface_points) = mesh_world_geometry(gltf, mesh, transform, &buffers)?;
        if vertices.is_empty() {
            continue;
        }
        let bounds = aabb_from_points(&vertices)?;
        let obb = mesh_local_aabb(gltf, mesh)?.transformed_obb(transform);
        shapes.push(PlayerPosePreviewShape {
            node_name: node_name.to_string(),
            vertices,
            surface_points,
            bounds,
            obb,
            color: player_pose_preview_color(node_name),
        });
    }
    Some(shapes)
}

fn glider_pose_preview_shapes(
    gltf: &Value,
    spec: GliderPosePreviewSpec,
) -> Option<Vec<PlayerPosePreviewShape>> {
    glider_pose_preview_shapes_with_transform(gltf, glider_pose_preview_transform(spec))
}

fn player_glider_attachment_preview_shapes(
    player_gltf: &Value,
    glider_gltf: &Value,
    spec: GliderPosePreviewSpec,
) -> Option<(Vec<PlayerPosePreviewShape>, Vec<PlayerPosePreviewShape>)> {
    let overrides = player_pose_node_overrides(player_gltf, spec.context, spec.phase)?;
    let player_shapes = player_pose_preview_shapes(player_gltf, &overrides)?;
    let glider_shapes = glider_pose_preview_shapes_with_transform(
        glider_gltf,
        player_aligned_glider_transform(spec),
    )?;
    Some((player_shapes, glider_shapes))
}

fn glider_pose_preview_shapes_with_transform(
    gltf: &Value,
    pose_transform: Mat4,
) -> Option<Vec<PlayerPosePreviewShape>> {
    let nodes = gltf.get("nodes")?.as_array()?;
    let buffers = embedded_gltf_buffers(gltf)?;
    let mut shapes = Vec::new();
    for node in nodes {
        let node_name = node.get("name").and_then(Value::as_str)?;
        let Some(mesh_index) = node
            .get("mesh")
            .and_then(Value::as_u64)
            .map(|mesh| mesh as usize)
        else {
            continue;
        };
        let mesh = gltf.get("meshes")?.as_array()?.get(mesh_index)?;
        let transform = pose_transform * world_node_transform(gltf, node_name)?;
        let (vertices, surface_points) = mesh_world_geometry(gltf, mesh, transform, &buffers)?;
        if vertices.is_empty() {
            continue;
        }
        let bounds = aabb_from_points(&vertices)?;
        let obb = mesh_local_aabb(gltf, mesh)?.transformed_obb(transform);
        shapes.push(PlayerPosePreviewShape {
            node_name: node_name.to_string(),
            vertices,
            surface_points,
            bounds,
            obb,
            color: glider_pose_preview_color(node_name),
        });
    }
    Some(shapes)
}

fn player_aligned_glider_transform(spec: GliderPosePreviewSpec) -> Mat4 {
    preview_glider_player_anchor_transform() * glider_pose_preview_transform(spec)
}

fn glider_pose_preview_transform(spec: GliderPosePreviewSpec) -> Mat4 {
    let deployment = spec.deployment.clamp(0.0, 1.0);
    let pose = glider_traversal_pose(spec.context, spec.phase);
    let translation =
        preview_glider_stowed_translation_offset().lerp(pose.translation_offset, deployment);
    let rotation = preview_glider_stowed_rotation_offset().slerp(pose.rotation_offset, deployment);
    let scale = preview_glider_stowed_scale().lerp(Vec3::ONE, deployment);

    Mat4::from_scale_rotation_translation(scale, rotation, translation)
}

fn preview_glider_player_anchor_transform() -> Mat4 {
    Mat4::from_translation(Vec3::new(0.0, 1.35, -0.45))
}

fn preview_glider_stowed_translation_offset() -> Vec3 {
    Vec3::new(0.0, -0.58, 0.64)
}

fn preview_glider_stowed_rotation_offset() -> Quat {
    Quat::from_rotation_x(-1.08) * Quat::from_rotation_z(0.10)
}

fn preview_glider_stowed_scale() -> Vec3 {
    Vec3::new(0.18, 0.72, 0.58)
}

fn render_player_pose_preview_view(
    svg: &mut String,
    shapes: &[PlayerPosePreviewShape],
    view: PlayerPosePreviewView,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    writeln!(
        svg,
        "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{width:.1}\" height=\"{height:.1}\" rx=\"4\" fill=\"#151d29\" stroke=\"#263749\" stroke-width=\"1\"/>"
    )
    .expect("writing to string should not fail");
    let Some((min_u, max_u, min_v, max_v)) = preview_projected_extent(shapes, view) else {
        return;
    };
    let span_u = (max_u - min_u).max(0.01);
    let span_v = (max_v - min_v).max(0.01);
    let scale = ((width - 28.0) / span_u).min((height - 28.0) / span_v);
    let origin_x = x + width * 0.5 - (min_u + max_u) * 0.5 * scale;
    let origin_y = y + height * 0.5 + (min_v + max_v) * 0.5 * scale;

    let mut ordered = shapes.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        preview_depth(left.bounds, view)
            .partial_cmp(&preview_depth(right.bounds, view))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for shape in ordered {
        let hull = preview_projected_hull(&shape.vertices, view);
        if hull.len() < 3 {
            continue;
        }
        let mut path = String::new();
        for (index, point) in hull.iter().enumerate() {
            let x = origin_x + point.x * scale;
            let y = origin_y - point.y * scale;
            if index == 0 {
                write!(path, "M {x:.2} {y:.2}").expect("writing to string should not fail");
            } else {
                write!(path, " L {x:.2} {y:.2}").expect("writing to string should not fail");
            }
        }
        path.push_str(" Z");
        writeln!(
            svg,
            "<path d=\"{}\" fill=\"{}\" fill-opacity=\"0.68\" stroke=\"#e6eef7\" stroke-opacity=\"0.38\" stroke-width=\"0.55\"><title>{}</title></path>",
            path,
            shape.color,
            escape_xml(&shape.node_name)
        )
        .expect("writing to string should not fail");
    }
    render_player_pose_contact_overlay(svg, shapes, view, origin_x, origin_y, scale);
    render_player_pose_overlap_overlay(svg, shapes, view, origin_x, origin_y, scale);
}

fn render_glider_pose_preview_view(
    svg: &mut String,
    shapes: &[PlayerPosePreviewShape],
    view: PlayerPosePreviewView,
    extent: Option<(f32, f32, f32, f32)>,
    panel: PosePreviewPanel,
) {
    let PosePreviewPanel {
        x,
        y,
        width,
        height,
    } = panel;
    writeln!(
        svg,
        "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{width:.1}\" height=\"{height:.1}\" rx=\"4\" fill=\"#121923\" stroke=\"#294150\" stroke-width=\"1\"/>"
    )
    .expect("writing to string should not fail");
    let Some((min_u, max_u, min_v, max_v)) = extent else {
        return;
    };
    let span_u = (max_u - min_u).max(0.01);
    let span_v = (max_v - min_v).max(0.01);
    let scale = ((width - 30.0) / span_u).min((height - 30.0) / span_v);
    let origin_x = x + width * 0.5 - (min_u + max_u) * 0.5 * scale;
    let origin_y = y + height * 0.5 + (min_v + max_v) * 0.5 * scale;

    let mut ordered = shapes.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        preview_depth(left.bounds, view)
            .partial_cmp(&preview_depth(right.bounds, view))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for shape in ordered {
        let hull = preview_projected_hull(&shape.vertices, view);
        if hull.len() < 3 {
            continue;
        }
        let mut path = String::new();
        for (index, point) in hull.iter().enumerate() {
            let x = origin_x + point.x * scale;
            let y = origin_y - point.y * scale;
            if index == 0 {
                write!(path, "M {x:.2} {y:.2}").expect("writing to string should not fail");
            } else {
                write!(path, " L {x:.2} {y:.2}").expect("writing to string should not fail");
            }
        }
        path.push_str(" Z");
        writeln!(
            svg,
            "<path d=\"{}\" fill=\"{}\" fill-opacity=\"0.72\" stroke=\"#f1f8ff\" stroke-opacity=\"0.42\" stroke-width=\"0.55\"><title>{}</title></path>",
            path,
            shape.color,
            escape_xml(&shape.node_name)
        )
        .expect("writing to string should not fail");
    }
}

fn render_player_glider_attachment_preview_view(
    svg: &mut String,
    preview_row: GliderAttachmentPreviewRow<'_>,
    view: PlayerPosePreviewView,
    extent: Option<(f32, f32, f32, f32)>,
    panel: PosePreviewPanel,
) {
    let PosePreviewPanel {
        x,
        y,
        width,
        height,
    } = panel;
    writeln!(
        svg,
        "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{width:.1}\" height=\"{height:.1}\" rx=\"4\" fill=\"#121923\" stroke=\"#294150\" stroke-width=\"1\"/>"
    )
    .expect("writing to string should not fail");
    let Some((min_u, max_u, min_v, max_v)) = extent else {
        return;
    };
    let span_u = (max_u - min_u).max(0.01);
    let span_v = (max_v - min_v).max(0.01);
    let scale = ((width - 34.0) / span_u).min((height - 34.0) / span_v);
    let origin_x = x + width * 0.5 - (min_u + max_u) * 0.5 * scale;
    let origin_y = y + height * 0.5 + (min_v + max_v) * 0.5 * scale;

    let mut ordered = preview_row.combined_shapes.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        preview_depth(left.bounds, view)
            .partial_cmp(&preview_depth(right.bounds, view))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for shape in ordered {
        let hull = preview_projected_hull(&shape.vertices, view);
        if hull.len() < 3 {
            continue;
        }
        let mut path = String::new();
        for (index, point) in hull.iter().enumerate() {
            let x = origin_x + point.x * scale;
            let y = origin_y - point.y * scale;
            if index == 0 {
                write!(path, "M {x:.2} {y:.2}").expect("writing to string should not fail");
            } else {
                write!(path, " L {x:.2} {y:.2}").expect("writing to string should not fail");
            }
        }
        path.push_str(" Z");
        writeln!(
            svg,
            "<path d=\"{}\" fill=\"{}\" fill-opacity=\"0.64\" stroke=\"#f1f8ff\" stroke-opacity=\"0.40\" stroke-width=\"0.55\"><title>{}</title></path>",
            path,
            shape.color,
            escape_xml(&shape.node_name)
        )
        .expect("writing to string should not fail");
    }
    render_player_glider_attachment_overlay(
        svg,
        preview_row,
        view,
        Vec2::new(origin_x, origin_y),
        scale,
    );
}

fn preview_shape_rows_projected_extent(
    shape_rows: &[Vec<PlayerPosePreviewShape>],
    view: PlayerPosePreviewView,
) -> Option<(f32, f32, f32, f32)> {
    shape_rows
        .iter()
        .flat_map(|shapes| shapes.iter())
        .filter_map(|shape| preview_vertex_extent(&shape.vertices, view))
        .reduce(|accumulator, bounds| {
            (
                accumulator.0.min(bounds.0),
                accumulator.1.max(bounds.1),
                accumulator.2.min(bounds.2),
                accumulator.3.max(bounds.3),
            )
        })
}

fn preview_projected_extent(
    shapes: &[PlayerPosePreviewShape],
    view: PlayerPosePreviewView,
) -> Option<(f32, f32, f32, f32)> {
    shapes
        .iter()
        .filter_map(|shape| preview_vertex_extent(&shape.vertices, view))
        .reduce(|accumulator, bounds| {
            (
                accumulator.0.min(bounds.0),
                accumulator.1.max(bounds.1),
                accumulator.2.min(bounds.2),
                accumulator.3.max(bounds.3),
            )
        })
}

fn preview_vertex_extent(
    vertices: &[Vec3],
    view: PlayerPosePreviewView,
) -> Option<(f32, f32, f32, f32)> {
    vertices
        .iter()
        .map(|vertex| project_preview_point(*vertex, view))
        .map(|point| (point.x, point.x, point.y, point.y))
        .reduce(|accumulator, bounds| {
            (
                accumulator.0.min(bounds.0),
                accumulator.1.max(bounds.1),
                accumulator.2.min(bounds.2),
                accumulator.3.max(bounds.3),
            )
        })
}

fn player_mesh_silhouette_audit(gltf: &Value) -> Option<Value> {
    let specs = player_pose_preview_specs();
    let mut samples = Vec::new();
    let mut min_projected_span_m = f64::INFINITY;

    for spec in &specs {
        let overrides = player_pose_node_overrides(gltf, spec.context, spec.phase)?;
        let shapes = player_pose_preview_shapes(gltf, &overrides)?;
        for view in PLAYER_POSE_PREVIEW_VIEWS {
            let (min_u, max_u, min_v, max_v) = preview_projected_extent(&shapes, view)?;
            let width_m = f64::from(max_u - min_u);
            let height_m = f64::from(max_v - min_v);
            min_projected_span_m = min_projected_span_m.min(width_m).min(height_m);
            samples.push(PlayerMeshSilhouetteSample {
                label: spec.label,
                title: spec.title,
                pose_intent: spec.context.intent().label(),
                view,
                width_m,
                height_m,
            });
        }
    }

    let fall_top_width_m =
        mesh_silhouette_sample_value(&samples, "falling_belly_down", PlayerPosePreviewView::Top)
            .map(|sample| sample.width_m)?;
    let fall_top_depth_m =
        mesh_silhouette_sample_value(&samples, "falling_belly_down", PlayerPosePreviewView::Top)
            .map(|sample| sample.height_m)?;
    let fall_front_width_m =
        mesh_silhouette_sample_value(&samples, "falling_belly_down", PlayerPosePreviewView::Front)
            .map(|sample| sample.width_m)?;
    let glide_front_width_m =
        mesh_silhouette_sample_value(&samples, "gliding", PlayerPosePreviewView::Front)
            .map(|sample| sample.width_m)?;
    let dive_front_width_m =
        mesh_silhouette_sample_value(&samples, "diving_head_down", PlayerPosePreviewView::Front)
            .map(|sample| sample.width_m)?;
    let dive_front_height_m =
        mesh_silhouette_sample_value(&samples, "diving_head_down", PlayerPosePreviewView::Front)
            .map(|sample| sample.height_m)?;
    let dive_side_width_m =
        mesh_silhouette_sample_value(&samples, "diving_head_down", PlayerPosePreviewView::Side)
            .map(|sample| sample.width_m)?;
    let dive_side_height_m =
        mesh_silhouette_sample_value(&samples, "diving_head_down", PlayerPosePreviewView::Side)
            .map(|sample| sample.height_m)?;
    let dive_front_to_fall_front_width_ratio = dive_front_width_m / fall_front_width_m.max(0.001);
    let dive_front_width_to_height_ratio = dive_front_width_m / dive_front_height_m.max(0.001);
    let dive_side_width_to_height_ratio = dive_side_width_m / dive_side_height_m.max(0.001);

    Some(json!({
        "schema": "nau_player_mesh_silhouette_audit.v1",
        "pose_count": specs.len(),
        "view_count": PLAYER_POSE_PREVIEW_VIEWS.len(),
        "sample_count": samples.len(),
        "min_projected_span_m": if min_projected_span_m.is_finite() {
            min_projected_span_m
        } else {
            0.0
        },
        "fall_top_width_m": fall_top_width_m,
        "fall_top_depth_m": fall_top_depth_m,
        "fall_front_width_m": fall_front_width_m,
        "glide_front_width_m": glide_front_width_m,
        "dive_front_width_m": dive_front_width_m,
        "dive_front_height_m": dive_front_height_m,
        "dive_front_width_to_height_ratio": dive_front_width_to_height_ratio,
        "dive_side_width_m": dive_side_width_m,
        "dive_side_height_m": dive_side_height_m,
        "dive_side_width_to_height_ratio": dive_side_width_to_height_ratio,
        "dive_front_to_fall_front_width_ratio": dive_front_to_fall_front_width_ratio,
        "thresholds": {
            "projected_span_min_m": PLAYER_MESH_SILHOUETTE_MIN_PROJECTED_SPAN_M,
            "fall_top_width_min_m": PLAYER_MESH_SILHOUETTE_MIN_FALL_TOP_WIDTH_M,
            "fall_top_depth_min_m": PLAYER_MESH_SILHOUETTE_MIN_FALL_TOP_DEPTH_M,
            "glide_front_width_min_m": PLAYER_MESH_SILHOUETTE_MIN_GLIDE_FRONT_WIDTH_M,
            "dive_front_to_fall_front_width_ratio_max": PLAYER_MESH_SILHOUETTE_MAX_DIVE_FRONT_TO_FALL_FRONT_WIDTH_RATIO,
            "dive_front_height_min_m": PLAYER_MESH_SILHOUETTE_MIN_DIVE_FRONT_HEIGHT_M,
            "dive_front_width_to_height_ratio_max": PLAYER_MESH_SILHOUETTE_MAX_DIVE_FRONT_WIDTH_TO_HEIGHT_RATIO,
            "dive_side_width_to_height_ratio_max": PLAYER_MESH_SILHOUETTE_MAX_DIVE_SIDE_WIDTH_TO_HEIGHT_RATIO,
        },
        "samples": samples.iter().map(|sample| json!({
            "label": sample.label,
            "title": sample.title,
            "pose_intent": sample.pose_intent,
            "view": sample.view.key(),
            "width_m": sample.width_m,
            "height_m": sample.height_m,
            "aspect_width_over_height": sample.width_m / sample.height_m.max(0.001),
        })).collect::<Vec<_>>(),
    }))
}

fn mesh_silhouette_sample_value<'a>(
    samples: &'a [PlayerMeshSilhouetteSample],
    label: &str,
    view: PlayerPosePreviewView,
) -> Option<&'a PlayerMeshSilhouetteSample> {
    samples
        .iter()
        .find(|sample| sample.label == label && sample.view == view)
}

fn preview_projected_hull(vertices: &[Vec3], view: PlayerPosePreviewView) -> Vec<Vec2> {
    let mut points = vertices
        .iter()
        .map(|vertex| project_preview_point(*vertex, view))
        .collect::<Vec<_>>();
    points.sort_by(|left, right| {
        left.x
            .partial_cmp(&right.x)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                left.y
                    .partial_cmp(&right.y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    points.dedup_by(|left, right| {
        (left.x - right.x).abs() <= 0.0001 && (left.y - right.y).abs() <= 0.0001
    });
    if points.len() <= 2 {
        return points;
    }

    let mut lower = Vec::new();
    for point in points.iter().copied() {
        while lower.len() >= 2
            && preview_cross(lower[lower.len() - 2], lower[lower.len() - 1], point) <= 0.0
        {
            lower.pop();
        }
        lower.push(point);
    }

    let mut upper = Vec::new();
    for point in points.iter().rev().copied() {
        while upper.len() >= 2
            && preview_cross(upper[upper.len() - 2], upper[upper.len() - 1], point) <= 0.0
        {
            upper.pop();
        }
        upper.push(point);
    }

    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

fn preview_cross(origin: Vec2, left: Vec2, right: Vec2) -> f32 {
    (left.x - origin.x) * (right.y - origin.y) - (left.y - origin.y) * (right.x - origin.x)
}

fn project_preview_point(vertex: Vec3, view: PlayerPosePreviewView) -> Vec2 {
    match view {
        PlayerPosePreviewView::Front => Vec2::new(vertex.x, vertex.y),
        PlayerPosePreviewView::Rear => Vec2::new(-vertex.x, vertex.y),
        PlayerPosePreviewView::Side => Vec2::new(vertex.z, vertex.y),
        PlayerPosePreviewView::Top => Vec2::new(vertex.x, vertex.z),
    }
}

fn render_player_pose_contact_overlay(
    svg: &mut String,
    shapes: &[PlayerPosePreviewShape],
    view: PlayerPosePreviewView,
    origin_x: f32,
    origin_y: f32,
    scale: f32,
) {
    for pair in player_surface_contact_pairs() {
        let Some(left) = shapes.iter().find(|shape| shape.node_name == pair.left) else {
            continue;
        };
        let Some(right) = shapes.iter().find(|shape| shape.node_name == pair.right) else {
            continue;
        };
        let Some(contact) = closest_surface_points(&left.surface_points, &right.surface_points)
        else {
            continue;
        };
        if contact.distance_m <= 0.012 {
            continue;
        }
        let left = project_preview_point(contact.left, view);
        let right = project_preview_point(contact.right, view);
        let x1 = origin_x + left.x * scale;
        let y1 = origin_y - left.y * scale;
        let x2 = origin_x + right.x * scale;
        let y2 = origin_y - right.y * scale;
        let color = if contact.distance_m > PLAYER_POSE_MAX_SURFACE_CONTACT_DISTANCE_M {
            "#ff5364"
        } else if contact.distance_m > 0.040 {
            "#ffb84d"
        } else {
            "#68d391"
        };
        writeln!(
            svg,
            "<line x1=\"{x1:.2}\" y1=\"{y1:.2}\" x2=\"{x2:.2}\" y2=\"{y2:.2}\" stroke=\"{color}\" stroke-opacity=\"0.58\" stroke-width=\"0.9\" stroke-dasharray=\"3 3\"><title>{}: {} to {} surface distance {:.3} m</title></line>",
            pair.category,
            escape_xml(pair.left),
            escape_xml(pair.right),
            contact.distance_m
        )
        .expect("writing to string should not fail");
    }
}

fn render_player_glider_attachment_overlay(
    svg: &mut String,
    preview_row: GliderAttachmentPreviewRow<'_>,
    view: PlayerPosePreviewView,
    origin: Vec2,
    scale: f32,
) {
    if !preview_row.requires_hand_grip {
        return;
    }

    for side in [Side::Left, Side::Right] {
        let Some(contact) = player_glider_hand_grip_contact_from_shapes(
            preview_row.player_shapes,
            preview_row.glider_shapes,
            side,
        ) else {
            continue;
        };
        let left = project_preview_point(contact.left, view);
        let right = project_preview_point(contact.right, view);
        let x1 = origin.x + left.x * scale;
        let y1 = origin.y - left.y * scale;
        let x2 = origin.x + right.x * scale;
        let y2 = origin.y - right.y * scale;
        let color = if contact.distance_m > PLAYER_GLIDER_MAX_HAND_GRIP_DISTANCE_M {
            "#ff5364"
        } else if contact.distance_m > PLAYER_GLIDER_MAX_HAND_GRIP_DISTANCE_M * 0.78 {
            "#ffb84d"
        } else {
            "#68d391"
        };
        writeln!(
            svg,
            "<line x1=\"{x1:.2}\" y1=\"{y1:.2}\" x2=\"{x2:.2}\" y2=\"{y2:.2}\" stroke=\"{color}\" stroke-opacity=\"0.86\" stroke-width=\"1.2\" stroke-dasharray=\"4 3\"><title>{} hand grip distance {:.3} m</title></line>",
            side_label(side),
            contact.distance_m
        )
        .expect("writing to string should not fail");
    }
}

fn player_glider_hand_grip_contact_from_shapes(
    player_shapes: &[PlayerPosePreviewShape],
    glider_shapes: &[PlayerPosePreviewShape],
    side: Side,
) -> Option<ClosestSurfacePoints> {
    let player_points = shape_surface_points(player_shapes, player_hand_attachment_nodes(side));
    let glider_points = shape_surface_points(glider_shapes, glider_grip_attachment_nodes(side));
    closest_surface_points(&player_points, &glider_points)
}

fn shape_surface_points(shapes: &[PlayerPosePreviewShape], names: &[&str]) -> Vec<Vec3> {
    shapes
        .iter()
        .filter(|shape| names.iter().any(|name| *name == shape.node_name))
        .flat_map(|shape| shape.surface_points.iter().copied())
        .collect()
}

fn player_hand_attachment_nodes(side: Side) -> &'static [&'static str] {
    match side {
        Side::Left => &[
            "Nau Left Leather Hand Palm",
            "Nau Left Leather Index Finger Grip",
            "Nau Left Leather Finger Grip",
            "Nau Left Leather Ring Finger Grip",
            "Nau Left Leather Pinky Finger Grip",
            "Nau Left Leather Thumb Grip",
        ],
        Side::Right => &[
            "Nau Right Leather Hand Palm",
            "Nau Right Leather Index Finger Grip",
            "Nau Right Leather Finger Grip",
            "Nau Right Leather Ring Finger Grip",
            "Nau Right Leather Pinky Finger Grip",
            "Nau Right Leather Thumb Grip",
        ],
    }
}

fn glider_grip_attachment_nodes(side: Side) -> &'static [&'static str] {
    match side {
        Side::Left => &["Nau Glider Left Grip"],
        Side::Right => &["Nau Glider Right Grip"],
    }
}

fn side_label(side: Side) -> &'static str {
    match side {
        Side::Left => "left",
        Side::Right => "right",
    }
}

fn player_glider_attachment_audit(player_gltf: &Value, glider_gltf: &Value) -> Option<Value> {
    let specs = player_glider_attachment_audit_specs();
    let mut samples = Vec::new();
    let mut max_distance_m = 0.0_f64;
    let mut max_projected_gap_m = 0.0_f64;
    let mut breach_count = 0_u64;
    let mut worst_sample = Value::Null;

    for spec in specs {
        let (player_shapes, glider_shapes) =
            player_glider_attachment_preview_shapes(player_gltf, glider_gltf, spec)?;
        for side in [Side::Left, Side::Right] {
            let contact =
                player_glider_hand_grip_contact_from_shapes(&player_shapes, &glider_shapes, side)?;
            let player_bounds = shapes_aabb(&player_shapes, player_hand_attachment_nodes(side))?;
            let glider_bounds = shapes_aabb(&glider_shapes, glider_grip_attachment_nodes(side))?;
            let (projected_gap_m, projected_gap_view) =
                projected_contact_gap_m(player_bounds, glider_bounds);
            let within_threshold = contact.distance_m <= PLAYER_GLIDER_MAX_HAND_GRIP_DISTANCE_M
                && projected_gap_m <= PLAYER_GLIDER_MAX_HAND_GRIP_PROJECTED_GAP_M;
            if !within_threshold {
                breach_count += 1;
            }
            max_distance_m = max_distance_m.max(contact.distance_m);
            max_projected_gap_m = max_projected_gap_m.max(projected_gap_m);
            let sample = json!({
                "label": spec.label,
                "title": spec.title,
                "side": side_label(side),
                "phase": spec.phase,
                "deployment": spec.deployment,
                "pose_intent": spec.context.intent().label(),
                "distance_m": contact.distance_m,
                "projected_gap_m": projected_gap_m,
                "projected_gap_view": projected_gap_view.key(),
                "within_threshold": within_threshold,
                "player_contact": [contact.left.x, contact.left.y, contact.left.z],
                "glider_contact": [contact.right.x, contact.right.y, contact.right.z],
            });
            if worst_sample.is_null() || contact.distance_m >= max_distance_m {
                worst_sample = sample.clone();
            }
            samples.push(sample);
        }
    }

    Some(json!({
        "schema": "nau_player_glider_attachment_audit.v1",
        "sample_count": samples.len(),
        "pose_count": player_glider_attachment_audit_specs().len(),
        "max_distance_m": max_distance_m,
        "max_projected_gap_m": max_projected_gap_m,
        "breach_count": breach_count,
        "thresholds": {
            "hand_grip_distance_max_m": PLAYER_GLIDER_MAX_HAND_GRIP_DISTANCE_M,
            "hand_grip_projected_gap_max_m": PLAYER_GLIDER_MAX_HAND_GRIP_PROJECTED_GAP_M,
        },
        "worst_sample": worst_sample,
        "samples": samples,
    }))
}

fn shapes_aabb(shapes: &[PlayerPosePreviewShape], names: &[&str]) -> Option<Aabb3> {
    shapes
        .iter()
        .filter(|shape| names.iter().any(|name| *name == shape.node_name))
        .map(|shape| shape.bounds)
        .reduce(|mut aggregate, bounds| {
            aggregate.include_aabb(bounds);
            aggregate
        })
}

fn render_player_pose_overlap_overlay(
    svg: &mut String,
    shapes: &[PlayerPosePreviewShape],
    view: PlayerPosePreviewView,
    origin_x: f32,
    origin_y: f32,
    scale: f32,
) {
    let transform = PosePreviewTransform {
        origin_x,
        origin_y,
        scale,
    };
    for rule in player_pose_overlap_rules() {
        render_player_pose_overlap_marker(svg, shapes, view, transform, rule);
    }
}

#[derive(Clone, Copy)]
struct PosePreviewTransform {
    origin_x: f32,
    origin_y: f32,
    scale: f32,
}

#[derive(Clone, Copy)]
struct PlayerPoseOverlapRule {
    category: &'static str,
    left: &'static str,
    right: &'static str,
    warn_threshold_m: f64,
    fail_threshold_m: f64,
}

#[derive(Clone, Copy, Debug)]
struct PlayerPoseOverlapMarker {
    category: &'static str,
    left: &'static str,
    right: &'static str,
    overlap_m: f64,
    failed: bool,
}

fn player_pose_overlap_rules() -> Vec<PlayerPoseOverlapRule> {
    let mut rules = Vec::new();
    for (left, right) in player_joint_cover_mesh_pairs() {
        rules.push(PlayerPoseOverlapRule {
            category: "joint cover mesh overlap",
            left,
            right,
            warn_threshold_m: PLAYER_POSE_MAX_JOINT_COVER_MESH_OVERLAP_M * 0.95,
            fail_threshold_m: PLAYER_POSE_MAX_JOINT_COVER_MESH_OVERLAP_M,
        });
    }
    for (left, right) in player_joint_bridge_mesh_pairs() {
        rules.push(PlayerPoseOverlapRule {
            category: "joint bridge mesh overlap",
            left,
            right,
            warn_threshold_m: PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_OVERLAP_M * 0.95,
            fail_threshold_m: PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_OVERLAP_M,
        });
    }
    for (left, right) in player_rest_non_adjacent_mesh_overlap_pairs() {
        rules.push(PlayerPoseOverlapRule {
            category: "non-adjacent mesh overlap",
            left,
            right,
            warn_threshold_m: PLAYER_POSE_MAX_NON_ADJACENT_MESH_OVERLAP_M,
            fail_threshold_m: PLAYER_POSE_MAX_NON_ADJACENT_MESH_OVERLAP_M,
        });
    }
    rules
}

fn render_player_pose_overlap_marker(
    svg: &mut String,
    shapes: &[PlayerPosePreviewShape],
    view: PlayerPosePreviewView,
    transform: PosePreviewTransform,
    rule: PlayerPoseOverlapRule,
) {
    let Some(left) = shapes.iter().find(|shape| shape.node_name == rule.left) else {
        return;
    };
    let Some(right) = shapes.iter().find(|shape| shape.node_name == rule.right) else {
        return;
    };
    let Some(marker) = player_pose_overlap_marker_from_shapes(left, right, rule) else {
        return;
    };
    let Some((min_u, max_u, min_v, max_v)) =
        projected_aabb_overlap_rect(left.bounds, right.bounds, view)
    else {
        return;
    };
    let x = transform.origin_x + min_u * transform.scale;
    let y = transform.origin_y - max_v * transform.scale;
    let width = (max_u - min_u) * transform.scale;
    let height = (max_v - min_v) * transform.scale;
    if width < 0.7 || height < 0.7 {
        return;
    }
    let color = if marker.failed { "#ff5364" } else { "#ffb84d" };
    writeln!(
        svg,
        "<rect x=\"{x:.2}\" y=\"{y:.2}\" width=\"{width:.2}\" height=\"{height:.2}\" fill=\"{color}\" fill-opacity=\"0.18\" stroke=\"{color}\" stroke-opacity=\"0.92\" stroke-width=\"1.0\"><title>{}: {} into {} overlap {:.3} m</title></rect>",
        escape_xml(marker.category),
        escape_xml(marker.left),
        escape_xml(marker.right),
        marker.overlap_m
    )
    .expect("writing to string should not fail");
}

fn player_pose_overlap_marker_for_rule(
    shapes: &[PlayerPosePreviewShape],
    rule: PlayerPoseOverlapRule,
) -> Option<PlayerPoseOverlapMarker> {
    let left = shapes.iter().find(|shape| shape.node_name == rule.left)?;
    let right = shapes.iter().find(|shape| shape.node_name == rule.right)?;
    player_pose_overlap_marker_from_shapes(left, right, rule)
}

fn player_pose_overlap_marker_from_shapes(
    left: &PlayerPosePreviewShape,
    right: &PlayerPosePreviewShape,
    rule: PlayerPoseOverlapRule,
) -> Option<PlayerPoseOverlapMarker> {
    let overlap_m = left.obb.overlap_depth_m(right.obb);
    if overlap_m <= rule.warn_threshold_m {
        return None;
    }
    Some(PlayerPoseOverlapMarker {
        category: rule.category,
        left: rule.left,
        right: rule.right,
        overlap_m,
        failed: overlap_m > rule.fail_threshold_m,
    })
}

fn player_motion_integrity_overlay_warning_audit(gltf: &Value) -> Option<Value> {
    let specs = player_motion_integrity_review_specs();
    let rules = player_pose_overlap_rules();
    let mut warning_count = 0_u64;
    let mut fail_count = 0_u64;
    let mut max_overlap_m = 0.0_f64;
    let mut worst_warning = Value::Null;
    let mut panels = Vec::new();

    for spec in &specs {
        let overrides = player_pose_node_overrides(gltf, spec.context, spec.phase)?;
        let shapes = player_pose_preview_shapes(gltf, &overrides)?;
        let review_shapes = filter_player_pose_preview_shapes(&shapes, spec.nodes);
        let mut panel_warning_count = 0_u64;
        let mut panel_fail_count = 0_u64;

        for rule in &rules {
            let Some(marker) = player_pose_overlap_marker_for_rule(&review_shapes, *rule) else {
                continue;
            };
            warning_count += 1;
            panel_warning_count += 1;
            if marker.failed {
                fail_count += 1;
                panel_fail_count += 1;
            }
            if marker.overlap_m > max_overlap_m {
                max_overlap_m = marker.overlap_m;
                worst_warning = json!({
                    "label": spec.label,
                    "title": spec.title,
                    "view": spec.view.key(),
                    "category": marker.category,
                    "left_node": marker.left,
                    "right_node": marker.right,
                    "overlap_m": marker.overlap_m,
                    "failed": marker.failed,
                });
            }
        }

        panels.push(json!({
            "label": spec.label,
            "title": spec.title,
            "view": spec.view.key(),
            "node_count": review_shapes.len(),
            "warning_count": panel_warning_count,
            "fail_count": panel_fail_count,
        }));
    }

    Some(json!({
        "schema": "nau_player_motion_integrity_overlay_warning_audit.v1",
        "panel_count": specs.len(),
        "rule_count": rules.len(),
        "warning_count": warning_count,
        "fail_count": fail_count,
        "max_overlap_m": max_overlap_m,
        "worst_warning": worst_warning,
        "thresholds": {
            "warning_count_max": PLAYER_MOTION_INTEGRITY_OVERLAY_MAX_WARNING_COUNT,
        },
        "panels": panels,
    }))
}

fn projected_aabb_overlap_rect(
    left: Aabb3,
    right: Aabb3,
    view: PlayerPosePreviewView,
) -> Option<(f32, f32, f32, f32)> {
    let min = left.min.max(right.min);
    let max = left.max.min(right.max);
    if min.x >= max.x || min.y >= max.y || min.z >= max.z {
        return None;
    }
    let (min_u, max_u, min_v, max_v) = match view {
        PlayerPosePreviewView::Front => (min.x, max.x, min.y, max.y),
        PlayerPosePreviewView::Rear => (-max.x, -min.x, min.y, max.y),
        PlayerPosePreviewView::Side => (min.z, max.z, min.y, max.y),
        PlayerPosePreviewView::Top => (min.x, max.x, min.z, max.z),
    };
    Some((min_u, max_u, min_v, max_v))
}

fn preview_depth(bounds: Aabb3, view: PlayerPosePreviewView) -> f32 {
    match view {
        PlayerPosePreviewView::Front => (bounds.min.z + bounds.max.z) * 0.5,
        PlayerPosePreviewView::Rear => -(bounds.min.z + bounds.max.z) * 0.5,
        PlayerPosePreviewView::Side => (bounds.min.x + bounds.max.x) * 0.5,
        PlayerPosePreviewView::Top => (bounds.min.y + bounds.max.y) * 0.5,
    }
}

fn embedded_gltf_buffers(gltf: &Value) -> Option<Vec<Vec<u8>>> {
    gltf.get("buffers")?
        .as_array()?
        .iter()
        .map(|buffer| {
            let uri = buffer.get("uri").and_then(Value::as_str)?;
            let encoded = uri.strip_prefix("data:application/octet-stream;base64,")?;
            STANDARD.decode(encoded).ok()
        })
        .collect()
}

fn mesh_world_geometry(
    gltf: &Value,
    mesh: &Value,
    transform: Mat4,
    buffers: &[Vec<u8>],
) -> Option<(Vec<Vec3>, Vec<Vec3>)> {
    let primitives = mesh.get("primitives")?.as_array()?;
    let mut vertices = Vec::new();
    let mut surface_points = Vec::new();
    for primitive in primitives {
        let position_accessor_index = primitive
            .get("attributes")?
            .get("POSITION")
            .and_then(Value::as_u64)? as usize;
        let local_vertices = read_vec3_accessor(gltf, position_accessor_index, buffers)?;
        let transformed_vertices = local_vertices
            .iter()
            .map(|vertex| transform.transform_point3(*vertex))
            .collect::<Vec<_>>();
        surface_points.extend(transformed_vertices.iter().copied());
        if let Some(indices) = primitive
            .get("indices")
            .and_then(Value::as_u64)
            .and_then(|index| read_index_accessor(gltf, index as usize, buffers))
        {
            for triangle in indices.chunks_exact(3) {
                let Some(a) = transformed_vertices.get(triangle[0]).copied() else {
                    continue;
                };
                let Some(b) = transformed_vertices.get(triangle[1]).copied() else {
                    continue;
                };
                let Some(c) = transformed_vertices.get(triangle[2]).copied() else {
                    continue;
                };
                surface_points.push((a + b + c) / 3.0);
            }
        }
        vertices.extend(transformed_vertices);
    }
    Some((
        vertices,
        reduce_surface_points(surface_points, PLAYER_SURFACE_CONTACT_SAMPLE_LIMIT),
    ))
}

fn reduce_surface_points(points: Vec<Vec3>, limit: usize) -> Vec<Vec3> {
    if points.len() <= limit || limit == 0 {
        return points;
    }
    (0..limit)
        .filter_map(|index| points.get(index * points.len() / limit).copied())
        .collect()
}

const PLAYER_SURFACE_CONTACT_SAMPLE_LIMIT: usize = 96;

fn read_index_accessor(
    gltf: &Value,
    accessor_index: usize,
    buffers: &[Vec<u8>],
) -> Option<Vec<usize>> {
    let accessors = gltf.get("accessors")?.as_array()?;
    let buffer_views = gltf.get("bufferViews")?.as_array()?;
    let accessor = accessors.get(accessor_index)?;
    if accessor.get("type").and_then(Value::as_str)? != "SCALAR" {
        return None;
    }
    let component_type = accessor.get("componentType").and_then(Value::as_u64)?;
    let component_size = match component_type {
        5121 => 1,
        5123 => 2,
        5125 => 4,
        _ => return None,
    };
    let count = accessor.get("count").and_then(Value::as_u64)? as usize;
    let view_index = accessor.get("bufferView").and_then(Value::as_u64)? as usize;
    let view = buffer_views.get(view_index)?;
    let buffer_index = view.get("buffer").and_then(Value::as_u64)? as usize;
    let buffer = buffers.get(buffer_index)?;
    let view_offset = view.get("byteOffset").and_then(Value::as_u64).unwrap_or(0) as usize;
    let accessor_offset = accessor
        .get("byteOffset")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let stride = view
        .get("byteStride")
        .and_then(Value::as_u64)
        .unwrap_or(component_size) as usize;
    let start = view_offset + accessor_offset;

    (0..count)
        .map(|index| {
            let offset = start + index * stride;
            match component_type {
                5121 => buffer.get(offset).map(|value| *value as usize),
                5123 => read_u16_le(buffer, offset).map(|value| value as usize),
                5125 => read_u32_le(buffer, offset).map(|value| value as usize),
                _ => None,
            }
        })
        .collect()
}

fn read_u16_le(buffer: &[u8], offset: usize) -> Option<u16> {
    let bytes = buffer.get(offset..offset + 2)?;
    Some(u16::from_le_bytes(bytes.try_into().ok()?))
}

fn read_u32_le(buffer: &[u8], offset: usize) -> Option<u32> {
    let bytes = buffer.get(offset..offset + 4)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

fn node_world_surface_points_with_pose(
    gltf: &Value,
    node_name: &str,
    overrides: &[PoseNodeOverride],
    buffers: &[Vec<u8>],
) -> Option<Vec<Vec3>> {
    let node = gltf
        .get("nodes")?
        .as_array()?
        .iter()
        .find(|node| node.get("name").and_then(Value::as_str) == Some(node_name))?;
    let mesh_index = node.get("mesh").and_then(Value::as_u64)? as usize;
    let mesh = gltf.get("meshes")?.as_array()?.get(mesh_index)?;
    let transform = world_node_transform_with_pose(gltf, node_name, overrides)?;
    Some(mesh_world_geometry(gltf, mesh, transform, buffers)?.1)
}

fn closest_surface_points(left: &[Vec3], right: &[Vec3]) -> Option<ClosestSurfacePoints> {
    let mut best = ClosestSurfacePoints {
        distance_m: f64::INFINITY,
        left: Vec3::ZERO,
        right: Vec3::ZERO,
    };
    for left_point in left {
        for right_point in right {
            let distance_m = left_point.distance(*right_point) as f64;
            if distance_m < best.distance_m {
                best = ClosestSurfacePoints {
                    distance_m,
                    left: *left_point,
                    right: *right_point,
                };
            }
        }
    }
    best.distance_m.is_finite().then_some(best)
}

fn read_vec3_accessor(
    gltf: &Value,
    accessor_index: usize,
    buffers: &[Vec<u8>],
) -> Option<Vec<Vec3>> {
    let accessors = gltf.get("accessors")?.as_array()?;
    let buffer_views = gltf.get("bufferViews")?.as_array()?;
    let accessor = accessors.get(accessor_index)?;
    if accessor.get("componentType").and_then(Value::as_u64)? != 5126 {
        return None;
    }
    if accessor.get("type").and_then(Value::as_str)? != "VEC3" {
        return None;
    }

    let count = accessor.get("count").and_then(Value::as_u64)? as usize;
    let view_index = accessor.get("bufferView").and_then(Value::as_u64)? as usize;
    let view = buffer_views.get(view_index)?;
    let buffer_index = view.get("buffer").and_then(Value::as_u64)? as usize;
    let buffer = buffers.get(buffer_index)?;
    let view_offset = view.get("byteOffset").and_then(Value::as_u64).unwrap_or(0) as usize;
    let accessor_offset = accessor
        .get("byteOffset")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let stride = view.get("byteStride").and_then(Value::as_u64).unwrap_or(12) as usize;
    let start = view_offset + accessor_offset;

    (0..count)
        .map(|index| {
            let offset = start + index * stride;
            Some(Vec3::new(
                read_f32_le(buffer, offset)?,
                read_f32_le(buffer, offset + 4)?,
                read_f32_le(buffer, offset + 8)?,
            ))
        })
        .collect()
}

fn read_f32_le(buffer: &[u8], offset: usize) -> Option<f32> {
    let bytes = buffer.get(offset..offset + 4)?;
    Some(f32::from_le_bytes(bytes.try_into().ok()?))
}

fn aabb_from_points(points: &[Vec3]) -> Option<Aabb3> {
    let mut iter = points.iter().copied();
    let first = iter.next()?;
    let mut bounds = Aabb3::from_min_max(first, first);
    for point in iter {
        bounds.include_point(point);
    }
    Some(bounds)
}

impl PlayerPosePreviewView {
    fn key(self) -> &'static str {
        match self {
            Self::Front => "front",
            Self::Rear => "rear",
            Self::Side => "side",
            Self::Top => "top",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Front => "front mesh silhouette",
            Self::Rear => "rear mesh silhouette",
            Self::Side => "side mesh silhouette",
            Self::Top => "top mesh footprint",
        }
    }
}

fn player_pose_preview_color(node_name: &str) -> &'static str {
    let name = node_name.to_ascii_lowercase();
    if name.contains("skin") || name.contains("face") {
        "#c98a62"
    } else if name.contains("eye") || name.contains("focus") {
        "#ff9d2d"
    } else if name.contains("accent") || name.contains("scarf") || name.contains("tunic") {
        "#168894"
    } else if name.contains("belt") || name.contains("collarbone") || name.contains("pelvis side") {
        "#b88738"
    } else if name.contains("leather") || name.contains("boot") {
        "#3d281f"
    } else {
        "#26384f"
    }
}

fn glider_pose_preview_color(node_name: &str) -> &'static str {
    let name = node_name.to_ascii_lowercase();
    if name.contains("cloth") {
        "#2cc8b8"
    } else if name.contains("seam") {
        "#7ce8de"
    } else if name.contains("spar") || name.contains("rib") || name.contains("keel") {
        "#d8a449"
    } else if name.contains("tether") {
        "#b98b6a"
    } else if name.contains("handle") || name.contains("grip") {
        "#74513d"
    } else {
        "#91a9bc"
    }
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn audit_all_fixtures() -> Result<Value, String> {
    let mut fixtures = Vec::new();
    let mut checks = Vec::new();

    for spec in VISUAL_ASSET_SPECS {
        let Some(requirement) = REQUIREMENTS
            .iter()
            .find(|requirement| requirement.kind == spec.kind)
        else {
            checks.push(check_bool(
                "fixture_requirement_declared",
                false,
                kind_name(spec.kind),
            ));
            continue;
        };
        let path = Path::new("assets").join(spec.gltf_scene_path);
        let fixture = audit_fixture(&path, &spec, requirement)?;
        checks.push(check_bool(
            "fixture_present",
            fixture["present"].as_bool().unwrap_or(false),
            kind_name(spec.kind),
        ));
        checks.push(check_bool(
            "fixture_passed",
            fixture["passed"].as_bool().unwrap_or(false),
            kind_name(spec.kind),
        ));
        fixtures.push(fixture);
    }

    let player_glider_attachment_audit = player_glider_attachment_audit_from_assets()?;
    checks.push(check_eq_f64(
        "player_glider_attachment_sample_count",
        number_field(&player_glider_attachment_audit, "sample_count"),
        PLAYER_GLIDER_HAND_GRIP_EXPECTED_SAMPLE_COUNT,
        "samples",
    ));
    checks.push(check_at_most_f64(
        "player_glider_attachment_hand_grip_distance_max",
        number_field(&player_glider_attachment_audit, "max_distance_m"),
        PLAYER_GLIDER_MAX_HAND_GRIP_DISTANCE_M,
        "m",
    ));
    checks.push(check_at_most_f64(
        "player_glider_attachment_projected_gap_max",
        number_field(&player_glider_attachment_audit, "max_projected_gap_m"),
        PLAYER_GLIDER_MAX_HAND_GRIP_PROJECTED_GAP_M,
        "m",
    ));
    checks.push(check_at_most_f64(
        "player_glider_attachment_breach_count",
        number_field(&player_glider_attachment_audit, "breach_count"),
        0.0,
        "breaches",
    ));
    checks.push(check_eq_u64(
        "fixture_count",
        fixtures.len() as u64,
        VISUAL_ASSET_SPECS.len() as u64,
        "fixtures",
    ));
    let passed = checks_passed(&checks);

    Ok(json!({
        "passed": passed,
        "fixture_count": fixtures.len(),
        "checks": checks,
        "fixtures": fixtures,
        "player_glider_attachment_audit": player_glider_attachment_audit,
    }))
}

fn player_glider_attachment_audit_from_assets() -> Result<Value, String> {
    let player_path = Path::new("assets/models/player/player.gltf");
    let glider_path = Path::new("assets/models/player/glider.gltf");
    let player_text = fs::read_to_string(player_path)
        .map_err(|error| format!("failed to read {player_path:?}: {error}"))?;
    let glider_text = fs::read_to_string(glider_path)
        .map_err(|error| format!("failed to read {glider_path:?}: {error}"))?;
    let player_gltf = serde_json::from_str::<Value>(&player_text)
        .map_err(|error| format!("invalid player glTF JSON: {error}"))?;
    let glider_gltf = serde_json::from_str::<Value>(&glider_text)
        .map_err(|error| format!("invalid glider glTF JSON: {error}"))?;
    player_glider_attachment_audit(&player_gltf, &glider_gltf)
        .ok_or_else(|| "failed to compute player/glider attachment audit".to_string())
}

fn audit_fixture(
    path: &Path,
    spec: &VisualAssetSpec,
    requirement: &Requirement,
) -> Result<Value, String> {
    let path_string = path.to_string_lossy().into_owned();
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            return Ok(json!({
                "kind": kind_name(requirement.kind),
                "path": path_string,
                "present": false,
                "passed": false,
                "error": error.to_string(),
                "checks": [check_bool("present", false, "file")],
            }));
        }
    };
    let gltf = serde_json::from_str::<Value>(&text)
        .map_err(|error| format!("could not parse {}: {error}", path.display()))?;

    let asset = gltf.get("asset").unwrap_or(&Value::Null);
    let generator = asset.get("generator").and_then(Value::as_str).unwrap_or("");
    let copyright = asset.get("copyright").and_then(Value::as_str).unwrap_or("");
    let nau_metadata = gltf.pointer("/extras/nau").unwrap_or(&Value::Null);
    let nodes = array_len(&gltf, "nodes");
    let meshes = array_len(&gltf, "meshes");
    let materials = array_len(&gltf, "materials");
    let component_names = named_components(&gltf);
    let metrics = geometry_metrics(&gltf);
    let blend_material_count = gltf
        .get("materials")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|material| {
            material
                .get("alphaMode")
                .and_then(Value::as_str)
                .is_some_and(|mode| mode == "BLEND")
        })
        .count() as u64;
    let ready_player_clip_count = if requirement.require_player_clips {
        player_clip_count(&gltf)
    } else {
        0
    };
    let player_pose_shape_audit = requirement
        .require_player_clips
        .then(player_pose_shape_audit);
    let player_pose_contact_audit = requirement
        .require_player_clips
        .then(|| player_pose_contact_audit(&gltf))
        .flatten();
    let player_pose_transition_contact_audit = requirement
        .require_player_clips
        .then(|| player_pose_transition_contact_audit(&gltf))
        .flatten();
    let player_pose_surface_contact_audit = requirement
        .require_player_clips
        .then(|| player_pose_surface_contact_audit(&gltf))
        .flatten();
    let player_joint_bridge_contact_audit = requirement
        .require_player_clips
        .then(|| player_joint_bridge_contact_audit(&gltf))
        .flatten();
    let player_proximal_contact_audit = requirement
        .require_player_clips
        .then(|| player_proximal_contact_audit(&gltf))
        .flatten();
    let player_joint_seam_contact_audit = requirement
        .require_player_clips
        .then(|| player_joint_seam_contact_audit(&gltf))
        .flatten();
    let player_limb_anatomy_detail_audit = requirement
        .require_player_clips
        .then(|| player_limb_anatomy_detail_audit(&gltf))
        .flatten();
    let player_mesh_silhouette_audit = requirement
        .require_player_clips
        .then(|| player_mesh_silhouette_audit(&gltf))
        .flatten();
    let player_motion_integrity_overlay_warning_audit = requirement
        .require_player_clips
        .then(|| player_motion_integrity_overlay_warning_audit(&gltf))
        .flatten();

    let mut checks = vec![
        check_bool("present", true, "file"),
        check_bool(
            "self_authored_generator",
            generator.starts_with("NAU Engine self-authored"),
            "provenance",
        ),
        check_eq_str(
            "self_authored_copyright",
            copyright,
            "Self-authored for NAU Engine; no third-party assets.",
            "provenance",
        ),
        check_bool("nau_metadata_present", nau_metadata.is_object(), "metadata"),
        check_eq_str(
            "nau_metadata_schema",
            nau_metadata_str(nau_metadata, "schema"),
            NAU_FIXTURE_SCHEMA,
            "metadata",
        ),
        check_eq_str(
            "nau_metadata_asset_kind",
            nau_metadata_str(nau_metadata, "asset_kind"),
            kind_name(spec.kind),
            "metadata",
        ),
        check_eq_str(
            "nau_metadata_asset_label",
            nau_metadata_str(nau_metadata, "asset_label"),
            spec.label,
            "metadata",
        ),
        check_eq_str(
            "nau_metadata_residency",
            nau_metadata_str(nau_metadata, "residency"),
            residency_name(spec.residency),
            "metadata",
        ),
        check_eq_str(
            "nau_metadata_license",
            nau_metadata_str(nau_metadata, "license"),
            NAU_FIXTURE_LICENSE,
            "metadata",
        ),
        check_at_least_u64("node_count", nodes, requirement.min_nodes, "nodes"),
        check_at_least_u64("mesh_count", meshes, requirement.min_meshes, "meshes"),
        check_at_least_u64(
            "material_count",
            materials,
            requirement.min_materials,
            "materials",
        ),
        check_at_least_u64(
            "position_vertices",
            metrics.position_vertices,
            requirement.min_vertices,
            "vertices",
        ),
        check_at_least_u64(
            "indexed_triangles",
            metrics.indexed_triangles,
            requirement.min_triangles,
            "triangles",
        ),
        check_bool(
            "all_primitives_have_normals",
            metrics.missing_normals == 0,
            "primitives",
        ),
        check_bool(
            "all_primitives_have_uvs",
            metrics.missing_uvs == 0,
            "primitives",
        ),
    ];
    for name_fragment in requirement.required_name_fragments {
        checks.push(check_bool(
            "semantic_name_present",
            has_name_fragment(&component_names, name_fragment),
            name_fragment,
        ));
    }

    if requirement.require_blend_material {
        checks.push(check_at_least_u64(
            "blend_material_count",
            blend_material_count,
            1,
            "materials",
        ));
    }
    if requirement.require_player_clips {
        checks.push(check_at_least_f64(
            "player_rest_arm_half_width",
            symmetric_node_half_width_m(&gltf, "Nau Left Arm", "Nau Right Arm").unwrap_or(0.0),
            0.42,
            "m",
        ));
        checks.push(check_at_least_f64(
            "player_rest_leg_half_width",
            symmetric_node_half_width_m(&gltf, "Nau Left Leg", "Nau Right Leg").unwrap_or(0.0),
            0.20,
            "m",
        ));
        checks.push(check_at_least_f64(
            "player_scarf_back_offset",
            world_node_translation(&gltf, "Nau Wind Scarf Accent")
                .map_or(0.0, |translation| translation[2]),
            0.30,
            "m",
        ));
        let (finger_grip_length_min_m, finger_grip_length_max_m) =
            player_finger_grip_length_range_m(&gltf).unwrap_or((0.0, f64::INFINITY));
        checks.push(check_at_least_f64(
            "player_finger_grip_length_min",
            finger_grip_length_min_m,
            PLAYER_MIN_FINGER_GRIP_LENGTH_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_finger_grip_length_max",
            finger_grip_length_max_m,
            PLAYER_MAX_FINGER_GRIP_LENGTH_M,
            "m",
        ));
        checks.push(check_at_least_f64(
            "player_boot_sole_length_min",
            player_boot_sole_length_min_m(&gltf).unwrap_or(0.0),
            PLAYER_MIN_BOOT_SOLE_LENGTH_M,
            "m",
        ));
        let limb_anatomy = player_limb_anatomy_detail_audit
            .as_ref()
            .expect("player limb anatomy detail audit should be present for player fixture");
        checks.push(check_eq_f64(
            "player_limb_anatomy_detail_node_count",
            number_field(limb_anatomy, "present_node_count"),
            PLAYER_LIMB_ANATOMY_EXPECTED_NODE_COUNT,
            "nodes",
        ));
        checks.push(check_at_least_f64(
            "player_limb_anatomy_detail_major_extent_min",
            number_field(limb_anatomy, "min_major_extent_m"),
            PLAYER_MIN_LIMB_ANATOMY_MAJOR_EXTENT_M,
            "m",
        ));
        checks.push(check_eq_f64(
            "player_limb_anatomy_detail_missing_count",
            number_field(limb_anatomy, "missing_count"),
            0.0,
            "nodes",
        ));
        let mesh_silhouette = player_mesh_silhouette_audit
            .as_ref()
            .expect("player mesh silhouette audit should be present for player fixture");
        checks.push(check_eq_f64(
            "player_mesh_silhouette_pose_count",
            number_field(mesh_silhouette, "pose_count"),
            PLAYER_MESH_SILHOUETTE_EXPECTED_POSE_COUNT,
            "poses",
        ));
        checks.push(check_eq_f64(
            "player_mesh_silhouette_sample_count",
            number_field(mesh_silhouette, "sample_count"),
            PLAYER_MESH_SILHOUETTE_EXPECTED_SAMPLE_COUNT,
            "samples",
        ));
        checks.push(check_at_least_f64(
            "player_mesh_silhouette_projected_span_min",
            number_field(mesh_silhouette, "min_projected_span_m"),
            PLAYER_MESH_SILHOUETTE_MIN_PROJECTED_SPAN_M,
            "m",
        ));
        checks.push(check_at_least_f64(
            "player_mesh_silhouette_fall_top_width",
            number_field(mesh_silhouette, "fall_top_width_m"),
            PLAYER_MESH_SILHOUETTE_MIN_FALL_TOP_WIDTH_M,
            "m",
        ));
        checks.push(check_at_least_f64(
            "player_mesh_silhouette_fall_top_depth",
            number_field(mesh_silhouette, "fall_top_depth_m"),
            PLAYER_MESH_SILHOUETTE_MIN_FALL_TOP_DEPTH_M,
            "m",
        ));
        checks.push(check_at_least_f64(
            "player_mesh_silhouette_glide_front_width",
            number_field(mesh_silhouette, "glide_front_width_m"),
            PLAYER_MESH_SILHOUETTE_MIN_GLIDE_FRONT_WIDTH_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_mesh_silhouette_dive_front_to_fall_front_width_ratio",
            number_field(mesh_silhouette, "dive_front_to_fall_front_width_ratio"),
            PLAYER_MESH_SILHOUETTE_MAX_DIVE_FRONT_TO_FALL_FRONT_WIDTH_RATIO,
            "ratio",
        ));
        checks.push(check_at_least_f64(
            "player_mesh_silhouette_dive_front_height",
            number_field(mesh_silhouette, "dive_front_height_m"),
            PLAYER_MESH_SILHOUETTE_MIN_DIVE_FRONT_HEIGHT_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_mesh_silhouette_dive_front_width_to_height_ratio",
            number_field(mesh_silhouette, "dive_front_width_to_height_ratio"),
            PLAYER_MESH_SILHOUETTE_MAX_DIVE_FRONT_WIDTH_TO_HEIGHT_RATIO,
            "ratio",
        ));
        checks.push(check_at_most_f64(
            "player_mesh_silhouette_dive_side_width_to_height_ratio",
            number_field(mesh_silhouette, "dive_side_width_to_height_ratio"),
            PLAYER_MESH_SILHOUETTE_MAX_DIVE_SIDE_WIDTH_TO_HEIGHT_RATIO,
            "ratio",
        ));
        checks.push(check_eq_u64(
            "player_named_clip_count",
            ready_player_clip_count,
            PLAYER_ANIMATION_CLIP_NAMES.len() as u64,
            "clips",
        ));
        checks.push(check_bool(
            "player_core_pose_nodes_present",
            player_core_pose_nodes_present(&gltf),
            "nodes",
        ));
        checks.push(check_bool(
            "player_articulated_pose_nodes_present",
            player_articulated_pose_nodes_present(&gltf),
            "nodes",
        ));
        checks.push(check_bool(
            "player_joint_bridge_nodes_present",
            player_joint_bridge_nodes_present(&gltf),
            "nodes",
        ));
        checks.push(check_bool(
            "player_joint_seam_nodes_present",
            player_joint_seam_nodes_present(&gltf),
            "nodes",
        ));
        checks.push(check_bool(
            "player_rest_limb_attachment_hierarchy",
            player_rest_limb_attachment_hierarchy_valid(&gltf),
            "nodes",
        ));
        checks.push(check_bool(
            "player_rest_mesh_bounds_present",
            player_rest_mesh_bounds_present(&gltf),
            "mesh_bounds",
        ));
        checks.push(check_bool(
            "player_animation_channels_cover_core_signals",
            player_animation_channels_cover_core_signals(&gltf),
            "clips",
        ));
        checks.push(check_bool(
            "player_animation_channels_avoid_runtime_pose_nodes",
            player_animation_channels_avoid_runtime_pose_nodes(&gltf),
            "clips",
        ));
        checks.push(check_at_most_f64(
            "player_rest_joint_gap_max",
            player_rest_joint_gap_max_m(&gltf).unwrap_or(f64::INFINITY),
            0.03,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_rest_articulated_joint_gap_max",
            player_rest_articulated_joint_gap_max_m(&gltf).unwrap_or(f64::INFINITY),
            PLAYER_REST_MAX_ARTICULATED_JOINT_GAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_rest_non_adjacent_mesh_overlap_max",
            player_rest_non_adjacent_mesh_overlap_max_m(&gltf).unwrap_or(f64::INFINITY),
            PLAYER_REST_MAX_NON_ADJACENT_MESH_OVERLAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_rest_shoulder_mesh_overlap_max",
            player_rest_shoulder_mesh_overlap_max_m(&gltf).unwrap_or(f64::INFINITY),
            PLAYER_REST_MAX_SHOULDER_MESH_OVERLAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_non_adjacent_mesh_overlap_max",
            player_pose_non_adjacent_mesh_overlap_max_m(&gltf).unwrap_or(f64::INFINITY),
            PLAYER_POSE_MAX_NON_ADJACENT_MESH_OVERLAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_articulated_joint_gap_max",
            player_pose_articulated_joint_gap_max_m(&gltf).unwrap_or(f64::INFINITY),
            PLAYER_POSE_MAX_ARTICULATED_JOINT_GAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_joint_cover_mesh_gap_max",
            player_pose_joint_cover_mesh_gap_max_m(&gltf).unwrap_or(f64::INFINITY),
            PLAYER_POSE_MAX_JOINT_COVER_MESH_GAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_joint_cover_mesh_overlap_max",
            player_pose_joint_cover_mesh_overlap_max_m(&gltf).unwrap_or(f64::INFINITY),
            PLAYER_POSE_MAX_JOINT_COVER_MESH_OVERLAP_M,
            "m",
        ));
        let bridge_contact = player_joint_bridge_contact_audit
            .as_ref()
            .expect("player joint bridge contact audit should be present for player fixture");
        checks.push(check_at_least_f64(
            "player_joint_bridge_contact_node_count",
            number_field(bridge_contact, "bridge_node_count"),
            PLAYER_JOINT_BRIDGE_EXPECTED_NODE_COUNT,
            "nodes",
        ));
        checks.push(check_eq_f64(
            "player_joint_bridge_contact_pair_count",
            number_field(bridge_contact, "pair_count"),
            PLAYER_JOINT_BRIDGE_EXPECTED_PAIR_COUNT,
            "pairs",
        ));
        checks.push(check_at_most_f64(
            "player_pose_joint_bridge_mesh_gap_max",
            number_field(bridge_contact, "max_gap_m"),
            PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_GAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_joint_bridge_mesh_overlap_max",
            number_field(bridge_contact, "max_overlap_m"),
            PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_OVERLAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_joint_bridge_contact_breach_count",
            number_field(bridge_contact, "breach_count"),
            0.0,
            "breaches",
        ));
        let proximal_contact = player_proximal_contact_audit
            .as_ref()
            .expect("player proximal contact audit should be present for player fixture");
        checks.push(check_eq_f64(
            "player_proximal_contact_pair_count",
            number_field(proximal_contact, "pair_count"),
            PLAYER_PROXIMAL_CONTACT_EXPECTED_PAIR_COUNT,
            "pairs",
        ));
        checks.push(check_at_most_f64(
            "player_proximal_contact_gap_max",
            number_field(proximal_contact, "max_gap_m"),
            PLAYER_POSE_MAX_SURFACE_CONTACT_DISTANCE_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_proximal_contact_overlap_max",
            number_field(proximal_contact, "max_overlap_m"),
            PLAYER_POSE_MAX_PROXIMAL_CONTACT_MESH_OVERLAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_proximal_contact_breach_count",
            number_field(proximal_contact, "breach_count"),
            0.0,
            "breaches",
        ));
        let seam_contact = player_joint_seam_contact_audit
            .as_ref()
            .expect("player joint seam contact audit should be present for player fixture");
        checks.push(check_at_least_f64(
            "player_joint_seam_contact_node_count",
            number_field(seam_contact, "seam_node_count"),
            PLAYER_JOINT_SEAM_EXPECTED_NODE_COUNT,
            "nodes",
        ));
        checks.push(check_eq_f64(
            "player_joint_seam_contact_pair_count",
            number_field(seam_contact, "pair_count"),
            PLAYER_JOINT_SEAM_EXPECTED_PAIR_COUNT,
            "pairs",
        ));
        checks.push(check_at_most_f64(
            "player_pose_joint_seam_mesh_gap_max",
            number_field(seam_contact, "max_gap_m"),
            PLAYER_POSE_MAX_JOINT_SEAM_MESH_GAP_M,
            "m",
        ));
        checks.push(check_at_least_f64(
            "player_pose_joint_seam_mesh_overlap_min",
            number_field(seam_contact, "min_overlap_m"),
            PLAYER_POSE_MIN_JOINT_SEAM_MESH_OVERLAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_joint_seam_mesh_overlap_max",
            number_field(seam_contact, "max_overlap_m"),
            PLAYER_POSE_MAX_JOINT_SEAM_MESH_OVERLAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_joint_seam_contact_breach_count",
            number_field(seam_contact, "breach_count"),
            0.0,
            "breaches",
        ));
        let surface_contact = player_pose_surface_contact_audit
            .as_ref()
            .expect("player pose surface contact audit should be present for player fixture");
        checks.push(check_eq_f64(
            "player_pose_surface_contact_pair_count",
            number_field(surface_contact, "pair_count"),
            PLAYER_SURFACE_CONTACT_EXPECTED_PAIR_COUNT,
            "pairs",
        ));
        checks.push(check_at_most_f64(
            "player_pose_surface_contact_distance_max",
            number_field(surface_contact, "max_distance_m"),
            PLAYER_POSE_MAX_SURFACE_CONTACT_DISTANCE_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_surface_contact_projected_gap_max",
            number_field(surface_contact, "max_projected_gap_m"),
            PLAYER_POSE_MAX_PROJECTED_CONTACT_GAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_surface_contact_breach_count",
            number_field(surface_contact, "breach_count"),
            0.0,
            "breaches",
        ));
        let motion_overlay = player_motion_integrity_overlay_warning_audit
            .as_ref()
            .expect("player motion overlay warning audit should be present for player fixture");
        checks.push(check_eq_f64(
            "player_motion_integrity_overlay_panel_count",
            number_field(motion_overlay, "panel_count"),
            PLAYER_MOTION_INTEGRITY_REVIEW_EXPECTED_PANEL_COUNT,
            "panels",
        ));
        checks.push(check_at_most_f64(
            "player_motion_integrity_overlay_warning_count",
            number_field(motion_overlay, "warning_count"),
            PLAYER_MOTION_INTEGRITY_OVERLAY_MAX_WARNING_COUNT,
            "warnings",
        ));
        checks.push(check_at_most_f64(
            "player_motion_integrity_overlay_fail_count",
            number_field(motion_overlay, "fail_count"),
            0.0,
            "failures",
        ));
        let pose_contact = player_pose_contact_audit
            .as_ref()
            .expect("player pose contact audit should be present for player fixture");
        checks.push(check_at_least_f64(
            "player_pose_contact_report_pose_count",
            number_field(pose_contact, "pose_count"),
            PLAYER_POSE_CONTACT_EXPECTED_POSE_COUNT,
            "poses",
        ));
        checks.push(check_at_least_f64(
            "player_pose_contact_report_phase_count",
            number_field(pose_contact, "phase_count"),
            PLAYER_POSE_CONTACT_EXPECTED_PHASE_COUNT,
            "phases",
        ));
        checks.push(check_at_most_f64(
            "player_pose_contact_report_breach_count",
            number_field(pose_contact, "breach_count"),
            0.0,
            "breaches",
        ));
        let transition_contact = player_pose_transition_contact_audit
            .as_ref()
            .expect("player pose transition contact audit should be present for player fixture");
        checks.push(check_eq_f64(
            "player_pose_transition_contact_report_transition_count",
            number_field(transition_contact, "transition_count"),
            PLAYER_POSE_TRANSITION_EXPECTED_TRANSITION_COUNT,
            "transitions",
        ));
        checks.push(check_eq_f64(
            "player_pose_transition_contact_report_blend_count",
            number_field(transition_contact, "blend_count"),
            PLAYER_POSE_TRANSITION_EXPECTED_BLEND_COUNT,
            "blends",
        ));
        checks.push(check_eq_f64(
            "player_pose_transition_contact_report_phase_count",
            number_field(transition_contact, "phase_count"),
            PLAYER_POSE_CONTACT_EXPECTED_PHASE_COUNT,
            "phases",
        ));
        checks.push(check_at_most_f64(
            "player_pose_transition_contact_report_breach_count",
            number_field(transition_contact, "breach_count"),
            0.0,
            "breaches",
        ));
        checks.push(check_at_most_f64(
            "player_pose_transition_articulated_joint_gap_max",
            number_field(transition_contact, "articulated_joint_gap_max_m"),
            PLAYER_POSE_MAX_ARTICULATED_JOINT_GAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_transition_joint_cover_mesh_gap_max",
            number_field(transition_contact, "joint_cover_mesh_gap_max_m"),
            PLAYER_POSE_MAX_JOINT_COVER_MESH_GAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_transition_joint_cover_mesh_overlap_max",
            number_field(transition_contact, "joint_cover_mesh_overlap_max_m"),
            PLAYER_POSE_MAX_JOINT_COVER_MESH_OVERLAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_transition_joint_bridge_mesh_gap_max",
            number_field(transition_contact, "joint_bridge_mesh_gap_max_m"),
            PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_GAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_transition_joint_bridge_mesh_overlap_max",
            number_field(transition_contact, "joint_bridge_mesh_overlap_max_m"),
            PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_OVERLAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_transition_non_adjacent_mesh_overlap_max",
            number_field(transition_contact, "non_adjacent_mesh_overlap_max_m"),
            PLAYER_POSE_MAX_NON_ADJACENT_MESH_OVERLAP_M,
            "m",
        ));
        let pose_shape = player_pose_shape_audit
            .as_ref()
            .expect("player pose shape audit should be present for player fixture");
        checks.push(check_at_least_f64(
            "player_pose_falling_belly_down_readability",
            number_field(pose_shape, "falling_key_pose_readability_score"),
            MIN_KEY_POSE_READABILITY_SCORE as f64,
            "score",
        ));
        checks.push(check_at_least_f64(
            "player_pose_falling_torso_pitch",
            number_field(pose_shape, "falling_torso_pitch_degrees"),
            PLAYER_POSE_MIN_FALLING_TORSO_PITCH_DEGREES,
            "deg",
        ));
        checks.push(check_at_least_f64(
            "player_pose_falling_hips_pitch",
            number_field(pose_shape, "falling_hips_pitch_degrees"),
            PLAYER_POSE_MIN_FALLING_HIPS_PITCH_DEGREES,
            "deg",
        ));
        checks.push(check_at_most_f64(
            "player_pose_falling_torso_local_pitch",
            number_field(pose_shape, "falling_torso_local_pitch_degrees"),
            PLAYER_POSE_MAX_FALLING_TORSO_LOCAL_PITCH_DEGREES,
            "deg",
        ));
        checks.push(check_at_least_f64(
            "player_pose_falling_arm_spread",
            number_field(pose_shape, "falling_arm_spread_degrees"),
            PLAYER_POSE_MIN_FALLING_ARM_SPREAD_DEGREES,
            "deg",
        ));
        checks.push(check_at_least_f64(
            "player_pose_dive_streamline_readability",
            number_field(pose_shape, "dive_key_pose_readability_score"),
            MIN_KEY_POSE_READABILITY_SCORE as f64,
            "score",
        ));
        checks.push(check_at_least_f64(
            "player_pose_dive_torso_pitch",
            number_field(pose_shape, "dive_torso_pitch_degrees"),
            PLAYER_POSE_MIN_DIVE_TORSO_PITCH_DEGREES,
            "deg",
        ));
        checks.push(check_at_least_f64(
            "player_pose_dive_hips_pitch",
            number_field(pose_shape, "dive_hips_pitch_degrees"),
            PLAYER_POSE_MIN_DIVE_HIPS_PITCH_DEGREES,
            "deg",
        ));
        checks.push(check_at_most_f64(
            "player_pose_dive_torso_local_pitch",
            number_field(pose_shape, "dive_torso_local_pitch_degrees"),
            PLAYER_POSE_MAX_DIVE_TORSO_LOCAL_PITCH_DEGREES,
            "deg",
        ));
        checks.push(check_at_most_f64(
            "player_pose_dive_arm_spread",
            number_field(pose_shape, "dive_arm_spread_degrees"),
            PLAYER_POSE_MAX_DIVE_ARM_SPREAD_DEGREES,
            "deg",
        ));
        checks.push(check_at_least_f64(
            "player_pose_dive_leg_tuck",
            number_field(pose_shape, "dive_leg_tuck_degrees"),
            PLAYER_POSE_MIN_DIVE_LEG_TUCK_DEGREES,
            "deg",
        ));
        checks.push(check_at_least_f64(
            "player_pose_landing_recovery_readability",
            number_field(pose_shape, "landing_recovery_key_pose_readability_score"),
            MIN_KEY_POSE_READABILITY_SCORE as f64,
            "score",
        ));
        checks.push(check_at_most_f64(
            "player_pose_connected_limb_root_translation",
            number_field(pose_shape, "max_connected_limb_translation_m"),
            PLAYER_POSE_MAX_CONNECTED_LIMB_TRANSLATION_M,
            "m",
        ));
        checks.push(check_at_least_f64(
            "player_pose_launch_overhead_arm_score",
            number_field(pose_shape, "launch_overhead_arm_score"),
            PLAYER_POSE_MIN_LAUNCH_OVERHEAD_ARM_SCORE,
            "score",
        ));
        checks.push(check_at_least_f64(
            "player_glider_launch_takeout_deployment",
            number_field(pose_shape, "glider_launch_deployment"),
            PLAYER_GLIDER_MIN_LAUNCH_DEPLOYMENT,
            "ratio",
        ));
        checks.push(check_at_most_f64(
            "player_glider_launch_takeout_not_full_deployment",
            number_field(pose_shape, "glider_launch_deployment"),
            PLAYER_GLIDER_MAX_LAUNCH_DEPLOYMENT,
            "ratio",
        ));
        checks.push(check_at_least_f64(
            "player_glider_launch_takeout_response",
            number_field(pose_shape, "glider_launch_response_degrees"),
            PLAYER_GLIDER_MIN_LAUNCH_RESPONSE_DEGREES,
            "deg",
        ));
        checks.push(check_at_least_f64(
            "player_glider_launch_takeout_motion",
            number_field(pose_shape, "glider_launch_motion_m"),
            PLAYER_GLIDER_MIN_LAUNCH_MOTION_M,
            "m",
        ));
        checks.push(check_at_least_f64(
            "player_glider_glide_full_deployment",
            number_field(pose_shape, "glider_glide_deployment"),
            1.0,
            "ratio",
        ));
        checks.push(check_at_most_f64(
            "player_glider_stowed_grounded_deployment",
            number_field(pose_shape, "glider_grounded_deployment"),
            0.0,
            "ratio",
        ));
        checks.push(check_at_least_f64(
            "player_glider_dive_release_deployment",
            number_field(pose_shape, "glider_dive_release_deployment"),
            PLAYER_GLIDER_MIN_DIVE_RELEASE_DEPLOYMENT,
            "ratio",
        ));
        checks.push(check_at_most_f64(
            "player_glider_dive_release_not_full_deployment",
            number_field(pose_shape, "glider_dive_release_deployment"),
            PLAYER_GLIDER_MAX_DIVE_RELEASE_DEPLOYMENT,
            "ratio",
        ));
        checks.push(check_at_least_f64(
            "player_glider_dive_response",
            number_field(pose_shape, "glider_dive_response_degrees"),
            PLAYER_GLIDER_MIN_DIVE_RESPONSE_DEGREES,
            "deg",
        ));
        checks.push(check_at_least_f64(
            "player_glider_dive_motion",
            number_field(pose_shape, "glider_dive_motion_m"),
            PLAYER_GLIDER_MIN_DIVE_MOTION_M,
            "m",
        ));
        checks.push(check_bool(
            "player_bank_clip_motion_distinct",
            player_bank_clip_motion_is_distinct(&gltf),
            "clips",
        ));
        checks.push(check_bool(
            "player_launch_glide_land_clip_motion_distinct",
            player_launch_glide_land_clip_motion_is_distinct(&gltf),
            "clips",
        ));
        checks.push(check_bool(
            "player_grounded_locomotion_clip_motion_distinct",
            player_grounded_locomotion_clip_motion_is_distinct(&gltf),
            "clips",
        ));
        checks.push(check_bool(
            "player_fall_clip_motion_distinct",
            player_fall_clip_motion_is_distinct(&gltf),
            "clips",
        ));
    }

    let passed = checks_passed(&checks);
    Ok(json!({
        "kind": kind_name(requirement.kind),
        "path": path_string,
        "present": true,
        "passed": passed,
        "generator": generator,
        "nau_metadata": nau_metadata,
        "node_count": nodes,
        "mesh_count": meshes,
        "material_count": materials,
        "semantic_name_count": component_names.len(),
        "position_vertices": metrics.position_vertices,
        "indexed_triangles": metrics.indexed_triangles,
        "missing_normal_primitives": metrics.missing_normals,
        "missing_uv_primitives": metrics.missing_uvs,
        "blend_material_count": blend_material_count,
        "player_named_clip_count": ready_player_clip_count,
        "player_core_pose_nodes_present": player_core_pose_nodes_present(&gltf),
        "player_articulated_pose_nodes_present": player_articulated_pose_nodes_present(&gltf),
        "player_joint_bridge_nodes_present": player_joint_bridge_nodes_present(&gltf),
        "player_joint_seam_nodes_present": player_joint_seam_nodes_present(&gltf),
        "player_rest_limb_attachment_hierarchy": player_rest_limb_attachment_hierarchy_valid(&gltf),
        "player_rest_mesh_bounds_present": player_rest_mesh_bounds_present(&gltf),
        "player_animation_channels_cover_core_signals": player_animation_channels_cover_core_signals(&gltf),
        "player_animation_channels_avoid_runtime_pose_nodes": player_animation_channels_avoid_runtime_pose_nodes(&gltf),
        "player_rest_joint_gap_max_m": player_rest_joint_gap_max_m(&gltf),
        "player_rest_articulated_joint_gap_max_m": player_rest_articulated_joint_gap_max_m(&gltf),
        "player_finger_grip_length_range_m": player_finger_grip_length_range_m(&gltf).map(|(min, max)| json!({"min": min, "max": max})),
        "player_boot_sole_length_min_m": player_boot_sole_length_min_m(&gltf),
        "player_limb_anatomy_detail_audit": player_limb_anatomy_detail_audit,
        "player_mesh_silhouette_audit": player_mesh_silhouette_audit,
        "player_motion_integrity_overlay_warning_audit": player_motion_integrity_overlay_warning_audit,
        "player_rest_non_adjacent_mesh_overlap_max_m": player_rest_non_adjacent_mesh_overlap_max_m(&gltf),
        "player_rest_shoulder_mesh_overlap_max_m": player_rest_shoulder_mesh_overlap_max_m(&gltf),
        "player_pose_non_adjacent_mesh_overlap_max_m": player_pose_non_adjacent_mesh_overlap_max_m(&gltf),
        "player_pose_non_adjacent_mesh_overlap_worst_pair": player_pose_non_adjacent_mesh_overlap_report(&gltf).map(|report| report.to_json()),
        "player_pose_articulated_joint_gap_max_m": player_pose_articulated_joint_gap_max_m(&gltf),
        "player_pose_articulated_joint_gap_worst_pair": player_pose_articulated_joint_gap_report(&gltf).map(|report| report.to_json()),
        "player_pose_joint_cover_mesh_gap_max_m": player_pose_joint_cover_mesh_gap_max_m(&gltf),
        "player_pose_joint_cover_mesh_gap_worst_pair": player_pose_joint_cover_mesh_gap_report(&gltf).map(|report| report.to_json()),
        "player_pose_joint_cover_mesh_overlap_max_m": player_pose_joint_cover_mesh_overlap_max_m(&gltf),
        "player_pose_joint_cover_mesh_overlap_worst_pair": player_pose_joint_cover_mesh_overlap_report(&gltf).map(|report| report.to_json()),
        "player_joint_bridge_contact_audit": player_joint_bridge_contact_audit,
        "player_proximal_contact_audit": player_proximal_contact_audit,
        "player_joint_seam_contact_audit": player_joint_seam_contact_audit,
        "player_pose_contact_audit": player_pose_contact_audit,
        "player_pose_transition_contact_audit": player_pose_transition_contact_audit,
        "player_pose_surface_contact_audit": player_pose_surface_contact_audit,
        "player_bank_clip_motion_distinct": player_bank_clip_motion_is_distinct(&gltf),
        "player_launch_glide_land_clip_motion_distinct": player_launch_glide_land_clip_motion_is_distinct(&gltf),
        "player_grounded_locomotion_clip_motion_distinct": player_grounded_locomotion_clip_motion_is_distinct(&gltf),
        "player_fall_clip_motion_distinct": player_fall_clip_motion_is_distinct(&gltf),
        "player_pose_shape_audit": player_pose_shape_audit,
        "checks": checks,
    }))
}

fn nau_metadata_str<'a>(metadata: &'a Value, key: &str) -> &'a str {
    metadata.get(key).and_then(Value::as_str).unwrap_or("")
}

fn named_components(gltf: &Value) -> Vec<String> {
    ["nodes", "meshes"]
        .into_iter()
        .flat_map(|key| {
            gltf.get(key)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|value| value.get("name").and_then(Value::as_str))
                .map(|name| name.to_ascii_lowercase())
        })
        .collect()
}

fn has_name_fragment(component_names: &[String], fragment: &str) -> bool {
    let fragment = fragment.to_ascii_lowercase();
    component_names
        .iter()
        .any(|component_name| component_name.contains(&fragment))
}

#[derive(Default)]
struct GeometryMetrics {
    position_vertices: u64,
    indexed_triangles: u64,
    missing_normals: u64,
    missing_uvs: u64,
}

#[derive(Clone, Copy, Debug)]
struct Aabb3 {
    min: Vec3,
    max: Vec3,
}

#[derive(Clone, Copy, Debug)]
struct Obb3 {
    center: Vec3,
    axes: [Vec3; 3],
    half_extents: Vec3,
}

#[derive(Clone, Copy, Debug)]
struct PoseNodeOverride {
    node_name: &'static str,
    pose: PartPose,
}

#[derive(Clone, Copy, Debug)]
struct PlayerPoseTransition {
    label: &'static str,
    from: PlayerPoseContext,
    to: PlayerPoseContext,
}

#[derive(Clone, Copy, Debug)]
struct MeshOverlapReport {
    max_overlap_m: f64,
    overlap_axes_m: [f64; 3],
    left_node: &'static str,
    right_node: &'static str,
    pose_intent: &'static str,
    phase: f32,
}

impl MeshOverlapReport {
    fn zero() -> Self {
        Self {
            max_overlap_m: 0.0,
            overlap_axes_m: [0.0; 3],
            left_node: "",
            right_node: "",
            pose_intent: "none",
            phase: 0.0,
        }
    }

    fn observe(
        &mut self,
        overlap_m: f64,
        overlap_axes_m: [f64; 3],
        left_node: &'static str,
        right_node: &'static str,
        pose_intent: PlayerPoseIntent,
        phase: f32,
    ) {
        if overlap_m > self.max_overlap_m {
            self.max_overlap_m = overlap_m;
            self.overlap_axes_m = overlap_axes_m;
            self.left_node = left_node;
            self.right_node = right_node;
            self.pose_intent = pose_intent.label();
            self.phase = phase;
        }
    }

    fn observe_label(
        &mut self,
        overlap_m: f64,
        overlap_axes_m: [f64; 3],
        left_node: &'static str,
        right_node: &'static str,
        pose_label: &'static str,
        phase: f32,
    ) {
        if overlap_m > self.max_overlap_m {
            self.max_overlap_m = overlap_m;
            self.overlap_axes_m = overlap_axes_m;
            self.left_node = left_node;
            self.right_node = right_node;
            self.pose_intent = pose_label;
            self.phase = phase;
        }
    }

    fn to_json(self) -> Value {
        json!({
            "max_overlap_m": self.max_overlap_m,
            "overlap_x_m": self.overlap_axes_m[0],
            "overlap_y_m": self.overlap_axes_m[1],
            "overlap_z_m": self.overlap_axes_m[2],
            "left_node": self.left_node,
            "right_node": self.right_node,
            "pose_intent": self.pose_intent,
            "phase": self.phase,
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct JointGapReport {
    max_gap_m: f64,
    socket_node: &'static str,
    joint_node: &'static str,
    pose_intent: &'static str,
    phase: f32,
}

impl JointGapReport {
    fn zero() -> Self {
        Self {
            max_gap_m: 0.0,
            socket_node: "",
            joint_node: "",
            pose_intent: "none",
            phase: 0.0,
        }
    }

    fn observe(
        &mut self,
        gap_m: f64,
        socket_node: &'static str,
        joint_node: &'static str,
        pose_intent: PlayerPoseIntent,
        phase: f32,
    ) {
        if gap_m > self.max_gap_m {
            self.max_gap_m = gap_m;
            self.socket_node = socket_node;
            self.joint_node = joint_node;
            self.pose_intent = pose_intent.label();
            self.phase = phase;
        }
    }

    fn observe_label(
        &mut self,
        gap_m: f64,
        socket_node: &'static str,
        joint_node: &'static str,
        pose_label: &'static str,
        phase: f32,
    ) {
        if gap_m > self.max_gap_m {
            self.max_gap_m = gap_m;
            self.socket_node = socket_node;
            self.joint_node = joint_node;
            self.pose_intent = pose_label;
            self.phase = phase;
        }
    }

    fn to_json(self) -> Value {
        json!({
            "max_gap_m": self.max_gap_m,
            "socket_node": self.socket_node,
            "joint_node": self.joint_node,
            "pose_intent": self.pose_intent,
            "phase": self.phase,
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct MeshGapReport {
    max_gap_m: f64,
    left_node: &'static str,
    right_node: &'static str,
    pose_intent: &'static str,
    phase: f32,
}

impl MeshGapReport {
    fn zero() -> Self {
        Self {
            max_gap_m: 0.0,
            left_node: "",
            right_node: "",
            pose_intent: "none",
            phase: 0.0,
        }
    }

    fn observe(
        &mut self,
        gap_m: f64,
        left_node: &'static str,
        right_node: &'static str,
        pose_intent: PlayerPoseIntent,
        phase: f32,
    ) {
        if gap_m > self.max_gap_m {
            self.max_gap_m = gap_m;
            self.left_node = left_node;
            self.right_node = right_node;
            self.pose_intent = pose_intent.label();
            self.phase = phase;
        }
    }

    fn observe_label(
        &mut self,
        gap_m: f64,
        left_node: &'static str,
        right_node: &'static str,
        pose_label: &'static str,
        phase: f32,
    ) {
        if gap_m > self.max_gap_m {
            self.max_gap_m = gap_m;
            self.left_node = left_node;
            self.right_node = right_node;
            self.pose_intent = pose_label;
            self.phase = phase;
        }
    }

    fn to_json(self) -> Value {
        json!({
            "max_gap_m": self.max_gap_m,
            "left_node": self.left_node,
            "right_node": self.right_node,
            "pose_intent": self.pose_intent,
            "phase": self.phase,
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct MeshMinOverlapReport {
    min_overlap_m: f64,
    left_node: &'static str,
    right_node: &'static str,
    pose_intent: &'static str,
    phase: f32,
}

impl MeshMinOverlapReport {
    fn zero() -> Self {
        Self {
            min_overlap_m: f64::INFINITY,
            left_node: "",
            right_node: "",
            pose_intent: "none",
            phase: 0.0,
        }
    }

    fn observe_label(
        &mut self,
        overlap_m: f64,
        left_node: &'static str,
        right_node: &'static str,
        pose_label: &'static str,
        phase: f32,
    ) {
        if overlap_m < self.min_overlap_m {
            self.min_overlap_m = overlap_m;
            self.left_node = left_node;
            self.right_node = right_node;
            self.pose_intent = pose_label;
            self.phase = phase;
        }
    }

    fn value(self) -> f64 {
        if self.min_overlap_m.is_finite() {
            self.min_overlap_m
        } else {
            0.0
        }
    }

    fn to_json(self) -> Value {
        json!({
            "min_overlap_m": self.value(),
            "left_node": self.left_node,
            "right_node": self.right_node,
            "pose_intent": self.pose_intent,
            "phase": self.phase,
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct SurfaceContactDistanceReport {
    max_distance_m: f64,
    category: &'static str,
    left_node: &'static str,
    right_node: &'static str,
    pose_intent: &'static str,
    phase: f32,
}

impl SurfaceContactDistanceReport {
    fn zero() -> Self {
        Self {
            max_distance_m: 0.0,
            category: "",
            left_node: "",
            right_node: "",
            pose_intent: "none",
            phase: 0.0,
        }
    }

    fn observe_label(
        &mut self,
        distance_m: f64,
        pair: PlayerSurfaceContactPair,
        pose_label: &'static str,
        phase: f32,
    ) {
        if distance_m > self.max_distance_m {
            self.max_distance_m = distance_m;
            self.category = pair.category;
            self.left_node = pair.left;
            self.right_node = pair.right;
            self.pose_intent = pose_label;
            self.phase = phase;
        }
    }

    fn to_json(self) -> Value {
        json!({
            "max_distance_m": self.max_distance_m,
            "category": self.category,
            "left_node": self.left_node,
            "right_node": self.right_node,
            "pose_intent": self.pose_intent,
            "phase": self.phase,
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct ProjectedContactGapReport {
    max_gap_m: f64,
    view: PlayerPosePreviewView,
    category: &'static str,
    left_node: &'static str,
    right_node: &'static str,
    pose_intent: &'static str,
    phase: f32,
}

impl ProjectedContactGapReport {
    fn zero() -> Self {
        Self {
            max_gap_m: 0.0,
            view: PlayerPosePreviewView::Front,
            category: "",
            left_node: "",
            right_node: "",
            pose_intent: "none",
            phase: 0.0,
        }
    }

    fn observe_label(
        &mut self,
        gap_m: f64,
        view: PlayerPosePreviewView,
        pair: PlayerSurfaceContactPair,
        pose_label: &'static str,
        phase: f32,
    ) {
        if gap_m > self.max_gap_m {
            self.max_gap_m = gap_m;
            self.view = view;
            self.category = pair.category;
            self.left_node = pair.left;
            self.right_node = pair.right;
            self.pose_intent = pose_label;
            self.phase = phase;
        }
    }

    fn to_json(self) -> Value {
        json!({
            "max_gap_m": self.max_gap_m,
            "view": self.view.key(),
            "category": self.category,
            "left_node": self.left_node,
            "right_node": self.right_node,
            "pose_intent": self.pose_intent,
            "phase": self.phase,
        })
    }
}

impl Aabb3 {
    fn from_min_max(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    fn include_aabb(&mut self, bounds: Self) {
        self.min = self.min.min(bounds.min);
        self.max = self.max.max(bounds.max);
    }

    fn include_point(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    fn transformed(self, transform: Mat4) -> Self {
        let mut transformed = Self::from_min_max(
            transform.transform_point3(Vec3::new(self.min.x, self.min.y, self.min.z)),
            transform.transform_point3(Vec3::new(self.min.x, self.min.y, self.min.z)),
        );
        for x in [self.min.x, self.max.x] {
            for y in [self.min.y, self.max.y] {
                for z in [self.min.z, self.max.z] {
                    transformed.include_point(transform.transform_point3(Vec3::new(x, y, z)));
                }
            }
        }
        transformed
    }

    fn transformed_obb(self, transform: Mat4) -> Obb3 {
        let local_center = (self.min + self.max) * 0.5;
        let local_half_extents = (self.max - self.min) * 0.5;
        let transformed_axes = [
            transform.transform_vector3(Vec3::X * local_half_extents.x),
            transform.transform_vector3(Vec3::Y * local_half_extents.y),
            transform.transform_vector3(Vec3::Z * local_half_extents.z),
        ];
        Obb3 {
            center: transform.transform_point3(local_center),
            axes: transformed_axes.map(|axis| axis.normalize_or_zero()),
            half_extents: Vec3::new(
                transformed_axes[0].length(),
                transformed_axes[1].length(),
                transformed_axes[2].length(),
            ),
        }
    }

    fn overlap_depth_m(self, other: Self) -> f64 {
        let overlap = self.overlap_axes_m(other);
        if overlap.iter().all(|depth| *depth > 0.0) {
            overlap.into_iter().fold(f64::INFINITY, f64::min)
        } else {
            0.0
        }
    }

    fn overlap_axes_m(self, other: Self) -> [f64; 3] {
        [
            self.max.x.min(other.max.x) - self.min.x.max(other.min.x),
            self.max.y.min(other.max.y) - self.min.y.max(other.min.y),
            self.max.z.min(other.max.z) - self.min.z.max(other.min.z),
        ]
        .map(|depth| depth.max(0.0) as f64)
    }

    fn separation_m(self, other: Self) -> f64 {
        let gap = Vec3::new(
            axis_separation_m(self.min.x, self.max.x, other.min.x, other.max.x),
            axis_separation_m(self.min.y, self.max.y, other.min.y, other.max.y),
            axis_separation_m(self.min.z, self.max.z, other.min.z, other.max.z),
        );
        gap.length() as f64
    }
}

fn axis_separation_m(left_min: f32, left_max: f32, right_min: f32, right_max: f32) -> f32 {
    if left_max < right_min {
        right_min - left_max
    } else if right_max < left_min {
        left_min - right_max
    } else {
        0.0
    }
}

impl Obb3 {
    fn overlap_depth_m(self, other: Self) -> f64 {
        let mut min_overlap = f32::INFINITY;
        for axis in self.separating_axes(other) {
            if axis.length_squared() <= 1e-8 {
                continue;
            }
            let axis = axis.normalize();
            let center_distance = (other.center - self.center).dot(axis).abs();
            let overlap =
                self.projected_radius(axis) + other.projected_radius(axis) - center_distance;
            if overlap <= 0.0 {
                return 0.0;
            }
            min_overlap = min_overlap.min(overlap);
        }
        min_overlap as f64
    }

    fn projected_radius(self, axis: Vec3) -> f32 {
        self.axes[0].dot(axis).abs() * self.half_extents.x
            + self.axes[1].dot(axis).abs() * self.half_extents.y
            + self.axes[2].dot(axis).abs() * self.half_extents.z
    }

    fn separating_axes(self, other: Self) -> [Vec3; 15] {
        [
            self.axes[0],
            self.axes[1],
            self.axes[2],
            other.axes[0],
            other.axes[1],
            other.axes[2],
            self.axes[0].cross(other.axes[0]),
            self.axes[0].cross(other.axes[1]),
            self.axes[0].cross(other.axes[2]),
            self.axes[1].cross(other.axes[0]),
            self.axes[1].cross(other.axes[1]),
            self.axes[1].cross(other.axes[2]),
            self.axes[2].cross(other.axes[0]),
            self.axes[2].cross(other.axes[1]),
            self.axes[2].cross(other.axes[2]),
        ]
    }
}

fn geometry_metrics(gltf: &Value) -> GeometryMetrics {
    let accessors = gltf
        .get("accessors")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut metrics = GeometryMetrics::default();
    let Some(meshes) = gltf.get("meshes").and_then(Value::as_array) else {
        return metrics;
    };

    for mesh in meshes {
        let Some(primitives) = mesh.get("primitives").and_then(Value::as_array) else {
            continue;
        };
        for primitive in primitives {
            let attributes = primitive.get("attributes").unwrap_or(&Value::Null);
            if let Some(position_accessor) = attributes.get("POSITION").and_then(Value::as_u64) {
                metrics.position_vertices += accessor_count(&accessors, position_accessor);
            }
            if attributes.get("NORMAL").and_then(Value::as_u64).is_none() {
                metrics.missing_normals += 1;
            }
            if attributes
                .get("TEXCOORD_0")
                .and_then(Value::as_u64)
                .is_none()
            {
                metrics.missing_uvs += 1;
            }
            if let Some(index_accessor) = primitive.get("indices").and_then(Value::as_u64) {
                metrics.indexed_triangles += accessor_count(&accessors, index_accessor) / 3;
            }
        }
    }
    metrics
}

fn accessor_count(accessors: &[Value], accessor_index: u64) -> u64 {
    accessors
        .get(accessor_index as usize)
        .and_then(|accessor| accessor.get("count"))
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

fn player_clip_count(gltf: &Value) -> u64 {
    let animations = gltf
        .get("animations")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    PLAYER_ANIMATION_CLIP_NAMES
        .iter()
        .filter(|clip_name| {
            animations.iter().any(|animation| {
                animation
                    .get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|name| name == **clip_name)
            })
        })
        .count() as u64
}

fn player_core_pose_nodes_present(gltf: &Value) -> bool {
    [
        "Nau Hips",
        "Nau Torso",
        "Nau Head",
        "Nau Left Arm",
        "Nau Right Arm",
        "Nau Left Leg",
        "Nau Right Leg",
        "Nau Neck Socket",
        "Nau Left Shoulder Socket",
        "Nau Right Shoulder Socket",
        "Nau Left Hip Socket",
        "Nau Right Hip Socket",
    ]
    .into_iter()
    .all(|name| node_index(gltf, name).is_some())
}

fn player_articulated_pose_nodes_present(gltf: &Value) -> bool {
    [
        "Nau Left Forearm",
        "Nau Right Forearm",
        "Nau Left Leather Hand Palm",
        "Nau Right Leather Hand Palm",
        "Nau Left Lower Leg",
        "Nau Right Lower Leg",
        "Nau Left Boot",
        "Nau Right Boot",
        "Nau Left Elbow Socket",
        "Nau Right Elbow Socket",
        "Nau Left Wrist Socket",
        "Nau Right Wrist Socket",
        "Nau Left Knee Socket",
        "Nau Right Knee Socket",
        "Nau Left Ankle Socket",
        "Nau Right Ankle Socket",
    ]
    .into_iter()
    .all(|name| node_index(gltf, name).is_some())
}

fn player_joint_bridge_nodes_present(gltf: &Value) -> bool {
    player_joint_bridge_node_names()
        .into_iter()
        .all(|name| node_index(gltf, name).is_some())
}

fn player_joint_seam_nodes_present(gltf: &Value) -> bool {
    player_joint_seam_node_names()
        .into_iter()
        .all(|name| node_index(gltf, name).is_some())
}

fn player_rest_limb_attachment_hierarchy_valid(gltf: &Value) -> bool {
    [
        ("Nau Torso", "Nau Head"),
        ("Nau Torso", "Nau Left Arm"),
        ("Nau Torso", "Nau Right Arm"),
        ("Nau Left Arm", "Nau Left Forearm"),
        ("Nau Right Arm", "Nau Right Forearm"),
        ("Nau Left Forearm", "Nau Left Leather Hand Palm"),
        ("Nau Right Forearm", "Nau Right Leather Hand Palm"),
        ("Nau Hips", "Nau Left Leg"),
        ("Nau Hips", "Nau Right Leg"),
        ("Nau Left Leg", "Nau Left Lower Leg"),
        ("Nau Right Leg", "Nau Right Lower Leg"),
        ("Nau Left Lower Leg", "Nau Left Boot"),
        ("Nau Right Lower Leg", "Nau Right Boot"),
    ]
    .into_iter()
    .all(|(parent, child)| node_is_direct_child(gltf, parent, child))
}

fn player_animation_channels_cover_core_signals(gltf: &Value) -> bool {
    PLAYER_ANIMATION_CLIP_NAMES.iter().all(|clip_name| {
        let Some(targets) = animation_target_node_names(gltf, clip_name) else {
            return false;
        };
        [
            "Nau Animation Signal Hips",
            "Nau Animation Signal Torso",
            "Nau Animation Signal Left Arm",
            "Nau Animation Signal Right Arm",
            "Nau Animation Signal Left Leg",
            "Nau Animation Signal Right Leg",
        ]
        .into_iter()
        .all(|required| targets.iter().any(|target| target == required))
    })
}

fn player_animation_channels_avoid_runtime_pose_nodes(gltf: &Value) -> bool {
    let runtime_pose_nodes = [
        "Nau Hips",
        "Nau Torso",
        "Nau Head",
        "Nau Left Arm",
        "Nau Right Arm",
        "Nau Left Forearm",
        "Nau Right Forearm",
        "Nau Left Leather Hand Palm",
        "Nau Right Leather Hand Palm",
        "Nau Left Leg",
        "Nau Right Leg",
        "Nau Left Lower Leg",
        "Nau Right Lower Leg",
        "Nau Left Boot",
        "Nau Right Boot",
        "Nau Back Scarf Anchor Accent",
        "Nau Wind Scarf Accent",
    ];
    PLAYER_ANIMATION_CLIP_NAMES.iter().all(|clip_name| {
        animation_target_node_names(gltf, clip_name).is_some_and(|targets| {
            targets
                .iter()
                .all(|target| !runtime_pose_nodes.contains(&target.as_str()))
        })
    })
}

fn player_rest_joint_gap_max_m(gltf: &Value) -> Option<f64> {
    let pairs = [
        ("Nau Neck Socket", "Nau Head"),
        ("Nau Left Shoulder Socket", "Nau Left Arm"),
        ("Nau Right Shoulder Socket", "Nau Right Arm"),
        ("Nau Left Hip Socket", "Nau Left Leg"),
        ("Nau Right Hip Socket", "Nau Right Leg"),
    ];
    pairs
        .into_iter()
        .map(|(socket, joint)| {
            let socket = world_node_translation(gltf, socket)?;
            let joint = world_node_translation(gltf, joint)?;
            Some(distance3(socket, joint))
        })
        .collect::<Option<Vec<_>>>()
        .map(|gaps| gaps.into_iter().fold(0.0, f64::max))
}

fn player_rest_articulated_joint_gap_max_m(gltf: &Value) -> Option<f64> {
    player_articulated_joint_gap_pairs()
        .into_iter()
        .map(|(socket, joint)| {
            let socket = world_node_translation(gltf, socket)?;
            let joint = world_node_translation(gltf, joint)?;
            Some(distance3(socket, joint))
        })
        .collect::<Option<Vec<_>>>()
        .map(|gaps| gaps.into_iter().fold(0.0, f64::max))
}

fn player_articulated_joint_gap_pairs() -> [(&'static str, &'static str); 8] {
    [
        ("Nau Left Elbow Socket", "Nau Left Forearm"),
        ("Nau Right Elbow Socket", "Nau Right Forearm"),
        ("Nau Left Wrist Socket", "Nau Left Leather Hand Palm"),
        ("Nau Right Wrist Socket", "Nau Right Leather Hand Palm"),
        ("Nau Left Knee Socket", "Nau Left Lower Leg"),
        ("Nau Right Knee Socket", "Nau Right Lower Leg"),
        ("Nau Left Ankle Socket", "Nau Left Boot"),
        ("Nau Right Ankle Socket", "Nau Right Boot"),
    ]
}

fn player_pose_articulated_joint_gap_max_m(gltf: &Value) -> Option<f64> {
    player_pose_articulated_joint_gap_report(gltf).map(|report| report.max_gap_m)
}

fn player_pose_contact_phases() -> [f32; 12] {
    let step = std::f32::consts::TAU / 12.0;
    std::array::from_fn(|index| step * index as f32)
}

fn player_pose_transition_contact_blends() -> [f32; 4] {
    [0.2, 0.4, 0.6, 0.8]
}

#[cfg(test)]
fn player_pose_contact_samples_per_pair() -> f64 {
    (player_pose_mesh_overlap_contexts().len() * player_pose_contact_phases().len()
        + player_pose_transition_contact_transitions().len()
            * player_pose_transition_contact_blends().len()
            * player_pose_contact_phases().len()) as f64
}

fn player_pose_contact_audit(gltf: &Value) -> Option<Value> {
    let phases = player_pose_contact_phases();
    let non_adjacent_pairs = player_rest_non_adjacent_mesh_overlap_pairs();
    let mut pose_reports = Vec::new();
    let mut breach_count = 0_u64;

    for context in player_pose_mesh_overlap_contexts() {
        let mut articulated_gap = JointGapReport::zero();
        let mut joint_cover_gap = MeshGapReport::zero();
        let mut joint_cover_overlap = MeshOverlapReport::zero();
        let mut non_adjacent_overlap = MeshOverlapReport::zero();

        for phase in phases {
            let overrides = player_pose_node_overrides(gltf, context, phase)?;

            for (socket, joint) in player_articulated_joint_gap_pairs() {
                let socket_translation =
                    world_node_translation_with_pose(gltf, socket, &overrides)?;
                let joint_translation = world_node_translation_with_pose(gltf, joint, &overrides)?;
                articulated_gap.observe(
                    distance3(socket_translation, joint_translation),
                    socket,
                    joint,
                    context.intent(),
                    phase,
                );
            }

            for (left, right) in player_joint_cover_mesh_pairs() {
                let left_bounds = node_world_mesh_aabb_with_pose(gltf, left, &overrides)?;
                let right_bounds = node_world_mesh_aabb_with_pose(gltf, right, &overrides)?;
                joint_cover_gap.observe(
                    left_bounds.separation_m(right_bounds),
                    left,
                    right,
                    context.intent(),
                    phase,
                );
                joint_cover_overlap.observe(
                    node_world_mesh_obb_with_pose(gltf, left, &overrides)?
                        .overlap_depth_m(node_world_mesh_obb_with_pose(gltf, right, &overrides)?),
                    left_bounds.overlap_axes_m(right_bounds),
                    left,
                    right,
                    context.intent(),
                    phase,
                );
            }

            for (left, right) in non_adjacent_pairs.iter().copied() {
                let left_bounds = node_world_mesh_aabb_with_pose(gltf, left, &overrides)?;
                let right_bounds = node_world_mesh_aabb_with_pose(gltf, right, &overrides)?;
                non_adjacent_overlap.observe(
                    node_world_mesh_obb_with_pose(gltf, left, &overrides)?
                        .overlap_depth_m(node_world_mesh_obb_with_pose(gltf, right, &overrides)?),
                    left_bounds.overlap_axes_m(right_bounds),
                    left,
                    right,
                    context.intent(),
                    phase,
                );
            }
        }

        let within_thresholds = articulated_gap.max_gap_m
            <= PLAYER_POSE_MAX_ARTICULATED_JOINT_GAP_M
            && joint_cover_gap.max_gap_m <= PLAYER_POSE_MAX_JOINT_COVER_MESH_GAP_M
            && joint_cover_overlap.max_overlap_m <= PLAYER_POSE_MAX_JOINT_COVER_MESH_OVERLAP_M
            && non_adjacent_overlap.max_overlap_m <= PLAYER_POSE_MAX_NON_ADJACENT_MESH_OVERLAP_M;
        if !within_thresholds {
            breach_count += 1;
        }

        pose_reports.push(json!({
            "pose_intent": context.intent().label(),
            "phase_count": phases.len(),
            "within_thresholds": within_thresholds,
            "articulated_joint_gap_max_m": articulated_gap.max_gap_m,
            "joint_cover_mesh_gap_max_m": joint_cover_gap.max_gap_m,
            "joint_cover_mesh_overlap_max_m": joint_cover_overlap.max_overlap_m,
            "non_adjacent_mesh_overlap_max_m": non_adjacent_overlap.max_overlap_m,
            "articulated_joint_gap_worst_pair": articulated_gap.to_json(),
            "joint_cover_mesh_gap_worst_pair": joint_cover_gap.to_json(),
            "joint_cover_mesh_overlap_worst_pair": joint_cover_overlap.to_json(),
            "non_adjacent_mesh_overlap_worst_pair": non_adjacent_overlap.to_json(),
        }));
    }

    Some(json!({
        "schema": "nau_player_pose_contact_audit.v1",
        "pose_count": pose_reports.len(),
        "phase_count": phases.len(),
        "breach_count": breach_count,
        "thresholds": {
            "articulated_joint_gap_max_m": PLAYER_POSE_MAX_ARTICULATED_JOINT_GAP_M,
            "joint_cover_mesh_gap_max_m": PLAYER_POSE_MAX_JOINT_COVER_MESH_GAP_M,
            "joint_cover_mesh_overlap_max_m": PLAYER_POSE_MAX_JOINT_COVER_MESH_OVERLAP_M,
            "non_adjacent_mesh_overlap_max_m": PLAYER_POSE_MAX_NON_ADJACENT_MESH_OVERLAP_M,
        },
        "poses": pose_reports,
    }))
}

fn player_pose_transition_contact_audit(gltf: &Value) -> Option<Value> {
    let phases = player_pose_contact_phases();
    let blends = player_pose_transition_contact_blends();
    let transitions = player_pose_transition_contact_transitions();
    let non_adjacent_pairs = player_rest_non_adjacent_mesh_overlap_pairs();
    let mut transition_reports = Vec::new();
    let mut breach_count = 0_u64;
    let mut overall_articulated_gap = JointGapReport::zero();
    let mut overall_joint_cover_gap = MeshGapReport::zero();
    let mut overall_joint_cover_overlap = MeshOverlapReport::zero();
    let mut overall_joint_bridge_gap = MeshGapReport::zero();
    let mut overall_joint_bridge_overlap = MeshOverlapReport::zero();
    let mut overall_non_adjacent_overlap = MeshOverlapReport::zero();

    for transition in transitions {
        let mut articulated_gap = JointGapReport::zero();
        let mut joint_cover_gap = MeshGapReport::zero();
        let mut joint_cover_overlap = MeshOverlapReport::zero();
        let mut joint_bridge_gap = MeshGapReport::zero();
        let mut joint_bridge_overlap = MeshOverlapReport::zero();
        let mut non_adjacent_overlap = MeshOverlapReport::zero();

        for phase in phases {
            for blend in blends {
                let overrides = player_pose_transition_node_overrides(
                    gltf,
                    transition.from,
                    transition.to,
                    phase,
                    blend,
                )?;

                for (socket, joint) in player_articulated_joint_gap_pairs() {
                    let socket_translation =
                        world_node_translation_with_pose(gltf, socket, &overrides)?;
                    let joint_translation =
                        world_node_translation_with_pose(gltf, joint, &overrides)?;
                    let gap = distance3(socket_translation, joint_translation);
                    articulated_gap.observe_label(gap, socket, joint, transition.label, phase);
                    overall_articulated_gap.observe_label(
                        gap,
                        socket,
                        joint,
                        transition.label,
                        phase,
                    );
                }

                for (left, right) in player_joint_cover_mesh_pairs() {
                    let left_bounds = node_world_mesh_aabb_with_pose(gltf, left, &overrides)?;
                    let right_bounds = node_world_mesh_aabb_with_pose(gltf, right, &overrides)?;
                    let gap = left_bounds.separation_m(right_bounds);
                    joint_cover_gap.observe_label(gap, left, right, transition.label, phase);
                    overall_joint_cover_gap.observe_label(
                        gap,
                        left,
                        right,
                        transition.label,
                        phase,
                    );
                    let overlap_axes_m = left_bounds.overlap_axes_m(right_bounds);
                    let overlap_m = node_world_mesh_obb_with_pose(gltf, left, &overrides)?
                        .overlap_depth_m(node_world_mesh_obb_with_pose(gltf, right, &overrides)?);
                    joint_cover_overlap.observe_label(
                        overlap_m,
                        overlap_axes_m,
                        left,
                        right,
                        transition.label,
                        phase,
                    );
                    overall_joint_cover_overlap.observe_label(
                        overlap_m,
                        overlap_axes_m,
                        left,
                        right,
                        transition.label,
                        phase,
                    );
                }

                for (bridge, contact) in player_joint_bridge_mesh_pairs() {
                    let bridge_bounds = node_world_mesh_aabb_with_pose(gltf, bridge, &overrides)?;
                    let contact_bounds = node_world_mesh_aabb_with_pose(gltf, contact, &overrides)?;
                    let gap = bridge_bounds.separation_m(contact_bounds);
                    joint_bridge_gap.observe_label(gap, bridge, contact, transition.label, phase);
                    overall_joint_bridge_gap.observe_label(
                        gap,
                        bridge,
                        contact,
                        transition.label,
                        phase,
                    );
                    let overlap_axes_m = bridge_bounds.overlap_axes_m(contact_bounds);
                    let overlap_m = node_world_mesh_obb_with_pose(gltf, bridge, &overrides)?
                        .overlap_depth_m(node_world_mesh_obb_with_pose(gltf, contact, &overrides)?);
                    joint_bridge_overlap.observe_label(
                        overlap_m,
                        overlap_axes_m,
                        bridge,
                        contact,
                        transition.label,
                        phase,
                    );
                    overall_joint_bridge_overlap.observe_label(
                        overlap_m,
                        overlap_axes_m,
                        bridge,
                        contact,
                        transition.label,
                        phase,
                    );
                }

                for (left, right) in non_adjacent_pairs.iter().copied() {
                    let left_bounds = node_world_mesh_aabb_with_pose(gltf, left, &overrides)?;
                    let right_bounds = node_world_mesh_aabb_with_pose(gltf, right, &overrides)?;
                    let overlap_axes_m = left_bounds.overlap_axes_m(right_bounds);
                    let overlap_m = node_world_mesh_obb_with_pose(gltf, left, &overrides)?
                        .overlap_depth_m(node_world_mesh_obb_with_pose(gltf, right, &overrides)?);
                    non_adjacent_overlap.observe_label(
                        overlap_m,
                        overlap_axes_m,
                        left,
                        right,
                        transition.label,
                        phase,
                    );
                    overall_non_adjacent_overlap.observe_label(
                        overlap_m,
                        overlap_axes_m,
                        left,
                        right,
                        transition.label,
                        phase,
                    );
                }
            }
        }

        let within_thresholds = articulated_gap.max_gap_m
            <= PLAYER_POSE_MAX_ARTICULATED_JOINT_GAP_M
            && joint_cover_gap.max_gap_m <= PLAYER_POSE_MAX_JOINT_COVER_MESH_GAP_M
            && joint_cover_overlap.max_overlap_m <= PLAYER_POSE_MAX_JOINT_COVER_MESH_OVERLAP_M
            && joint_bridge_gap.max_gap_m <= PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_GAP_M
            && joint_bridge_overlap.max_overlap_m <= PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_OVERLAP_M
            && non_adjacent_overlap.max_overlap_m <= PLAYER_POSE_MAX_NON_ADJACENT_MESH_OVERLAP_M;
        if !within_thresholds {
            breach_count += 1;
        }

        transition_reports.push(json!({
            "transition": transition.label,
            "from_pose_intent": transition.from.intent().label(),
            "to_pose_intent": transition.to.intent().label(),
            "phase_count": phases.len(),
            "blend_count": blends.len(),
            "within_thresholds": within_thresholds,
            "articulated_joint_gap_max_m": articulated_gap.max_gap_m,
            "joint_cover_mesh_gap_max_m": joint_cover_gap.max_gap_m,
            "joint_cover_mesh_overlap_max_m": joint_cover_overlap.max_overlap_m,
            "joint_bridge_mesh_gap_max_m": joint_bridge_gap.max_gap_m,
            "joint_bridge_mesh_overlap_max_m": joint_bridge_overlap.max_overlap_m,
            "non_adjacent_mesh_overlap_max_m": non_adjacent_overlap.max_overlap_m,
            "articulated_joint_gap_worst_pair": articulated_gap.to_json(),
            "joint_cover_mesh_gap_worst_pair": joint_cover_gap.to_json(),
            "joint_cover_mesh_overlap_worst_pair": joint_cover_overlap.to_json(),
            "joint_bridge_mesh_gap_worst_pair": joint_bridge_gap.to_json(),
            "joint_bridge_mesh_overlap_worst_pair": joint_bridge_overlap.to_json(),
            "non_adjacent_mesh_overlap_worst_pair": non_adjacent_overlap.to_json(),
        }));
    }

    Some(json!({
        "schema": "nau_player_pose_transition_contact_audit.v1",
        "transition_count": transition_reports.len(),
        "phase_count": phases.len(),
        "blend_count": blends.len(),
        "breach_count": breach_count,
        "articulated_joint_gap_max_m": overall_articulated_gap.max_gap_m,
        "joint_cover_mesh_gap_max_m": overall_joint_cover_gap.max_gap_m,
        "joint_cover_mesh_overlap_max_m": overall_joint_cover_overlap.max_overlap_m,
        "joint_bridge_mesh_gap_max_m": overall_joint_bridge_gap.max_gap_m,
        "joint_bridge_mesh_overlap_max_m": overall_joint_bridge_overlap.max_overlap_m,
        "non_adjacent_mesh_overlap_max_m": overall_non_adjacent_overlap.max_overlap_m,
        "thresholds": {
            "articulated_joint_gap_max_m": PLAYER_POSE_MAX_ARTICULATED_JOINT_GAP_M,
            "joint_cover_mesh_gap_max_m": PLAYER_POSE_MAX_JOINT_COVER_MESH_GAP_M,
            "joint_cover_mesh_overlap_max_m": PLAYER_POSE_MAX_JOINT_COVER_MESH_OVERLAP_M,
            "joint_bridge_mesh_gap_max_m": PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_GAP_M,
            "joint_bridge_mesh_overlap_max_m": PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_OVERLAP_M,
            "non_adjacent_mesh_overlap_max_m": PLAYER_POSE_MAX_NON_ADJACENT_MESH_OVERLAP_M,
        },
        "articulated_joint_gap_worst_pair": overall_articulated_gap.to_json(),
        "joint_cover_mesh_gap_worst_pair": overall_joint_cover_gap.to_json(),
        "joint_cover_mesh_overlap_worst_pair": overall_joint_cover_overlap.to_json(),
        "joint_bridge_mesh_gap_worst_pair": overall_joint_bridge_gap.to_json(),
        "joint_bridge_mesh_overlap_worst_pair": overall_joint_bridge_overlap.to_json(),
        "non_adjacent_mesh_overlap_worst_pair": overall_non_adjacent_overlap.to_json(),
        "transitions": transition_reports,
    }))
}

fn player_pose_articulated_joint_gap_report(gltf: &Value) -> Option<JointGapReport> {
    let phases = player_pose_contact_phases();
    let mut report = JointGapReport::zero();

    for context in player_pose_mesh_overlap_contexts() {
        for phase in phases {
            let overrides = player_pose_node_overrides(gltf, context, phase)?;
            for (socket, joint) in player_articulated_joint_gap_pairs() {
                let socket_translation =
                    world_node_translation_with_pose(gltf, socket, &overrides)?;
                let joint_translation = world_node_translation_with_pose(gltf, joint, &overrides)?;
                report.observe(
                    distance3(socket_translation, joint_translation),
                    socket,
                    joint,
                    context.intent(),
                    phase,
                );
            }
        }
    }

    Some(report)
}

fn player_joint_cover_mesh_pairs() -> [(&'static str, &'static str); 13] {
    [
        ("Nau Neck Joint Cover", "Nau Skin Rounded Head"),
        ("Nau Left Shoulder Joint Cover", "Nau Left Suit Upper Arm"),
        ("Nau Right Shoulder Joint Cover", "Nau Right Suit Upper Arm"),
        (
            "Nau Left Elbow Joint Cover",
            "Nau Left Leather Forearm Wrap",
        ),
        (
            "Nau Right Elbow Joint Cover",
            "Nau Right Leather Forearm Wrap",
        ),
        ("Nau Left Wrist Joint Cover", "Nau Left Leather Hand Palm"),
        ("Nau Right Wrist Joint Cover", "Nau Right Leather Hand Palm"),
        ("Nau Left Hip Joint Cover", "Nau Left Suit Thigh Guard"),
        ("Nau Right Hip Joint Cover", "Nau Right Suit Thigh Guard"),
        (
            "Nau Left Knee Joint Cover",
            "Nau Left Suit Lower Leg Greave",
        ),
        (
            "Nau Right Knee Joint Cover",
            "Nau Right Suit Lower Leg Greave",
        ),
        ("Nau Left Ankle Joint Cover", "Nau Left Leather Boot Shell"),
        (
            "Nau Right Ankle Joint Cover",
            "Nau Right Leather Boot Shell",
        ),
    ]
}

fn player_joint_bridge_node_names() -> [&'static str; 12] {
    [
        "Nau Left Shoulder Bridge Sleeve",
        "Nau Right Shoulder Bridge Sleeve",
        "Nau Left Elbow Bridge Sleeve",
        "Nau Right Elbow Bridge Sleeve",
        "Nau Left Wrist Bridge Sleeve",
        "Nau Right Wrist Bridge Sleeve",
        "Nau Left Hip Bridge Sleeve",
        "Nau Right Hip Bridge Sleeve",
        "Nau Left Knee Bridge Sleeve",
        "Nau Right Knee Bridge Sleeve",
        "Nau Left Ankle Bridge Sleeve",
        "Nau Right Ankle Bridge Sleeve",
    ]
}

fn player_joint_bridge_mesh_pairs() -> [(&'static str, &'static str); 12] {
    [
        ("Nau Left Shoulder Bridge Sleeve", "Nau Left Suit Upper Arm"),
        (
            "Nau Right Shoulder Bridge Sleeve",
            "Nau Right Suit Upper Arm",
        ),
        (
            "Nau Left Elbow Bridge Sleeve",
            "Nau Left Leather Forearm Wrap",
        ),
        (
            "Nau Right Elbow Bridge Sleeve",
            "Nau Right Leather Forearm Wrap",
        ),
        ("Nau Left Wrist Bridge Sleeve", "Nau Left Leather Hand Palm"),
        (
            "Nau Right Wrist Bridge Sleeve",
            "Nau Right Leather Hand Palm",
        ),
        ("Nau Left Hip Bridge Sleeve", "Nau Left Suit Thigh Guard"),
        ("Nau Right Hip Bridge Sleeve", "Nau Right Suit Thigh Guard"),
        (
            "Nau Left Knee Bridge Sleeve",
            "Nau Left Suit Lower Leg Greave",
        ),
        (
            "Nau Right Knee Bridge Sleeve",
            "Nau Right Suit Lower Leg Greave",
        ),
        (
            "Nau Left Ankle Bridge Sleeve",
            "Nau Left Leather Boot Shell",
        ),
        (
            "Nau Right Ankle Bridge Sleeve",
            "Nau Right Leather Boot Shell",
        ),
    ]
}

fn player_joint_seam_node_names() -> [&'static str; 12] {
    [
        "Nau Left Seamless Shoulder Flex Cover",
        "Nau Right Seamless Shoulder Flex Cover",
        "Nau Left Seamless Elbow Flex Cover",
        "Nau Right Seamless Elbow Flex Cover",
        "Nau Left Seamless Wrist Flex Cover",
        "Nau Right Seamless Wrist Flex Cover",
        "Nau Left Seamless Hip Flex Cover",
        "Nau Right Seamless Hip Flex Cover",
        "Nau Left Seamless Knee Flex Cover",
        "Nau Right Seamless Knee Flex Cover",
        "Nau Left Seamless Ankle Flex Cover",
        "Nau Right Seamless Ankle Flex Cover",
    ]
}

fn player_joint_seam_mesh_pairs() -> [(&'static str, &'static str); 26] {
    [
        (
            "Nau Left Seamless Shoulder Flex Cover",
            "Nau Left Shoulder Joint Cover",
        ),
        (
            "Nau Left Seamless Shoulder Flex Cover",
            "Nau Left Suit Upper Arm",
        ),
        (
            "Nau Left Seamless Shoulder Flex Cover",
            "Nau Left Suit Deltoid Filler",
        ),
        (
            "Nau Right Seamless Shoulder Flex Cover",
            "Nau Right Shoulder Joint Cover",
        ),
        (
            "Nau Right Seamless Shoulder Flex Cover",
            "Nau Right Suit Upper Arm",
        ),
        (
            "Nau Right Seamless Shoulder Flex Cover",
            "Nau Right Suit Deltoid Filler",
        ),
        (
            "Nau Left Seamless Elbow Flex Cover",
            "Nau Left Elbow Joint Cover",
        ),
        (
            "Nau Left Seamless Elbow Flex Cover",
            "Nau Left Leather Forearm Wrap",
        ),
        (
            "Nau Right Seamless Elbow Flex Cover",
            "Nau Right Elbow Joint Cover",
        ),
        (
            "Nau Right Seamless Elbow Flex Cover",
            "Nau Right Leather Forearm Wrap",
        ),
        (
            "Nau Left Seamless Wrist Flex Cover",
            "Nau Left Wrist Joint Cover",
        ),
        (
            "Nau Left Seamless Wrist Flex Cover",
            "Nau Left Leather Hand Palm",
        ),
        (
            "Nau Right Seamless Wrist Flex Cover",
            "Nau Right Wrist Joint Cover",
        ),
        (
            "Nau Right Seamless Wrist Flex Cover",
            "Nau Right Leather Hand Palm",
        ),
        (
            "Nau Left Seamless Hip Flex Cover",
            "Nau Left Hip Joint Cover",
        ),
        (
            "Nau Left Seamless Hip Flex Cover",
            "Nau Left Suit Thigh Guard",
        ),
        (
            "Nau Right Seamless Hip Flex Cover",
            "Nau Right Hip Joint Cover",
        ),
        (
            "Nau Right Seamless Hip Flex Cover",
            "Nau Right Suit Thigh Guard",
        ),
        (
            "Nau Left Seamless Knee Flex Cover",
            "Nau Left Knee Joint Cover",
        ),
        (
            "Nau Left Seamless Knee Flex Cover",
            "Nau Left Suit Lower Leg Greave",
        ),
        (
            "Nau Right Seamless Knee Flex Cover",
            "Nau Right Knee Joint Cover",
        ),
        (
            "Nau Right Seamless Knee Flex Cover",
            "Nau Right Suit Lower Leg Greave",
        ),
        (
            "Nau Left Seamless Ankle Flex Cover",
            "Nau Left Ankle Joint Cover",
        ),
        (
            "Nau Left Seamless Ankle Flex Cover",
            "Nau Left Leather Boot Shell",
        ),
        (
            "Nau Right Seamless Ankle Flex Cover",
            "Nau Right Ankle Joint Cover",
        ),
        (
            "Nau Right Seamless Ankle Flex Cover",
            "Nau Right Leather Boot Shell",
        ),
    ]
}

fn player_surface_contact_pairs() -> Vec<PlayerSurfaceContactPair> {
    let mut pairs = Vec::with_capacity(PLAYER_SURFACE_CONTACT_EXPECTED_PAIR_COUNT as usize);
    for (left, right) in player_joint_cover_mesh_pairs() {
        pairs.push(PlayerSurfaceContactPair {
            category: "cover",
            left,
            right,
        });
    }
    for (left, right) in player_joint_bridge_mesh_pairs() {
        pairs.push(PlayerSurfaceContactPair {
            category: "bridge",
            left,
            right,
        });
    }
    for (left, right) in player_joint_seam_mesh_pairs() {
        pairs.push(PlayerSurfaceContactPair {
            category: "seam",
            left,
            right,
        });
    }
    for (left, right) in player_proximal_contact_mesh_pairs() {
        pairs.push(PlayerSurfaceContactPair {
            category: "proximal",
            left,
            right,
        });
    }
    pairs
}

fn player_proximal_contact_mesh_pairs() -> [(&'static str, &'static str); 94] {
    [
        ("Nau Skin Neck Column", "Nau Skin Rounded Head"),
        ("Nau Skin Neck Column", "Nau Neck Joint Cover"),
        ("Nau Suit Neck Collar Pad", "Nau Neck Joint Cover"),
        ("Nau Suit Neck Collar Pad", "Nau Suit Shoulder Yoke Plate"),
        (
            "Nau Suit Lower Rib Flex Lip",
            "Nau Suit Ribcage Soft Volume",
        ),
        (
            "Nau Suit Abdominal Flex Gasket",
            "Nau Suit Waist Soft Volume",
        ),
        (
            "Nau Left Suit Oblique Flex Connector",
            "Nau Suit Abdominal Flex Gasket",
        ),
        (
            "Nau Right Suit Oblique Flex Connector",
            "Nau Suit Abdominal Flex Gasket",
        ),
        (
            "Nau Left Suit Oblique Flex Connector",
            "Nau Suit Waist Soft Volume",
        ),
        (
            "Nau Right Suit Oblique Flex Connector",
            "Nau Suit Waist Soft Volume",
        ),
        (
            "Nau Left Shoulder Joint Cover",
            "Nau Suit Shoulder Yoke Plate",
        ),
        (
            "Nau Right Shoulder Joint Cover",
            "Nau Suit Shoulder Yoke Plate",
        ),
        (
            "Nau Left Shoulder Bridge Sleeve",
            "Nau Suit Shoulder Yoke Plate",
        ),
        (
            "Nau Right Shoulder Bridge Sleeve",
            "Nau Suit Shoulder Yoke Plate",
        ),
        (
            "Nau Left Suit Deltoid Filler",
            "Nau Suit Shoulder Yoke Plate",
        ),
        (
            "Nau Right Suit Deltoid Filler",
            "Nau Suit Shoulder Yoke Plate",
        ),
        ("Nau Left Shoulder Accent", "Nau Suit Shoulder Yoke Plate"),
        ("Nau Right Shoulder Accent", "Nau Suit Shoulder Yoke Plate"),
        (
            "Nau Left Suit Shoulder Chest Blend",
            "Nau Suit Shoulder Yoke Plate",
        ),
        (
            "Nau Right Suit Shoulder Chest Blend",
            "Nau Suit Shoulder Yoke Plate",
        ),
        (
            "Nau Left Suit Pectoral Soft Volume",
            "Nau Suit Ribcage Soft Volume",
        ),
        (
            "Nau Right Suit Pectoral Soft Volume",
            "Nau Suit Ribcage Soft Volume",
        ),
        (
            "Nau Left Suit Pectoral Soft Volume",
            "Nau Left Suit Shoulder Chest Blend",
        ),
        (
            "Nau Right Suit Pectoral Soft Volume",
            "Nau Right Suit Shoulder Chest Blend",
        ),
        (
            "Nau Left Suit Scapula Soft Volume",
            "Nau Suit Ribcage Soft Volume",
        ),
        (
            "Nau Right Suit Scapula Soft Volume",
            "Nau Suit Ribcage Soft Volume",
        ),
        (
            "Nau Left Suit Shoulder Root Blend",
            "Nau Left Suit Upper Arm",
        ),
        (
            "Nau Right Suit Shoulder Root Blend",
            "Nau Right Suit Upper Arm",
        ),
        (
            "Nau Left Suit Lat Shoulder Connector",
            "Nau Left Suit Shoulder Web Capsule",
        ),
        (
            "Nau Right Suit Lat Shoulder Connector",
            "Nau Right Suit Shoulder Web Capsule",
        ),
        (
            "Nau Left Suit Lat Shoulder Connector",
            "Nau Left Suit Upper Arm",
        ),
        (
            "Nau Right Suit Lat Shoulder Connector",
            "Nau Right Suit Upper Arm",
        ),
        (
            "Nau Left Suit Shoulder Torso Motion Cowl",
            "Nau Suit Shoulder Yoke Plate",
        ),
        (
            "Nau Right Suit Shoulder Torso Motion Cowl",
            "Nau Suit Shoulder Yoke Plate",
        ),
        (
            "Nau Left Suit Shoulder Torso Motion Cowl",
            "Nau Left Shoulder Joint Cover",
        ),
        (
            "Nau Right Suit Shoulder Torso Motion Cowl",
            "Nau Right Shoulder Joint Cover",
        ),
        (
            "Nau Left Suit Shoulder Arm Motion Cowl",
            "Nau Left Seamless Shoulder Flex Cover",
        ),
        (
            "Nau Right Suit Shoulder Arm Motion Cowl",
            "Nau Right Seamless Shoulder Flex Cover",
        ),
        (
            "Nau Left Suit Shoulder Arm Motion Cowl",
            "Nau Left Suit Upper Arm",
        ),
        (
            "Nau Right Suit Shoulder Arm Motion Cowl",
            "Nau Right Suit Upper Arm",
        ),
        (
            "Nau Left Suit Elbow Upper Motion Cowl",
            "Nau Left Suit Upper Arm",
        ),
        (
            "Nau Right Suit Elbow Upper Motion Cowl",
            "Nau Right Suit Upper Arm",
        ),
        (
            "Nau Left Suit Elbow Upper Motion Cowl",
            "Nau Left Elbow Joint Cover",
        ),
        (
            "Nau Right Suit Elbow Upper Motion Cowl",
            "Nau Right Elbow Joint Cover",
        ),
        (
            "Nau Left Suit Elbow Forearm Motion Cowl",
            "Nau Left Leather Forearm Wrap",
        ),
        (
            "Nau Right Suit Elbow Forearm Motion Cowl",
            "Nau Right Leather Forearm Wrap",
        ),
        (
            "Nau Left Suit Elbow Forearm Motion Cowl",
            "Nau Left Seamless Elbow Flex Cover",
        ),
        (
            "Nau Right Suit Elbow Forearm Motion Cowl",
            "Nau Right Seamless Elbow Flex Cover",
        ),
        ("Nau Left Hip Joint Cover", "Nau Suit Pelvis Hip Yoke"),
        ("Nau Right Hip Joint Cover", "Nau Suit Pelvis Hip Yoke"),
        (
            "Nau Left Seamless Hip Flex Cover",
            "Nau Suit Pelvis Hip Yoke",
        ),
        (
            "Nau Right Seamless Hip Flex Cover",
            "Nau Suit Pelvis Hip Yoke",
        ),
        ("Nau Left Suit Thigh Guard", "Nau Suit Pelvis Hip Yoke"),
        ("Nau Right Suit Thigh Guard", "Nau Suit Pelvis Hip Yoke"),
        ("Nau Left Suit Hip Root Blend", "Nau Left Suit Thigh Guard"),
        (
            "Nau Right Suit Hip Root Blend",
            "Nau Right Suit Thigh Guard",
        ),
        (
            "Nau Left Suit Glute Hip Connector",
            "Nau Left Suit Hip Web Capsule",
        ),
        (
            "Nau Right Suit Glute Hip Connector",
            "Nau Right Suit Hip Web Capsule",
        ),
        (
            "Nau Left Suit Glute Hip Connector",
            "Nau Left Suit Thigh Guard",
        ),
        (
            "Nau Right Suit Glute Hip Connector",
            "Nau Right Suit Thigh Guard",
        ),
        (
            "Nau Left Suit Hip Thigh Fairing",
            "Nau Left Suit Thigh Guard",
        ),
        (
            "Nau Right Suit Hip Thigh Fairing",
            "Nau Right Suit Thigh Guard",
        ),
        (
            "Nau Left Suit Hip Thigh Fairing",
            "Nau Left Seamless Hip Flex Cover",
        ),
        (
            "Nau Right Suit Hip Thigh Fairing",
            "Nau Right Seamless Hip Flex Cover",
        ),
        (
            "Nau Left Suit Hip Pelvis Motion Cowl",
            "Nau Suit Pelvis Hip Yoke",
        ),
        (
            "Nau Right Suit Hip Pelvis Motion Cowl",
            "Nau Suit Pelvis Hip Yoke",
        ),
        (
            "Nau Left Suit Hip Pelvis Motion Cowl",
            "Nau Left Suit Hip Web Capsule",
        ),
        (
            "Nau Right Suit Hip Pelvis Motion Cowl",
            "Nau Right Suit Hip Web Capsule",
        ),
        (
            "Nau Left Suit Hip Thigh Motion Cowl",
            "Nau Left Suit Thigh Guard",
        ),
        (
            "Nau Right Suit Hip Thigh Motion Cowl",
            "Nau Right Suit Thigh Guard",
        ),
        (
            "Nau Left Suit Hip Thigh Motion Cowl",
            "Nau Left Seamless Hip Flex Cover",
        ),
        (
            "Nau Right Suit Hip Thigh Motion Cowl",
            "Nau Right Seamless Hip Flex Cover",
        ),
        (
            "Nau Left Suit Knee Thigh Motion Cowl",
            "Nau Left Suit Thigh Guard",
        ),
        (
            "Nau Right Suit Knee Thigh Motion Cowl",
            "Nau Right Suit Thigh Guard",
        ),
        (
            "Nau Left Suit Knee Thigh Motion Cowl",
            "Nau Left Knee Joint Cover",
        ),
        (
            "Nau Right Suit Knee Thigh Motion Cowl",
            "Nau Right Knee Joint Cover",
        ),
        (
            "Nau Left Suit Knee Lower Motion Cowl",
            "Nau Left Suit Lower Leg Greave",
        ),
        (
            "Nau Right Suit Knee Lower Motion Cowl",
            "Nau Right Suit Lower Leg Greave",
        ),
        (
            "Nau Left Suit Knee Lower Motion Cowl",
            "Nau Left Seamless Knee Flex Cover",
        ),
        (
            "Nau Right Suit Knee Lower Motion Cowl",
            "Nau Right Seamless Knee Flex Cover",
        ),
        (
            "Nau Left Leather Wrist Palm Gusset",
            "Nau Left Leather Hand Palm",
        ),
        (
            "Nau Right Leather Wrist Palm Gusset",
            "Nau Right Leather Hand Palm",
        ),
        (
            "Nau Left Leather Index Finger Grip",
            "Nau Left Leather Index Finger Tip Pad",
        ),
        (
            "Nau Right Leather Index Finger Grip",
            "Nau Right Leather Index Finger Tip Pad",
        ),
        (
            "Nau Left Leather Finger Grip",
            "Nau Left Leather Middle Finger Tip Pad",
        ),
        (
            "Nau Right Leather Finger Grip",
            "Nau Right Leather Middle Finger Tip Pad",
        ),
        (
            "Nau Left Leather Ring Finger Grip",
            "Nau Left Leather Ring Finger Tip Pad",
        ),
        (
            "Nau Right Leather Ring Finger Grip",
            "Nau Right Leather Ring Finger Tip Pad",
        ),
        (
            "Nau Left Leather Pinky Finger Grip",
            "Nau Left Leather Pinky Finger Tip Pad",
        ),
        (
            "Nau Right Leather Pinky Finger Grip",
            "Nau Right Leather Pinky Finger Tip Pad",
        ),
        (
            "Nau Left Leather Thumb Grip",
            "Nau Left Leather Thumb Tip Pad",
        ),
        (
            "Nau Right Leather Thumb Grip",
            "Nau Right Leather Thumb Tip Pad",
        ),
        (
            "Nau Left Leather Ankle Boot Tongue",
            "Nau Left Leather Boot Shell",
        ),
        (
            "Nau Right Leather Ankle Boot Tongue",
            "Nau Right Leather Boot Shell",
        ),
    ]
}

fn player_joint_bridge_contact_audit(gltf: &Value) -> Option<Value> {
    let phases = player_pose_contact_phases();
    let contexts = player_pose_mesh_overlap_contexts();
    let mut bridge_reports = Vec::new();
    let mut overall_gap = MeshGapReport::zero();
    let mut overall_overlap = MeshOverlapReport::zero();
    let mut breach_count = 0_u64;

    for (bridge, contact) in player_joint_bridge_mesh_pairs() {
        let mut pair_gap = MeshGapReport::zero();
        let mut pair_overlap = MeshOverlapReport::zero();
        for context in contexts {
            for phase in phases {
                let overrides = player_pose_node_overrides(gltf, context, phase)?;
                let bridge_bounds = node_world_mesh_aabb_with_pose(gltf, bridge, &overrides)?;
                let contact_bounds = node_world_mesh_aabb_with_pose(gltf, contact, &overrides)?;
                let gap = bridge_bounds.separation_m(contact_bounds);
                pair_gap.observe(gap, bridge, contact, context.intent(), phase);
                overall_gap.observe(gap, bridge, contact, context.intent(), phase);
                let overlap_axes_m = bridge_bounds.overlap_axes_m(contact_bounds);
                let overlap_m = node_world_mesh_obb_with_pose(gltf, bridge, &overrides)?
                    .overlap_depth_m(node_world_mesh_obb_with_pose(gltf, contact, &overrides)?);
                pair_overlap.observe(
                    overlap_m,
                    overlap_axes_m,
                    bridge,
                    contact,
                    context.intent(),
                    phase,
                );
                overall_overlap.observe(
                    overlap_m,
                    overlap_axes_m,
                    bridge,
                    contact,
                    context.intent(),
                    phase,
                );
            }
        }
        let within_threshold = pair_gap.max_gap_m <= PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_GAP_M
            && pair_overlap.max_overlap_m <= PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_OVERLAP_M;
        if !within_threshold {
            breach_count += 1;
        }
        bridge_reports.push(json!({
            "bridge_node": bridge,
            "contact_node": contact,
            "max_gap_m": pair_gap.max_gap_m,
            "max_overlap_m": pair_overlap.max_overlap_m,
            "within_threshold": within_threshold,
            "worst_gap_pair": pair_gap.to_json(),
            "worst_overlap_pair": pair_overlap.to_json(),
        }));
    }

    Some(json!({
        "schema": "nau_player_joint_bridge_contact_audit.v1",
        "bridge_node_count": player_joint_bridge_node_names().len(),
        "pair_count": bridge_reports.len(),
        "pose_count": contexts.len(),
        "phase_count": phases.len(),
        "max_gap_m": overall_gap.max_gap_m,
        "max_overlap_m": overall_overlap.max_overlap_m,
        "breach_count": breach_count,
        "thresholds": {
            "joint_bridge_mesh_gap_max_m": PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_GAP_M,
            "joint_bridge_mesh_overlap_max_m": PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_OVERLAP_M,
        },
        "worst_gap_pair": overall_gap.to_json(),
        "worst_overlap_pair": overall_overlap.to_json(),
        "pairs": bridge_reports,
    }))
}

fn player_proximal_contact_audit(gltf: &Value) -> Option<Value> {
    let phases = player_pose_contact_phases();
    let contexts = player_pose_mesh_overlap_contexts();
    let transitions = player_pose_transition_contact_transitions();
    let blends = player_pose_transition_contact_blends();
    let mut pair_reports = Vec::new();
    let mut overall_gap = MeshGapReport::zero();
    let mut overall_overlap = MeshOverlapReport::zero();
    let mut breach_count = 0_u64;
    let samples_per_pair =
        contexts.len() * phases.len() + transitions.len() * phases.len() * blends.len();

    for (left, right) in player_proximal_contact_mesh_pairs() {
        let mut pair_gap = MeshGapReport::zero();
        let mut pair_overlap = MeshOverlapReport::zero();

        for context in contexts {
            for phase in phases {
                let overrides = player_pose_node_overrides(gltf, context, phase)?;
                observe_proximal_contact_sample(
                    gltf,
                    left,
                    right,
                    &overrides,
                    context.intent().label(),
                    phase,
                    &mut pair_gap,
                    &mut pair_overlap,
                    &mut overall_gap,
                    &mut overall_overlap,
                )?;
            }
        }

        for transition in transitions {
            for phase in phases {
                for blend in blends {
                    let overrides = player_pose_transition_node_overrides(
                        gltf,
                        transition.from,
                        transition.to,
                        phase,
                        blend,
                    )?;
                    observe_proximal_contact_sample(
                        gltf,
                        left,
                        right,
                        &overrides,
                        transition.label,
                        phase,
                        &mut pair_gap,
                        &mut pair_overlap,
                        &mut overall_gap,
                        &mut overall_overlap,
                    )?;
                }
            }
        }

        let within_threshold = pair_gap.max_gap_m <= PLAYER_POSE_MAX_SURFACE_CONTACT_DISTANCE_M
            && pair_overlap.max_overlap_m <= PLAYER_POSE_MAX_PROXIMAL_CONTACT_MESH_OVERLAP_M;
        if !within_threshold {
            breach_count += 1;
        }
        pair_reports.push(json!({
            "left_node": left,
            "right_node": right,
            "max_gap_m": pair_gap.max_gap_m,
            "max_overlap_m": pair_overlap.max_overlap_m,
            "within_threshold": within_threshold,
            "worst_gap_pair": pair_gap.to_json(),
            "worst_overlap_pair": pair_overlap.to_json(),
        }));
    }

    Some(json!({
        "schema": "nau_player_proximal_contact_audit.v1",
        "pair_count": pair_reports.len(),
        "pose_count": contexts.len(),
        "phase_count": phases.len(),
        "transition_count": transitions.len(),
        "blend_count": blends.len(),
        "samples_per_pair": samples_per_pair,
        "max_gap_m": overall_gap.max_gap_m,
        "max_overlap_m": overall_overlap.max_overlap_m,
        "breach_count": breach_count,
        "thresholds": {
            "proximal_contact_gap_max_m": PLAYER_POSE_MAX_SURFACE_CONTACT_DISTANCE_M,
            "proximal_contact_overlap_max_m": PLAYER_POSE_MAX_PROXIMAL_CONTACT_MESH_OVERLAP_M,
        },
        "worst_gap_pair": overall_gap.to_json(),
        "worst_overlap_pair": overall_overlap.to_json(),
        "pairs": pair_reports,
    }))
}

#[allow(clippy::too_many_arguments)]
fn observe_proximal_contact_sample(
    gltf: &Value,
    left: &'static str,
    right: &'static str,
    overrides: &[PoseNodeOverride],
    label: &'static str,
    phase: f32,
    pair_gap: &mut MeshGapReport,
    pair_overlap: &mut MeshOverlapReport,
    overall_gap: &mut MeshGapReport,
    overall_overlap: &mut MeshOverlapReport,
) -> Option<()> {
    let left_bounds = node_world_mesh_aabb_with_pose(gltf, left, overrides)?;
    let right_bounds = node_world_mesh_aabb_with_pose(gltf, right, overrides)?;
    let gap = left_bounds.separation_m(right_bounds);
    pair_gap.observe_label(gap, left, right, label, phase);
    overall_gap.observe_label(gap, left, right, label, phase);
    let overlap_axes_m = left_bounds.overlap_axes_m(right_bounds);
    let overlap_m = node_world_mesh_obb_with_pose(gltf, left, overrides)?
        .overlap_depth_m(node_world_mesh_obb_with_pose(gltf, right, overrides)?);
    pair_overlap.observe_label(overlap_m, overlap_axes_m, left, right, label, phase);
    overall_overlap.observe_label(overlap_m, overlap_axes_m, left, right, label, phase);
    Some(())
}

fn player_joint_seam_contact_audit(gltf: &Value) -> Option<Value> {
    let phases = player_pose_contact_phases();
    let contexts = player_pose_mesh_overlap_contexts();
    let transitions = player_pose_transition_contact_transitions();
    let blends = player_pose_transition_contact_blends();
    let mut seam_reports = Vec::new();
    let mut overall_gap = MeshGapReport::zero();
    let mut overall_min_overlap = MeshMinOverlapReport::zero();
    let mut overall_max_overlap = MeshOverlapReport::zero();
    let mut breach_count = 0_u64;
    let samples_per_pair =
        contexts.len() * phases.len() + transitions.len() * phases.len() * blends.len();

    for (seam, contact) in player_joint_seam_mesh_pairs() {
        let mut pair_gap = MeshGapReport::zero();
        let mut pair_min_overlap = MeshMinOverlapReport::zero();
        let mut pair_max_overlap = MeshOverlapReport::zero();

        for context in contexts {
            for phase in phases {
                let overrides = player_pose_node_overrides(gltf, context, phase)?;
                observe_joint_seam_sample(
                    gltf,
                    seam,
                    contact,
                    &overrides,
                    context.intent().label(),
                    phase,
                    &mut pair_gap,
                    &mut pair_min_overlap,
                    &mut pair_max_overlap,
                    &mut overall_gap,
                    &mut overall_min_overlap,
                    &mut overall_max_overlap,
                )?;
            }
        }

        for transition in transitions {
            for phase in phases {
                for blend in blends {
                    let overrides = player_pose_transition_node_overrides(
                        gltf,
                        transition.from,
                        transition.to,
                        phase,
                        blend,
                    )?;
                    observe_joint_seam_sample(
                        gltf,
                        seam,
                        contact,
                        &overrides,
                        transition.label,
                        phase,
                        &mut pair_gap,
                        &mut pair_min_overlap,
                        &mut pair_max_overlap,
                        &mut overall_gap,
                        &mut overall_min_overlap,
                        &mut overall_max_overlap,
                    )?;
                }
            }
        }

        let within_threshold = pair_gap.max_gap_m <= PLAYER_POSE_MAX_JOINT_SEAM_MESH_GAP_M
            && pair_min_overlap.value() >= PLAYER_POSE_MIN_JOINT_SEAM_MESH_OVERLAP_M
            && pair_max_overlap.max_overlap_m <= PLAYER_POSE_MAX_JOINT_SEAM_MESH_OVERLAP_M;
        if !within_threshold {
            breach_count += 1;
        }
        seam_reports.push(json!({
            "seam_node": seam,
            "contact_node": contact,
            "max_gap_m": pair_gap.max_gap_m,
            "min_overlap_m": pair_min_overlap.value(),
            "max_overlap_m": pair_max_overlap.max_overlap_m,
            "within_threshold": within_threshold,
            "worst_gap_pair": pair_gap.to_json(),
            "worst_min_overlap_pair": pair_min_overlap.to_json(),
            "worst_max_overlap_pair": pair_max_overlap.to_json(),
        }));
    }

    Some(json!({
        "schema": "nau_player_joint_seam_contact_audit.v1",
        "seam_node_count": player_joint_seam_node_names().len(),
        "pair_count": seam_reports.len(),
        "pose_count": contexts.len(),
        "phase_count": phases.len(),
        "transition_count": transitions.len(),
        "blend_count": blends.len(),
        "samples_per_pair": samples_per_pair,
        "max_gap_m": overall_gap.max_gap_m,
        "min_overlap_m": overall_min_overlap.value(),
        "max_overlap_m": overall_max_overlap.max_overlap_m,
        "breach_count": breach_count,
        "thresholds": {
            "joint_seam_mesh_gap_max_m": PLAYER_POSE_MAX_JOINT_SEAM_MESH_GAP_M,
            "joint_seam_mesh_overlap_min_m": PLAYER_POSE_MIN_JOINT_SEAM_MESH_OVERLAP_M,
            "joint_seam_mesh_overlap_max_m": PLAYER_POSE_MAX_JOINT_SEAM_MESH_OVERLAP_M,
        },
        "worst_gap_pair": overall_gap.to_json(),
        "worst_min_overlap_pair": overall_min_overlap.to_json(),
        "worst_max_overlap_pair": overall_max_overlap.to_json(),
        "pairs": seam_reports,
    }))
}

#[allow(clippy::too_many_arguments)]
fn observe_joint_seam_sample(
    gltf: &Value,
    seam: &'static str,
    contact: &'static str,
    overrides: &[PoseNodeOverride],
    label: &'static str,
    phase: f32,
    pair_gap: &mut MeshGapReport,
    pair_min_overlap: &mut MeshMinOverlapReport,
    pair_max_overlap: &mut MeshOverlapReport,
    overall_gap: &mut MeshGapReport,
    overall_min_overlap: &mut MeshMinOverlapReport,
    overall_max_overlap: &mut MeshOverlapReport,
) -> Option<()> {
    let seam_bounds = node_world_mesh_aabb_with_pose(gltf, seam, overrides)?;
    let contact_bounds = node_world_mesh_aabb_with_pose(gltf, contact, overrides)?;
    let gap = seam_bounds.separation_m(contact_bounds);
    pair_gap.observe_label(gap, seam, contact, label, phase);
    overall_gap.observe_label(gap, seam, contact, label, phase);
    let overlap_axes_m = seam_bounds.overlap_axes_m(contact_bounds);
    let overlap_m = node_world_mesh_obb_with_pose(gltf, seam, overrides)?
        .overlap_depth_m(node_world_mesh_obb_with_pose(gltf, contact, overrides)?);
    pair_min_overlap.observe_label(overlap_m, seam, contact, label, phase);
    overall_min_overlap.observe_label(overlap_m, seam, contact, label, phase);
    pair_max_overlap.observe_label(overlap_m, overlap_axes_m, seam, contact, label, phase);
    overall_max_overlap.observe_label(overlap_m, overlap_axes_m, seam, contact, label, phase);
    Some(())
}

fn player_pose_surface_contact_audit(gltf: &Value) -> Option<Value> {
    let phases = player_pose_contact_phases();
    let contexts = player_pose_mesh_overlap_contexts();
    let transitions = player_pose_transition_contact_transitions();
    let blends = player_pose_transition_contact_blends();
    let pairs = player_surface_contact_pairs();
    let buffers = embedded_gltf_buffers(gltf)?;
    let samples_per_pair =
        contexts.len() * phases.len() + transitions.len() * phases.len() * blends.len();
    let mut pair_reports = Vec::new();
    let mut overall = SurfaceContactDistanceReport::zero();
    let mut overall_projected_gap = ProjectedContactGapReport::zero();
    let mut breach_count = 0_u64;

    for pair in pairs.iter().copied() {
        let mut pair_report = SurfaceContactDistanceReport::zero();
        let mut pair_projected_gap = ProjectedContactGapReport::zero();

        for context in contexts.iter().copied() {
            for phase in phases {
                let overrides = player_pose_node_overrides(gltf, context, phase)?;
                let contact = player_pose_surface_contact_sample(gltf, pair, &overrides, &buffers)?;
                pair_report.observe_label(
                    contact.distance_m,
                    pair,
                    context.intent().label(),
                    phase,
                );
                overall.observe_label(contact.distance_m, pair, context.intent().label(), phase);
                observe_projected_contact_gap(
                    gltf,
                    pair,
                    &overrides,
                    context.intent().label(),
                    phase,
                    &mut pair_projected_gap,
                    &mut overall_projected_gap,
                )?;
            }
        }

        for transition in transitions.iter().copied() {
            for phase in phases {
                for blend in blends {
                    let overrides = player_pose_transition_node_overrides(
                        gltf,
                        transition.from,
                        transition.to,
                        phase,
                        blend,
                    )?;
                    let contact =
                        player_pose_surface_contact_sample(gltf, pair, &overrides, &buffers)?;
                    pair_report.observe_label(contact.distance_m, pair, transition.label, phase);
                    overall.observe_label(contact.distance_m, pair, transition.label, phase);
                    observe_projected_contact_gap(
                        gltf,
                        pair,
                        &overrides,
                        transition.label,
                        phase,
                        &mut pair_projected_gap,
                        &mut overall_projected_gap,
                    )?;
                }
            }
        }

        let within_threshold = pair_report.max_distance_m
            <= PLAYER_POSE_MAX_SURFACE_CONTACT_DISTANCE_M
            && pair_projected_gap.max_gap_m <= PLAYER_POSE_MAX_PROJECTED_CONTACT_GAP_M;
        if !within_threshold {
            breach_count += 1;
        }
        pair_reports.push(json!({
            "category": pair.category,
            "left_node": pair.left,
            "right_node": pair.right,
            "max_distance_m": pair_report.max_distance_m,
            "max_projected_gap_m": pair_projected_gap.max_gap_m,
            "within_threshold": within_threshold,
            "worst_sample": pair_report.to_json(),
            "worst_projected_gap": pair_projected_gap.to_json(),
        }));
    }

    Some(json!({
        "schema": "nau_player_pose_surface_contact_audit.v1",
        "pair_count": pair_reports.len(),
        "pose_count": contexts.len(),
        "phase_count": phases.len(),
        "transition_count": transitions.len(),
        "blend_count": blends.len(),
        "samples_per_pair": samples_per_pair,
        "sample_points_per_mesh_limit": PLAYER_SURFACE_CONTACT_SAMPLE_LIMIT,
        "max_distance_m": overall.max_distance_m,
        "max_projected_gap_m": overall_projected_gap.max_gap_m,
        "breach_count": breach_count,
        "thresholds": {
            "surface_contact_distance_max_m": PLAYER_POSE_MAX_SURFACE_CONTACT_DISTANCE_M,
            "projected_contact_gap_max_m": PLAYER_POSE_MAX_PROJECTED_CONTACT_GAP_M,
        },
        "worst_pair": overall.to_json(),
        "worst_projected_gap": overall_projected_gap.to_json(),
        "pairs": pair_reports,
    }))
}

fn observe_projected_contact_gap(
    gltf: &Value,
    pair: PlayerSurfaceContactPair,
    overrides: &[PoseNodeOverride],
    label: &'static str,
    phase: f32,
    pair_report: &mut ProjectedContactGapReport,
    overall_report: &mut ProjectedContactGapReport,
) -> Option<()> {
    let left_bounds = node_world_mesh_aabb_with_pose(gltf, pair.left, overrides)?;
    let right_bounds = node_world_mesh_aabb_with_pose(gltf, pair.right, overrides)?;
    let (gap_m, view) = projected_contact_gap_m(left_bounds, right_bounds);
    pair_report.observe_label(gap_m, view, pair, label, phase);
    overall_report.observe_label(gap_m, view, pair, label, phase);
    Some(())
}

fn projected_contact_gap_m(left: Aabb3, right: Aabb3) -> (f64, PlayerPosePreviewView) {
    [
        (
            projected_rect_gap_m(left, right, PlayerPosePreviewView::Front),
            PlayerPosePreviewView::Front,
        ),
        (
            projected_rect_gap_m(left, right, PlayerPosePreviewView::Side),
            PlayerPosePreviewView::Side,
        ),
        (
            projected_rect_gap_m(left, right, PlayerPosePreviewView::Top),
            PlayerPosePreviewView::Top,
        ),
    ]
    .into_iter()
    .max_by(|left, right| {
        left.0
            .partial_cmp(&right.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
    .expect("projected contact gap views should not be empty")
}

fn projected_rect_gap_m(left: Aabb3, right: Aabb3, view: PlayerPosePreviewView) -> f64 {
    let (left_min_u, left_max_u, left_min_v, left_max_v) = projected_aabb_rect(left, view);
    let (right_min_u, right_max_u, right_min_v, right_max_v) = projected_aabb_rect(right, view);
    let gap_u = axis_separation_m(left_min_u, left_max_u, right_min_u, right_max_u);
    let gap_v = axis_separation_m(left_min_v, left_max_v, right_min_v, right_max_v);
    Vec2::new(gap_u, gap_v).length() as f64
}

fn projected_aabb_rect(bounds: Aabb3, view: PlayerPosePreviewView) -> (f32, f32, f32, f32) {
    match view {
        PlayerPosePreviewView::Front => (bounds.min.x, bounds.max.x, bounds.min.y, bounds.max.y),
        PlayerPosePreviewView::Rear => (-bounds.max.x, -bounds.min.x, bounds.min.y, bounds.max.y),
        PlayerPosePreviewView::Side => (bounds.min.z, bounds.max.z, bounds.min.y, bounds.max.y),
        PlayerPosePreviewView::Top => (bounds.min.x, bounds.max.x, bounds.min.z, bounds.max.z),
    }
}

fn player_pose_surface_contact_sample(
    gltf: &Value,
    pair: PlayerSurfaceContactPair,
    overrides: &[PoseNodeOverride],
    buffers: &[Vec<u8>],
) -> Option<ClosestSurfacePoints> {
    let left = node_world_surface_points_with_pose(gltf, pair.left, overrides, buffers)?;
    let right = node_world_surface_points_with_pose(gltf, pair.right, overrides, buffers)?;
    closest_surface_points(&left, &right)
}

fn player_pose_joint_cover_mesh_gap_max_m(gltf: &Value) -> Option<f64> {
    player_pose_joint_cover_mesh_gap_report(gltf).map(|report| report.max_gap_m)
}

fn player_pose_joint_cover_mesh_gap_report(gltf: &Value) -> Option<MeshGapReport> {
    let phases = player_pose_contact_phases();
    let mut report = MeshGapReport::zero();

    for context in player_pose_mesh_overlap_contexts() {
        for phase in phases {
            let overrides = player_pose_node_overrides(gltf, context, phase)?;
            for (left, right) in player_joint_cover_mesh_pairs() {
                let left_bounds = node_world_mesh_aabb_with_pose(gltf, left, &overrides)?;
                let right_bounds = node_world_mesh_aabb_with_pose(gltf, right, &overrides)?;
                report.observe(
                    left_bounds.separation_m(right_bounds),
                    left,
                    right,
                    context.intent(),
                    phase,
                );
            }
        }
    }

    Some(report)
}

fn player_pose_joint_cover_mesh_overlap_max_m(gltf: &Value) -> Option<f64> {
    player_pose_joint_cover_mesh_overlap_report(gltf).map(|report| report.max_overlap_m)
}

fn player_pose_joint_cover_mesh_overlap_report(gltf: &Value) -> Option<MeshOverlapReport> {
    let phases = player_pose_contact_phases();
    let mut report = MeshOverlapReport::zero();

    for context in player_pose_mesh_overlap_contexts() {
        for phase in phases {
            let overrides = player_pose_node_overrides(gltf, context, phase)?;
            for (left, right) in player_joint_cover_mesh_pairs() {
                let left_bounds = node_world_mesh_aabb_with_pose(gltf, left, &overrides)?;
                let right_bounds = node_world_mesh_aabb_with_pose(gltf, right, &overrides)?;
                let overlap_axes_m = left_bounds.overlap_axes_m(right_bounds);
                let overlap_m = node_world_mesh_obb_with_pose(gltf, left, &overrides)?
                    .overlap_depth_m(node_world_mesh_obb_with_pose(gltf, right, &overrides)?);
                report.observe(
                    overlap_m,
                    overlap_axes_m,
                    left,
                    right,
                    context.intent(),
                    phase,
                );
            }
        }
    }

    Some(report)
}

fn player_rest_mesh_bounds_present(gltf: &Value) -> bool {
    player_rest_mesh_overlap_node_names()
        .into_iter()
        .all(|node_name| node_world_mesh_aabb(gltf, node_name).is_some())
}

fn player_rest_non_adjacent_mesh_overlap_max_m(gltf: &Value) -> Option<f64> {
    mesh_overlap_max_m(gltf, &player_rest_non_adjacent_mesh_overlap_pairs())
}

fn player_rest_shoulder_mesh_overlap_max_m(gltf: &Value) -> Option<f64> {
    mesh_overlap_max_m(
        gltf,
        &[
            ("Nau Left Suit Upper Arm", "Nau Suit Armored Torso Shell"),
            ("Nau Right Suit Upper Arm", "Nau Suit Armored Torso Shell"),
        ],
    )
}

fn player_limb_anatomy_detail_node_names() -> &'static [&'static str] {
    &[
        "Nau Skin Neck Column",
        "Nau Suit Neck Collar Pad",
        "Nau Suit Ribcage Soft Volume",
        "Nau Suit Lower Rib Flex Lip",
        "Nau Suit Waist Soft Volume",
        "Nau Suit Abdominal Flex Gasket",
        "Nau Left Suit Oblique Flex Connector",
        "Nau Right Suit Oblique Flex Connector",
        "Nau Left Suit Pectoral Soft Volume",
        "Nau Right Suit Pectoral Soft Volume",
        "Nau Left Suit Scapula Soft Volume",
        "Nau Right Suit Scapula Soft Volume",
        "Nau Left Suit Bicep Volume",
        "Nau Right Suit Bicep Volume",
        "Nau Left Suit Tricep Sweep",
        "Nau Right Suit Tricep Sweep",
        "Nau Left Leather Forearm Tendon Strap",
        "Nau Right Leather Forearm Tendon Strap",
        "Nau Left Leather Finger Web Bridge",
        "Nau Right Leather Finger Web Bridge",
        "Nau Left Suit Shoulder Root Blend",
        "Nau Right Suit Shoulder Root Blend",
        "Nau Left Suit Shoulder Chest Blend",
        "Nau Right Suit Shoulder Chest Blend",
        "Nau Left Suit Axilla Blend",
        "Nau Right Suit Axilla Blend",
        "Nau Left Suit Shoulder Web Capsule",
        "Nau Right Suit Shoulder Web Capsule",
        "Nau Left Suit Lat Shoulder Connector",
        "Nau Right Suit Lat Shoulder Connector",
        "Nau Left Suit Shoulder Torso Motion Cowl",
        "Nau Right Suit Shoulder Torso Motion Cowl",
        "Nau Left Suit Shoulder Arm Motion Cowl",
        "Nau Right Suit Shoulder Arm Motion Cowl",
        "Nau Left Suit Elbow Upper Motion Cowl",
        "Nau Right Suit Elbow Upper Motion Cowl",
        "Nau Left Suit Elbow Forearm Motion Cowl",
        "Nau Right Suit Elbow Forearm Motion Cowl",
        "Nau Left Suit Outer Thigh Sweep",
        "Nau Right Suit Outer Thigh Sweep",
        "Nau Left Suit Inner Thigh Sweep",
        "Nau Right Suit Inner Thigh Sweep",
        "Nau Left Suit Hip Root Blend",
        "Nau Right Suit Hip Root Blend",
        "Nau Left Suit Hip Inguinal Blend",
        "Nau Right Suit Hip Inguinal Blend",
        "Nau Left Suit Hip Web Capsule",
        "Nau Right Suit Hip Web Capsule",
        "Nau Left Suit Glute Hip Connector",
        "Nau Right Suit Glute Hip Connector",
        "Nau Left Suit Hip Thigh Fairing",
        "Nau Right Suit Hip Thigh Fairing",
        "Nau Left Suit Hip Pelvis Motion Cowl",
        "Nau Right Suit Hip Pelvis Motion Cowl",
        "Nau Left Suit Hip Thigh Motion Cowl",
        "Nau Right Suit Hip Thigh Motion Cowl",
        "Nau Left Suit Calf Volume",
        "Nau Right Suit Calf Volume",
        "Nau Left Suit Shin Ridge",
        "Nau Right Suit Shin Ridge",
        "Nau Left Suit Knee Tendon Strap",
        "Nau Right Suit Knee Tendon Strap",
        "Nau Left Suit Knee Thigh Motion Cowl",
        "Nau Right Suit Knee Thigh Motion Cowl",
        "Nau Left Suit Knee Lower Motion Cowl",
        "Nau Right Suit Knee Lower Motion Cowl",
        "Nau Left Leather Wrist Palm Gusset",
        "Nau Right Leather Wrist Palm Gusset",
        "Nau Left Leather Heel Tendon Guard",
        "Nau Right Leather Heel Tendon Guard",
        "Nau Left Leather Boot Instep Plate",
        "Nau Right Leather Boot Instep Plate",
        "Nau Left Leather Outer Boot Side Guard",
        "Nau Right Leather Outer Boot Side Guard",
        "Nau Left Leather Boot Arch Rib",
        "Nau Right Leather Boot Arch Rib",
        "Nau Left Leather Lace Cross Strap A",
        "Nau Right Leather Lace Cross Strap A",
        "Nau Left Leather Lace Cross Strap B",
        "Nau Right Leather Lace Cross Strap B",
        "Nau Left Leather Ankle Boot Tongue",
        "Nau Right Leather Ankle Boot Tongue",
    ]
}

fn player_limb_anatomy_detail_audit(gltf: &Value) -> Option<Value> {
    let mut samples = Vec::new();
    let mut missing = Vec::new();
    let mut min_major_extent_m = f64::INFINITY;
    let mut present_node_count = 0_u64;

    for node in player_limb_anatomy_detail_node_names() {
        let Some(bounds) = node_world_mesh_obb_with_pose(gltf, node, &[]) else {
            missing.push(*node);
            continue;
        };
        let major_extent_m = (bounds.half_extents.x as f64)
            .max(bounds.half_extents.y as f64)
            .max(bounds.half_extents.z as f64)
            * 2.0;
        min_major_extent_m = min_major_extent_m.min(major_extent_m);
        present_node_count += 1;
        samples.push(json!({
            "node": node,
            "major_extent_m": major_extent_m,
            "half_extents_m": [
                bounds.half_extents.x,
                bounds.half_extents.y,
                bounds.half_extents.z,
            ],
        }));
    }

    Some(json!({
        "schema": "nau_player_limb_anatomy_detail_audit.v1",
        "expected_node_count": player_limb_anatomy_detail_node_names().len(),
        "present_node_count": present_node_count,
        "missing_count": missing.len(),
        "missing": missing,
        "min_major_extent_m": if min_major_extent_m.is_finite() {
            min_major_extent_m
        } else {
            0.0
        },
        "samples": samples,
    }))
}

fn player_finger_grip_node_names() -> [&'static str; 10] {
    [
        "Nau Left Leather Index Finger Grip",
        "Nau Left Leather Finger Grip",
        "Nau Left Leather Ring Finger Grip",
        "Nau Left Leather Pinky Finger Grip",
        "Nau Left Leather Thumb Grip",
        "Nau Right Leather Index Finger Grip",
        "Nau Right Leather Finger Grip",
        "Nau Right Leather Ring Finger Grip",
        "Nau Right Leather Pinky Finger Grip",
        "Nau Right Leather Thumb Grip",
    ]
}

fn player_finger_grip_length_range_m(gltf: &Value) -> Option<(f64, f64)> {
    player_finger_grip_node_names()
        .into_iter()
        .map(|node| {
            let bounds = node_world_mesh_obb_with_pose(gltf, node, &[])?;
            Some(bounds.half_extents.y as f64 * 2.0)
        })
        .collect::<Option<Vec<_>>>()
        .map(|lengths| {
            lengths
                .into_iter()
                .fold((f64::INFINITY, 0.0_f64), |(min, max), length| {
                    (min.min(length), max.max(length))
                })
        })
}

fn player_boot_sole_length_min_m(gltf: &Value) -> Option<f64> {
    ["Nau Left Leather Boot Sole", "Nau Right Leather Boot Sole"]
        .into_iter()
        .map(|node| {
            let bounds = node_world_mesh_obb_with_pose(gltf, node, &[])?;
            Some(bounds.half_extents.z as f64 * 2.0)
        })
        .collect::<Option<Vec<_>>>()
        .map(|lengths| lengths.into_iter().fold(f64::INFINITY, f64::min))
}

fn player_pose_non_adjacent_mesh_overlap_max_m(gltf: &Value) -> Option<f64> {
    player_pose_non_adjacent_mesh_overlap_report(gltf).map(|report| report.max_overlap_m)
}

fn player_pose_non_adjacent_mesh_overlap_report(gltf: &Value) -> Option<MeshOverlapReport> {
    let pairs = player_rest_non_adjacent_mesh_overlap_pairs();
    let phases = player_pose_contact_phases();
    let mut report = MeshOverlapReport::zero();

    for context in player_pose_mesh_overlap_contexts() {
        for phase in phases {
            let overrides = player_pose_node_overrides(gltf, context, phase)?;
            for (left, right) in pairs.iter().copied() {
                let left_bounds = node_world_mesh_aabb_with_pose(gltf, left, &overrides)?;
                let right_bounds = node_world_mesh_aabb_with_pose(gltf, right, &overrides)?;
                let overlap_axes_m = left_bounds.overlap_axes_m(right_bounds);
                let overlap_m = node_world_mesh_obb_with_pose(gltf, left, &overrides)?
                    .overlap_depth_m(node_world_mesh_obb_with_pose(gltf, right, &overrides)?);
                report.observe(
                    overlap_m,
                    overlap_axes_m,
                    left,
                    right,
                    context.intent(),
                    phase,
                );
            }
        }
    }

    Some(report)
}

fn mesh_overlap_max_m(gltf: &Value, pairs: &[(&'static str, &'static str)]) -> Option<f64> {
    pairs
        .iter()
        .map(|(left, right)| {
            Some(
                node_world_mesh_aabb(gltf, left)?
                    .overlap_depth_m(node_world_mesh_aabb(gltf, right)?),
            )
        })
        .collect::<Option<Vec<_>>>()
        .map(|overlaps| overlaps.into_iter().fold(0.0, f64::max))
}

fn player_pose_mesh_overlap_contexts() -> [PlayerPoseContext; 10] {
    [
        PlayerPoseContext::new(
            FlightMode::Grounded,
            Vec3::ZERO,
            FlightInput::default(),
            0.0,
        )
        .with_resolved_intent(PlayerPoseIntent::GroundedIdle),
        PlayerPoseContext::new(
            FlightMode::Grounded,
            Vec3::new(0.0, 0.0, -4.5),
            FlightInput::default(),
            0.0,
        )
        .with_resolved_intent(PlayerPoseIntent::GroundedWalk),
        PlayerPoseContext::new(
            FlightMode::Grounded,
            Vec3::new(0.0, 0.0, -10.0),
            FlightInput::default(),
            0.0,
        )
        .with_resolved_intent(PlayerPoseIntent::GroundedRun),
        PlayerPoseContext::new(
            FlightMode::Launching,
            Vec3::new(0.0, 24.0, -18.0),
            FlightInput::default(),
            80.0,
        )
        .with_resolved_intent(PlayerPoseIntent::Launching),
        PlayerPoseContext::new(
            FlightMode::Airborne,
            Vec3::new(0.0, -22.0, -24.0),
            FlightInput::default(),
            80.0,
        )
        .with_resolved_intent(PlayerPoseIntent::Falling),
        PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -4.0, -38.0),
            FlightInput::default(),
            80.0,
        )
        .with_resolved_intent(PlayerPoseIntent::Gliding),
        PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -31.0, -49.0),
            FlightInput {
                dive: true,
                ..FlightInput::default()
            },
            80.0,
        )
        .with_resolved_intent(PlayerPoseIntent::Diving),
        PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -6.0, -20.0),
            FlightInput {
                backward: true,
                ..FlightInput::default()
            },
            80.0,
        )
        .with_resolved_intent(PlayerPoseIntent::AirBrake),
        PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -18.0, -24.0),
            FlightInput::default(),
            5.0,
        )
        .with_resolved_intent(PlayerPoseIntent::LandingAnticipation),
        PlayerPoseContext::new(
            FlightMode::Grounded,
            Vec3::new(0.0, 0.0, -8.0),
            FlightInput::default(),
            0.0,
        )
        .with_landing_recovery(0.36, 16.0)
        .with_resolved_intent(PlayerPoseIntent::LandingRecovery),
    ]
}

fn player_pose_transition_contact_transitions() -> [PlayerPoseTransition; 9] {
    let [
        _grounded_idle,
        _grounded_walk,
        _grounded_run,
        launching,
        falling,
        gliding,
        diving,
        air_brake,
        landing_anticipation,
        landing_recovery,
    ] = player_pose_mesh_overlap_contexts();

    [
        PlayerPoseTransition {
            label: "launching_to_gliding",
            from: launching,
            to: gliding,
        },
        PlayerPoseTransition {
            label: "falling_to_gliding",
            from: falling,
            to: gliding,
        },
        PlayerPoseTransition {
            label: "gliding_to_falling",
            from: gliding,
            to: falling,
        },
        PlayerPoseTransition {
            label: "gliding_to_diving",
            from: gliding,
            to: diving,
        },
        PlayerPoseTransition {
            label: "diving_to_gliding",
            from: diving,
            to: gliding,
        },
        PlayerPoseTransition {
            label: "gliding_to_air_brake",
            from: gliding,
            to: air_brake,
        },
        PlayerPoseTransition {
            label: "air_brake_to_gliding",
            from: air_brake,
            to: gliding,
        },
        PlayerPoseTransition {
            label: "gliding_to_landing_anticipation",
            from: gliding,
            to: landing_anticipation,
        },
        PlayerPoseTransition {
            label: "landing_anticipation_to_landing_recovery",
            from: landing_anticipation,
            to: landing_recovery,
        },
    ]
}

fn player_pose_node_overrides(
    gltf: &Value,
    context: PlayerPoseContext,
    phase: f32,
) -> Option<Vec<PoseNodeOverride>> {
    PLAYER_RUNTIME_POSE_NODE_ROLES
        .iter()
        .map(|(node_name, role)| {
            let part = character_part_from_node(gltf, node_name, *role)?;
            Some(PoseNodeOverride {
                node_name,
                pose: part_pose_with_context(&part, context, phase),
            })
        })
        .collect()
}

fn player_pose_transition_node_overrides(
    gltf: &Value,
    from: PlayerPoseContext,
    to: PlayerPoseContext,
    phase: f32,
    blend: f32,
) -> Option<Vec<PoseNodeOverride>> {
    PLAYER_RUNTIME_POSE_NODE_ROLES
        .iter()
        .map(|(node_name, role)| {
            let part = character_part_from_node(gltf, node_name, *role)?;
            let from_pose = part_pose_with_context(&part, from, phase);
            let to_pose = part_pose_with_context(&part, to, phase);
            Some(PoseNodeOverride {
                node_name,
                pose: blend_part_pose(from_pose, to_pose, blend),
            })
        })
        .collect()
}

fn blend_part_pose(from: PartPose, to: PartPose, blend: f32) -> PartPose {
    let blend = blend.clamp(0.0, 1.0);
    PartPose {
        translation: from.translation.lerp(to.translation, blend),
        rotation: from.rotation.slerp(to.rotation, blend),
        visibility: if blend < 0.5 {
            from.visibility
        } else {
            to.visibility
        },
    }
}

fn character_part_from_node(
    gltf: &Value,
    node_name: &str,
    role: CharacterPartRole,
) -> Option<CharacterPart> {
    let nodes = gltf.get("nodes")?.as_array()?;
    let index = node_index(gltf, node_name)?;
    let (translation, rotation) = node_local_translation_rotation_by_index(nodes, index)?;
    Some(CharacterPart::new(role, translation, rotation))
}

fn player_rest_mesh_overlap_node_names() -> Vec<&'static str> {
    let mut names = Vec::new();
    names.extend_from_slice(PLAYER_REST_TORSO_BLOCKING_MESH_NODES);
    names.extend_from_slice(PLAYER_REST_LEFT_ARM_MESH_NODES);
    names.extend_from_slice(PLAYER_REST_RIGHT_ARM_MESH_NODES);
    names.extend_from_slice(PLAYER_REST_LEFT_LEG_MESH_NODES);
    names.extend_from_slice(PLAYER_REST_RIGHT_LEG_MESH_NODES);
    names.sort_unstable();
    names.dedup();
    names
}

fn player_rest_non_adjacent_mesh_overlap_pairs() -> Vec<(&'static str, &'static str)> {
    let mut pairs = Vec::new();
    append_cross_pairs(
        &mut pairs,
        PLAYER_REST_LEFT_DISTAL_ARM_MESH_NODES,
        PLAYER_REST_TORSO_BLOCKING_MESH_NODES,
    );
    append_cross_pairs(
        &mut pairs,
        PLAYER_REST_RIGHT_DISTAL_ARM_MESH_NODES,
        PLAYER_REST_TORSO_BLOCKING_MESH_NODES,
    );
    append_cross_pairs(
        &mut pairs,
        PLAYER_REST_LEFT_DISTAL_LEG_MESH_NODES,
        PLAYER_REST_TORSO_BLOCKING_MESH_NODES,
    );
    append_cross_pairs(
        &mut pairs,
        PLAYER_REST_RIGHT_DISTAL_LEG_MESH_NODES,
        PLAYER_REST_TORSO_BLOCKING_MESH_NODES,
    );
    append_cross_pairs(
        &mut pairs,
        PLAYER_REST_LEFT_ARM_MESH_NODES,
        PLAYER_REST_RIGHT_ARM_MESH_NODES,
    );
    append_cross_pairs(
        &mut pairs,
        PLAYER_REST_LEFT_DISTAL_LEG_MESH_NODES,
        PLAYER_REST_RIGHT_DISTAL_LEG_MESH_NODES,
    );
    append_cross_pairs(
        &mut pairs,
        PLAYER_REST_LEFT_DISTAL_ARM_MESH_NODES,
        PLAYER_REST_LEFT_LEG_MESH_NODES,
    );
    append_cross_pairs(
        &mut pairs,
        PLAYER_REST_RIGHT_DISTAL_ARM_MESH_NODES,
        PLAYER_REST_RIGHT_LEG_MESH_NODES,
    );
    append_cross_pairs(
        &mut pairs,
        PLAYER_REST_LEFT_DISTAL_ARM_MESH_NODES,
        PLAYER_REST_RIGHT_LEG_MESH_NODES,
    );
    append_cross_pairs(
        &mut pairs,
        PLAYER_REST_RIGHT_DISTAL_ARM_MESH_NODES,
        PLAYER_REST_LEFT_LEG_MESH_NODES,
    );
    pairs
}

fn append_cross_pairs(
    pairs: &mut Vec<(&'static str, &'static str)>,
    left: &'static [&'static str],
    right: &'static [&'static str],
) {
    pairs.extend(
        left.iter()
            .flat_map(|left| right.iter().map(move |right| (*left, *right))),
    );
}

fn player_pose_shape_audit() -> Value {
    let falling_context = PlayerPoseContext::new(
        FlightMode::Airborne,
        Vec3::new(0.0, -22.0, -24.0),
        FlightInput::default(),
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::Falling);
    let gliding_context = PlayerPoseContext::new(
        FlightMode::Gliding,
        Vec3::new(0.0, -4.0, -38.0),
        FlightInput::default(),
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::Gliding);
    let launch_context = PlayerPoseContext::new(
        FlightMode::Launching,
        Vec3::new(0.0, 8.0, -20.0),
        FlightInput::default(),
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::Launching);
    let dive_context = PlayerPoseContext::new(
        FlightMode::Gliding,
        Vec3::new(0.0, -28.0, -42.0),
        FlightInput {
            dive: true,
            ..Default::default()
        },
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::Diving);
    let air_brake_context = PlayerPoseContext::new(
        FlightMode::Gliding,
        Vec3::new(0.0, -5.0, 16.0),
        FlightInput {
            backward: true,
            ..Default::default()
        },
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::AirBrake);
    let landing_context = PlayerPoseContext::new(
        FlightMode::Gliding,
        Vec3::new(0.0, -18.0, -24.0),
        FlightInput::default(),
        5.0,
    )
    .with_resolved_intent(PlayerPoseIntent::LandingAnticipation);
    let landing_recovery_context = PlayerPoseContext::new(
        FlightMode::Grounded,
        Vec3::new(0.0, 0.0, -8.0),
        FlightInput::default(),
        0.0,
    )
    .with_landing_recovery(0.36, 16.0)
    .with_resolved_intent(PlayerPoseIntent::LandingRecovery);

    let falling = pose_readability_metrics(falling_context, 0.0);
    let launch = pose_readability_metrics(launch_context, 0.0);
    let dive = pose_readability_metrics(dive_context, 0.0);
    let landing_recovery = pose_readability_metrics(landing_recovery_context, 0.0);
    let hips_part = CharacterPart::new(CharacterPartRole::Hips, Vec3::ZERO, Quat::IDENTITY);
    let torso_part = CharacterPart::new(CharacterPartRole::Torso, Vec3::ZERO, Quat::IDENTITY);
    let falling_hips = part_pose_with_context(&hips_part, falling_context, 0.0);
    let falling_torso = part_pose_with_context(&torso_part, falling_context, 0.0);
    let dive_hips = part_pose_with_context(&hips_part, dive_context, 0.0);
    let dive_torso = part_pose_with_context(&torso_part, dive_context, 0.0);
    let glider_launch = glider_traversal_pose(launch_context, 0.0);
    let glider_dive = glider_traversal_pose(dive_context, 0.0);
    let max_connected_limb_translation_m = max_connected_limb_translation_m(&[
        falling_context,
        gliding_context,
        dive_context,
        air_brake_context,
        landing_context,
        landing_recovery_context,
    ]);

    json!({
        "falling_key_pose_readability_score": falling.key_pose_readability_score,
        "falling_torso_pitch_degrees": falling.torso_pitch_degrees,
        "falling_hips_pitch_degrees": falling_hips.rotation.angle_between(Quat::IDENTITY).to_degrees(),
        "falling_torso_local_pitch_degrees": falling_torso.rotation.angle_between(Quat::IDENTITY).to_degrees(),
        "falling_arm_spread_degrees": falling.arm_spread_degrees,
        "dive_key_pose_readability_score": dive.key_pose_readability_score,
        "dive_torso_pitch_degrees": dive.torso_pitch_degrees,
        "dive_hips_pitch_degrees": dive_hips.rotation.angle_between(Quat::IDENTITY).to_degrees(),
        "dive_torso_local_pitch_degrees": dive_torso.rotation.angle_between(Quat::IDENTITY).to_degrees(),
        "dive_arm_spread_degrees": dive.arm_spread_degrees,
        "dive_leg_tuck_degrees": dive.leg_tuck_degrees,
        "landing_recovery_key_pose_readability_score": landing_recovery.key_pose_readability_score,
        "landing_recovery_flip_degrees": landing_recovery.landing_recovery_flip_degrees,
        "max_connected_limb_translation_m": max_connected_limb_translation_m,
        "launch_key_pose_readability_score": launch.key_pose_readability_score,
        "launch_overhead_arm_score": launch.launch_overhead_arm_score,
        "glider_launch_deployment": glider_deployment_for_mode(FlightMode::Launching),
        "glider_launch_response_degrees": glider_launch.response_degrees(),
        "glider_launch_motion_m": glider_launch.motion_m(),
        "glider_glide_deployment": glider_deployment_for_mode(FlightMode::Gliding),
        "glider_grounded_deployment": glider_deployment_for_mode(FlightMode::Grounded),
        "glider_dive_release_deployment": glider_deployment_for_context(dive_context),
        "glider_dive_response_degrees": glider_dive.response_degrees(),
        "glider_dive_motion_m": glider_dive.motion_m(),
    })
}

fn max_connected_limb_translation_m(contexts: &[PlayerPoseContext]) -> f64 {
    let limbs = [
        CharacterPart::new(
            CharacterPartRole::Arm(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::Arm(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::Forearm(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::Forearm(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::Hand(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::Hand(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::Leg(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::Leg(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::LowerLeg(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::LowerLeg(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::Foot(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::Foot(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
    ];

    contexts
        .iter()
        .flat_map(|context| {
            limbs
                .iter()
                .map(|limb| part_pose_with_context(limb, *context, 0.0))
        })
        .map(|pose| pose.translation.length() as f64)
        .fold(0.0, f64::max)
}

fn player_bank_clip_motion_is_distinct(gltf: &Value) -> bool {
    let Some(glide) = animation_signature(gltf, "Glide_Loop") else {
        return false;
    };
    let Some(bank_left) = animation_signature(gltf, "Bank_Left") else {
        return false;
    };
    let Some(bank_right) = animation_signature(gltf, "Bank_Right") else {
        return false;
    };

    bank_left.len() >= 4
        && bank_right.len() >= 4
        && bank_left != glide
        && bank_right != glide
        && bank_left != bank_right
}

fn player_launch_glide_land_clip_motion_is_distinct(gltf: &Value) -> bool {
    let Some(launch) = animation_signature(gltf, "Launch_Start") else {
        return false;
    };
    let Some(glide) = animation_signature(gltf, "Glide_Loop") else {
        return false;
    };
    let Some(land) = animation_signature(gltf, "Land") else {
        return false;
    };

    !launch.is_empty()
        && !glide.is_empty()
        && !land.is_empty()
        && launch != glide
        && launch != land
        && glide != land
}

fn player_grounded_locomotion_clip_motion_is_distinct(gltf: &Value) -> bool {
    let Some(idle) = animation_signature(gltf, "Idle_Loop") else {
        return false;
    };
    let Some(walk) = animation_signature(gltf, "Walk_Fwd_Loop") else {
        return false;
    };
    let Some(run) = animation_signature(gltf, "Run_Fwd_Loop") else {
        return false;
    };

    !idle.is_empty()
        && walk.len() >= 4
        && run.len() >= 4
        && idle != walk
        && idle != run
        && walk != run
}

fn player_fall_clip_motion_is_distinct(gltf: &Value) -> bool {
    let Some(fall) = animation_signature(gltf, "Fall_Loop") else {
        return false;
    };
    let Some(glide) = animation_signature(gltf, "Glide_Loop") else {
        return false;
    };
    let Some(air_brake) = animation_signature(gltf, "Air_Brake") else {
        return false;
    };
    let Some(land) = animation_signature(gltf, "Land") else {
        return false;
    };

    fall.len() >= 4 && fall != glide && fall != air_brake && fall != land
}

fn animation_target_node_names(gltf: &Value, animation_name: &str) -> Option<Vec<String>> {
    let animation = gltf
        .get("animations")
        .and_then(Value::as_array)?
        .iter()
        .find(|animation| {
            animation
                .get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| name == animation_name)
        })?;
    let channels = animation.get("channels").and_then(Value::as_array)?;
    channels
        .iter()
        .map(|channel| {
            let node = channel.get("target")?.get("node").and_then(Value::as_u64)? as usize;
            node_name(gltf, node).map(str::to_owned)
        })
        .collect()
}

fn animation_signature(gltf: &Value, animation_name: &str) -> Option<Vec<String>> {
    let accessors = gltf.get("accessors").and_then(Value::as_array)?;
    let animation = gltf
        .get("animations")
        .and_then(Value::as_array)?
        .iter()
        .find(|animation| {
            animation
                .get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| name == animation_name)
        })?;
    let samplers = animation.get("samplers").and_then(Value::as_array)?;
    let channels = animation.get("channels").and_then(Value::as_array)?;

    channels
        .iter()
        .map(|channel| {
            let sampler_index = channel.get("sampler").and_then(Value::as_u64)? as usize;
            let output_accessor = samplers
                .get(sampler_index)?
                .get("output")
                .and_then(Value::as_u64)? as usize;
            let accessor = accessors.get(output_accessor)?;
            let target = channel.get("target")?;
            let node = target.get("node").and_then(Value::as_u64)?;
            let path = target.get("path").and_then(Value::as_str)?;
            let min = accessor.get("min").unwrap_or(&Value::Null);
            let max = accessor.get("max").unwrap_or(&Value::Null);
            Some(format!("{node}:{path}:{}:{}", min, max))
        })
        .collect()
}

fn array_len(value: &Value, key: &str) -> u64 {
    value
        .get(key)
        .and_then(Value::as_array)
        .map_or(0, |values| values.len() as u64)
}

fn symmetric_node_half_width_m(gltf: &Value, left_name: &str, right_name: &str) -> Option<f64> {
    let left_x = world_node_translation(gltf, left_name)?[0];
    let right_x = world_node_translation(gltf, right_name)?[0];

    if left_x < 0.0 && right_x > 0.0 {
        Some((-left_x).min(right_x))
    } else {
        None
    }
}

fn world_node_translation(gltf: &Value, node_name: &str) -> Option<[f64; 3]> {
    let translation = world_node_transform(gltf, node_name)?.transform_point3(Vec3::ZERO);
    Some([
        translation.x as f64,
        translation.y as f64,
        translation.z as f64,
    ])
}

fn world_node_translation_with_pose(
    gltf: &Value,
    node_name: &str,
    overrides: &[PoseNodeOverride],
) -> Option<[f64; 3]> {
    let translation =
        world_node_transform_with_pose(gltf, node_name, overrides)?.transform_point3(Vec3::ZERO);
    Some([
        translation.x as f64,
        translation.y as f64,
        translation.z as f64,
    ])
}

fn world_node_transform(gltf: &Value, node_name: &str) -> Option<Mat4> {
    let nodes = gltf.get("nodes")?.as_array()?;
    let index = node_index(gltf, node_name)?;
    let parents = parent_indices(nodes);
    let mut cursor = Some(index);
    let mut chain = Vec::new();
    while let Some(node_index) = cursor {
        chain.push(node_index);
        cursor = parents[node_index];
    }
    chain
        .into_iter()
        .rev()
        .try_fold(Mat4::IDENTITY, |transform, node_index| {
            Some(transform * node_local_transform_by_index(nodes, node_index)?)
        })
}

fn node_world_mesh_aabb(gltf: &Value, node_name: &str) -> Option<Aabb3> {
    let nodes = gltf.get("nodes")?.as_array()?;
    let node_index = node_index(gltf, node_name)?;
    let mesh_index = nodes.get(node_index)?.get("mesh").and_then(Value::as_u64)? as usize;
    let mesh = gltf.get("meshes")?.as_array()?.get(mesh_index)?;
    Some(mesh_local_aabb(gltf, mesh)?.transformed(world_node_transform(gltf, node_name)?))
}

fn node_world_mesh_aabb_with_pose(
    gltf: &Value,
    node_name: &str,
    overrides: &[PoseNodeOverride],
) -> Option<Aabb3> {
    let nodes = gltf.get("nodes")?.as_array()?;
    let node_index = node_index(gltf, node_name)?;
    let mesh_index = nodes.get(node_index)?.get("mesh").and_then(Value::as_u64)? as usize;
    let mesh = gltf.get("meshes")?.as_array()?.get(mesh_index)?;
    Some(
        mesh_local_aabb(gltf, mesh)?
            .transformed(world_node_transform_with_pose(gltf, node_name, overrides)?),
    )
}

fn node_world_mesh_obb_with_pose(
    gltf: &Value,
    node_name: &str,
    overrides: &[PoseNodeOverride],
) -> Option<Obb3> {
    let nodes = gltf.get("nodes")?.as_array()?;
    let node_index = node_index(gltf, node_name)?;
    let mesh_index = nodes.get(node_index)?.get("mesh").and_then(Value::as_u64)? as usize;
    let mesh = gltf.get("meshes")?.as_array()?.get(mesh_index)?;
    Some(
        mesh_local_aabb(gltf, mesh)?
            .transformed_obb(world_node_transform_with_pose(gltf, node_name, overrides)?),
    )
}

fn world_node_transform_with_pose(
    gltf: &Value,
    node_name: &str,
    overrides: &[PoseNodeOverride],
) -> Option<Mat4> {
    let nodes = gltf.get("nodes")?.as_array()?;
    let index = node_index(gltf, node_name)?;
    let parents = parent_indices(nodes);
    let mut cursor = Some(index);
    let mut chain = Vec::new();
    while let Some(node_index) = cursor {
        chain.push(node_index);
        cursor = parents[node_index];
    }
    chain
        .into_iter()
        .rev()
        .try_fold(Mat4::IDENTITY, |transform, node_index| {
            Some(transform * node_local_transform_by_index_with_pose(nodes, node_index, overrides)?)
        })
}

fn mesh_local_aabb(gltf: &Value, mesh: &Value) -> Option<Aabb3> {
    let accessors = gltf.get("accessors")?.as_array()?;
    let primitives = mesh.get("primitives")?.as_array()?;
    primitives
        .iter()
        .map(|primitive| {
            let position_accessor_index = primitive
                .get("attributes")?
                .get("POSITION")
                .and_then(Value::as_u64)? as usize;
            accessor_aabb(accessors.get(position_accessor_index)?)
        })
        .collect::<Option<Vec<_>>>()
        .map(|bounds| {
            bounds
                .into_iter()
                .reduce(|mut aggregate, bounds| {
                    aggregate.include_aabb(bounds);
                    aggregate
                })
                .expect("mesh with primitives should have at least one bounds")
        })
}

fn accessor_aabb(accessor: &Value) -> Option<Aabb3> {
    let min = vec3_field(accessor.get("min")?)?;
    let max = vec3_field(accessor.get("max")?)?;
    Some(Aabb3::from_min_max(min, max))
}

fn vec3_field(value: &Value) -> Option<Vec3> {
    let values = value.as_array()?;
    Some(Vec3::new(
        values.first()?.as_f64()? as f32,
        values.get(1)?.as_f64()? as f32,
        values.get(2)?.as_f64()? as f32,
    ))
}

fn node_local_transform_by_index(nodes: &[Value], index: usize) -> Option<Mat4> {
    let node = nodes.get(index)?;
    if let Some(matrix) = node.get("matrix").and_then(Value::as_array) {
        let matrix = <[f32; 16]>::try_from(
            matrix
                .iter()
                .map(|value| value.as_f64().map(|value| value as f32))
                .collect::<Option<Vec<_>>>()?,
        )
        .ok()?;
        return Some(Mat4::from_cols_array(&matrix));
    }

    let translation = if let Some(translation) = node.get("translation").and_then(Value::as_array) {
        Vec3::new(
            translation.first()?.as_f64()? as f32,
            translation.get(1)?.as_f64()? as f32,
            translation.get(2)?.as_f64()? as f32,
        )
    } else {
        Vec3::ZERO
    };
    let rotation = if let Some(rotation) = node.get("rotation").and_then(Value::as_array) {
        Quat::from_xyzw(
            rotation.first()?.as_f64()? as f32,
            rotation.get(1)?.as_f64()? as f32,
            rotation.get(2)?.as_f64()? as f32,
            rotation.get(3)?.as_f64()? as f32,
        )
    } else {
        Quat::IDENTITY
    };
    let scale = if let Some(scale) = node.get("scale").and_then(Value::as_array) {
        Vec3::new(
            scale.first()?.as_f64()? as f32,
            scale.get(1)?.as_f64()? as f32,
            scale.get(2)?.as_f64()? as f32,
        )
    } else {
        Vec3::ONE
    };

    Some(Mat4::from_scale_rotation_translation(
        scale,
        rotation,
        translation,
    ))
}

fn node_local_transform_by_index_with_pose(
    nodes: &[Value],
    index: usize,
    overrides: &[PoseNodeOverride],
) -> Option<Mat4> {
    let Some(override_pose) = pose_override_by_index(nodes, index, overrides) else {
        return node_local_transform_by_index(nodes, index);
    };
    Some(Mat4::from_scale_rotation_translation(
        node_local_scale_by_index(nodes, index)?,
        override_pose.pose.rotation,
        override_pose.pose.translation,
    ))
}

fn pose_override_by_index(
    nodes: &[Value],
    index: usize,
    overrides: &[PoseNodeOverride],
) -> Option<PoseNodeOverride> {
    let name = nodes.get(index)?.get("name").and_then(Value::as_str)?;
    overrides
        .iter()
        .copied()
        .find(|override_pose| override_pose.node_name == name)
}

fn node_local_translation_rotation_by_index(nodes: &[Value], index: usize) -> Option<(Vec3, Quat)> {
    let node = nodes.get(index)?;
    if node.get("matrix").and_then(Value::as_array).is_some() {
        let (_scale, rotation, translation) =
            node_local_transform_by_index(nodes, index)?.to_scale_rotation_translation();
        return Some((translation, rotation));
    }
    let translation = if let Some(translation) = node.get("translation").and_then(Value::as_array) {
        Vec3::new(
            translation.first()?.as_f64()? as f32,
            translation.get(1)?.as_f64()? as f32,
            translation.get(2)?.as_f64()? as f32,
        )
    } else {
        Vec3::ZERO
    };
    let rotation = if let Some(rotation) = node.get("rotation").and_then(Value::as_array) {
        Quat::from_xyzw(
            rotation.first()?.as_f64()? as f32,
            rotation.get(1)?.as_f64()? as f32,
            rotation.get(2)?.as_f64()? as f32,
            rotation.get(3)?.as_f64()? as f32,
        )
    } else {
        Quat::IDENTITY
    };
    Some((translation, rotation))
}

fn node_local_scale_by_index(nodes: &[Value], index: usize) -> Option<Vec3> {
    let node = nodes.get(index)?;
    if node.get("matrix").and_then(Value::as_array).is_some() {
        let (scale, _rotation, _translation) =
            node_local_transform_by_index(nodes, index)?.to_scale_rotation_translation();
        return Some(scale);
    }
    if let Some(scale) = node.get("scale").and_then(Value::as_array) {
        Some(Vec3::new(
            scale.first()?.as_f64()? as f32,
            scale.get(1)?.as_f64()? as f32,
            scale.get(2)?.as_f64()? as f32,
        ))
    } else {
        Some(Vec3::ONE)
    }
}

fn node_index(gltf: &Value, node_name: &str) -> Option<usize> {
    gltf.get("nodes")?
        .as_array()?
        .iter()
        .position(|node| node.get("name").and_then(Value::as_str) == Some(node_name))
}

fn node_name(gltf: &Value, index: usize) -> Option<&str> {
    gltf.get("nodes")?
        .as_array()?
        .get(index)?
        .get("name")
        .and_then(Value::as_str)
}

fn node_is_direct_child(gltf: &Value, parent_name: &str, child_name: &str) -> bool {
    let Some(nodes) = gltf.get("nodes").and_then(Value::as_array) else {
        return false;
    };
    let Some(parent) = node_index(gltf, parent_name) else {
        return false;
    };
    let Some(child) = node_index(gltf, child_name) else {
        return false;
    };
    nodes
        .get(parent)
        .and_then(|node| node.get("children"))
        .and_then(Value::as_array)
        .is_some_and(|children| {
            children
                .iter()
                .any(|candidate| candidate.as_u64() == Some(child as u64))
        })
}

fn parent_indices(nodes: &[Value]) -> Vec<Option<usize>> {
    let mut parents = vec![None; nodes.len()];
    for (parent, node) in nodes.iter().enumerate() {
        let Some(children) = node.get("children").and_then(Value::as_array) else {
            continue;
        };
        for child in children {
            if let Some(child) = child.as_u64().map(|child| child as usize)
                && child < parents.len()
            {
                parents[child] = Some(parent);
            }
        }
    }
    parents
}

fn distance3(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn checks_passed(checks: &[Value]) -> bool {
    checks.iter().all(|check| {
        check
            .get("passed")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    })
}

fn number_field(value: &Value, key: &str) -> f64 {
    value.get(key).and_then(Value::as_f64).unwrap_or(f64::NAN)
}

fn check_bool(name: &'static str, passed: bool, unit: &'static str) -> Value {
    json!({
        "name": name,
        "passed": passed,
        "value": if passed { 1 } else { 0 },
        "comparator": "==",
        "threshold": 1,
        "unit": unit,
    })
}

fn check_at_least_u64(name: &'static str, value: u64, threshold: u64, unit: &'static str) -> Value {
    json!({
        "name": name,
        "passed": value >= threshold,
        "value": value,
        "comparator": ">=",
        "threshold": threshold,
        "unit": unit,
    })
}

fn check_at_least_f64(name: &'static str, value: f64, threshold: f64, unit: &'static str) -> Value {
    json!({
        "name": name,
        "passed": value >= threshold,
        "value": value,
        "comparator": ">=",
        "threshold": threshold,
        "unit": unit,
    })
}

fn check_at_most_f64(name: &'static str, value: f64, threshold: f64, unit: &'static str) -> Value {
    json!({
        "name": name,
        "passed": value <= threshold,
        "value": value,
        "comparator": "<=",
        "threshold": threshold,
        "unit": unit,
    })
}

fn check_eq_f64(name: &'static str, value: f64, threshold: f64, unit: &'static str) -> Value {
    json!({
        "name": name,
        "passed": (value - threshold).abs() <= f64::EPSILON,
        "value": value,
        "comparator": "==",
        "threshold": threshold,
        "unit": unit,
    })
}

fn check_eq_u64(name: &'static str, value: u64, threshold: u64, unit: &'static str) -> Value {
    json!({
        "name": name,
        "passed": value == threshold,
        "value": value,
        "comparator": "==",
        "threshold": threshold,
        "unit": unit,
    })
}

fn check_eq_str(name: &'static str, value: &str, threshold: &str, unit: &'static str) -> Value {
    json!({
        "name": name,
        "passed": value == threshold,
        "value": value,
        "comparator": "==",
        "threshold": threshold,
        "unit": unit,
    })
}

fn kind_name(kind: VisualAssetKind) -> &'static str {
    match kind {
        VisualAssetKind::PlayerCharacter => "player_character",
        VisualAssetKind::Glider => "glider",
        VisualAssetKind::IslandTerrain => "island_terrain",
        VisualAssetKind::IslandFoliage => "island_foliage",
        VisualAssetKind::IslandRock => "island_rock",
        VisualAssetKind::IslandWater => "island_water",
        VisualAssetKind::RouteMarker => "route_marker",
        VisualAssetKind::WeatherLayer => "weather_layer",
        VisualAssetKind::DistantImpostor => "distant_impostor",
    }
}

fn residency_name(residency: VisualAssetResidency) -> &'static str {
    match residency {
        VisualAssetResidency::Always => "always",
        VisualAssetResidency::StreamWindow => "stream_window",
        VisualAssetResidency::NearLod => "near_lod",
        VisualAssetResidency::FarLod => "far_lod",
        VisualAssetResidency::Weather => "weather",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_name_fragments_can_match_nodes_or_meshes() {
        let gltf = json!({
            "nodes": [
                {"name": "Readable Landing Soil Strip"},
                {"name": "World Root"}
            ],
            "meshes": [
                {"name": "Authored Cliff Skirt"},
                {"name": "Underside Rock Mass"}
            ]
        });
        let names = named_components(&gltf);

        assert!(has_name_fragment(&names, "landing"));
        assert!(has_name_fragment(&names, "cliff"));
        assert!(has_name_fragment(&names, "underside"));
        assert!(!has_name_fragment(&names, "missing route ring"));
    }

    #[test]
    fn nau_fixture_metadata_uses_registry_contract_names() {
        let metadata = json!({
            "schema": NAU_FIXTURE_SCHEMA,
            "asset_kind": "island_terrain",
            "asset_label": "island terrain kit",
            "residency": "stream_window",
            "license": NAU_FIXTURE_LICENSE
        });

        assert_eq!(nau_metadata_str(&metadata, "schema"), NAU_FIXTURE_SCHEMA);
        assert_eq!(
            kind_name(VisualAssetKind::IslandTerrain),
            nau_metadata_str(&metadata, "asset_kind")
        );
        assert_eq!(
            residency_name(VisualAssetResidency::StreamWindow),
            nau_metadata_str(&metadata, "residency")
        );
    }

    #[test]
    fn world_node_transform_applies_parent_rotation() {
        let rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2).to_array();
        let gltf = json!({
            "nodes": [
                {"name": "root", "rotation": rotation, "children": [1]},
                {"name": "child", "translation": [1.0, 0.0, 0.0]}
            ]
        });

        let translation = world_node_translation(&gltf, "child").expect("child translation");

        assert!(translation[0].abs() < 0.0001);
        assert!((translation[1] - 1.0).abs() < 0.0001);
    }

    #[test]
    fn world_node_transform_applies_parent_scale() {
        let gltf = json!({
            "nodes": [
                {"name": "root", "scale": [2.0, 3.0, 4.0], "children": [1]},
                {"name": "child", "translation": [1.0, 0.0, 0.0]}
            ]
        });

        let translation = world_node_translation(&gltf, "child").expect("child translation");

        assert!((translation[0] - 2.0).abs() < 0.0001);
        assert!(translation[1].abs() < 0.0001);
    }

    #[test]
    fn world_node_transform_reads_matrix_nodes() {
        let matrix = Mat4::from_translation(Vec3::new(2.0, 3.0, 4.0)).to_cols_array();
        let gltf = json!({
            "nodes": [
                {"name": "root", "matrix": matrix, "children": [1]},
                {"name": "child", "translation": [1.0, 0.0, 0.0]}
            ]
        });

        let translation = world_node_translation(&gltf, "child").expect("child translation");

        assert!((translation[0] - 3.0).abs() < 0.0001);
        assert!((translation[1] - 3.0).abs() < 0.0001);
        assert!((translation[2] - 4.0).abs() < 0.0001);
    }

    #[test]
    fn node_world_mesh_aabb_uses_accessor_bounds_and_node_transform() {
        let gltf = json!({
            "nodes": [
                {"name": "root", "translation": [1.0, 0.0, 0.0], "children": [1]},
                {"name": "box", "mesh": 0, "translation": [0.0, 2.0, 0.0], "scale": [2.0, 1.0, 1.0]}
            ],
            "meshes": [
                {"primitives": [{"attributes": {"POSITION": 0}}]}
            ],
            "accessors": [
                {"min": [-1.0, -0.5, -0.25], "max": [1.0, 0.5, 0.25]}
            ]
        });

        let bounds = node_world_mesh_aabb(&gltf, "box").expect("world mesh bounds");

        assert!((bounds.min.x + 1.0).abs() < 0.0001);
        assert!((bounds.max.x - 3.0).abs() < 0.0001);
        assert!((bounds.min.y - 1.5).abs() < 0.0001);
        assert!((bounds.max.y - 2.5).abs() < 0.0001);
        assert!((bounds.min.z + 0.25).abs() < 0.0001);
        assert!((bounds.max.z - 0.25).abs() < 0.0001);
    }

    #[test]
    fn node_world_mesh_aabb_with_pose_preserves_node_scale() {
        let gltf = json!({
            "nodes": [
                {"name": "box", "mesh": 0, "translation": [0.0, 0.0, 0.0], "scale": [2.0, 1.0, 1.0]}
            ],
            "meshes": [
                {"primitives": [{"attributes": {"POSITION": 0}}]}
            ],
            "accessors": [
                {"min": [-1.0, -0.5, -0.25], "max": [1.0, 0.5, 0.25]}
            ]
        });
        let overrides = [PoseNodeOverride {
            node_name: "box",
            pose: PartPose {
                translation: Vec3::new(3.0, 0.0, 0.0),
                rotation: Quat::IDENTITY,
                visibility: nau_engine::animation::PartVisibility::Inherited,
            },
        }];

        let bounds =
            node_world_mesh_aabb_with_pose(&gltf, "box", &overrides).expect("posed mesh bounds");

        assert!((bounds.min.x - 1.0).abs() < 0.0001);
        assert!((bounds.max.x - 5.0).abs() < 0.0001);
        assert!((bounds.min.y + 0.5).abs() < 0.0001);
        assert!((bounds.max.y - 0.5).abs() < 0.0001);
    }

    #[test]
    fn aabb_overlap_depth_reports_smallest_axis_penetration() {
        let left = Aabb3::from_min_max(Vec3::ZERO, Vec3::new(1.0, 2.0, 3.0));
        let right = Aabb3::from_min_max(Vec3::new(0.75, -1.0, 1.0), Vec3::new(2.0, 1.0, 2.0));
        let separated = Aabb3::from_min_max(Vec3::new(2.0, 0.0, 0.0), Vec3::new(3.0, 1.0, 1.0));

        assert!((left.overlap_depth_m(right) - 0.25).abs() < 0.0001);
        assert_eq!(left.overlap_depth_m(separated), 0.0);
    }

    #[test]
    fn aabb_separation_reports_surface_gap() {
        let left = Aabb3::from_min_max(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
        let touching = Aabb3::from_min_max(Vec3::new(1.0, 0.25, 0.25), Vec3::new(2.0, 0.75, 0.75));
        let separated = Aabb3::from_min_max(Vec3::new(1.03, 1.04, 0.25), Vec3::new(2.0, 2.0, 0.75));

        assert_eq!(left.separation_m(touching), 0.0);
        assert!((left.separation_m(separated) - 0.05).abs() < 0.0001);
    }

    #[test]
    fn joint_cover_mesh_pairs_use_visible_cover_nodes() {
        let pairs = player_joint_cover_mesh_pairs();

        assert!(
            pairs
                .iter()
                .any(|(left, _right)| *left == "Nau Neck Joint Cover")
        );
        assert!(
            pairs
                .iter()
                .all(|(left, _right)| !left.ends_with(" Socket"))
        );
    }

    #[test]
    fn joint_bridge_mesh_pairs_cover_each_bridge_to_two_surfaces() {
        let bridges = player_joint_bridge_node_names();
        let pairs = player_joint_bridge_mesh_pairs();

        assert_eq!(
            bridges.len() as f64,
            PLAYER_JOINT_BRIDGE_EXPECTED_NODE_COUNT
        );
        assert_eq!(pairs.len() as f64, PLAYER_JOINT_BRIDGE_EXPECTED_PAIR_COUNT);
        for bridge in bridges {
            assert_eq!(
                pairs
                    .iter()
                    .filter(|(candidate, _contact)| *candidate == bridge)
                    .count(),
                1,
                "{bridge} should stay attached to its animated distal fixture surface"
            );
        }
    }

    #[test]
    fn player_joint_bridge_contact_audit_reports_connected_bridges() {
        let text = fs::read_to_string("assets/models/player/player.gltf").expect("player fixture");
        let gltf = serde_json::from_str::<Value>(&text).expect("player gltf");
        let audit = player_joint_bridge_contact_audit(&gltf).expect("bridge contact audit");

        assert_eq!(
            number_field(&audit, "bridge_node_count"),
            PLAYER_JOINT_BRIDGE_EXPECTED_NODE_COUNT
        );
        assert_eq!(number_field(&audit, "breach_count"), 0.0);
        assert!(number_field(&audit, "max_gap_m") <= PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_GAP_M);
        assert!(
            number_field(&audit, "max_overlap_m") <= PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_OVERLAP_M
        );
    }

    #[test]
    fn player_joint_seam_contact_audit_reports_transition_safe_flex_covers() {
        let text = fs::read_to_string("assets/models/player/player.gltf").expect("player fixture");
        let gltf = serde_json::from_str::<Value>(&text).expect("player gltf");
        let audit = player_joint_seam_contact_audit(&gltf).expect("seam contact audit");

        assert_eq!(
            number_field(&audit, "seam_node_count"),
            PLAYER_JOINT_SEAM_EXPECTED_NODE_COUNT
        );
        assert_eq!(
            number_field(&audit, "pair_count"),
            PLAYER_JOINT_SEAM_EXPECTED_PAIR_COUNT
        );
        assert_eq!(number_field(&audit, "breach_count"), 0.0);
        assert!(number_field(&audit, "max_gap_m") <= PLAYER_POSE_MAX_JOINT_SEAM_MESH_GAP_M);
        assert!(number_field(&audit, "min_overlap_m") >= PLAYER_POSE_MIN_JOINT_SEAM_MESH_OVERLAP_M);
        assert!(number_field(&audit, "max_overlap_m") <= PLAYER_POSE_MAX_JOINT_SEAM_MESH_OVERLAP_M);
        assert_eq!(number_field(&audit, "transition_count"), 9.0);
        assert_eq!(
            number_field(&audit, "samples_per_pair"),
            player_pose_contact_samples_per_pair()
        );
    }

    #[test]
    fn player_proximal_contact_audit_reports_connected_shoulders_and_hips() {
        let text = fs::read_to_string("assets/models/player/player.gltf").expect("player fixture");
        let gltf = serde_json::from_str::<Value>(&text).expect("player gltf");
        let audit = player_proximal_contact_audit(&gltf).expect("proximal contact audit");

        assert_eq!(
            number_field(&audit, "pair_count"),
            PLAYER_PROXIMAL_CONTACT_EXPECTED_PAIR_COUNT
        );
        assert_eq!(number_field(&audit, "breach_count"), 0.0);
        assert!(number_field(&audit, "max_gap_m") <= PLAYER_POSE_MAX_SURFACE_CONTACT_DISTANCE_M);
        assert!(
            number_field(&audit, "max_overlap_m")
                <= PLAYER_POSE_MAX_PROXIMAL_CONTACT_MESH_OVERLAP_M
        );
    }

    #[test]
    fn player_pose_surface_contact_audit_reports_mesh_sample_distances() {
        let text = fs::read_to_string("assets/models/player/player.gltf").expect("player fixture");
        let gltf = serde_json::from_str::<Value>(&text).expect("player gltf");
        let audit = player_pose_surface_contact_audit(&gltf).expect("surface contact audit");

        assert_eq!(
            number_field(&audit, "pair_count"),
            PLAYER_SURFACE_CONTACT_EXPECTED_PAIR_COUNT
        );
        assert_eq!(
            number_field(&audit, "phase_count"),
            PLAYER_POSE_CONTACT_EXPECTED_PHASE_COUNT
        );
        assert_eq!(
            number_field(&audit, "transition_count"),
            PLAYER_POSE_TRANSITION_EXPECTED_TRANSITION_COUNT
        );
        assert_eq!(
            number_field(&audit, "blend_count"),
            PLAYER_POSE_TRANSITION_EXPECTED_BLEND_COUNT
        );
        assert_eq!(
            number_field(&audit, "samples_per_pair"),
            player_pose_contact_samples_per_pair()
        );
        assert_eq!(number_field(&audit, "breach_count"), 0.0);
        assert!(
            number_field(&audit, "max_distance_m") <= PLAYER_POSE_MAX_SURFACE_CONTACT_DISTANCE_M
        );
        assert!(
            number_field(&audit, "max_projected_gap_m") <= PLAYER_POSE_MAX_PROJECTED_CONTACT_GAP_M
        );

        let hip_contact = audit
            .get("pairs")
            .and_then(Value::as_array)
            .and_then(|pairs| {
                pairs.iter().find(|pair| {
                    pair.get("left_node").and_then(Value::as_str)
                        == Some("Nau Left Seamless Hip Flex Cover")
                        && pair.get("right_node").and_then(Value::as_str)
                            == Some("Nau Left Hip Joint Cover")
                })
            })
            .expect("hip flex contact pair");
        assert!(
            number_field(hip_contact, "max_distance_m")
                <= PLAYER_POSE_MAX_SURFACE_CONTACT_DISTANCE_M
        );
        assert!(
            number_field(hip_contact, "max_projected_gap_m")
                <= PLAYER_POSE_MAX_PROJECTED_CONTACT_GAP_M
        );
    }

    #[test]
    fn player_glider_attachment_audit_reports_hands_on_grips() {
        let player_text =
            fs::read_to_string("assets/models/player/player.gltf").expect("player fixture");
        let player_gltf = serde_json::from_str::<Value>(&player_text).expect("player gltf");
        let glider_text =
            fs::read_to_string("assets/models/player/glider.gltf").expect("glider fixture");
        let glider_gltf = serde_json::from_str::<Value>(&glider_text).expect("glider gltf");
        let audit = player_glider_attachment_audit(&player_gltf, &glider_gltf)
            .expect("player glider attachment audit");

        assert_eq!(
            number_field(&audit, "sample_count"),
            PLAYER_GLIDER_HAND_GRIP_EXPECTED_SAMPLE_COUNT
        );
        assert_eq!(number_field(&audit, "breach_count"), 0.0);
        assert!(number_field(&audit, "max_distance_m") <= PLAYER_GLIDER_MAX_HAND_GRIP_DISTANCE_M);
        assert!(
            number_field(&audit, "max_projected_gap_m")
                <= PLAYER_GLIDER_MAX_HAND_GRIP_PROJECTED_GAP_M
        );
    }

    #[test]
    fn player_hand_and_boot_silhouette_audit_rejects_marker_rod_proportions() {
        let text = fs::read_to_string("assets/models/player/player.gltf").expect("player fixture");
        let gltf = serde_json::from_str::<Value>(&text).expect("player gltf");
        let (finger_min, finger_max) =
            player_finger_grip_length_range_m(&gltf).expect("finger grip length range");

        assert!(finger_min >= PLAYER_MIN_FINGER_GRIP_LENGTH_M);
        assert!(finger_max <= PLAYER_MAX_FINGER_GRIP_LENGTH_M);
        assert!(
            player_boot_sole_length_min_m(&gltf).expect("boot sole length")
                >= PLAYER_MIN_BOOT_SOLE_LENGTH_M
        );
    }

    #[test]
    fn player_limb_anatomy_detail_audit_requires_extremity_shape_nodes() {
        let text = fs::read_to_string("assets/models/player/player.gltf").expect("player fixture");
        let gltf = serde_json::from_str::<Value>(&text).expect("player gltf");
        let audit = player_limb_anatomy_detail_audit(&gltf).expect("limb anatomy detail audit");

        assert_eq!(
            number_field(&audit, "present_node_count"),
            PLAYER_LIMB_ANATOMY_EXPECTED_NODE_COUNT
        );
        assert_eq!(number_field(&audit, "missing_count"), 0.0);
        assert!(
            number_field(&audit, "min_major_extent_m") >= PLAYER_MIN_LIMB_ANATOMY_MAJOR_EXTENT_M
        );
        for expected in [
            "Nau Left Suit Bicep Volume",
            "Nau Left Suit Shoulder Root Blend",
            "Nau Right Suit Shoulder Chest Blend",
            "Nau Left Suit Pectoral Soft Volume",
            "Nau Right Suit Scapula Soft Volume",
            "Nau Left Suit Axilla Blend",
            "Nau Left Suit Hip Root Blend",
            "Nau Left Suit Hip Thigh Fairing",
            "Nau Right Suit Hip Inguinal Blend",
            "Nau Right Suit Tricep Sweep",
            "Nau Left Leather Finger Web Bridge",
            "Nau Right Suit Calf Volume",
            "Nau Left Suit Knee Tendon Strap",
            "Nau Left Suit Shin Ridge",
            "Nau Left Leather Wrist Palm Gusset",
            "Nau Right Leather Boot Instep Plate",
            "Nau Left Leather Outer Boot Side Guard",
            "Nau Right Leather Boot Arch Rib",
            "Nau Right Leather Ankle Boot Tongue",
            "Nau Left Leather Lace Cross Strap A",
        ] {
            assert!(
                audit
                    .get("samples")
                    .and_then(Value::as_array)
                    .is_some_and(|samples| {
                        samples.iter().any(|sample| {
                            sample.get("node").and_then(Value::as_str) == Some(expected)
                        })
                    })
            );
        }
    }

    #[test]
    fn player_mesh_silhouette_audit_measures_actual_pose_hulls() {
        let text = fs::read_to_string("assets/models/player/player.gltf").expect("player fixture");
        let gltf = serde_json::from_str::<Value>(&text).expect("player gltf");
        let audit = player_mesh_silhouette_audit(&gltf).expect("mesh silhouette audit");

        assert_eq!(
            number_field(&audit, "pose_count"),
            PLAYER_MESH_SILHOUETTE_EXPECTED_POSE_COUNT
        );
        assert_eq!(
            number_field(&audit, "sample_count"),
            PLAYER_MESH_SILHOUETTE_EXPECTED_SAMPLE_COUNT
        );
        assert!(
            number_field(&audit, "min_projected_span_m")
                >= PLAYER_MESH_SILHOUETTE_MIN_PROJECTED_SPAN_M
        );
        assert!(
            number_field(&audit, "fall_top_width_m") >= PLAYER_MESH_SILHOUETTE_MIN_FALL_TOP_WIDTH_M
        );
        assert!(
            number_field(&audit, "fall_top_depth_m") >= PLAYER_MESH_SILHOUETTE_MIN_FALL_TOP_DEPTH_M
        );
        assert!(
            number_field(&audit, "glide_front_width_m")
                >= PLAYER_MESH_SILHOUETTE_MIN_GLIDE_FRONT_WIDTH_M
        );
        assert!(
            number_field(&audit, "dive_front_to_fall_front_width_ratio")
                <= PLAYER_MESH_SILHOUETTE_MAX_DIVE_FRONT_TO_FALL_FRONT_WIDTH_RATIO
        );
        assert!(
            number_field(&audit, "dive_front_height_m")
                >= PLAYER_MESH_SILHOUETTE_MIN_DIVE_FRONT_HEIGHT_M
        );
        assert!(
            number_field(&audit, "dive_front_width_to_height_ratio")
                <= PLAYER_MESH_SILHOUETTE_MAX_DIVE_FRONT_WIDTH_TO_HEIGHT_RATIO
        );
        assert!(
            number_field(&audit, "dive_side_width_to_height_ratio")
                <= PLAYER_MESH_SILHOUETTE_MAX_DIVE_SIDE_WIDTH_TO_HEIGHT_RATIO
        );
        assert!(
            audit
                .get("samples")
                .and_then(Value::as_array)
                .is_some_and(|samples| samples.iter().any(|sample| {
                    sample.get("label").and_then(Value::as_str) == Some("diving_head_down")
                        && sample.get("view").and_then(Value::as_str) == Some("front")
                }))
        );
    }

    #[test]
    fn player_pose_contact_audit_reports_each_sampled_pose() {
        let text = fs::read_to_string("assets/models/player/player.gltf").expect("player fixture");
        let gltf = serde_json::from_str::<Value>(&text).expect("player gltf");
        let audit = player_pose_contact_audit(&gltf).expect("pose contact audit");
        let poses = audit
            .get("poses")
            .and_then(Value::as_array)
            .expect("pose rows");

        assert_eq!(
            number_field(&audit, "pose_count"),
            PLAYER_POSE_CONTACT_EXPECTED_POSE_COUNT
        );
        assert_eq!(
            number_field(&audit, "phase_count"),
            PLAYER_POSE_CONTACT_EXPECTED_PHASE_COUNT
        );
        assert_eq!(number_field(&audit, "breach_count"), 0.0);
        for expected in [
            "grounded_idle",
            "grounded_walk",
            "grounded_run",
            "launching",
            "falling",
            "gliding",
            "diving",
            "air_brake",
            "landing_anticipation",
            "landing_recovery",
        ] {
            assert!(poses.iter().any(|pose| {
                pose.get("pose_intent").and_then(Value::as_str) == Some(expected)
                    && pose
                        .get("within_thresholds")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
            }));
        }
    }

    #[test]
    fn player_pose_transition_contact_audit_reports_in_between_samples() {
        let text = fs::read_to_string("assets/models/player/player.gltf").expect("player fixture");
        let gltf = serde_json::from_str::<Value>(&text).expect("player gltf");
        let audit =
            player_pose_transition_contact_audit(&gltf).expect("pose transition contact audit");
        let transitions = audit
            .get("transitions")
            .and_then(Value::as_array)
            .expect("transition rows");

        assert_eq!(
            number_field(&audit, "transition_count"),
            PLAYER_POSE_TRANSITION_EXPECTED_TRANSITION_COUNT
        );
        assert_eq!(
            number_field(&audit, "blend_count"),
            PLAYER_POSE_TRANSITION_EXPECTED_BLEND_COUNT
        );
        assert_eq!(
            number_field(&audit, "phase_count"),
            PLAYER_POSE_CONTACT_EXPECTED_PHASE_COUNT
        );
        assert_eq!(number_field(&audit, "breach_count"), 0.0);
        assert!(
            number_field(&audit, "joint_bridge_mesh_overlap_max_m")
                <= PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_OVERLAP_M
        );
        for expected in [
            "launching_to_gliding",
            "falling_to_gliding",
            "gliding_to_diving",
            "diving_to_gliding",
            "gliding_to_landing_anticipation",
        ] {
            assert!(transitions.iter().any(|transition| {
                transition.get("transition").and_then(Value::as_str) == Some(expected)
                    && transition
                        .get("within_thresholds")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
            }));
        }
    }

    #[test]
    fn player_pose_preview_renders_mesh_hulls_for_fixture_details() {
        let text = fs::read_to_string("assets/models/player/player.gltf").expect("player fixture");
        let gltf = serde_json::from_str::<Value>(&text).expect("player gltf");
        let specs = player_pose_preview_specs();
        let overrides = player_pose_node_overrides(&gltf, specs[0].context, specs[0].phase)
            .expect("pose overrides");
        let shapes = player_pose_preview_shapes(&gltf, &overrides).expect("preview shapes");
        let pinky = shapes
            .iter()
            .find(|shape| shape.node_name == "Nau Left Leather Pinky Finger Grip")
            .expect("pinky finger shape");

        assert!(pinky.vertices.len() > 12);

        let sheet = render_player_pose_preview_sheet(&gltf, &specs).expect("preview sheet");
        assert!(sheet.contains("front mesh silhouette"));
        assert!(sheet.contains("rear mesh silhouette"));
        assert!(sheet.contains("<path d=\""));
        assert!(sheet.contains("surface distance"));
        assert!(sheet.contains("Nau Left Leather Pinky Finger Grip"));
        assert!(sheet.contains("Nau Left Leather Outer Toe Lug"));
        assert!(sheet.contains("Grounded Walk"));
        assert!(sheet.contains("Grounded Run"));
    }

    #[test]
    fn player_anatomy_review_preview_renders_closeup_panels() {
        let text = fs::read_to_string("assets/models/player/player.gltf").expect("player fixture");
        let gltf = serde_json::from_str::<Value>(&text).expect("player gltf");
        let specs = player_anatomy_review_specs();
        let sheet = render_player_anatomy_review_sheet(&gltf, &specs).expect("anatomy sheet");

        assert_eq!(specs.len(), 18);
        assert!(sheet.contains("player anatomy review sheet"));
        assert!(sheet.contains("Core Connectors Front"));
        assert!(sheet.contains("Shoulders Front"));
        assert!(sheet.contains("Left Hand Top"));
        assert!(sheet.contains("Right Hand Top"));
        assert!(sheet.contains("Left Boot Side"));
        assert!(sheet.contains("Right Boot Side"));
        assert!(sheet.contains("Nau Skin Neck Column"));
        assert!(sheet.contains("Nau Suit Abdominal Flex Gasket"));
        assert!(sheet.contains("Nau Left Suit Axilla Blend"));
        assert!(sheet.contains("Nau Left Suit Pectoral Soft Volume"));
        assert!(sheet.contains("Nau Right Suit Scapula Soft Volume"));
        assert!(sheet.contains("Nau Left Suit Hip Thigh Fairing"));
        assert!(sheet.contains("Nau Left Leather Thumb Web Pad"));
        assert!(sheet.contains("Nau Left Leather Boot Arch Rib"));
        assert!(sheet.contains("Nau Right Leather Thumb Web Pad"));
        assert!(sheet.contains("Nau Right Leather Boot Arch Rib"));
    }

    #[test]
    fn player_rig_stress_review_preview_renders_extreme_pose_closeups() {
        let text = fs::read_to_string("assets/models/player/player.gltf").expect("player fixture");
        let gltf = serde_json::from_str::<Value>(&text).expect("player gltf");
        let specs = player_rig_stress_review_specs();
        let sheet =
            render_player_rig_stress_review_sheet(&gltf, &specs).expect("stress review sheet");

        assert_eq!(specs.len(), 13);
        assert!(sheet.contains("player rig stress review sheet"));
        assert!(sheet.contains("Launch Shoulders Front"));
        assert!(sheet.contains("Launch Right Hand Front"));
        assert!(sheet.contains("Fall Core Top"));
        assert!(sheet.contains("Dive Core Side"));
        assert!(sheet.contains("Landing Hips Side"));
        assert!(sheet.contains("Nau Left Suit Shoulder Web Capsule"));
        assert!(sheet.contains("Nau Suit Lower Rib Flex Lip"));
        assert!(sheet.contains("Nau Left Suit Pectoral Soft Volume"));
        assert!(sheet.contains("Nau Left Suit Glute Hip Connector"));
        assert!(sheet.contains("Nau Right Leather Thumb Tip Pad"));
    }

    #[test]
    fn player_motion_integrity_review_preview_renders_large_full_body_panels() {
        let text = fs::read_to_string("assets/models/player/player.gltf").expect("player fixture");
        let gltf = serde_json::from_str::<Value>(&text).expect("player gltf");
        let specs = player_motion_integrity_review_specs();
        let sheet =
            render_player_motion_integrity_review_sheet(&gltf, &specs).expect("motion sheet");

        assert_eq!(specs.len(), 15);
        assert!(sheet.contains("player motion integrity review sheet"));
        assert!(sheet.contains("Launch Front Full"));
        assert!(sheet.contains("Fall Top Full"));
        assert!(sheet.contains("Glide Rear Full"));
        assert!(sheet.contains("Dive Side Full"));
        assert!(sheet.contains("Landing Recovery Front Full"));
        assert!(sheet.contains("Nau Left Suit Pectoral Soft Volume"));
        assert!(sheet.contains("Nau Left Suit Hip Thigh Fairing"));
        assert!(sheet.contains("surface distance"));

        let overlay_audit =
            player_motion_integrity_overlay_warning_audit(&gltf).expect("overlay audit");
        assert_eq!(
            number_field(&overlay_audit, "panel_count"),
            PLAYER_MOTION_INTEGRITY_REVIEW_EXPECTED_PANEL_COUNT
        );
        assert_eq!(number_field(&overlay_audit, "warning_count"), 0.0);
        assert_eq!(number_field(&overlay_audit, "fail_count"), 0.0);
    }

    #[test]
    fn player_transition_pose_preview_renders_blend_frames() {
        let text = fs::read_to_string("assets/models/player/player.gltf").expect("player fixture");
        let gltf = serde_json::from_str::<Value>(&text).expect("player gltf");
        let specs = player_transition_pose_preview_specs();
        let sheet =
            render_player_transition_pose_preview_sheet(&gltf, &specs).expect("transition sheet");

        assert_eq!(
            specs.len() as f64,
            PLAYER_POSE_TRANSITION_EXPECTED_TRANSITION_COUNT
                * PLAYER_POSE_TRANSITION_EXPECTED_BLEND_COUNT
        );
        assert!(sheet.contains("player fixture transition pose preview"));
        assert!(sheet.contains("gliding_to_diving"));
        assert!(sheet.contains("landing_anticipation_to_landing_recovery"));
        for expected in ["blend: 20%", "blend: 40%", "blend: 60%", "blend: 80%"] {
            assert!(sheet.contains(expected));
        }
        assert!(sheet.contains("surface distance"));
        assert!(sheet.contains("mesh overlap"));
        assert!(sheet.contains("rear mesh silhouette"));
    }

    #[test]
    fn glider_pose_preview_renders_takeout_deployment_sheet() {
        let text = fs::read_to_string("assets/models/player/glider.gltf").expect("glider fixture");
        let gltf = serde_json::from_str::<Value>(&text).expect("glider gltf");
        let specs = glider_pose_preview_specs();
        let stowed_shapes = glider_pose_preview_shapes(&gltf, specs[0]).expect("stowed shapes");
        let takeout_shapes = glider_pose_preview_shapes(&gltf, specs[1]).expect("glider shapes");
        let open_shapes = glider_pose_preview_shapes(&gltf, specs[3]).expect("open shapes");
        let left_panel = takeout_shapes
            .iter()
            .find(|shape| shape.node_name == "Nau Glider Left Cloth Panel")
            .expect("left panel shape");
        let stowed_width = preview_projected_extent(&stowed_shapes, PlayerPosePreviewView::Top)
            .expect("stowed top extent");
        let takeout_width = preview_projected_extent(&takeout_shapes, PlayerPosePreviewView::Top)
            .expect("takeout top extent");
        let open_width = preview_projected_extent(&open_shapes, PlayerPosePreviewView::Top)
            .expect("open top extent");

        assert!(takeout_shapes.len() >= 14);
        assert!(left_panel.vertices.len() >= 8);
        assert!((stowed_width.1 - stowed_width.0) < (open_width.1 - open_width.0) * 0.35);
        assert!((takeout_width.1 - takeout_width.0) > (stowed_width.1 - stowed_width.0));

        let sheet = render_glider_pose_preview_sheet(&gltf, &specs).expect("glider preview sheet");
        assert!(sheet.contains("NAU glider deployment pose preview"));
        assert!(sheet.contains("Takeout 35%"));
        assert!(sheet.contains("Launch Takeout"));
        assert!(sheet.contains("Dive Release"));
        assert!(sheet.contains("deploy: 52%"));
        assert!(sheet.contains("<path d=\""));
        assert!(sheet.contains("Nau Glider Left Cloth Panel"));
        assert!(sheet.contains("Nau Glider Handle Bar"));
        assert!(sheet.contains("rear mesh silhouette"));
    }

    #[test]
    fn player_glider_attachment_preview_renders_combined_pose_sheet() {
        let player_text =
            fs::read_to_string("assets/models/player/player.gltf").expect("player fixture");
        let player_gltf = serde_json::from_str::<Value>(&player_text).expect("player gltf");
        let glider_text =
            fs::read_to_string("assets/models/player/glider.gltf").expect("glider fixture");
        let glider_gltf = serde_json::from_str::<Value>(&glider_text).expect("glider gltf");
        let specs = player_glider_attachment_preview_specs();
        let sheet =
            render_player_glider_attachment_preview_sheet(&player_gltf, &glider_gltf, &specs)
                .expect("attachment preview sheet");

        assert!(sheet.contains("player/glider attachment preview"));
        assert!(sheet.contains("Launch Takeout"));
        assert!(sheet.contains("Dive Release"));
        assert!(sheet.contains("hand grip distance"));
        assert!(sheet.contains("Nau Glider Left Grip"));
        assert!(sheet.contains("Nau Left Leather Hand Palm"));
    }

    #[test]
    fn obb_overlap_depth_uses_oriented_axes() {
        let local_box = Aabb3::from_min_max(Vec3::new(-1.0, -0.1, -0.1), Vec3::new(1.0, 0.1, 0.1));
        let left = local_box.transformed_obb(Mat4::IDENTITY);
        let right = local_box.transformed_obb(
            Mat4::from_translation(Vec3::new(0.0, 1.25, 0.0))
                * Mat4::from_rotation_z(std::f32::consts::FRAC_PI_2),
        );

        assert_eq!(left.overlap_depth_m(right), 0.0);
    }

    #[test]
    fn player_bank_clip_motion_rejects_glide_reuse() {
        let gltf = bank_clip_test_gltf(false);

        assert!(!player_bank_clip_motion_is_distinct(&gltf));
    }

    #[test]
    fn player_bank_clip_motion_accepts_distinct_bank_tracks() {
        let gltf = bank_clip_test_gltf(true);

        assert!(player_bank_clip_motion_is_distinct(&gltf));
    }

    #[test]
    fn player_launch_glide_land_clip_motion_rejects_reused_tracks() {
        let gltf = launch_glide_land_clip_test_gltf(false);

        assert!(!player_launch_glide_land_clip_motion_is_distinct(&gltf));
    }

    #[test]
    fn player_launch_glide_land_clip_motion_accepts_distinct_tracks() {
        let gltf = launch_glide_land_clip_test_gltf(true);

        assert!(player_launch_glide_land_clip_motion_is_distinct(&gltf));
    }

    #[test]
    fn player_fall_clip_motion_rejects_air_brake_reuse() {
        let gltf = fall_clip_test_gltf(false);

        assert!(!player_fall_clip_motion_is_distinct(&gltf));
    }

    #[test]
    fn player_fall_clip_motion_accepts_distinct_fall_track() {
        let gltf = fall_clip_test_gltf(true);

        assert!(player_fall_clip_motion_is_distinct(&gltf));
    }

    #[test]
    fn player_grounded_locomotion_clip_motion_rejects_walk_run_reuse() {
        let gltf = grounded_locomotion_clip_test_gltf(false);

        assert!(!player_grounded_locomotion_clip_motion_is_distinct(&gltf));
    }

    #[test]
    fn player_grounded_locomotion_clip_motion_accepts_distinct_walk_run() {
        let gltf = grounded_locomotion_clip_test_gltf(true);

        assert!(player_grounded_locomotion_clip_motion_is_distinct(&gltf));
    }

    fn bank_clip_test_gltf(distinct_banks: bool) -> Value {
        let accessors = json!([
            {"min": [0.0], "max": [1.0]},
            {"min": [0.0], "max": [0.5]},
            {"min": [0.0], "max": [0.6]},
            {"min": [0.0], "max": [0.7]},
            {"min": [0.0], "max": [0.8]},
            {"min": [-0.8], "max": [0.0]},
            {"min": [-0.7], "max": [0.0]},
            {"min": [-0.6], "max": [0.0]},
            {"min": [-0.5], "max": [0.0]}
        ]);
        let left_outputs = if distinct_banks {
            [1, 2, 3, 4]
        } else {
            [0, 0, 0, 0]
        };
        let right_outputs = if distinct_banks {
            [5, 6, 7, 8]
        } else {
            [0, 0, 0, 0]
        };

        json!({
            "accessors": accessors,
            "animations": [
                clip("Glide_Loop", [0, 0, 0, 0]),
                clip("Bank_Left", left_outputs),
                clip("Bank_Right", right_outputs)
            ]
        })
    }

    fn launch_glide_land_clip_test_gltf(distinct_launch_and_land: bool) -> Value {
        let accessors = json!([
            {"min": [0.0], "max": [0.2]},
            {"min": [0.1], "max": [0.4]},
            {"min": [0.2], "max": [0.6]},
            {"min": [0.3], "max": [0.8]},
            {"min": [-0.8], "max": [-0.2]},
            {"min": [-0.6], "max": [-0.1]},
            {"min": [-0.4], "max": [0.0]},
            {"min": [-0.2], "max": [0.1]},
            {"min": [0.4], "max": [1.0]},
            {"min": [0.5], "max": [1.1]},
            {"min": [0.6], "max": [1.2]},
            {"min": [0.7], "max": [1.3]}
        ]);
        let glide_outputs = [0, 1, 2, 3];
        let launch_outputs = if distinct_launch_and_land {
            [4, 5, 6, 7]
        } else {
            glide_outputs
        };
        let land_outputs = if distinct_launch_and_land {
            [8, 9, 10, 11]
        } else {
            glide_outputs
        };

        json!({
            "accessors": accessors,
            "animations": [
                clip("Launch_Start", launch_outputs),
                clip("Glide_Loop", glide_outputs),
                clip("Land", land_outputs)
            ]
        })
    }

    fn fall_clip_test_gltf(distinct_fall: bool) -> Value {
        let accessors = json!([
            {"min": [0.0], "max": [0.2]},
            {"min": [0.0], "max": [0.4]},
            {"min": [0.0], "max": [0.6]},
            {"min": [0.0], "max": [0.8]},
            {"min": [-0.2], "max": [0.0]},
            {"min": [-0.4], "max": [0.0]},
            {"min": [-0.6], "max": [0.0]},
            {"min": [-0.8], "max": [0.0]},
            {"min": [0.1], "max": [0.5]},
            {"min": [0.2], "max": [0.6]},
            {"min": [0.3], "max": [0.7]},
            {"min": [0.4], "max": [0.8]},
            {"min": [0.5], "max": [0.9]}
        ]);
        let air_brake_outputs = [1, 2, 3, 4];
        let fall_outputs = if distinct_fall {
            [9, 10, 11, 12]
        } else {
            air_brake_outputs
        };

        json!({
            "accessors": accessors,
            "animations": [
                clip("Glide_Loop", [0, 0, 0, 0]),
                clip("Air_Brake", air_brake_outputs),
                clip("Land", [5, 6, 7, 8]),
                clip("Fall_Loop", fall_outputs)
            ]
        })
    }

    fn grounded_locomotion_clip_test_gltf(distinct_run: bool) -> Value {
        let accessors = json!([
            {"min": [0.0], "max": [0.1]},
            {"min": [0.0], "max": [0.2]},
            {"min": [0.0], "max": [0.3]},
            {"min": [0.0], "max": [0.4]},
            {"min": [-0.4], "max": [0.0]},
            {"min": [-0.3], "max": [0.0]},
            {"min": [-0.2], "max": [0.0]},
            {"min": [-0.1], "max": [0.0]},
            {"min": [0.2], "max": [0.7]},
            {"min": [0.3], "max": [0.8]},
            {"min": [0.4], "max": [0.9]},
            {"min": [0.5], "max": [1.0]}
        ]);
        let walk_outputs = [1, 2, 3, 4];
        let run_outputs = if distinct_run {
            [8, 9, 10, 11]
        } else {
            walk_outputs
        };

        json!({
            "accessors": accessors,
            "animations": [
                clip("Idle_Loop", [0, 0, 0, 0]),
                clip("Walk_Fwd_Loop", walk_outputs),
                clip("Run_Fwd_Loop", run_outputs)
            ]
        })
    }

    fn clip(name: &str, outputs: [u64; 4]) -> Value {
        json!({
            "name": name,
            "samplers": [
                {"output": outputs[0]},
                {"output": outputs[1]},
                {"output": outputs[2]},
                {"output": outputs[3]}
            ],
            "channels": [
                {"sampler": 0, "target": {"node": 2, "path": "rotation"}},
                {"sampler": 1, "target": {"node": 3, "path": "rotation"}},
                {"sampler": 2, "target": {"node": 4, "path": "rotation"}},
                {"sampler": 3, "target": {"node": 5, "path": "rotation"}}
            ]
        })
    }
}
