# LazyLora Refactoring Task Board

## Quick Start

1. **Find your role**: Check assignment below
2. **Read your task file**: Navigate to the appropriate stage
3. **Follow the checklist**: Update checkboxes as you progress
4. **Report blockers**: Document issues in your task file

---

## Current Status

| Stage | Status | Workers | Estimated Duration |
|-------|--------|---------|-------------------|
| **Stage 0** | `COMPLETE` | A, B, C (parallel) | 2 days |
| **Stage 1** | `COMPLETE` | Coordinator | 1 day |
| **Stage 1.5** | `COMPLETE` | A, B, C (parallel) | 3 days |
| **Stage 2** | `COMPLETE` | Coordinator | 1 day |
| **Stage 2.5** | `COMPLETE` | A, B (parallel) | 3 days |
| **Stage 3** | `COMPLETE` | All | 2 days |
| **Stage 4** | `COMPLETE` | Cleanup | 1 day |

**Total Timeline**: 13 days

---

## Task Navigation

### Stage 0 - Foundation (Parallel Work)

| Worker | Task | File | Creates |
|--------|------|------|---------|
| **A** | Theme & Constants | [WORKER_A_STAGE_0.md](./WORKER_A_STAGE_0.md) | `src/theme.rs`, `src/constants.rs` |
| **B** | Domain Types | [WORKER_B_STAGE_0.md](./WORKER_B_STAGE_0.md) | `src/domain/*` (8 files) |
| **C** | HTTP Client | [WORKER_C_STAGE_0.md](./WORKER_C_STAGE_0.md) | `src/client/*` (5 files) |

### Stage 1 - Sync Point

| Role | Task | File |
|------|------|------|
| **Coordinator** | Merge & Integration | [COORDINATOR_STAGE_1.md](./COORDINATOR_STAGE_1.md) |

### Stage 1.5 - Core Split (Parallel Work)

| Worker | Task | File | Creates |
|--------|------|------|---------|
| **A** | Widgets: List + Common | [WORKER_A_STAGE_1.5.md](./WORKER_A_STAGE_1.5.md) | `src/widgets/list/*`, `src/widgets/common/*` |
| **B** | Widgets: Graph + Detail | [WORKER_B_STAGE_1.5.md](./WORKER_B_STAGE_1.5.md) | `src/widgets/graph/*`, `src/widgets/detail/*` |
| **C** | State Module | [WORKER_C_STAGE_1.5.md](./WORKER_C_STAGE_1.5.md) | `src/state/*` (9 files) |

### Stage 2 - Sync Point

| Role | Task | File |
|------|------|------|
| **Coordinator** | Merge & Cleanup | [COORDINATOR_STAGE_2.md](./COORDINATOR_STAGE_2.md) |

### Stage 2.5 - UI Split (Parallel Work)

| Worker | Task | File | Creates |
|--------|------|------|---------|
| **A** | UI: Panels + Layout | [WORKER_A_STAGE_2.5.md](./WORKER_A_STAGE_2.5.md) | `src/ui/panels/*`, `src/ui/layout.rs`, etc. |
| **B** | UI: Popups + Components | [WORKER_B_STAGE_2.5.md](./WORKER_B_STAGE_2.5.md) | `src/ui/popups/*`, `src/ui/components/*` |

### Stage 3 - Final Integration

| Role | Task | File |
|------|------|------|
| **Coordinator** | Final Integration | [COORDINATOR_STAGE_3.md](./COORDINATOR_STAGE_3.md) |

### Stage 4 - Cleanup & Polish

| Role | Task | File |
|------|------|------|
| **Cleanup** | Code Quality | [STAGE_4_CLEANUP.md](./STAGE_4_CLEANUP.md) |

---

## File Ownership Matrix

**CRITICAL**: Each file is owned by exactly ONE worker during each stage. Never modify files outside your ownership.

### Stage 0 Ownership

```
src/theme.rs        -> Worker A (CREATE)
src/constants.rs    -> Worker A (CREATE)
src/domain/*        -> Worker B (CREATE)
src/client/*        -> Worker C (CREATE)
[All other files]   -> LOCKED (NO TOUCH)
```

### Stage 1.5 Ownership

```
src/widgets/mod.rs         -> Worker A (CREATE)
src/widgets/helpers.rs     -> Worker A (CREATE)
src/widgets/common/*       -> Worker A (CREATE)
src/widgets/list/*         -> Worker A (CREATE)
src/widgets/graph/*        -> Worker B (CREATE)
src/widgets/detail/*       -> Worker B (CREATE)
src/state/*                -> Worker C (CREATE)
src/widgets.rs             -> LOCKED (DELETE @ Stage 2)
src/app_state.rs           -> LOCKED (DELETE @ Stage 2)
src/ui.rs                  -> LOCKED (NO TOUCH)
```

### Stage 2.5 Ownership

```
src/ui/mod.rs              -> Worker A (CREATE)
src/ui/layout.rs           -> Worker A (CREATE)
src/ui/header.rs           -> Worker A (CREATE)
src/ui/footer.rs           -> Worker A (CREATE)
src/ui/panels/*            -> Worker A (CREATE)
src/ui/popups/*            -> Worker B (CREATE)
src/ui/components/*        -> Worker B (CREATE)
src/ui.rs                  -> LOCKED (DELETE @ Stage 3)
```

---

## Workflow for Workers

### Starting a Stage

1. **Check prerequisites**: Ensure previous stage is complete
2. **Create branch**: `refactor/stage{N}-worker-{X}-{description}`
3. **Read task file**: Open your assigned `.md` file
4. **Update status**: Change `NOT_STARTED` to `IN_PROGRESS`

### During Work

1. **Follow checklist**: Work through tasks sequentially
2. **Check boxes**: Mark completed items with `[x]`
3. **Document issues**: Add to "Issues Encountered" section
4. **Commit frequently**: Small, focused commits

### Completing a Stage

1. **Run verification**:
   ```bash
   cargo build
   cargo test --all-features
   cargo clippy --all-features -- -D warnings
   cargo fmt -- --check
   ```
2. **Update status**: Change to `COMPLETE`
3. **Fill handoff notes**: Document any blockers or notes
4. **Create PR**: Use branch name as PR title
5. **Notify coordinator**: Signal completion

---

## Workflow for Coordinator

### At Sync Points

1. **Verify all workers complete**: Check status in task files
2. **Review PRs**: Ensure no conflicting changes
3. **Merge order**: Follow documented merge sequence
4. **Run integration tests**: Full test suite
5. **Manual testing**: Verify app functionality
6. **Update this README**: Change stage statuses
7. **Notify workers**: Signal next stage can begin

---

## Verification Commands

```bash
# Build check
cargo build

# Run all tests
cargo test --all-features

# Lint check
cargo clippy --all-features -- -D warnings

# Format check
cargo fmt -- --check

# Release build (final stages)
cargo build --release
```

---

## Branch Naming Convention

```
refactor/stage0-worker-a-theme
refactor/stage0-worker-b-domain
refactor/stage0-worker-c-client
refactor/stage1-integration
refactor/stage1.5-worker-a-widgets-list
refactor/stage1.5-worker-b-widgets-graph
refactor/stage1.5-worker-c-state
refactor/stage2-integration
refactor/stage2.5-worker-a-ui-panels
refactor/stage2.5-worker-b-ui-popups
refactor/stage3-final
```

---

## Expected Outcomes

### Line Count Reduction

| Module | Before | After | Change |
|--------|--------|-------|--------|
| algorand.rs | 2,783 | 0 | **Removed** (split to domain/ + client/algo.rs) |
| widgets.rs | 4,112 | 2,670 | -35% |
| app_state.rs | 2,054 | 1,330 | -35% |
| ui.rs | 2,455 | 2,580 | +5% (better structure) |
| **Total** | **~12,631** | **~9,820** | **-22%** |

### New Module Structure

```
src/
├── domain/     (~990 lines)   - Domain types
├── client/     (~800 lines)   - API clients
├── widgets/    (~2,670 lines) - UI widgets
├── ui/         (~2,580 lines) - UI rendering
├── state/      (~1,330 lines) - Application state
├── theme.rs    (~150 lines)   - Colors & styles
├── constants.rs (~50 lines)   - App constants
└── [unchanged files]
```

---

## Related Documents

- [REFACTORING_PLAN.md](../REFACTORING_PLAN.md) - Full technical details
- [AGENTS.md](../AGENTS.md) - Coding guidelines

---

## Progress Tracking

### Stage 0 Progress

- [x] Worker A: Theme & Constants (`src/theme.rs` - 569 lines, `src/constants.rs` - 387 lines)
- [x] Worker B: Domain Types (`src/domain/*` - 8 files, ~2,100 lines total)
- [x] Worker C: HTTP Client (`src/client/*` - 5 files, ~250 lines total)

### Stage 1 Progress

- [x] All Stage 0 work integrated
- [x] main.rs updated with new modules (`mod domain;`, `mod client;`, `mod theme;`, `mod constants;`)
- [x] All 275 tests passing
- [x] Clippy passes with no warnings
- [x] Code formatting verified
- [ ] algorand.rs facade (deferred to Stage 2 - new modules use `#[allow(dead_code)]` during transition)

### Stage 1.5 Progress

- [x] Worker A: Widgets List + Common (`src/widgets/list/*`, `src/widgets/common/*`, `src/widgets/helpers.rs`)
- [x] Worker B: Widgets Graph + Detail (`src/widgets/graph/*`, `src/widgets/detail/*`)
- [x] Worker C: State Module (`src/state/*` - 9 files)

### Stage 2 Progress

- [x] All Stage 1.5 PRs merged
- [x] widgets.rs replaced with widgets/mod.rs
- [x] app_state.rs replaced with state/mod.rs
- [x] All 380 tests passing
- [x] Clippy passes with no warnings
- [x] Release build successful

### Stage 2.5 Progress

- [x] Worker A: UI Panels + Layout (`src/ui/panels/*`, `src/ui/layout.rs`, `src/ui/header.rs`, `src/ui/footer.rs`, `src/ui/helpers.rs`)
- [x] Worker B: UI Popups + Components (`src/ui/popups/*`, `src/ui/components/*`)
- [x] src/ui/mod.rs refactored as clean orchestrator (~183 lines)
- [x] All 450 tests passing
- [x] Clippy passes with no warnings
- [x] Release build successful

### Stage 3 Progress

- [x] Stage 2.5 integration verified (ui/ module structure complete)
- [x] Old ui.rs already replaced by ui/ directory (no separate ui.rs exists)
- [x] Domain types migration complete (all code imports from `crate::domain`)
- [x] **AlgoClient migrated to `src/client/algo.rs`** (1,765 lines with all tests)
- [x] **`src/algorand.rs` completely removed** - no longer exists
- [x] All imports updated: `crate::client::AlgoClient` and `crate::domain::*`
- [x] Backup files cleaned up (widgets.rs.bak removed)
- [x] All 450 tests passing
- [x] Clippy passes with no warnings
- [x] Release build successful (7.0MB binary)
- [x] Documentation updated

### Stage 4 Progress (Cleanup & Polish)

- [x] **Phase 1.1**: Removed 14 stale TODO comments referencing Stage 2
- [x] **Phase 1.2**: Replaced magic numbers with constants in `state/mod.rs`
  - Added `DEFAULT_TERMINAL_WIDTH`, `DEFAULT_VISIBLE_BLOCKS`, `DEFAULT_VISIBLE_TRANSACTIONS` to constants.rs
  - Updated all hardcoded values to use named constants
- [x] **Phase 1.3**: Audited `#![allow(dead_code)]` attributes
  - Removed from 25 files where code is actually used
  - Kept on 18 files with improved documentation
- [x] All 450 tests passing
- [x] Clippy passes with 0 warnings
- [x] Build successful with 0 warnings

---

## Final Statistics

### Line Counts (Actual)

| Module | Lines | Files | Description |
|--------|-------|-------|-------------|
| theme.rs | 573 | 1 | Colors & styles |
| constants.rs | 390 | 1 | App constants |
| domain/ | 3,126 | 8 | Domain types |
| client/ | 2,130 | 6 | HTTP clients + AlgoClient |
| widgets/ | 5,059 | 17 | UI widgets |
| ui/ | 5,015 | 18 | UI rendering |
| state/ | 5,550 | 9 | Application state |
| **Total New Modules** | **21,843** | **60** | - |

### Test Count
- **450 tests** passing (up from 275 at Stage 1)

### New Module Structure (Final)

```
src/
├── boot_screen.rs
├── commands.rs
├── constants.rs    (390 lines)    - App constants ✓
├── theme.rs        (573 lines)    - Colors & styles ✓
├── tui.rs
├── updater.rs
├── main.rs
├── domain/         (3,126 lines)  - Domain types ✓
│   ├── account.rs
│   ├── asset.rs
│   ├── block.rs
│   ├── error.rs
│   ├── mod.rs
│   ├── network.rs
│   ├── nfd.rs
│   └── transaction.rs
├── client/         (2,130 lines)  - HTTP clients + AlgoClient ✓
│   ├── algo.rs     (1,765 lines)  - Main API client (migrated from algorand.rs)
│   ├── http.rs
│   ├── indexer.rs
│   ├── mod.rs
│   ├── nfd.rs
│   └── node.rs
├── widgets/        (5,059 lines)  - UI widgets ✓
│   ├── common/
│   ├── detail/
│   ├── graph/
│   ├── list/
│   ├── helpers.rs
│   └── mod.rs
├── ui/             (5,015 lines)  - UI rendering ✓
│   ├── components/
│   ├── panels/
│   ├── popups/
│   ├── footer.rs
│   ├── header.rs
│   ├── helpers.rs
│   ├── layout.rs
│   └── mod.rs
└── state/          (5,550 lines)  - App state ✓
    ├── platform/
    ├── command_handler.rs
    ├── config.rs
    ├── data.rs
    ├── mod.rs
    ├── navigation.rs
    └── ui_state.rs
```

---

## Emergency Contacts

If you encounter blocking issues:

1. Document in your task file's "Issues Encountered" section
2. Create a GitHub issue with `refactor` label
3. Tag the coordinator in your PR

---

*Last updated: Stage 4 cleanup complete - Dec 5, 2025*
