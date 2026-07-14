# Local Build: 10-Minute Keep-Mic-Open Experiment

This is a fork-only note for `rsanheim/Handy`. It documents local development
workflow and should not be treated as upstream Handy documentation.

This branch changes the experimental **Keep Mic Open Between Transcriptions**
setting from 30 seconds to 10 minutes.

It also uses the prerelease version `0.8.3-rsanheim-rc1` so local artifacts are
easier to distinguish from the published upstream `0.8.3` release.

The warm lazy-stream path also skips the fixed 100ms startup-tone delay when the
microphone stream is already open. Cold starts keep the existing delay.

## What Changed

The backend idle timeout in `src-tauri/src/managers/audio.rs` is hard-coded to
10 minutes:

```rust
const STREAM_IDLE_TIMEOUT: Duration = Duration::from_secs(10 * 60);
```

The setting still has to be enabled in Handy:

1. Open Handy settings.
2. Go to **Advanced**.
3. Enable **Experimental Features**.
4. In the **Experimental** group, enable **Keep Mic Open Between Transcriptions**.

With that setting enabled, Handy keeps the microphone stream open for 10 minutes
after each recording stops. Starting another recording during that window should
avoid the cold microphone-open path.

When a later recording starts during the warm window, Handy skips the old fixed
100ms pause before the startup tone. That change only applies when **Keep Mic
Open Between Transcriptions** is enabled and the stream is already open.

## Existing Installed Release

The released Handy app uses the macOS app identifier `com.pais.handy`. Bundles
built by `script/local-signing` instead use the product name `Handy Local` and
identifier `com.rsanheim.handy`.

The identifier split gives the fork its own macOS permission records,
application data, logs, and Tauri single-instance namespace. The released app
and `Handy Local` can launch independently. Avoid assigning both apps the same
global shortcut if they are running at the same time.

The split is applied only by the fork-local signing script. It does not modify
upstream `src-tauri/tauri.conf.json`.

## One-Time Setup

From the repository root:

```bash
bun install
mkdir -p src-tauri/resources/models
curl -o src-tauri/resources/models/silero_vad_v4.onnx \
  https://blob.handy.computer/silero_vad_v4.onnx
```

If the VAD model already exists at
`src-tauri/resources/models/silero_vad_v4.onnx`, you do not need to download it
again.

## Run Without Installing

Do not use bare `bun run tauri dev` when testing Accessibility or single-instance
behavior for this fork. The upstream development configuration still uses
`com.pais.handy`, so it does not get the identifier isolation described here.

Use the signed local bundle instead:

```bash
script/local-signing all
script/handy
```

The signed local app uses its own application data directory:

```text
~/Library/Application Support/com.rsanheim.handy/
```

Settings and downloaded models from the released app are not shared
automatically. Expect first-run setup in `Handy Local`.

## Build a Local App Bundle

To create a local macOS bundle for this fork, use the local signing helper:

```bash
script/local-signing all
```

`script/local-signing all` creates a self-signed code-signing identity named
`Handy Local Code Signing` if it does not already exist, trusts it when needed,
builds `Handy Local.app` with identifier `com.rsanheim.handy`, and verifies both
the identifier and resulting signature. Run it from an interactive terminal
because macOS may prompt for permission to update certificate trust. The build
disables updater artifacts for this local-only app.

The individual `setup`, `trust`, `build`, and `verify` commands remain available
for troubleshooting or running one step at a time.

This is not Developer ID signing and it does not notarize the app. It is only
for repeatable local builds of this fork without a paid Apple developer account.

You can inspect the helper's command list and environment overrides with:

```bash
script/local-signing help
```

The bundle will be written under:

```text
src-tauri/target/release/bundle/macos/Handy Local.app
```

The local signing helper builds the `.app` bundle only. If you need a DMG, build
one separately after the app bundle is working.

When this fork's prerelease packaging builds a DMG, the DMG name includes the
prerelease suffix:

```text
src-tauri/target/release/bundle/dmg/Handy_0.8.3-rsanheim-rc1_aarch64.dmg
```

This does not overwrite `/Applications/Handy.app`. The different product name
and identifier keep the local bundle distinct even if both are running.

## Accessibility Permissions For Local Builds

The released app and local fork have separate Accessibility entries:

- Released Handy: `com.pais.handy`
- Handy Local: `com.rsanheim.handy`

Grant Accessibility to `Handy Local` when macOS first asks. Its stable local
signing identity keeps that grant valid across rebuilds without displacing the
released app's grant.

`script/handy` refuses to launch a bundle with the upstream identifier, an
ad-hoc signature, an invalid signature, or a signing identity other than
`Handy Local Code Signing`. Rebuild with `script/local-signing all` if that
check fails.

If the local grant needs to be reset, reset only the local identifier:

```bash
tccutil reset Accessibility com.rsanheim.handy
```

Do not reset `com.pais.handy` unless you intentionally want to reset permissions
for the released app.

## Verify the Experiment

After launching the local build:

1. Enable **Experimental Features** and **Keep Mic Open Between Transcriptions**.
2. Start and stop a transcription once.
3. Start another transcription within 10 minutes.
4. Check the Handy log for the absence of a fresh cold microphone-open delay.

On macOS, `Handy Local` logs are under:

```text
~/Library/Logs/com.rsanheim.handy/handy.log
```

You can follow them while testing:

```bash
tail -f ~/Library/Logs/com.rsanheim.handy/handy.log
```

## Trace The Startup Hot Path

Hot-path tracing uses the existing Handy log pipeline. It does not add a new
dependency, and it only creates traces when explicitly enabled, so normal
release-style logging does not include these events.

To enable tracing for a local app bundle, quit any running Handy instance and
launch the local build with `HANDY_PERF_TRACE=1`:

```bash
HANDY_PERF_TRACE=1 script/handy
```

You can still pass normal Handy arguments through the launcher, for example:

```bash
HANDY_PERF_TRACE=1 script/handy --debug
```

`--debug` enables broader app and dependency debug logging. For cleaner timing
logs, prefer `HANDY_PERF_TRACE=1` without `--debug`.

Then follow the log:

```bash
tail -f ~/Library/Logs/com.rsanheim.handy/handy.log | grep 'perf.hot_path'
```

The trace lines include a `trace_id`, event name, and elapsed milliseconds from
the shortcut or external trigger. For the flow this branch is investigating,
useful events include:

- `shortcut_event_received`
- `coordinator_start_dispatch`
- `try_start_recording_begin`
- `audio_manager_microphone_stream_already_open`
- `audio_manager_recorder_start_complete`
- `startup_feedback_delay_skipped_warm_stream`
- `audio_feedback_output_stream_open_complete`
- `audio_recorder_first_resampled_frame`

Launch without `HANDY_PERF_TRACE=1` to disable these hot-path trace events.
