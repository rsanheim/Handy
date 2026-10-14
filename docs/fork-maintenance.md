# Maintain this fork

The daily-use Mac app is `Handy Local` (`com.rsanheim.handy`). The fork keeps a
stable local signing certificate so updates preserve its macOS identity. This
workflow prepares and verifies a candidate while the existing app keeps running.

## Update from upstream main

1. Inspect `git status`, `git worktree list`, the running Handy executable path,
   and [CHANGELOG-RS.md](../CHANGELOG-RS.md). Preserve uncommitted work; do not
   reset it to make a merge easier.
2. Fetch both remotes and fast-forward the daily-use checkout to `fork/main`.
   `origin` is cjpais/Handy, and `fork` is rsanheim/Handy. Review the upstream
   commit range since the last merge.
3. Create an update branch and isolated worktree based on `fork/main`. For example,
   from the daily-use checkout:

   ```bash
   git fetch fork --prune
   git fetch origin main --tags
   git merge --ff-only fork/main
   git worktree add -b chore/upstream-sync ../Handy-update fork/main
   git -C ../Handy-update merge origin/main
   ```

   Use a fresh branch/directory if those names already exist. Resolve conflicts
   there; never build into the daily-use checkout's target directory.

4. Keep the fork's `AGENTS.md` and `CLAUDE.md` symlink. Review upstream's agent
   guidance with `git show origin/main:AGENTS.md` and incorporate useful changes
   deliberately. There is no `AGENTS.upstream.md` copy or custom merge driver.
5. Audit the retained fork changes: 10-minute microphone timeout and regression
   test; bundle ID/name; stable signing and Launch Services launcher; release
   identity guard; fork version suffix and updater isolation; pnpm pins/lockfile.
   Drop patches where upstream now supplies equivalent behavior, recording the
   reason in the changelog. Set matching fork versions in `package.json`,
   `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`; update lockfiles using
   pnpm/Cargo. Review any newly blocked dependency builds with `pnpm approve-builds`.

## Verify the candidate

From the update worktree, run:

```bash
script/test
```

This is the complete automated check entry point. It uses the existing Tart
launcher and a fresh snapshot of the current checkout, including uncommitted and
new non-ignored files. It does not test an old committed snapshot by accident.
The [Tart guide](tart-verification.md) covers first-time setup and environment
variables for the running VM. Tests fail if setup is missing; integration tests
are not silently skipped. Leave the mounted host directory in place until the VM
is stopped. If this Mac has no prepared VM or app bundle yet, perform the signed
build below first to supply the initial `tart-verify prepare` input.

The suite builds a debug worker, runs the normal Rust tests and the 16 real-model
worker tests, runs all three frontend assertion scripts and Playwright, and checks
lint, formatting, translations, model languages, Clippy, frontend build, and the
signing helper's self-test. It does not build a signed release or test physical
microphone/permissions/paste behavior.

Build the signed production candidate in the isolated update worktree:

```bash
mise install
pnpm install --frozen-lockfile
script/local-signing all
```

Use the existing host certificate. Do not create or export a signing certificate
just to run VM tests. On another development Mac, follow the existing local
signing setup once. For a release smoke test, use the Tart launcher's `prepare`,
`check`, and `smoke` commands against a copy of this bundle, as documented in the
Tart guide. If a VM is already running with another mounted share, keep it intact
until a stop/restart is authorized.

Record validation and remaining host checks in `CHANGELOG-RS.md`, commit the
candidate, push to `fork`, and use `gh --repo rsanheim/Handy` for its PR. After
merging, fast-forward the daily-use checkout. Do not push changes to `origin`.

## Stage and install

From the daily-use checkout, stage the verified candidate (the example assumes
that `../Handy-update` is the build worktree):

```bash
mkdir -p target/install
# Clear only the previous staged candidate so removed bundle resources cannot linger.
rm -rf 'target/install/Handy Local.app'
ditto '../Handy-update/src-tauri/target/release/bundle/macos/Handy Local.app' \
  'target/install/Handy Local.app'
script/local-signing verify 'target/install/Handy Local.app'
```

Staging does not touch the running app. When ready for the brief quit/relaunch:

```bash
script/install
```

The installer makes no backups. It keeps settings, models, history, and recordings
in place. After installation, check a short dictation, paste, and another dictation
within the warm microphone window. Do not reset permissions unless diagnosing an
actual permissions failure.

Clean up merged branches and unused worktrees once their artifacts are no longer
needed. A worktree that supplies a running VM's mounted directory is still in use.
Machine-specific paths, downloaded models, logs, and installed VM state are local
artifacts; source instructions and scripts must be committed to the fork.
