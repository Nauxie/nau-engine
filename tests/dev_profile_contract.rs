use std::fs;
use std::path::Path;

fn manifest_section<'a>(manifest: &'a str, heading: &str) -> &'a str {
    let header = format!("[{heading}]");
    let section_start = manifest
        .find(&header)
        .unwrap_or_else(|| panic!("missing {header} in Cargo.toml"));
    let body = &manifest[section_start + header.len()..];
    let section_end = body.find("\n[").unwrap_or(body.len());
    &body[..section_end]
}

fn setting<'a>(section: &'a str, key: &str) -> Option<&'a str> {
    section.lines().find_map(|line| {
        let line = line.split_once('#').map_or(line, |(value, _)| value).trim();
        let (candidate_key, value) = line.split_once('=')?;
        (candidate_key.trim() == key).then(|| value.trim())
    })
}

#[test]
fn development_profile_keeps_normal_cargo_run_playable() {
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", manifest_path.display()));

    let dev_profile = manifest_section(&manifest, "profile.dev");
    assert_eq!(
        setting(dev_profile, "opt-level"),
        Some("1"),
        "our crate needs light optimization so ordinary `cargo run` remains playable"
    );

    let dependency_profile = manifest_section(&manifest, "profile.dev.package.\"*\"");
    assert_eq!(
        setting(dependency_profile, "opt-level"),
        Some("3"),
        "Bevy and rendering dependencies must stay optimized in development builds"
    );
}
