# OxiJade UI / SFTP Polish Design
**Date**: 2026-04-05
**Project**: OxiJade
**Platform**: Windows

---

## Overview

This design refines the current OxiJade terminal UI and SFTP interaction model in four places the user called out:

1. Remove the purple strip shown above the active conversation tab.
2. Add visible upload feedback for drag-and-drop uploads, including real percentage progress and clear success/failure states.
3. Redesign the left local/SSH sidebar so it feels intentional instead of flat and unfinished.
4. Make the active tab more obvious with slightly larger size and stronger visual contrast.

The approved visual direction is:

- Modern terminal feel
- Cold neon palette
- Card-based session list
- Real upload percentage plus stage-based feedback
- Chosen implementation direction: `Approach A: Neon Card Terminal`

This is a focused polish pass. It improves UI language and upload feedback without changing unrelated terminal, split-pane, or SSH connection behavior.

---

## Problems

### 1. Active tab highlight is noisy and not effective

The current purple strip is not a bug in rendering; it is the current active-tab accent treatment in [tab_bar.rs](C:\Users\lccxxo\rust\rust_projects\OxiJade\oxijade-app\src\panels\tab_bar.rs). It reads as a stray bar instead of a deliberate selection indicator.

### 2. Upload feedback is too coarse

The current SFTP panel only models broad upload states like `Uploading(String)` and `UploadOk(String)` in [sftp_panel.rs](C:\Users\lccxxo\rust\rust_projects\OxiJade\oxijade-app\src\panels\sftp_panel.rs). Drag-and-drop uploads enqueue work, but the user cannot see:

- Whether the drop was accepted
- Which file is uploading
- How many files are queued
- Real upload percentage
- Whether the batch partially failed

### 3. Sidebar presentation is too flat

The current left sidebar is still visually close to a text list with light row highlighting. It does not express hierarchy, connection state, or active context strongly enough.

### 4. Theme tokens no longer match the desired product direction

The current theme in [theme.rs](C:\Users\lccxxo\rust\rust_projects\OxiJade\oxijade-app\src\theme.rs) is built around darker GitHub-like panels plus a purple SSH accent. The approved direction is colder and more terminal-like, with cyan/blue-green emphasis rather than purple.

---

## Options Considered

### Option A: Neon Card Terminal

Use pill-like active tabs, card-based session rows, cold neon accent tokens, and a dedicated upload task card with real progress. This is the approved option.

Pros:

- Stronger perceived quality without becoming gaudy
- Makes active/connected/idle states much clearer
- Supports both percentage progress and stage-based feedback cleanly
- Fits a terminal product better than a generic dashboard style

Cons:

- Requires light refactoring of SFTP state and response flow
- More UI state than a pure visual restyle

### Option B: Dense Utility Tool

Keep everything compact and information-dense with a lighter restyle.

Pros:

- Minimal implementation risk
- Feels efficient

Cons:

- Does not address the "ugly" feedback strongly enough
- Active tab emphasis remains weaker

### Option C: High-Glow Showcase

Push stronger gradients and glow-heavy neon styling.

Pros:

- High visual impact

Cons:

- Easier to overdo
- Competes with terminal content
- Higher fatigue during long sessions

Decision: `Option A`

---

## Goals

- Remove the visual impression of a stray purple line from the top tab bar.
- Make the active tab obviously current at a glance.
- Upgrade the left sidebar into a card-based session list with clear state hierarchy.
- Show upload lifecycle clearly for both file dialog upload and drag-and-drop upload.
- Report real upload percentage from the core transfer path, not fake UI progress.
- Keep the visual language consistent across tab bar, sidebar, SFTP panel, and status feedback.

---

## Non-Goals

- No redesign of terminal text rendering.
- No changes to split-pane behavior beyond visual harmony.
- No change to SSH connection semantics.
- No new persistence format for profiles or settings.
- No download progress tracking in this pass unless it naturally falls out of the same event model.

---

## Affected Files

### App UI

- [theme.rs](C:\Users\lccxxo\rust\rust_projects\OxiJade\oxijade-app\src\theme.rs)
- [app.rs](C:\Users\lccxxo\rust\rust_projects\OxiJade\oxijade-app\src\app.rs)
- [tab_bar.rs](C:\Users\lccxxo\rust\rust_projects\OxiJade\oxijade-app\src\panels\tab_bar.rs)
- [sidebar.rs](C:\Users\lccxxo\rust\rust_projects\OxiJade\oxijade-app\src\panels\sidebar.rs)
- [sftp_panel.rs](C:\Users\lccxxo\rust\rust_projects\OxiJade\oxijade-app\src\panels\sftp_panel.rs)

### Core Transfer Path

- [mod.rs](C:\Users\lccxxo\rust\rust_projects\OxiJade\oxijade-core\src\sftp\mod.rs)

---

## Chosen Design

### 1. Theme Tokens and Visual Language

Replace the current purple-heavy SSH accent language with a cold neon system:

- Primary accent: cyan-blue for active focus and progress
- Secondary accent: blue-green for connected/healthy states
- Success: soft mint
- Error: muted coral-red
- Surfaces: deep slate panels with subtle contrast steps
- Text: bright foreground with restrained muted text

Visual intensity stays controlled:

- Thin bright borders instead of thick glow
- Small glow or elevated contrast only on active surfaces
- Dark base surfaces so the terminal remains the visual anchor

This keeps the modern terminal feel while avoiding the "RGB tool" look.

---

### 2. Tab Bar Redesign

The active-tab top accent line will be removed entirely.

The tab bar will instead use:

- Slightly taller active tab
- Filled pill or soft capsule shape
- Brighter border in the new cyan-blue accent
- Stronger active label color
- Slightly larger active font size
- More subdued inactive tabs with hover lift

Behavior rules:

- Active tabs should be obvious from shape, fill, border, and text, not a tiny top strip.
- Inactive tabs should remain readable but clearly subordinate.
- The close button should stay visually integrated with the tab, not read as a detached glyph on a bar.

Result:

- The purple strip disappears.
- Active tab salience improves without adding clutter.

---

### 3. Sidebar Redesign

The left sidebar will move from row selection to session cards.

Each session item will show:

- Session type indicator: local vs SSH
- Session name
- Running/connected indicator
- Active state styling

Visual states:

- Active session: bright border, slightly brighter surface, stronger text, accent marker
- Running but not active: mid-contrast surface with state dot
- Selected but idle: soft hover-style card treatment
- Idle: dark resting card

Group headers will be restyled into cleaner section labels with improved spacing and less visual noise.

The card structure is intentionally visual but still compact enough for a terminal sidebar. It should feel more like a premium terminal workspace list than a plain tree view.

---

### 4. SFTP Upload UX

Uploads from both the file picker and drag-and-drop will use the same task model.

### Accepted interaction

When files are dragged onto the SFTP panel:

- The panel should visually enter a drop-ready state.
- On drop, the UI should immediately confirm acceptance.
- A task card should appear before the first upload completes.

### Upload task card contents

The upload card should show:

- Current file name
- Current item index and total item count
- Overall percentage
- Current file transferred bytes and total bytes
- Stage label

Stage labels:

- Accepted
- Uploading
- Refreshing directory
- Completed
- Failed
- Partially completed

### End-state messaging

Examples:

- `Accepted 3 files`
- `Uploading 2 / 3: build.tar.gz`
- `Completed: 3 files uploaded`
- `Partial failure: 2 succeeded, 1 failed`

If there is an error, the UI should preserve enough context to explain whether:

- The current file failed
- The batch partially failed
- The refresh after upload failed

This removes ambiguity about whether the transfer itself failed or just the follow-up listing failed.

---

### 5. Upload Progress Data Model

Real percentage requires core-to-app progress events. The current `UploadDone` response is not enough.

### Core changes

In [mod.rs](C:\Users\lccxxo\rust\rust_projects\OxiJade\oxijade-core\src\sftp\mod.rs), extend the response model to report upload progress while bytes are being written.

Planned response additions:

- `UploadStarted { local, remote, total_bytes }`
- `UploadProgress { local, remote, sent_bytes, total_bytes }`
- `UploadDone { local, remote, total_bytes }`
- `UploadFailed { local, remote, error }`

The implementation may choose slightly different Rust enum names, but the event shape above is mandatory in substance: explicit, per-file, and machine-readable.

The upload implementation should emit progress after each chunk write or at a throttled cadence tied to chunk writes. It must always finish with a terminal success or failure event.

### App changes

In the app layer, maintain a batch upload state that can aggregate per-file progress into:

- Per-file progress
- Current file index
- Total file count
- Overall bytes sent
- Overall batch percentage
- Completed and failed file counters

This state will back the upload task card in the SFTP panel.

---

### 6. Batch Upload Behavior

Batch uploads should be modeled explicitly rather than inferred from a single string.

Recommended app-side shape:

- A batch task object for the current upload group
- One item record per file
- Derived totals for progress and summary text

Rules:

- Drag-and-drop of multiple files creates one batch task.
- File-dialog multi-selection, if later enabled, should use the same path.
- A failure in one file should not erase successful file progress from the batch.
- The final batch state should distinguish full success from partial success.

---

### 7. Error Handling

Error reporting must be specific enough to support action.

### Upload errors

Surface:

- File name
- Short reason
- Batch summary if multiple files were involved

Examples:

- `Failed: nginx.conf (permission denied)`
- `Partial failure: 1 of 4 files failed`

### Directory refresh errors after upload

If upload succeeds but refresh fails:

- The task should not be converted into a generic upload failure.
- Show upload as completed with a separate note that directory refresh failed.

### Drag-and-drop rejection

If dropped files are invalid or unusable, show immediate rejection feedback instead of silently doing nothing.

---

### 8. State Flow

Unified state flow for each upload batch:

1. User drops files or selects file(s)
2. UI creates a visible task in `Accepted`
3. Core emits `UploadStarted`
4. Core emits repeated `UploadProgress`
5. Core emits per-file `UploadDone` or `UploadFailed`
6. App aggregates file results
7. App requests directory refresh
8. UI settles into `Completed`, `Failed`, or `Partially completed`

This is the minimum closed loop required to make uploads understandable.

---

### 9. Testing Strategy

### Core tests

Add focused tests around the progress-emitting upload path:

- Progress events are emitted in order
- `sent_bytes` never decreases
- Final progress reaches total bytes
- Terminal success event is emitted on completion

If the upload path cannot be directly unit-tested without a large fake SFTP harness, extract enough logic to test the progress aggregation independently.

### App/state tests

Add small tests for upload aggregation logic where feasible:

- Accepted -> Uploading -> Completed path
- Accepted -> Uploading -> Partial failure path
- Overall percentage increases monotonically
- Multiple file progress produces the correct batch summary

### Verification commands

At minimum, before implementation is claimed complete:

- `cargo check --workspace`
- Targeted tests for `oxijade-core` SFTP progress logic
- Targeted tests for any extracted app-side progress aggregation logic

---

### 10. Implementation Boundaries

This work should stay within the defined scope:

- Update theme tokens and shared visual treatment
- Redesign tab and sidebar presentation
- Add explicit upload task state and progress aggregation
- Extend core upload events to support real percentage progress

Do not:

- Refactor terminal grid behavior
- Rework split-pane mechanics
- Redesign dialogs unrelated to this flow
- Expand into a general settings/theming system

---

### 11. Open Decisions Already Resolved With User

- Visual style: modern terminal
- Palette: cold neon
- Sidebar structure: card-based
- Upload feedback: both real percentage and stage-based feedback
- Final design direction: `Approach A`

No unresolved product questions remain for this scope.

---

## Implementation Summary

The approved solution is a controlled modern-terminal polish pass:

- Remove the active tab's top accent strip
- Replace it with stronger active-tab shape, border, size, and text contrast
- Turn the left session list into a card-based workspace list
- Replace the coarse SFTP status line with a real upload task card
- Propagate upload progress from core to app so the UI can show honest percentages
- Unify all of the above with a cold neon theme token set

This yields a clearer, more premium interface without drifting into unrelated refactoring.
