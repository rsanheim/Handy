# Local Build: 10-Minute Keep-Mic-Open Experiment

This branch changes the experimental **Keep Mic Open Between Transcriptions**
setting from 30 seconds to 10 minutes.

It also uses the prerelease version `0.8.3-rsanheim-rc1` so local artifacts are
easier to distinguish from the published upstream `0.8.3` release.

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

## Existing Installed Release

Handy uses the macOS app identifier `com.pais.handy` and Tauri's single-instance
plugin. Running a development or locally built copy while the installed release
is already running will usually talk to the already-running instance instead of
starting a separate independent app.

For this experiment, quit the installed Handy release before launching the local
build. Use the tray/menu-bar quit action, or run:

```bash
pkill -x Handy
```

Do not leave the release build running while testing this branch, or you may be
testing the installed release instead of the local build.

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

This is the fastest way to try the change:

```bash
bun run tauri dev
```

If macOS or CMake complains during startup, use:

```bash
CMAKE_POLICY_VERSION_MINIMUM=3.5 bun run tauri dev
```

The dev app uses the same application data directory as the release build:

```text
~/Library/Application Support/com.pais.handy/
```

That means your existing Handy settings and downloaded transcription models are
shared. The source checkout is separate; your installed `.app` is not replaced.

## Build a Local App Bundle

To create a local macOS bundle:

```bash
bun run tauri build
```

The bundle will be written under:

```text
src-tauri/target/release/bundle/macos/
```

The DMG name includes the prerelease suffix:

```text
src-tauri/target/release/bundle/dmg/Handy_0.8.3-rsanheim-rc1_aarch64.dmg
```

This does not overwrite the installed release in `/Applications` unless you copy
the locally built app there yourself. If you launch the local bundle while the
installed release is running, the single-instance behavior still applies, so quit
the release first.

## Verify the Experiment

After launching the local build:

1. Enable **Experimental Features** and **Keep Mic Open Between Transcriptions**.
2. Start and stop a transcription once.
3. Start another transcription within 10 minutes.
4. Check the Handy log for the absence of a fresh cold microphone-open delay.

On macOS, Handy logs are under:

```text
~/Library/Logs/com.pais.handy/handy.log
```

You can follow them while testing:

```bash
tail -f ~/Library/Logs/com.pais.handy/handy.log
```
