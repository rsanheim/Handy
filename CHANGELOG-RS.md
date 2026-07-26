# CHANGELOG-RS

Personal changelog for rsanheim builds of Handy.

This file tracks fork-local versions, experiments, and release-candidate builds so
they can be compared against upstream Handy releases later. The upstream project
changelog remains the source of truth for official releases.

## 0.9.4-rsanheim-rc1 - 2026-07-25

Base: upstream `main` at `6cad594` (past the `v0.9.4` release tag).

Merge of 80 upstream commits into the fork, spanning upstream releases 0.9.0
through 0.9.4. See the upstream changelog for the full list.

### Fork-Local Changes Dropped

- Removed the fork-local `perf_trace` hot-path tracing module and the
  `HANDY_PERF_TRACE=1` environment switch. Upstream now instruments the same
  start path directly (`start-path pre-recording steps`, `Cmd::Start processed
  ... after send`, `first audio chunk arrived ...`, `tray icon change ...`), and
  shipped `faster mic initialization` (#1582). Keeping a parallel tracing
  mechanism through upstream's refactored `try_start_recording(binding_id,
  vad_policy)` signature would have duplicated that work.
- Dropped the fork-local tray icon panic fix. Upstream adopted the same fix in
  `fix(tray): log tray icon failures instead of panicking` (#1355), so the only
  remaining fork delta was error-message wording.

### Fork-Local Changes Retained

- `STREAM_IDLE_TIMEOUT` stays at 10 minutes instead of upstream's 30 seconds,
  with the guard test that fails if a future merge silently reverts it.
- The paired `lazyStreamClose` UI copy still says 10 minutes.
- Local macOS signing workflow (`script/local-signing`, `script/handy`) and the
  `com.rsanheim.handy` / `Handy Local` bundle identity are unchanged.

### Notes

- `docs/local-build-keep-mic-open.md` now documents upstream's debug-level
  startup timing logs in place of the removed `HANDY_PERF_TRACE` tracing.
- The `perf_trace` implementation can be recovered from commit `863b161`
  (`perf: trace warm microphone startup`), reachable from the local
  `pre-upstream-sync-2026-07-25` tag.

## 0.8.3-rsanheim-rc3 - 2026-07-14

Base: `0.8.3-rsanheim-rc2`.

### Fork-Local Changes

- Add `script/local-signing` for repeatable local macOS app bundle signing with
  a stable self-signed code-signing identity. `script/local-signing all` runs
  setup, trust, build, and verification as one interactive workflow.
- Build the fork as `Handy Local` with identifier `com.rsanheim.handy`, keeping
  its macOS permissions, data, logs, and single-instance namespace separate from
  the released `com.pais.handy` app.
- Refuse local builds and launches that accidentally reuse the upstream bundle
  identifier.
- Refuse to launch local bundles that are ad-hoc signed, have an invalid
  signature, or do not use the expected stable local signing identity.
- Normalize the default Keychain path returned by macOS and verify the built
  app against the signing identity's certificate fingerprint.
- Reuse `script/local-signing verify` from `script/handy` so build and launch
  checks cannot drift apart.
- Launch `Handy Local.app` through macOS Launch Services so Accessibility is
  attributed to the local app instead of the invoking terminal.
- Update `script/handy` to point missing local builds at
  `script/local-signing all` instead of ad-hoc Tauri build commands.
- Document the local signing and Accessibility/TCC reset workflow in
  `docs/local-build-keep-mic-open.md`.

### Notes

- This is for local development only. It is not Developer ID signing and does
  not notarize the app.
- The goal is to keep this fork's code signature stable across rebuilds so
  macOS Accessibility permissions do not have to be re-granted after every
  local build.

## 0.8.3-rsanheim-rc2 - 2026-04-30

Base: `0.8.3-rsanheim-rc1`.

### Fixes

- Avoid panicking when a tray icon resource cannot be resolved or loaded while
  changing recording state.
- Preserve the transcription coordinator thread when tray icon resources are
  missing, such as after accidentally running from and then deleting a temporary
  worktree app bundle.

### Notes

- Root cause observed locally: Handy was still running from
  `/private/tmp/Handy-lazy-stream-10min/.../Handy.app` after that worktree had
  been removed.

## 0.8.3-rsanheim-rc1 - 2026-04-30

Base: upstream `v0.8.3` plus `a385371` (`refactor(nix): rely on cargo-tauri.hook standard phases`).

### Fork-Local Changes

- Mark local builds as `0.8.3-rsanheim-rc1` so they are visually distinct from
  official upstream releases.
- Extend the experimental lazy microphone stream idle timeout from 30 seconds to
  10 minutes.
- Update the English settings copy and local build documentation to describe the
  10-minute keep-warm behavior.
- Add `script/handy` for launching the local app bundle with forwarded CLI args,
  including `script/handy --debug`.

### Latency Investigation

- Add release-build-friendly performance tracing gated behind
  `HANDY_PERF_TRACE=1`.
- Trace shortcut, signal, coordinator, audio manager, recorder, and startup tone
  milestones in the shortcut-to-recording path.
- Skip the 100 ms startup feedback delay only when the lazy microphone stream is
  enabled and the stream is already warm.
- Add recorder lifecycle coverage for warm stream open/close behavior.

### Notes

- Daily-driver build candidate.
- Upstream candidates to consider later: recorder lifecycle tests, gated tracing,
  and the conditional warm-stream startup delay skip.
- Fork-local changes to keep separate from upstream: hard-coded 10-minute warm
  mic timeout, prerelease version suffix, and local launch/build documentation.
