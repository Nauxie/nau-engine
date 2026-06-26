use super::{
    VisualAssetLoadAdmission, VisualAssetLoadPolicy, VisualAssetResidency, VisualAssetSpec,
};

pub fn visual_asset_load_admission_plan(
    specs: &[VisualAssetSpec],
    mut asset_exists: impl FnMut(&VisualAssetSpec) -> bool,
    policy: VisualAssetLoadPolicy,
) -> Vec<VisualAssetLoadAdmission> {
    let mut admissions = vec![VisualAssetLoadAdmission::Missing; specs.len()];
    let mut admitted_always_count = 0;
    let mut streaming_candidates = Vec::new();

    for (index, spec) in specs.iter().enumerate() {
        if !asset_exists(spec) {
            continue;
        }

        if spec.residency == VisualAssetResidency::Always {
            admissions[index] = VisualAssetLoadAdmission::Admitted;
            admitted_always_count += 1;
        } else {
            streaming_candidates.push((index, visual_asset_load_priority(*spec)));
        }
    }

    let remaining_total_budget = policy
        .max_admitted_scene_count
        .saturating_sub(admitted_always_count);
    let streaming_budget = policy
        .max_streaming_admitted_scene_count
        .min(remaining_total_budget);

    streaming_candidates.sort_by_key(|(index, priority)| (*priority, *index));
    for (rank, (index, _)) in streaming_candidates.into_iter().enumerate() {
        admissions[index] = if rank < streaming_budget {
            VisualAssetLoadAdmission::Admitted
        } else {
            VisualAssetLoadAdmission::Deferred
        };
    }

    admissions
}

fn visual_asset_load_priority(spec: VisualAssetSpec) -> u8 {
    match spec.residency {
        VisualAssetResidency::Always => 0,
        VisualAssetResidency::StreamWindow => 1,
        VisualAssetResidency::NearLod => 2,
        VisualAssetResidency::Weather => 3,
        VisualAssetResidency::FarLod => 4,
    }
}
