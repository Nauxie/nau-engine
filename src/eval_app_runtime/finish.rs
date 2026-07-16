use super::scene::EvalScene;
use super::semantics::capture_due_checkpoint_screenshots;
use crate::eval_runtime::{EvalRun, path_string};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use std::{
    fs,
    io::{ErrorKind, Write},
    path::Path,
    process,
};

const EVAL_SCREENSHOT_TIMEOUT_FRAMES: u32 = 180;

pub(crate) fn finish_eval_frame(
    mut commands: Commands,
    mut run: ResMut<EvalRun>,
    scene: EvalScene,
    screenshots: Query<(), With<Screenshot>>,
    mut app_exit: MessageWriter<AppExit>,
) {
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
            if run.screenshot_path.is_none() {
                app_exit.write(if exit_success {
                    AppExit::Success
                } else {
                    AppExit::error()
                });
                return;
            }

            let screenshot_path = run
                .screenshot_path
                .clone()
                .expect("screenshot path checked above");
            match screenshot_ready_to_exit(
                &screenshot_path,
                !screenshots.is_empty(),
                &mut run.screenshot_ready_frames,
            ) {
                Ok(true) => {
                    run.pending_screenshot_exit_success = None;
                    terminate_screenshot_eval(exit_success);
                }
                Ok(false) => {}
                Err(error) => {
                    run.pending_screenshot_exit_success = None;
                    eprintln!(
                        "failed to read eval screenshot {}: {error}",
                        path_string(&screenshot_path)
                    );
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

fn screenshot_ready_to_exit(
    path: &Path,
    screenshot_in_flight: bool,
    ready_frames: &mut u32,
) -> Result<bool, String> {
    if screenshot_file_ready(path)? && !screenshot_in_flight {
        *ready_frames += 1;
        Ok(*ready_frames >= 2)
    } else {
        *ready_frames = 0;
        Ok(false)
    }
}

fn screenshot_file_ready(path: &Path) -> Result<bool, String> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if retryable_screenshot_io_error(error.kind()) => return Ok(false),
        Err(error) => return Err(error.to_string()),
    };
    if metadata.len() == 0 {
        return Ok(false);
    }

    let reader = match image::ImageReader::open(path) {
        Ok(reader) => reader,
        Err(error) if retryable_screenshot_io_error(error.kind()) => return Ok(false),
        Err(error) => return Err(error.to_string()),
    };
    let reader = match reader.with_guessed_format() {
        Ok(reader) => reader,
        Err(error) if retryable_screenshot_io_error(error.kind()) => return Ok(false),
        Err(error) => return Err(error.to_string()),
    };

    match reader.decode() {
        Ok(image) => Ok(image.width() > 0 && image.height() > 0),
        Err(image::ImageError::IoError(error)) if retryable_screenshot_io_error(error.kind()) => {
            Ok(false)
        }
        Err(image::ImageError::IoError(error)) => Err(error.to_string()),
        Err(_) => Ok(false),
    }
}

fn retryable_screenshot_io_error(kind: ErrorKind) -> bool {
    matches!(
        kind,
        ErrorKind::NotFound
            | ErrorKind::Interrupted
            | ErrorKind::UnexpectedEof
            | ErrorKind::WouldBlock
    )
}

#[cfg(test)]
mod tests {
    use super::{eval_process_exit_code, screenshot_file_ready, screenshot_ready_to_exit};
    use image::{Rgb, RgbImage};
    use std::{env, fs, process};

    #[test]
    fn final_screenshot_readiness_requires_a_decodable_image() {
        let temp_dir = env::temp_dir().join(format!(
            "nau_final_screenshot_readiness_{}_{}",
            process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        fs::create_dir_all(&temp_dir).expect("temp dir");
        let screenshot_path = temp_dir.join("final.png");

        assert!(!screenshot_file_ready(&screenshot_path).expect("missing file is pending"));
        fs::write(&screenshot_path, b"not a complete png").expect("partial screenshot");
        assert!(!screenshot_file_ready(&screenshot_path).expect("invalid image is pending"));

        RgbImage::from_pixel(2, 2, Rgb([12, 34, 56]))
            .save(&screenshot_path)
            .expect("valid screenshot");
        assert!(screenshot_file_ready(&screenshot_path).expect("readable screenshot"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn final_screenshot_waits_for_entity_and_render_cleanup_before_exit() {
        let temp_dir = env::temp_dir().join(format!(
            "nau_final_screenshot_cleanup_{}_{}",
            process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        fs::create_dir_all(&temp_dir).expect("temp dir");
        let screenshot_path = temp_dir.join("final.png");
        RgbImage::from_pixel(2, 2, Rgb([12, 34, 56]))
            .save(&screenshot_path)
            .expect("valid screenshot");
        let mut ready_frames = 0;

        assert!(
            !screenshot_ready_to_exit(&screenshot_path, true, &mut ready_frames)
                .expect("screenshot entity still present")
        );
        assert!(
            !screenshot_ready_to_exit(&screenshot_path, false, &mut ready_frames)
                .expect("first render cleanup frame")
        );
        assert!(
            screenshot_ready_to_exit(&screenshot_path, false, &mut ready_frames)
                .expect("second render cleanup frame")
        );

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn screenshot_eval_exit_code_preserves_pass_or_fail_status() {
        assert_eq!(eval_process_exit_code(true), 0);
        assert_eq!(eval_process_exit_code(false), 1);
    }

    #[test]
    fn final_screenshot_readiness_surfaces_filesystem_errors() {
        let temp_dir = env::temp_dir().join(format!(
            "nau_final_screenshot_io_error_{}_{}",
            process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        fs::create_dir_all(&temp_dir).expect("temp dir");
        let parent_file = temp_dir.join("not-a-directory");
        fs::write(&parent_file, b"file").expect("parent file");

        assert!(screenshot_file_ready(&parent_file.join("final.png")).is_err());

        let _ = fs::remove_dir_all(temp_dir);
    }
}
