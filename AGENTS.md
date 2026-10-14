# Handy fork instructions

This is **rsanheim/Handy**. This file is committed, fork-owned guidance;
`CLAUDE.md` is a symlink to it. Preserve both during upstream merges. Review
upstream changes selectively with `git show origin/main:AGENTS.md`; there is no
second copy to maintain and no automatic “ours” merge driver.

## Start here

- [Fork maintenance](docs/fork-maintenance.md): upstream updates, verification,
  staging, and installation.
- [Tart verification](docs/tart-verification.md): pinned base image, first-time VM
  setup, Metal shim, fixtures, test coverage, and limits.
- [Local build and signing](docs/local-build-keep-mic-open.md): bundle identity,
  stable certificate, permissions, and microphone behavior.
- [CHANGELOG-RS.md](CHANGELOG-RS.md): retained fork changes and patches dropped
  when upstream supplied an equivalent.

## Remotes and scope

- `origin` is **cjpais/Handy**: fetch and merge from `origin/main`.
- `fork` is **rsanheim/Handy**: push and open PRs here. `main` tracks `fork/main`.
- All work is for this fork unless Rob explicitly requests an upstream submission.
  Use `gh --repo rsanheim/Handy` for PR operations. Upstream's submission policies
  apply to upstream submissions, not routine maintenance of this personal fork.
- `script/` is fork tooling; `scripts/` is upstream tooling. `tools/` and `target/`
  hold ignored local artifacts, never the sole copy of instructions or code.
- Commit completed instructions and scripts. Never discard unfamiliar local
  changes merely because they are absent from committed history.

## Protect the running Mac app

- The daily-use app may run directly from this checkout's
  `src-tauri/target/release/bundle/macos/Handy Local.app`. Inspect the running
  process before building. Build candidates in a separate worktree; run tests
  inside Tart. Do not overwrite that bundle while it is running.
- Use `Handy Local`, `com.rsanheim.handy`, and the existing **Handy Local Code
  Signing** certificate. `script/local-signing` applies these overrides; upstream
  `tauri.conf.json` still uses `com.pais.handy`.
- `script/handy` verifies the signature and launches through Launch Services.
  Preserve this behavior: direct binary launches previously confused permissions.
- The release identity guard does not protect debug builds or builds with `CI`
  set. Avoid bare `tauri dev`/`tauri build` for the daily-use fork.
- `script/install` consumes `target/install/Handy Local.app`, verifies it, quits
  the fork, replaces its usual bundle, and relaunches. It does not build or back up.
  Only run it when installation/restart is authorized. Never reset TCC as routine
  update work.

## Commands

Use **pnpm**, never npm/npx. `mise.toml` pins pnpm and Bun; Bun is retained as the
runtime for upstream TypeScript checks. The GGUF Silero VAD resource is tracked;
no manual VAD download is needed.

- `mise install` then `pnpm install --frozen-lockfile`: local dependency setup.
- `script/test`: all automated checks in the prepared, running Tart VM, including
  the model-dependent worker tests. It snapshots current working files and leaves
  the VM running. First-time setup is in the Tart guide; missing setup is an error.
- `script/local-signing all`: signed production build, **in an isolated worktree**.
- `script/install`: install the staged, tested candidate from the daily-use checkout.

Keep fork-specific behavior only where upstream does not supply an equivalent.
Retain the 10-minute microphone timeout and its regression test unless measurements
justify changing it. Fork versions use an `-rsanheim-` suffix and disable upstream
update checks without rewriting the user's saved preference.

User-facing strings use i18next, with English keys in
`src/i18n/locales/en/translation.json`. CLI overrides must not persist settings;
headless inference and worker processes must remain separate from the GUI's
single-instance forwarding. Automated tests do not cover physical microphone,
Accessibility, global shortcut, and paste behavior; check those after installation.

Use conventional commit prefixes and explain why the change is needed. For an
explicitly requested upstream submission, read the relevant `.github/` template
and `CONTRIBUTING.md`, keep fork-only tooling/instructions out of that submission,
and do not invent the human-written description.
