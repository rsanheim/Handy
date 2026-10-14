# Verify the fork in a Tart VM

For upstream updates and installation, start with
[Fork maintenance](fork-maintenance.md). Once the VM below is prepared and running,
`script/test` runs the complete automated suite from the current checkout.

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

## Run all automated checks

With the prepared VM running, from the checkout you want to test:

```bash
script/test
```

Use the same absolute `HANDY_TART_DIR`, `HANDY_TART_VM`, and `TART_BIN` as the
launcher when testing from another worktree. If `TART_BIN` is unset, the test
command uses `tools/tart-spike/tart.app/Contents/MacOS/tart` when present, otherwise
`tart` from PATH. Machine-specific paths are not committed. On a fresh Mac, the
`prepare`, `init`, and `run` steps above are required once; the test command does
not create or stop VMs.

If a fresh checkout has no built app for `prepare`, first make a signed candidate
in an isolated worktree using the build step in the maintenance guide. That copied
bundle supplies the launcher smoke test; `script/test` builds fresh source itself.

On a Mac with an existing session, `tools/tart-verify` may be a local symlink to
the directory already mounted by its VM. This keeps plain `script/test` usable
without committing machine-specific paths or restarting the VM. Do not remove
the symlink target while the VM is using it.

The runner snapshots current tracked files and new non-ignored files into the
mounted share, including uncommitted edits. Deleted files are omitted, and ignored
builds/models/dependencies are not copied. A uniquely named archive avoids stale
mounted-file caches. The guest extracts it onto its own disk and runs the copy of
`script/test` from that snapshot. It does not reuse the old `prepare` source tree.

The base image supplies Command Line Tools, CMake, Node, mise, and Ruby. The runner
installs the repo-pinned Bun/pnpm and stable Rust with mise, installs dependencies
from the pnpm lockfile, and installs Playwright Chromium. It downloads Parakeet
Unified EN Q8_0 using the catalog's pinned revision and SHA256, caching it in the
guest. The prepared `sample.wav` must be nonempty 16 kHz mono 16-bit PCM audio.
The first run needs network access and several GB of guest disk space.

Checks include all three frontend assertion scripts, both Playwright HTML checks,
translation/model-language consistency, lint, frontend build, signing-script
self-test, normal Rust tests, every ignored engine-supervisor test, Clippy, and
formatting. Missing setup or a failed check exits nonzero; no model tests are
silently skipped. The guest's debug worker enables fault injection, and those
tests run serially. Metal stays enabled for these inference tests; it is removed
from the environment of build tooling.

Output is streamed and saved to `$HANDY_TART_DIR/test.log` (by default
`tools/tart-verify/test.log`), with the checkout HEAD and source archive checksum.
The log is replaced on each run. Temporary source copies are removed afterwards;
Rust output remains in guest `~/handy-test-target` and models in
`~/handy-test-fixtures` for reuse. The VM remains running. The runner does not
modify the host app, keychain, settings, or permissions, or build a signed release.

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
The guest used Rust 1.99.0, Bun 1.3.13, and pnpm 11.10.0. The fork's Playwright
server command now uses `pnpm exec vite dev`. Repeat the suite with `script/test`.

The debug worker is required because fault injection is compiled out of release
builds. Run those tests serially because they change shared fault settings.
Disabling debug symbols and incremental builds kept the build within the base
VM's disk capacity. Build on the guest's disk, using the mount to transfer inputs.

This does not validate physical microphone input, Accessibility/TCC permissions,
global shortcuts, end-to-end paste, or updater behavior. Guest OS and virtual GPU
behavior also differ from the host.
