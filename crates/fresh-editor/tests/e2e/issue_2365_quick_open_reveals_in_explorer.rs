//! E2E regression test for issue #2365: using Quick Open (Ctrl+P) to jump to
//! a file should also expand the file explorer down to that file.
//!
//! Repro (manual):
//!   1. Open the file explorer (Ctrl+B) with a nested project; leave a
//!      sub-directory collapsed.
//!   2. With focus in the editor, press Ctrl+P and jump to a file inside that
//!      collapsed sub-directory.
//!   3. Expected: the explorer expands the ancestor directories and reveals
//!      the freshly opened file.
//!   4. Actual (before the fix): the explorer stays put — the sub-directory
//!      remains collapsed — unless `file_explorer.follow_active_buffer` is
//!      turned on (off by default).
//!
//! Quick Open is an *explicit* "take me to this file" gesture, so it reveals
//! the target regardless of `follow_active_buffer` (which only governs passive
//! following as you cycle tabs). This test therefore runs with the default
//! config (`follow_active_buffer = false`) to prove the reveal is driven by the
//! jump itself.
//!
//! <https://github.com/sinelaw/fresh/issues/2365>

use crate::common::harness::EditorTestHarness;
use crossterm::event::{KeyCode, KeyModifiers};
use std::fs;

#[test]
fn test_quick_open_jump_reveals_file_in_explorer() {
    // Layout:
    //   <root>/
    //     nested/deep_target.txt   ← jumped-to file, initially hidden
    //     sibling/other.txt        ← keeps root from collapsing into a chain
    let mut harness = EditorTestHarness::with_temp_project(120, 40).unwrap();
    let project_root = harness.project_dir().unwrap();

    fs::create_dir_all(project_root.join("nested")).unwrap();
    fs::create_dir_all(project_root.join("sibling")).unwrap();
    fs::write(
        project_root.join("nested/deep_target.txt"),
        "needle-content",
    )
    .unwrap();
    fs::write(project_root.join("sibling/other.txt"), "other").unwrap();

    // Open the explorer but keep focus in the editor — this is the exact
    // scenario from the report (sidebar open while editing). With focus in the
    // editor the explorer renders its tree to the side.
    harness.editor_mut().toggle_file_explorer();
    harness.editor_mut().active_window_mut().focus_editor();

    // Wait until the tree has loaded by waiting for the collapsed `nested`
    // directory row to appear.
    harness.wait_for_file_explorer_item("nested").unwrap();

    // Precondition: `nested` is collapsed, so its child is not in the tree yet.
    assert!(
        !harness.screen_to_string().contains("deep_target.txt"),
        "Precondition: `nested` should be collapsed (deep_target.txt not yet \
         visible) before the Quick Open jump.\nScreen:\n{}",
        harness.screen_to_string()
    );

    // Jump to the nested file via Quick Open. Ctrl+P opens command mode by
    // default; Backspace drops the command prefix to switch to file mode.
    harness
        .send_key(KeyCode::Char('p'), KeyModifiers::CONTROL)
        .unwrap();
    harness
        .send_key(KeyCode::Backspace, KeyModifiers::NONE)
        .unwrap();
    harness.type_text("deep_target.txt").unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();

    // The buffer should open (its content renders)...
    harness
        .wait_until(|h| h.screen_to_string().contains("needle-content"))
        .expect("Quick Open should open the jumped-to file");

    // ...and the explorer should now have expanded `nested` to reveal the
    // freshly opened file. Without the fix this row never appears and the wait
    // times out.
    harness
        .wait_for_file_explorer_item("deep_target.txt")
        .expect("Quick Open jump should expand the explorer down to the opened file");
}
