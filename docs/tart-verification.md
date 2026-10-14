# Verify the fork in a Tart VM

This fork-only spike runs a **copy** of Handy in a stock Cirrus Labs Sequoia base
VM, mounting a committed source snapshot, app bundle, model, and test audio. It
uses Tart's guest agent: no SSH setup or guest Git clone is needed. The base
image includes Homebrew and the Metal shim; full Xcode and mobile simulators
are unnecessary for running the copied app. It does not
build, install, launch, stop, or re-sign the host Handy app.

## Requirements

- Apple Silicon Mac with Tart **2.41.0** or newer and Ruby (the script uses only
  its standard library).
- An existing `com.rsanheim.handy` app bundle with the fork's headless CLI.
- A local GGUF model and a **16 kHz, mono, 16-bit PCM WAV** sample.
- Space for the stock base image: approximately 25.5 GB to download and 34 GB
  of actual storage for the tested image and VM together (APFS shares blocks).
  Its virtual disk capacity is 50 GB. The VM uses 4 CPUs and 16 GiB RAM.

Use an installed `tart`, or set `TART_BIN` to an absolute executable path. The
original spike downloaded the official 2.41.0 release and verified its published
SHA256 checksum; it did not replace a global installation.

Sources: [Tart VM images and mounts](https://tart.run/quick-start/#vm-images),
[Cirrus Labs image templates](https://github.com/cirruslabs/macos-image-templates),
[Metal shim](https://github.com/cirruslabs/macos-image-templates/tree/main/data/tart-metal-capabilities),
and [Tart releases](https://github.com/cirruslabs/tart/releases).

## Run it

From this checkout, set the input paths for your machine:

```bash
# Optional: use the locally retained spike executable.
export TART_BIN="$PWD/tools/tart-spike/tart.app/Contents/MacOS/tart"

script/tart-verify prepare \
  "$PWD/src-tauri/target/release/bundle/macos/Handy Local.app" \
  /absolute/path/to/model.gguf \
  /absolute/path/to/sample.wav

# Skip init if handy-sequoia-spike already exists with this base image.
script/tart-verify init
script/tart-verify run
```

Leave `run` in its terminal. In another terminal, with the same environment:

```bash
script/tart-verify check
script/tart-verify smoke
script/tart-verify stop
```

`smoke` saves JSON and diagnostic logs under ignored `tools/tart-verify/` and
fails if the transcription is empty or its reported backend is not Metal. Device
index 0 was Metal in the tested VM; inspect `check` output if this changes.

The prepared share is mounted at `/Volumes/My Shared Files/handy`. Files in it
are copies; the mount is writable. `prepare` archives **committed HEAD**, excluding
uncommitted changes, `.git`, build outputs, and dependencies. It refuses to
replace an existing share. To test another build, stop the VM, select a fresh
absolute `HANDY_TART_DIR`, and prepare that build's bundle. The bundle may come
from another checkout; the source snapshot always comes from the checkout
containing this script. Build future candidates in a separate worktree and target
directory, since the running host app currently lives inside its build output.

`HANDY_TART_VM` overrides the VM name. The stock image pins the exact digest
tested in the spike rather than tracking `latest`. `init` is for creating a new
VM, not resetting an existing one. `stop` retains its disk and guest settings
for reuse. Checks do not stop the VM automatically; leave it running between
checks and use `stop` when finished. Keep the host share directory in place while
the VM is running, even if its source worktree's branch has been merged.

## Metal defaults

`run` persistently enables this preference for the host user running Tart:

```bash
defaults write com.apple.gpusw.ParavirtualizedGraphics ForceUnrestrictedDeviceFeatureLevel -bool true
```

The stock image contains `/usr/local/lib/TartMetalCapabilities.dylib`, but the
host preference alone does not inject it. `exec`, `check`'s Handy invocation, and
`smoke` set `DYLD_INSERT_LIBRARIES` and `TART_METAL_APPLE_FAMILY_MAX=1009` for the
guest workload. Use `script/tart-verify exec EXECUTABLE ...` for other workloads.
These defaults apply to this launcher, not arbitrary GUI launches inside the VM.

For build tools, remove the injected library using the existing `exec` command:

```bash
script/tart-verify exec /usr/bin/env -u DYLD_INSERT_LIBRARIES /bin/bash -lc 'COMMAND'
```

The mise-installed pnpm executable crashes during startup with the Metal shim
injected. Keep the shim enabled for inference and worker integration tests.

To restore an originally absent host preference after the spike, stop the VM
and run:

```bash
defaults delete com.apple.gpusw.ParavirtualizedGraphics ForceUnrestrictedDeviceFeatureLevel
```

## Observed result and limits

The October 14, 2026 verification used the Sequoia **base** image pinned in the
script: guest macOS **15.7.7**, Homebrew **7.0.7**, on a host running **15.8.1**.
The bundled Metal shim and Tart guest agent worked without additional guest
installation. Command Line Tools, Swift, CMake, Node, and mise were also present.
Full Xcode was not installed or required. The guest build uses the Apple
Intelligence stub; it does not validate FoundationModels support.

The unchanged copied Handy bundle transcribed an 8.27-second synthetic sample
using Whisper Large V3 Q5_K_M on `MTL0` in 1.50 seconds, plus 1.90 seconds to load.
It heard “Tauri” as “Tori”; this is an execution smoke test, not an accuracy
benchmark. The host app retained its original PID and executable hash.

The updated `0.9.8-rsanheim-rc1` host bundle also passed Metal transcription and
GUI onboarding startup. Its source snapshot (`77d1d3d`) was copied from the mount
to `~/handy-test` in the guest with `ditto`. After `mise trust`, `mise install`,
and `mise install rust@stable`, the guest compiled the app and passed:

- 276 normal Rust tests and all 16 ignored worker integration tests (87.5 seconds).
- Three standalone frontend assertion scripts and both Playwright HTML checks.
- Translation/model-language checks, lint, formatting, frontend build, Clippy
  (with upstream warnings), and `script/local-signing test`.

The integration fixture was Parakeet Unified EN Q8_0, pinned to catalog revision
`7e948f21b7bdbac698d3318db9d350f1096f3b6c`, checked against the catalog SHA256,
with the same 16 kHz mono sample. Whisper cannot cover the streaming tests.
The guest used Rust 1.99.0, Bun 1.3.13, and pnpm 11.10.0. Its Playwright config's
server command was changed from `bunx vite dev` to `pnpm exec vite dev` locally.

To repeat the Rust tests in the prepared guest, run these commands through
`exec /usr/bin/env -u DYLD_INSERT_LIBRARIES /bin/bash -lc '...'`:

```bash
export PATH=/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
export CARGO_BUILD_JOBS=4 CMAKE_BUILD_PARALLEL_LEVEL=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cd "$HOME/handy-test/src-tauri"
mise exec rust@stable -- cargo test --locked
mise exec rust@stable -- cargo build --locked --bin handy

export HANDY_TRANSCRIBE_WORKER_EXE="$PWD/target/debug/handy"
export HANDY_TEST_MODEL="$HOME/handy-test-fixtures/parakeet-unified-en-Q8_0.gguf"
export HANDY_TEST_WAV="$HOME/handy-test-fixtures/sample.wav"
DYLD_INSERT_LIBRARIES=/usr/local/lib/TartMetalCapabilities.dylib \
  TART_METAL_APPLE_FAMILY_MAX=1009 \
  mise exec rust@stable -- cargo test --locked --lib engine_supervisor -- \
  --ignored --nocapture --test-threads=1
```

The debug worker is required because fault injection is compiled out of release
builds. Run those tests serially because they change shared fault settings.
Disabling debug symbols and incremental builds kept the build within the base
VM's disk capacity. Build on the guest's disk, using the mount to transfer inputs.

This does not validate physical microphone input, Accessibility/TCC permissions,
global shortcuts, end-to-end paste, or updater behavior. Guest OS and virtual GPU
behavior also differ from the host.
