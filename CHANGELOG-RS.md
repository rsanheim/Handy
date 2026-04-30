# CHANGELOG-RS

Personal changelog for rsanheim builds of Handy.

This file tracks fork-local versions, experiments, and release-candidate builds so
they can be compared against upstream Handy releases later. The upstream project
changelog remains the source of truth for official releases.

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
