use nau_engine::world::{IslandArtDirection, IslandWaterStory, island_art_directions};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
    process,
};

const EXPECTED_SCHEMA: &str = "nau_visual_content_export.v2";
const EXPECTED_ISLAND_COUNT: usize = 41;
const LARGE_ISLAND_GROUND_COVER_FOOTPRINT_M2: f64 = 5_000.0;
const MIN_LARGE_ISLAND_AUTHORED_FEATURE_FOOTPRINT_RATIO: f64 = 0.20;
const EXPECTED_BASE_VISUAL_COUNTS: [(&str, usize, usize); EXPECTED_ISLAND_COUNT] = [
    ("launch mesa", 4, 5),
    ("midpoint shelf", 2, 3),
    ("landing garden", 6, 7),
    ("distant crown", 2, 7),
    ("wind overlook", 3, 5),
    ("copper stair", 2, 3),
    ("sunlit terrace", 4, 7),
    ("western refuge", 4, 7),
    ("storm porch", 2, 7),
    ("high orchard", 9, 10),
    ("far needle", 1, 4),
    ("sapphire basin", 6, 10),
    ("broken stair", 2, 7),
    ("mist arch", 4, 7),
    ("cloud gate", 4, 7),
    ("launch spur", 1, 2),
    ("garden apron", 6, 7),
    ("storm shard", 1, 4),
    ("orchard spur", 7, 7),
    ("mist stepping stone", 1, 2),
    ("underbridge cay", 1, 2),
    ("low reef", 2, 3),
    ("quiet lower garden", 8, 10),
    ("lowwind shelf", 4, 7),
    ("upper thermal ring", 3, 5),
    ("needle crownlet", 1, 4),
    ("skyhook basin", 6, 10),
    ("stratos shelf", 6, 10),
    ("cloudfall meadow", 7, 7),
    ("highgate stair", 2, 3),
    ("thin air roost", 1, 4),
    ("summit anvil", 5, 12),
    ("upper sky shelf", 6, 10),
    ("east windchain", 3, 5),
    ("bluevault basin", 8, 10),
    ("outer switchback", 2, 7),
    ("sunspire garden", 8, 10),
    ("cloudbreak stair", 2, 7),
    ("great sky plateau", 14, 16),
    ("far horizon perch", 6, 10),
    ("upper crown", 5, 12),
];
const EXPECTED_ART_DIRECTION_SIGNATURES: [(&str, u64); EXPECTED_ISLAND_COUNT] = [
    ("launch mesa", 1_950_177_616_937_874_794),
    ("midpoint shelf", 1_039_402_781_253_510_536),
    ("landing garden", 5_032_637_163_532_818_367),
    ("distant crown", 2_376_811_426_781_462_688),
    ("wind overlook", 4_007_233_629_631_835_004),
    ("copper stair", 15_244_694_326_718_591_072),
    ("sunlit terrace", 7_785_631_309_120_133_352),
    ("western refuge", 13_325_815_721_683_284_521),
    ("storm porch", 17_202_764_494_286_165_064),
    ("high orchard", 18_271_345_858_129_255_628),
    ("far needle", 11_896_043_717_908_163_131),
    ("sapphire basin", 4_527_789_354_961_795_012),
    ("broken stair", 16_263_160_378_367_860_413),
    ("mist arch", 5_748_006_610_540_834_799),
    ("cloud gate", 8_342_005_617_059_280_315),
    ("launch spur", 6_565_222_993_077_040_768),
    ("garden apron", 6_727_386_422_512_113_634),
    ("storm shard", 9_595_854_373_753_673_989),
    ("orchard spur", 8_082_660_020_755_161_764),
    ("mist stepping stone", 17_094_372_378_274_215_205),
    ("underbridge cay", 6_751_237_824_848_857_753),
    ("low reef", 15_388_621_427_997_492_302),
    ("quiet lower garden", 14_071_350_252_705_351_669),
    ("lowwind shelf", 6_870_270_904_150_723_563),
    ("upper thermal ring", 4_677_079_475_381_317_229),
    ("needle crownlet", 6_636_334_345_419_473_805),
    ("skyhook basin", 7_765_649_081_827_931_181),
    ("stratos shelf", 299_656_254_442_603_734),
    ("cloudfall meadow", 7_578_576_924_348_886_889),
    ("highgate stair", 6_244_459_493_510_280_007),
    ("thin air roost", 5_610_419_615_527_883_258),
    ("summit anvil", 14_024_454_224_475_870_592),
    ("upper sky shelf", 1_162_891_289_449_086_799),
    ("east windchain", 17_762_636_332_851_021_492),
    ("bluevault basin", 13_427_733_260_443_675_892),
    ("outer switchback", 4_637_298_253_918_671_552),
    ("sunspire garden", 14_211_634_766_941_376_156),
    ("cloudbreak stair", 9_135_365_538_050_481_612),
    ("great sky plateau", 17_147_886_957_343_726_532),
    ("far horizon perch", 8_464_004_183_424_388_597),
    ("upper crown", 5_392_483_594_551_306_623),
];

#[derive(Default)]
struct IslandInventory {
    ground_cover_count: usize,
    ground_cover_footprint_m2: Option<f64>,
    tree_labels: Vec<String>,
    rock_labels: Vec<String>,
    landmarks: Vec<Landmark>,
}

struct Landmark {
    kind: String,
    label: String,
    family: Option<String>,
    footprint_m2: Option<f64>,
}

fn main() {
    let args = env::args().skip(1).map(PathBuf::from).collect::<Vec<_>>();
    if args.len() != 1 {
        eprintln!("Usage: cargo run --bin island_art_direction_audit -- <manifest.json>");
        process::exit(2);
    }

    match audit_manifest_path(&args[0]) {
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
            eprintln!("island art-direction audit failed: {error}");
            process::exit(2);
        }
    }
}

fn audit_manifest_path(path: &Path) -> Result<Value, String> {
    let manifest_text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let manifest = serde_json::from_str(&manifest_text).map_err(|error| error.to_string())?;
    Ok(audit_manifest(&manifest, &path.to_string_lossy()))
}

fn audit_manifest(manifest: &Value, manifest_path: &str) -> Value {
    let profiles = island_art_directions();
    let palettes = array(manifest, "palettes");
    let inventory = collect_inventory(manifest);
    let expected_names = profiles
        .iter()
        .map(|profile| profile.island_name.to_string())
        .collect::<BTreeSet<_>>();
    let palette_names = palettes
        .iter()
        .filter_map(|palette| string_field(palette, "island"))
        .collect::<Vec<_>>();
    let palette_name_set = palette_names.iter().cloned().collect::<BTreeSet<_>>();

    let ordered_palettes = profiles.len() == EXPECTED_ISLAND_COUNT
        && palettes.len() == EXPECTED_ISLAND_COUNT
        && profiles.iter().enumerate().all(|(index, profile)| {
            palettes.get(index).is_some_and(|palette| {
                usize_field(palette, "index") == Some(index)
                    && string_field(palette, "island").as_deref() == Some(profile.island_name)
            })
        });
    let unique_islands = palette_names.len() == EXPECTED_ISLAND_COUNT
        && palette_name_set.len() == EXPECTED_ISLAND_COUNT
        && palette_name_set == expected_names;

    let art_signatures = palettes
        .iter()
        .filter_map(|palette| u64_field(palette, "art_direction_signature"))
        .collect::<Vec<_>>();
    let art_signature_set = art_signatures.iter().copied().collect::<BTreeSet<_>>();
    let unique_art_signatures = art_signatures.len() == EXPECTED_ISLAND_COUNT
        && art_signature_set.len() == EXPECTED_ISLAND_COUNT;
    let accepted_art_signatures = accepted_profile_signatures(profiles);

    let palette_signatures = palettes
        .iter()
        .filter_map(palette_signature)
        .collect::<Vec<_>>();
    let palette_signature_set = palette_signatures.iter().cloned().collect::<BTreeSet<_>>();
    let unique_palette_signatures = palette_signatures.len() == EXPECTED_ISLAND_COUNT
        && palette_signature_set.len() == EXPECTED_ISLAND_COUNT;

    let metadata_matches = profiles.iter().enumerate().all(|(index, profile)| {
        palettes
            .get(index)
            .is_some_and(|palette| palette_matches_profile(palette, index, profile))
    });

    let known_inventory_islands = ["ground_cover", "trees", "rocks", "landmarks"]
        .into_iter()
        .flat_map(|field| array(manifest, field))
        .all(|entry| {
            string_field(entry, "island")
                .is_some_and(|island| expected_names.contains(island.as_str()))
        });

    let mut island_reports = Vec::with_capacity(profiles.len());
    let mut exact_features = true;
    let mut required_base_visuals = true;
    let mut hero_landmarks = true;
    let mut water_story_presence = true;
    let mut large_island_authored_feature_coverage = true;
    let mut aggregate_signatures = Vec::with_capacity(profiles.len());

    for (index, profile) in profiles.iter().enumerate() {
        let empty = IslandInventory::default();
        let island = inventory.get(profile.island_name).unwrap_or(&empty);
        let flora = feature_kinds(island, "flora_cluster");
        let formations = feature_kinds(island, "rock_formation");
        let ruins = feature_kinds(island, "ruin_complex");
        let expected_flora = flora_labels(profile);
        let expected_formations = formation_labels(profile);
        let expected_ruins = ruin_labels(profile);
        let features_passed =
            flora == expected_flora && formations == expected_formations && ruins == expected_ruins;
        let expected_base_visual_counts = expected_base_visual_counts(profile.island_name);
        let base_visuals_passed = expected_base_visual_counts.is_some_and(
            |(expected_tree_count, expected_rock_count)| {
                island.ground_cover_count == 1
                    && island.tree_labels.len() == expected_tree_count
                    && island.rock_labels.len() == expected_rock_count
            },
        );
        let matching_heroes = island
            .landmarks
            .iter()
            .filter(|landmark| landmark.kind == "hero_landmark")
            .collect::<Vec<_>>();
        let hero_passed =
            matching_heroes.len() == 1 && matching_heroes[0].label == profile.hero_landmark.label();
        let water_present = island.landmarks.iter().any(is_water_landmark);
        let expected_water = profile.water_story != IslandWaterStory::DryWindCarved;
        let water_passed = water_present == expected_water;
        let authored_feature_footprint_m2 = island
            .landmarks
            .iter()
            .filter(|landmark| is_authored_coverage_landmark(landmark))
            .map(|landmark| landmark.footprint_m2)
            .sum::<Option<f64>>();
        let coverage_ratio = island
            .ground_cover_footprint_m2
            .zip(authored_feature_footprint_m2)
            .filter(|(ground_cover_footprint_m2, _)| *ground_cover_footprint_m2 > 0.0)
            .map(
                |(ground_cover_footprint_m2, authored_feature_footprint_m2)| {
                    authored_feature_footprint_m2 / ground_cover_footprint_m2
                },
            );
        let coverage_passed = island
            .ground_cover_footprint_m2
            .zip(coverage_ratio)
            .is_some_and(|(ground_cover_footprint_m2, coverage_ratio)| {
                ground_cover_footprint_m2 < LARGE_ISLAND_GROUND_COVER_FOOTPRINT_M2
                    || coverage_ratio >= MIN_LARGE_ISLAND_AUTHORED_FEATURE_FOOTPRINT_RATIO
            });

        exact_features &= features_passed;
        required_base_visuals &= base_visuals_passed;
        hero_landmarks &= hero_passed;
        water_story_presence &= water_passed;
        large_island_authored_feature_coverage &= coverage_passed;

        let palette = palettes.get(index);
        if let Some(signature) = aggregate_visual_signature(palette, island) {
            aggregate_signatures.push(signature);
        }

        island_reports.push(json!({
            "index": index,
            "island": profile.island_name,
            "passed": features_passed
                && base_visuals_passed
                && hero_passed
                && water_passed
                && coverage_passed,
            "checks": {
                "surface_features": {
                    "passed": features_passed,
                    "flora": {"actual": flora, "expected": expected_flora},
                    "formations": {"actual": formations, "expected": expected_formations},
                    "ruins": {"actual": ruins, "expected": expected_ruins}
                },
                "base_visuals": {
                    "passed": base_visuals_passed,
                    "ground_cover_count": island.ground_cover_count,
                    "tree_count": island.tree_labels.len(),
                    "rock_count": island.rock_labels.len(),
                    "expected_ground_cover_count": 1,
                    "expected_tree_count":
                        expected_base_visual_counts.map(|(tree_count, _)| tree_count),
                    "expected_rock_count":
                        expected_base_visual_counts.map(|(_, rock_count)| rock_count)
                },
                "hero_landmark": {
                    "passed": hero_passed,
                    "actual_count": matching_heroes.len(),
                    "expected_label": profile.hero_landmark.label()
                },
                "water_presence": {
                    "passed": water_passed,
                    "actual": water_present,
                    "expected": expected_water
                },
                "authored_feature_coverage": {
                    "passed": coverage_passed,
                    "ground_cover_footprint_m2": island.ground_cover_footprint_m2,
                    "authored_feature_footprint_m2": authored_feature_footprint_m2,
                    "coverage_ratio": coverage_ratio,
                    "large_island_threshold_m2": LARGE_ISLAND_GROUND_COVER_FOOTPRINT_M2,
                    "minimum_large_island_ratio":
                        MIN_LARGE_ISLAND_AUTHORED_FEATURE_FOOTPRINT_RATIO
                }
            }
        }));
    }

    let aggregate_signature_set = aggregate_signatures
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let unique_aggregate_signatures = aggregate_signatures.len() == EXPECTED_ISLAND_COUNT
        && aggregate_signature_set.len() == EXPECTED_ISLAND_COUNT;

    let checks = vec![
        check_equal(
            "manifest_schema",
            manifest.get("schema").cloned().unwrap_or(Value::Null),
            json!(EXPECTED_SCHEMA),
        ),
        check_equal(
            "authored_profile_count",
            json!(profiles.len()),
            json!(EXPECTED_ISLAND_COUNT),
        ),
        check_bool(
            "ordered_palette_entries",
            ordered_palettes,
            format!("exactly {EXPECTED_ISLAND_COUNT} entries in profile order"),
        ),
        check_bool(
            "unique_island_entries",
            unique_islands,
            "each authored island appears exactly once",
        ),
        check_bool(
            "unique_art_direction_signatures",
            unique_art_signatures,
            "all art_direction_signature values are present and unique",
        ),
        check_bool(
            "accepted_art_direction_signatures",
            accepted_art_signatures,
            "every behavior-affecting authored profile field matches the accepted baseline",
        ),
        check_bool(
            "unique_palette_signatures",
            unique_palette_signatures,
            "all terrain/foliage/stone palette tuples are present and unique",
        ),
        check_bool(
            "profile_metadata_matches",
            metadata_matches,
            "ordered palette metadata matches island_art_directions()",
        ),
        check_bool(
            "known_inventory_islands",
            known_inventory_islands,
            "all visual entries reference authored islands",
        ),
        check_bool(
            "exact_surface_feature_inventories",
            exact_features,
            "flora, formation, and ruin kinds and counts match every profile",
        ),
        check_bool(
            "required_base_visuals",
            required_base_visuals,
            "every island exactly matches its accepted ground-cover, tree, and rock budgets",
        ),
        check_bool(
            "hero_landmarks",
            hero_landmarks,
            "every island has one matching hero_landmark entry",
        ),
        check_bool(
            "water_story_presence",
            water_story_presence,
            "water is present exactly for non-dry profile stories",
        ),
        check_bool(
            "large_island_authored_feature_coverage",
            large_island_authored_feature_coverage,
            format!(
                "islands with at least {LARGE_ISLAND_GROUND_COVER_FOOTPRINT_M2:.0} m2 of ground-cover footprint retain at least {:.0}% authored feature footprint coverage",
                MIN_LARGE_ISLAND_AUTHORED_FEATURE_FOOTPRINT_RATIO * 100.0
            ),
        ),
        check_bool(
            "unique_aggregate_visual_signatures",
            unique_aggregate_signatures,
            "all complete per-island aggregate visual signatures are unique",
        ),
    ];
    let passed = checks
        .iter()
        .all(|check| check.get("passed").and_then(Value::as_bool) == Some(true));

    json!({
        "schema": "nau_island_art_direction_audit.v1",
        "manifest": manifest_path,
        "passed": passed,
        "checks": checks,
        "islands": island_reports
    })
}

fn collect_inventory(manifest: &Value) -> BTreeMap<String, IslandInventory> {
    let mut inventory = BTreeMap::<String, IslandInventory>::new();

    for entry in array(manifest, "ground_cover") {
        if let Some(island) = string_field(entry, "island") {
            let island = inventory.entry(island).or_default();
            island.ground_cover_count += 1;
            if let Some(footprint_m2) = mesh_footprint_m2(entry) {
                island.ground_cover_footprint_m2 = Some(
                    island
                        .ground_cover_footprint_m2
                        .map_or(footprint_m2, |current| current.max(footprint_m2)),
                );
            }
        }
    }
    for entry in array(manifest, "trees") {
        if let Some(island) = string_field(entry, "island") {
            inventory
                .entry(island)
                .or_default()
                .tree_labels
                .push(string_field(entry, "label").unwrap_or_default());
        }
    }
    for entry in array(manifest, "rocks") {
        if let Some(island) = string_field(entry, "island") {
            inventory
                .entry(island)
                .or_default()
                .rock_labels
                .push(string_field(entry, "label").unwrap_or_default());
        }
    }
    for entry in array(manifest, "landmarks") {
        if let Some(island) = string_field(entry, "island") {
            inventory
                .entry(island)
                .or_default()
                .landmarks
                .push(Landmark {
                    kind: string_field(entry, "kind").unwrap_or_default(),
                    label: string_field(entry, "label").unwrap_or_default(),
                    family: string_field(entry, "surface_feature_family"),
                    footprint_m2: mesh_footprint_m2(entry),
                });
        }
    }

    inventory
}

fn expected_base_visual_counts(island_name: &str) -> Option<(usize, usize)> {
    EXPECTED_BASE_VISUAL_COUNTS
        .iter()
        .find_map(|(name, tree_count, rock_count)| {
            (*name == island_name).then_some((*tree_count, *rock_count))
        })
}

fn expected_art_direction_signature(island_name: &str) -> Option<u64> {
    EXPECTED_ART_DIRECTION_SIGNATURES
        .iter()
        .find_map(|(name, signature)| (*name == island_name).then_some(*signature))
}

fn accepted_profile_signatures(profiles: &[IslandArtDirection]) -> bool {
    profiles.len() == EXPECTED_ISLAND_COUNT
        && profiles.iter().all(|profile| {
            expected_art_direction_signature(profile.island_name) == Some(profile.signature())
        })
}

fn palette_matches_profile(palette: &Value, index: usize, profile: &IslandArtDirection) -> bool {
    usize_field(palette, "index") == Some(index)
        && string_field(palette, "island").as_deref() == Some(profile.island_name)
        && string_field(palette, "epithet").as_deref() == Some(profile.epithet)
        && string_field(palette, "palette_family").as_deref()
            == Some(profile.palette_family.label())
        && string_field(palette, "surface_pattern").as_deref()
            == Some(profile.surface_pattern.label())
        && string_field(palette, "hero_landmark").as_deref() == Some(profile.hero_landmark.label())
        && string_field(palette, "water_story").as_deref() == Some(profile.water_story.label())
        && u64_field(palette, "art_direction_signature") == Some(profile.signature())
        && string_array(palette, "flora_kinds") == flora_labels(profile)
        && string_array(palette, "formation_kinds") == formation_labels(profile)
        && string_array(palette, "ruin_kinds") == ruin_labels(profile)
}

fn flora_labels(profile: &IslandArtDirection) -> Vec<String> {
    profile
        .flora_kinds
        .iter()
        .take(usize::from(profile.flora_count))
        .map(|kind| kind.label().to_string())
        .collect()
}

fn formation_labels(profile: &IslandArtDirection) -> Vec<String> {
    profile
        .formation_kinds
        .iter()
        .take(usize::from(profile.formation_count))
        .map(|kind| kind.label().to_string())
        .collect()
}

fn ruin_labels(profile: &IslandArtDirection) -> Vec<String> {
    profile
        .ruin_kinds
        .iter()
        .take(usize::from(profile.ruin_count))
        .map(|kind| kind.label().to_string())
        .collect()
}

fn feature_kinds(inventory: &IslandInventory, family: &str) -> Vec<String> {
    inventory
        .landmarks
        .iter()
        .filter(|landmark| landmark.family.as_deref() == Some(family))
        .map(|landmark| landmark.kind.clone())
        .collect()
}

fn is_water_landmark(landmark: &Landmark) -> bool {
    landmark.family.as_deref() == Some("water_detail")
        || matches!(
            landmark.kind.as_str(),
            "pond_surface"
                | "plateau_lake_surface"
                | "river_channel"
                | "route_lake_surface"
                | "plateau_waterfall_ribbon"
                | "plateau_waterfall_mist"
                | "route_waterfall_ribbon"
                | "route_waterfall_mist"
        )
}

fn is_authored_coverage_landmark(landmark: &Landmark) -> bool {
    landmark.kind == "hero_landmark"
        || matches!(
            landmark.family.as_deref(),
            Some("flora_cluster" | "ruin_complex" | "rock_formation" | "water_detail")
        )
        || is_water_landmark(landmark)
}

fn mesh_footprint_m2(entry: &Value) -> Option<f64> {
    let mesh = entry.get("mesh")?;
    let horizontal_span_m = f64_field(mesh, "horizontal_span_m")?;
    let depth_span_m = f64_field(mesh, "depth_span_m")?;
    (horizontal_span_m.is_finite()
        && depth_span_m.is_finite()
        && horizontal_span_m > 0.0
        && depth_span_m > 0.0)
        .then_some(horizontal_span_m * depth_span_m)
}

fn palette_signature(palette: &Value) -> Option<String> {
    let terrain = color_key(palette, "terrain_key")?;
    let foliage = color_key(palette, "foliage_key")?;
    let stone = color_key(palette, "stone_key")?;
    Some(format!("{terrain:?}|{foliage:?}|{stone:?}"))
}

fn aggregate_visual_signature(
    palette: Option<&Value>,
    inventory: &IslandInventory,
) -> Option<String> {
    let palette = palette?;
    let palette_signature = palette_signature(palette)?;
    let mut trees = inventory.tree_labels.clone();
    let mut rocks = inventory.rock_labels.clone();
    trees.sort();
    rocks.sort();
    let mut landmarks = BTreeMap::<String, usize>::new();
    for landmark in &inventory.landmarks {
        let key = format!(
            "{}|{}|{}",
            landmark.family.as_deref().unwrap_or("landmark"),
            landmark.kind,
            landmark.label
        );
        *landmarks.entry(key).or_default() += 1;
    }

    serde_json::to_string(&json!({
        "palette": palette_signature,
        "surface_pattern": string_field(palette, "surface_pattern"),
        "hero_landmark": string_field(palette, "hero_landmark"),
        "water_story": string_field(palette, "water_story"),
        "flora": string_array(palette, "flora_kinds"),
        "formations": string_array(palette, "formation_kinds"),
        "ruins": string_array(palette, "ruin_kinds"),
        "ground_cover_count": inventory.ground_cover_count,
        "trees": trees,
        "rocks": rocks,
        "landmarks": landmarks
    }))
    .ok()
}

fn array<'a>(value: &'a Value, field: &str) -> &'a [Value] {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn string_field(value: &Value, field: &str) -> Option<String> {
    value.get(field).and_then(Value::as_str).map(str::to_string)
}

fn usize_field(value: &Value, field: &str) -> Option<usize> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .and_then(|number| usize::try_from(number).ok())
}

fn u64_field(value: &Value, field: &str) -> Option<u64> {
    value.get(field).and_then(|value| {
        value
            .as_u64()
            .or_else(|| value.as_str().and_then(|value| value.parse().ok()))
    })
}

fn f64_field(value: &Value, field: &str) -> Option<f64> {
    value.get(field).and_then(Value::as_f64)
}

fn string_array(value: &Value, field: &str) -> Vec<String> {
    array(value, field)
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

fn color_key(value: &Value, field: &str) -> Option<[u64; 3]> {
    let values = array(value, field);
    if values.len() != 3 {
        return None;
    }
    let color = [
        values[0].as_u64()?,
        values[1].as_u64()?,
        values[2].as_u64()?,
    ];
    color.iter().all(|channel| *channel <= 255).then_some(color)
}

fn check_bool(name: &str, passed: bool, requirement: impl Into<String>) -> Value {
    json!({
        "name": name,
        "passed": passed,
        "requirement": requirement.into()
    })
}

fn check_equal(name: &str, actual: Value, expected: Value) -> Value {
    json!({
        "name": name,
        "passed": actual == expected,
        "actual": actual,
        "expected": expected
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_synthetic_fixture_passes() {
        let report = audit_manifest(&synthetic_manifest(), "synthetic/manifest.json");
        assert!(report["passed"].as_bool().unwrap());
    }

    #[test]
    fn missing_island_is_rejected() {
        let mut manifest = synthetic_manifest();
        manifest["palettes"].as_array_mut().unwrap().remove(7);

        let report = audit_manifest(&manifest, "synthetic/manifest.json");
        assert_failed(&report, "ordered_palette_entries");
        assert_failed(&report, "unique_island_entries");
    }

    #[test]
    fn duplicated_island_is_rejected() {
        let mut manifest = synthetic_manifest();
        let duplicate = manifest["palettes"][4].clone();
        manifest["palettes"].as_array_mut().unwrap().push(duplicate);

        let report = audit_manifest(&manifest, "synthetic/manifest.json");
        assert_failed(&report, "ordered_palette_entries");
        assert_failed(&report, "unique_island_entries");
    }

    #[test]
    fn mismatched_island_is_rejected() {
        let mut manifest = synthetic_manifest();
        manifest["palettes"][0]["island"] = json!("not an authored island");

        let report = audit_manifest(&manifest, "synthetic/manifest.json");
        assert_failed(&report, "ordered_palette_entries");
        assert_failed(&report, "unique_island_entries");
        assert_failed(&report, "profile_metadata_matches");
    }

    #[test]
    fn sparse_large_island_authored_feature_coverage_is_rejected() {
        let mut manifest = synthetic_manifest();
        let island_name = island_art_directions()[22].island_name;
        let ground_cover = manifest["ground_cover"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|entry| entry["island"].as_str() == Some(island_name))
            .expect("synthetic ground cover");
        ground_cover["mesh"] = synthetic_mesh(100.0, 100.0);
        for landmark in manifest["landmarks"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .filter(|entry| entry["island"].as_str() == Some(island_name))
        {
            landmark["mesh"] = synthetic_mesh(1.0, 1.0);
        }

        let report = audit_manifest(&manifest, "synthetic/manifest.json");
        assert_failed(&report, "large_island_authored_feature_coverage");
    }

    #[test]
    fn reduced_base_visual_budget_is_rejected() {
        let mut manifest = synthetic_manifest();
        let island_name = "great sky plateau";
        for field in ["trees", "rocks"] {
            let entries = manifest[field].as_array_mut().unwrap();
            let mut kept_island_entry = false;
            entries.retain(|entry| {
                if entry["island"].as_str() != Some(island_name) {
                    return true;
                }
                if kept_island_entry {
                    false
                } else {
                    kept_island_entry = true;
                    true
                }
            });
        }

        let report = audit_manifest(&manifest, "synthetic/manifest.json");
        assert_failed(&report, "required_base_visuals");
        let island = report["islands"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["island"].as_str() == Some(island_name))
            .expect("great sky plateau report");
        assert_eq!(island["checks"]["base_visuals"]["tree_count"], json!(1));
        assert_eq!(
            island["checks"]["base_visuals"]["expected_tree_count"],
            json!(14)
        );
        assert_eq!(island["checks"]["base_visuals"]["rock_count"], json!(1));
        assert_eq!(
            island["checks"]["base_visuals"]["expected_rock_count"],
            json!(16)
        );
    }

    #[test]
    fn authored_profile_spatial_drift_is_rejected() {
        let mut profiles = island_art_directions().to_vec();
        profiles[0].hero_anchor[0] += 0.01;

        assert!(!accepted_profile_signatures(&profiles));
    }

    fn synthetic_manifest() -> Value {
        let profiles = island_art_directions();
        let mut palettes = Vec::with_capacity(profiles.len());
        let mut ground_cover = Vec::with_capacity(profiles.len());
        let mut trees = Vec::with_capacity(profiles.len());
        let mut rocks = Vec::with_capacity(profiles.len());
        let mut landmarks = Vec::new();

        for (index, profile) in profiles.iter().enumerate() {
            let channel = u8::try_from(index).unwrap();
            palettes.push(json!({
                "index": index,
                "island": profile.island_name,
                "epithet": profile.epithet,
                "palette_family": profile.palette_family.label(),
                "surface_pattern": profile.surface_pattern.label(),
                "hero_landmark": profile.hero_landmark.label(),
                "water_story": profile.water_story.label(),
                "art_direction_signature": profile.signature().to_string(),
                "flora_kinds": flora_labels(profile),
                "formation_kinds": formation_labels(profile),
                "ruin_kinds": ruin_labels(profile),
                "terrain_key": [channel, 17, 31],
                "foliage_key": [channel, 47, 61],
                "stone_key": [channel, 79, 97]
            }));
            ground_cover.push(json!({
                "island": profile.island_name,
                "mesh": synthetic_mesh(40.0, 40.0)
            }));
            let (tree_count, rock_count) =
                expected_base_visual_counts(profile.island_name).expect("accepted base budget");
            for tree_index in 0..tree_count {
                trees.push(json!({
                    "island": profile.island_name,
                    "label": format!("tree {index} {tree_index}")
                }));
            }
            for rock_index in 0..rock_count {
                rocks.push(json!({
                    "island": profile.island_name,
                    "label": format!("rock {index} {rock_index}")
                }));
            }
            landmarks.push(json!({
                "island": profile.island_name,
                "kind": "hero_landmark",
                "label": profile.hero_landmark.label(),
                "surface_feature_family": null,
                "mesh": synthetic_mesh(20.0, 20.0)
            }));
            for kind in flora_labels(profile) {
                landmarks.push(surface_feature(profile.island_name, "flora_cluster", &kind));
            }
            for kind in ruin_labels(profile) {
                landmarks.push(surface_feature(profile.island_name, "ruin_complex", &kind));
            }
            for kind in formation_labels(profile) {
                landmarks.push(surface_feature(
                    profile.island_name,
                    "rock_formation",
                    &kind,
                ));
            }
            if profile.water_story != IslandWaterStory::DryWindCarved {
                landmarks.push(json!({
                    "island": profile.island_name,
                    "kind": "pond_surface",
                    "label": "authored water",
                    "surface_feature_family": null,
                    "mesh": synthetic_mesh(20.0, 20.0)
                }));
            }
        }

        json!({
            "schema": EXPECTED_SCHEMA,
            "ground_cover": ground_cover,
            "trees": trees,
            "rocks": rocks,
            "landmarks": landmarks,
            "palettes": palettes
        })
    }

    fn surface_feature(island: &str, family: &str, kind: &str) -> Value {
        json!({
            "island": island,
            "kind": kind,
            "label": kind,
            "surface_feature_family": family,
            "mesh": synthetic_mesh(20.0, 20.0)
        })
    }

    fn synthetic_mesh(horizontal_span_m: f64, depth_span_m: f64) -> Value {
        json!({
            "horizontal_span_m": horizontal_span_m,
            "depth_span_m": depth_span_m
        })
    }

    fn assert_failed(report: &Value, name: &str) {
        assert!(!report["passed"].as_bool().unwrap());
        assert!(
            report["checks"].as_array().unwrap().iter().any(|check| {
                check["name"].as_str() == Some(name) && check["passed"].as_bool() == Some(false)
            }),
            "expected failed check {name}"
        );
    }
}
