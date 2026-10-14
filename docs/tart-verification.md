# Verify the fork in a Tart VM

This fork-only spike runs a **copy** of Handy in a stock Cirrus Labs Sequoia/Xcode
VM, mounting a committed source snapshot, app bundle, model, and test audio. It
uses Tart's guest agent: no SSH setup or guest Git clone is needed. It does not
build, install, launch, stop, or re-sign the host Handy app.

## Requirements

- Apple Silicon Mac with Tart **2.41.0** or newer and host Python 3.
- An existing `com.rsanheim.handy` app bundle with the fork's headless CLI.
- A local GGUF model and a **16 kHz, mono, 16-bit PCM WAV** sample.
- Space for the stock image (approximately 89 GB allocated in the initial spike;
  local APFS clones can share blocks). The VM uses 4 CPUs and 16 GiB RAM.

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

# Skip init if handy-sequoia-spike already exists from the original spike.
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

`HANDY_TART_VM` overrides the VM name. `HANDY_TART_IMAGE` overrides the stock image;
the default pins the exact digest tested in the spike rather than tracking
`latest`. `init` is for creating a new VM, not resetting an existing one. `stop`
retains its disk and guest settings for reuse.

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

For an unshimmed comparison, use plain `tart exec`, without the wrapper; injection
into an intermediate process can affect its children. To restore an originally
absent host preference after the spike, stop the VM and run:

```bash
defaults delete com.apple.gpusw.ParavirtualizedGraphics ForceUnrestrictedDeviceFeatureLevel
```

## Observed result and limits

The October 14, 2026 spike used
`ghcr.io/cirruslabs/macos-sequoia-xcode:latest`, resolved to the digest pinned in
the script: guest macOS **15.7.7**, Xcode **16.4**, Homebrew **6.0.22**, on a host
running **15.8.1**. With the shim, the GPU advertised Apple family 1009 and 64 KiB
threadgroup memory (without it: family unsupported and 32 KiB).

The unchanged copied Handy bundle transcribed an 8.27-second synthetic sample
using Whisper Large V3 Q5_K_M on `MTL0` in 2.05 seconds, plus 1.17 seconds to load.
It heard “Tauri” as “Tori”; this is an execution smoke test, not an accuracy
benchmark. The host app retained its original PID, executable hash, and signature.

This does not validate microphone input, Accessibility/TCC permissions, global
shortcuts, GUI behavior, or updater behavior. Guest OS and virtual GPU behavior
also differ from the host. No upstream merge is included in this tooling change.
