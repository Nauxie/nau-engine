use super::scene::EvalScene;
use super::semantics::capture_due_checkpoint_screenshots;
use crate::eval_runtime::{EvalCheckpointCapture, EvalRun, path_string};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use std::{fs, io::Write, path::Path, process};

const EVAL_SCREENSHOT_TIMEOUT_FRAMES: u32 = 180;

pub(crate) fn finish_eval_frame(
    mut commands: Commands,
    mut run: ResMut<EvalRun>,
    scene: EvalScene,
    screenshots: Query<(), With<Screenshot>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if !run.finalized {
        run.classify_current_frame_time(!screenshots.is_empty());
    }

    if let Some(error) = run.io_error.clone() {
        eprintln!("{error}");
        run.finalized = true;
        if run.screenshot_path.is_some() {
            terminate_screenshot_eval(false);
        }
        app_exit.write(AppExit::error());
        return;
    }

    if run.finalized {
        if let Some(exit_success) = run.pending_screenshot_exit_success {
            let screenshot_path = run.screenshot_path.clone();
            let mut ready_frames = run.screenshot_ready_frames;
            let artifacts_ready = eval_artifacts_ready_to_exit(
                screenshot_path.as_deref(),
                &run.checkpoint_captures,
                !screenshots.is_empty(),
                &mut ready_frames,
            );
            run.screenshot_ready_frames = ready_frames;
            match artifacts_ready {
                Ok(true) => {
                    if let Err(error) = run.write_island_review_manifest() {
                        run.pending_screenshot_exit_success = None;
                        eprintln!("failed to refresh island review manifest: {error}");
                        terminate_screenshot_eval(false);
                    }
                    run.pending_screenshot_exit_success = None;
                    terminate_screenshot_eval(exit_success);
                }
                Ok(false) => {}
                Err(error) => {
                    run.pending_screenshot_exit_success = None;
                    eprintln!("eval artifact validation failed: {error}");
                    terminate_screenshot_eval(false);
                }
            }

            run.screenshot_wait_frames += 1;
            if run.screenshot_wait_frames > EVAL_SCREENSHOT_TIMEOUT_FRAMES {
                run.pending_screenshot_exit_success = None;
                eprintln!(
                    "eval screenshot did not finish within {} frames",
                    EVAL_SCREENSHOT_TIMEOUT_FRAMES
                );
                terminate_screenshot_eval(false);
            }
        }
        return;
    }

    if let Err(error) = capture_due_checkpoint_screenshots(&mut commands, &mut run, &scene) {
        run.io_error = Some(format!(
            "failed to write checkpoint marker metadata: {error}"
        ));
        return;
    }

    if run.frame < run.scenario.frame_count {
        run.frame += 1;
        return;
    }

    let passed = match run.write_summary() {
        Ok(passed) => passed,
        Err(error) => {
            eprintln!("failed to write eval summary: {error}");
            run.finalized = true;
            if run.screenshot_path.is_some() {
                terminate_screenshot_eval(false);
            }
            app_exit.write(AppExit::error());
            return;
        }
    };

    run.finalized = true;
    eprintln!("eval summary: {}", path_string(&run.summary_path));

    if let Some(screenshot_path) = run.screenshot_path.clone() {
        run.screenshot_wait_frames = 0;
        run.screenshot_ready_frames = 0;
        run.pending_screenshot_exit_success = Some(passed);
        commands.spawn(Screenshot::primary_window()).observe(
            move |captured: On<ScreenshotCaptured>| {
                save_to_disk(screenshot_path.clone())(captured);
            },
        );
    } else if passed {
        run.pending_screenshot_exit_success = Some(true);
        app_exit.write(AppExit::Success);
    } else {
        run.pending_screenshot_exit_success = Some(false);
        app_exit.write(AppExit::error());
    }
}

fn terminate_screenshot_eval(exit_success: bool) -> ! {
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();
    process::exit(eval_process_exit_code(exit_success));
}

const fn eval_process_exit_code(exit_success: bool) -> i32 {
    if exit_success { 0 } else { 1 }
}

fn eval_artifacts_ready_to_exit(
    final_screenshot_path: Option<&Path>,
    checkpoints: &[EvalCheckpointCapture],
    screenshot_in_flight: bool,
    ready_frames: &mut u32,
) -> Result<bool, String> {
    let checkpoint_artifacts_expected = checkpoints
        .iter()
        .any(|checkpoint| checkpoint.capture_requested);
    if final_screenshot_path.is_none() && !checkpoint_artifacts_expected {
        return Ok(true);
    }
    if screenshot_in_flight {
        *ready_frames = 0;
        return Ok(false);
    }

    *ready_frames += 1;
    if *ready_frames < 2 {
        return Ok(false);
    }

    if let Some(path) = final_screenshot_path {
        validate_png_artifact(path, "final screenshot")?;
    }
    for checkpoint in checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.capture_requested)
    {
        if !checkpoint.captured {
            return Err(format!(
                "checkpoint {} was never requested from the renderer",
                checkpoint.name
            ));
        }
        if !checkpoint.marker_metadata_written {
            return Err(format!(
                "checkpoint {} marker metadata was never written",
                checkpoint.name
            ));
        }
        let metadata = decode_json_artifact(
            &checkpoint.marker_metadata_path,
            "checkpoint marker metadata",
        )?;
        if metadata.get("passed").and_then(serde_json::Value::as_bool) != Some(true) {
            return Err(format!(
                "checkpoint marker metadata {} did not pass",
                path_string(&checkpoint.marker_metadata_path)
            ));
        }
        validate_png_artifact(&checkpoint.path, "checkpoint screenshot")?;
    }

    Ok(true)
}

fn validate_png_artifact(path: &Path, label: &str) -> Result<(), String> {
    let metadata =
        fs::metadata(path).map_err(|error| format!("{label} {}: {error}", path_string(path)))?;
    if metadata.len() == 0 {
        return Err(format!("{label} {} is empty", path_string(path)));
    }

    let reader = image::ImageReader::open(path)
        .map_err(|error| format!("{label} {}: {error}", path_string(path)))?
        .with_guessed_format()
        .map_err(|error| format!("{label} {}: {error}", path_string(path)))?;
    let image = reader
        .decode()
        .map_err(|error| format!("{label} {} failed to decode: {error}", path_string(path)))?;
    if image.width() == 0 || image.height() == 0 {
        return Err(format!("{label} {} has no pixels", path_string(path)));
    }
    Ok(())
}

fn decode_json_artifact(path: &Path, label: &str) -> Result<serde_json::Value, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("{label} {}: {error}", path_string(path)))?;
    if bytes.is_empty() {
        return Err(format!("{label} {} is empty", path_string(path)));
    }
    serde_json::from_slice::<serde_json::Value>(&bytes)
        .map_err(|error| format!("{label} {} failed to decode: {error}", path_string(path)))
}

#[cfg(test)]
mod tests {
    use super::{eval_artifacts_ready_to_exit, eval_process_exit_code};
    use crate::eval_runtime::EvalCheckpointCapture;
    use image::{Rgb, RgbImage};
    use std::{
        env, fs, process,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_dir(label: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        env::temp_dir().join(format!(
            "nau_eval_artifacts_{label}_{}_{}",
            process::id(),
            nonce
        ))
    }

    fn write_png(path: &std::path::Path) {
        RgbImage::from_pixel(2, 2, Rgb([12, 34, 56]))
            .save(path)
            .expect("valid png");
    }

    fn checkpoint(
        screenshot_path: std::path::PathBuf,
        metadata_path: std::path::PathBuf,
    ) -> EvalCheckpointCapture {
        EvalCheckpointCapture {
            frame: 8,
            name: "capture".to_string(),
            path: screenshot_path,
            marker_metadata_path: metadata_path,
            capture_requested: true,
            target_island_name: None,
            target_view: None,
            island_index: None,
            pose: None,
            captured: true,
            marker_metadata_written: true,
        }
    }

    #[test]
    fn direct_eval_success_waits_for_and_validates_every_capture_artifact() {
        let temp_dir = temp_dir("complete");
        fs::create_dir_all(&temp_dir).expect("temp dir");
        let final_path = temp_dir.join("final.png");
        let checkpoint_path = temp_dir.join("checkpoint.png");
        let metadata_path = temp_dir.join("checkpoint.markers.json");
        write_png(&final_path);
        write_png(&checkpoint_path);
        fs::write(&metadata_path, br#"{"passed":true}"#).expect("metadata");
        let checkpoints = [checkpoint(checkpoint_path, metadata_path)];
        let mut ready_frames = 0;

        assert!(
            !eval_artifacts_ready_to_exit(Some(&final_path), &checkpoints, true, &mut ready_frames)
                .expect("screenshot entity still present")
        );
        assert!(
            !eval_artifacts_ready_to_exit(
                Some(&final_path),
                &checkpoints,
                false,
                &mut ready_frames
            )
            .expect("first render cleanup frame")
        );
        assert!(
            eval_artifacts_ready_to_exit(Some(&final_path), &checkpoints, false, &mut ready_frames)
                .expect("second render cleanup frame")
        );

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn direct_eval_success_rejects_missing_or_undecodable_checkpoint_pngs() {
        let temp_dir = temp_dir("invalid_png");
        fs::create_dir_all(&temp_dir).expect("temp dir");
        let metadata_path = temp_dir.join("checkpoint.markers.json");
        fs::write(&metadata_path, br#"{"passed":true}"#).expect("metadata");
        let checkpoint_path = temp_dir.join("checkpoint.png");
        let checkpoints = [checkpoint(checkpoint_path.clone(), metadata_path.clone())];

        let mut ready_frames = 1;
        let missing = eval_artifacts_ready_to_exit(None, &checkpoints, false, &mut ready_frames)
            .expect_err("missing checkpoint png must fail");
        assert!(missing.contains("checkpoint screenshot"));

        fs::write(&checkpoint_path, b"not a png").expect("invalid png");
        let mut ready_frames = 1;
        let invalid = eval_artifacts_ready_to_exit(None, &checkpoints, false, &mut ready_frames)
            .expect_err("invalid checkpoint png must fail");
        assert!(invalid.contains("failed to decode"));

        fs::write(&metadata_path, b"not json").expect("invalid metadata");
        write_png(&checkpoint_path);
        let mut ready_frames = 1;
        let invalid = eval_artifacts_ready_to_exit(None, &checkpoints, false, &mut ready_frames)
            .expect_err("invalid checkpoint metadata must fail");
        assert!(invalid.contains("failed to decode"));

        fs::write(&metadata_path, br#"{"passed":false}"#).expect("failed metadata");
        let mut ready_frames = 1;
        let failed = eval_artifacts_ready_to_exit(None, &checkpoints, false, &mut ready_frames)
            .expect_err("failed checkpoint metadata must fail");
        assert!(failed.contains("did not pass"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn direct_eval_success_rejects_unrequested_checkpoint_capture() {
        let temp_dir = temp_dir("not_requested");
        fs::create_dir_all(&temp_dir).expect("temp dir");
        let mut capture = checkpoint(
            temp_dir.join("checkpoint.png"),
            temp_dir.join("checkpoint.markers.json"),
        );
        capture.captured = false;
        let mut ready_frames = 1;
        let error = eval_artifacts_ready_to_exit(None, &[capture], false, &mut ready_frames)
            .expect_err("unrequested capture must fail");
        assert!(error.contains("never requested"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn screenshot_eval_exit_code_preserves_pass_or_fail_status() {
        assert_eq!(eval_process_exit_code(true), 0);
        assert_eq!(eval_process_exit_code(false), 1);
    }
}
