---
status: resolved
phase: 05-gui-integration
source: 05-01-SUMMARY.md, 05-02-SUMMARY.md, 05-03-SUMMARY.md, 05-04-SUMMARY.md
started: 2026-03-17T08:00:00Z
updated: 2026-03-17T08:00:00Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

[testing complete]

## Tests

### 1. Build Link Parse and Preview
expected: Paste a d2core.com build link. The app fetches the build data and displays a preview showing skill names and paragon board info. Invalid links show a bilingual error message.
result: pass

### 2. Variant Selector
expected: For builds with multiple variants, a dropdown selector appears above the build preview. Changing the selected variant updates the preview. For single-variant builds, no dropdown shown.
result: pass

### 3. Calibration Warning on Startup
expected: On first launch (no calibration.json saved), an amber warning banner appears and the Start button is disabled. The warning indicates calibration is needed.
result: pass

### 4. Calibration Wizard — Screenshot Capture
expected: Click the Calibrate button. The wizard opens and captures a screenshot of the Diablo IV game window, displayed as a fullscreen overlay with a crosshair cursor.
result: pass

### 5. Calibration Wizard — 5-Step Click Flow
expected: The wizard guides through 5 steps with bilingual instructions (skill allocate button, skill panel origin, paragon center, paragon nav next, paragon nav prev). Each click places a gold dot marker on the screenshot. After all 5 steps, calibration is saved and Start button becomes enabled.
result: issue
reported: "第4步定位完成后，只能返回上一步或者取消，流程无法继续到第5步。另外第4步说明不明确，不清楚具体点击哪里——是否要将技能面板向左全部展开，十字定位到底部的技能分配按钮？"
severity: major

### 6. Apply Controls — Start/Pause/Stop
expected: After calibration and build loaded, clicking Start begins automation. Pause button pauses mid-execution. Stop button halts and resets to Idle. Button states change correctly (disabled/enabled based on ApplyPhase).
result: skipped
reason: Depends on calibration wizard (Test 5 blocked), cannot complete calibration to test

### 7. Real-Time Progress Bar
expected: During automation, a progress bar updates in real-time showing current step progress. Step descriptions visible as automation proceeds.
result: skipped
reason: Depends on calibration wizard (Test 5 blocked), cannot start automation to test

### 8. Bilingual Error Messages
expected: When errors occur (window not found, safety check fail, etc.), error messages display in both Chinese and English format.
result: skipped
reason: Depends on calibration wizard (Test 5 blocked), cannot trigger automation errors to test

## Summary

total: 8
passed: 4
issues: 1
pending: 0
skipped: 3
skipped: 0

## Gaps

- truth: "Calibration wizard completes all 5 steps and saves calibration data"
  status: resolved
  reason: "User reported: 第4步定位完成后只能返回或取消，无法继续到第5步（流程卡住）。第4步说明不明确，不清楚具体点击位置。"
  severity: major
  test: 5
  root_cause: "Wizard captures one screenshot (skill tree) at start but steps 3-5 need paragon board screenshot. User sees skill tree screenshot while asked to identify paragon elements — cannot meaningfully click. Also step descriptions unclear."
  artifacts:
    - path: "src/App.tsx"
      issue: "No recapture mechanism between skill tree and paragon phases; vague step descriptions"
  missing:
    - "Recapture prompt between step 2 and step 3 for paragon board screenshot"
    - "Recapture button available at any step"
    - "Clearer Chinese descriptions specifying click targets"
  fix_commit: "70f9008"
