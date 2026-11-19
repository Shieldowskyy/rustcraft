use crate::app::{InputState, RustCraftApp};

use super::{AutoTestRunner, AutoTestStage, TestRecord};

#[test]
fn test_autotest_runner_initial_state() {
    let runner = AutoTestRunner::new();
    assert_eq!(runner.stage, AutoTestStage::Init);
    assert_eq!(runner.passed_count(), 0);
    assert_eq!(runner.total_count(), 0);
    assert!(!runner.is_positive());
    assert!(!runner.is_completed());
    assert!(!runner.dismissed);
}

#[test]
fn test_autotest_runner_scoring_logic() {
    let mut runner = AutoTestRunner::new();
    runner.records.push(TestRecord {
        id: 1,
        name: "Test 1",
        passed: true,
        message: "Ok".to_string(),
        duration_ms: 10,
    });
    runner.records.push(TestRecord {
        id: 2,
        name: "Test 2",
        passed: true,
        message: "Ok".to_string(),
        duration_ms: 12,
    });
    assert_eq!(runner.passed_count(), 2);
    assert_eq!(runner.total_count(), 2);
    assert!(runner.is_positive());

    runner.records.push(TestRecord {
        id: 3,
        name: "Test 3",
        passed: false,
        message: "Failed assertion".to_string(),
        duration_ms: 15,
    });
    assert_eq!(runner.passed_count(), 2);
    assert_eq!(runner.total_count(), 3);
    assert!(!runner.is_positive());
}

#[test]
fn test_autotest_dismissal_toggle() {
    let mut runner = AutoTestRunner::new();
    runner.stage = AutoTestStage::Completed;
    assert!(!runner.dismissed);

    let mut input = InputState::default();
    input.key_escape = true;
    runner.handle_input(&input);
    assert!(runner.dismissed);

    input.key_escape = false;
    input.key_enter = true;
    runner.handle_input(&input);
    assert!(!runner.dismissed);
}

#[test]
fn test_autotest_render_overlay_execution() {
    let runner = AutoTestRunner::new();
    let width = 320;
    let height = 180;
    let mut buffer = vec![0u32; width * height];

    runner.render_overlay(&mut buffer, width, height);
    assert!(buffer.iter().any(|&px| (px & 0x00FFFFFF) != 0));

    let mut completed_runner = AutoTestRunner::new();
    completed_runner.stage = AutoTestStage::Completed;
    completed_runner.records.push(TestRecord {
        id: 1,
        name: "World Gen",
        passed: true,
        message: "16 chunks".to_string(),
        duration_ms: 25,
    });
    let mut card_buffer = vec![0u32; width * height];
    completed_runner.render_overlay(&mut card_buffer, width, height);
    assert!(card_buffer.iter().any(|&px| (px & 0x00FFFFFF) != 0));

    completed_runner.dismissed = true;
    let mut badge_buffer = vec![0u32; width * height];
    completed_runner.render_overlay(&mut badge_buffer, width, height);
    assert!(badge_buffer.iter().any(|&px| (px & 0x00FFFFFF) != 0));
}

#[test]
fn test_autotest_full_execution_cycle() {
    let mut app = RustCraftApp::new(320, 180, "software", 12345);
    app.autotest = Some(AutoTestRunner::new());

    let input = InputState::default();
    let dt = 0.05;
    let max_frames = 200;

    for _ in 0..max_frames {
        let _ = app.update_and_render(dt, &input);
        if let Some(ref runner) = app.autotest {
            if runner.is_completed() {
                break;
            }
        }
    }

    let runner = app.autotest.as_ref().expect("Autotest runner should exist");
    assert!(
        runner.is_completed(),
        "Autotest runner did not reach Completed stage"
    );
    assert_eq!(runner.total_count(), 15, "Expected 15 tests executed");
    assert_eq!(runner.passed_count(), 15, "All 15 tests should pass");
    assert!(
        runner.is_positive(),
        "Overall test verdict should be POSITIVE"
    );
}
