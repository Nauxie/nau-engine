use crate::asset_pipeline::{
    DEFAULT_VISUAL_ASSET_LOAD_POLICY, VISUAL_ASSET_SPECS, VisualAssetKind,
    VisualAssetLoadAdmission, VisualAssetLoadPolicy, VisualAssetLoadState, VisualAssetResidency,
    VisualAssetSpec, visual_asset_load_admission_plan,
};

#[test]
fn default_asset_load_policy_admits_current_manifest() {
    let admissions = visual_asset_load_admission_plan(
        &VISUAL_ASSET_SPECS,
        |_| true,
        DEFAULT_VISUAL_ASSET_LOAD_POLICY,
    );

    assert_eq!(admissions.len(), VISUAL_ASSET_SPECS.len());
    assert!(admissions.iter().all(|admission| admission.is_admitted()));
    assert_eq!(
        admissions
            .iter()
            .filter(|admission| admission.load_state() == VisualAssetLoadState::Deferred)
            .count(),
        0
    );
}

#[test]
fn asset_load_policy_defers_lower_priority_streamed_assets() {
    let specs = [
        VisualAssetSpec {
            kind: VisualAssetKind::PlayerCharacter,
            label: "always player",
            gltf_scene_path: "player.gltf",
            animation_clip_names: &[],
            residency: VisualAssetResidency::Always,
        },
        VisualAssetSpec {
            kind: VisualAssetKind::Glider,
            label: "always glider",
            gltf_scene_path: "glider.gltf",
            animation_clip_names: &[],
            residency: VisualAssetResidency::Always,
        },
        VisualAssetSpec {
            kind: VisualAssetKind::IslandTerrain,
            label: "stream terrain",
            gltf_scene_path: "terrain.gltf",
            animation_clip_names: &[],
            residency: VisualAssetResidency::StreamWindow,
        },
        VisualAssetSpec {
            kind: VisualAssetKind::IslandRock,
            label: "stream rock",
            gltf_scene_path: "rock.gltf",
            animation_clip_names: &[],
            residency: VisualAssetResidency::StreamWindow,
        },
        VisualAssetSpec {
            kind: VisualAssetKind::IslandFoliage,
            label: "near foliage",
            gltf_scene_path: "foliage.gltf",
            animation_clip_names: &[],
            residency: VisualAssetResidency::NearLod,
        },
        VisualAssetSpec {
            kind: VisualAssetKind::WeatherLayer,
            label: "weather",
            gltf_scene_path: "weather.gltf",
            animation_clip_names: &[],
            residency: VisualAssetResidency::Weather,
        },
        VisualAssetSpec {
            kind: VisualAssetKind::DistantImpostor,
            label: "far impostor",
            gltf_scene_path: "far.gltf",
            animation_clip_names: &[],
            residency: VisualAssetResidency::FarLod,
        },
    ];

    let admissions = visual_asset_load_admission_plan(
        &specs,
        |_| true,
        VisualAssetLoadPolicy {
            max_admitted_scene_count: 4,
            max_streaming_admitted_scene_count: 2,
        },
    );

    assert_eq!(
        admissions,
        vec![
            VisualAssetLoadAdmission::Admitted,
            VisualAssetLoadAdmission::Admitted,
            VisualAssetLoadAdmission::Admitted,
            VisualAssetLoadAdmission::Admitted,
            VisualAssetLoadAdmission::Deferred,
            VisualAssetLoadAdmission::Deferred,
            VisualAssetLoadAdmission::Deferred,
        ]
    );
}
