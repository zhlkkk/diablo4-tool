# Milestones

## v1.0 MVP (Shipped: 2026-03-16)

**Phases completed:** 6 phases, 15 plans
**Commits:** 93
**Lines of code:** ~3,246 (2,631 Rust + 537 TypeScript + 78 YAML)
**Timeline:** 2026-03-16 (single day)

**Key accomplishments:**
1. Empirical d2core.com API investigation — confirmed Tencent CloudBase endpoint with full skills + paragon data
2. Tauri v2 scaffold with DPI-aware game capture, pixel-based safety detection, and F10 emergency stop
3. Web parser: direct TCB API integration with typed BuildPlan, pinned test fixtures, Chinese error messages
4. Auto applier: resolution-adaptive humanized click sequences with safety re-checks and cancel support
5. Complete React GUI: build preview, variant selector, calibration wizard, apply controls with real-time progress
6. CalibrationData pipeline wired end-to-end from wizard UI through executor coordinate scaling
7. GitHub Actions CI (frontend type-check + Rust clippy/test) and Release workflow (Windows installer via tauri-action)

**Delivered:** Windows desktop tool that takes a d2core.com build link, parses it, and applies skills + paragon selections to a Diablo IV character via safe UI automation. Includes CI/CD for automated checks and release builds.

**Archive:** `.planning/milestones/v1.0-ROADMAP.md`, `.planning/milestones/v1.0-REQUIREMENTS.md`

---
