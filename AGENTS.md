# AGENTS.md

Guidance for AI coding assistants working in rsanheim's fork of Handy.

This file is fork-local, and `CLAUDE.md` is a symlink to it. Upstream's version is
preserved verbatim at [AGENTS.upstream.md](AGENTS.upstream.md) — read it for the
architecture tour, directory map, and tech stack. Keep all three out of any PR sent
upstream.

## Fork layout

- `origin` is **upstream** (cjpais/Handy) — merge from it, never push to it. `fork` is
  **rsanheim/Handy** — push there. `main` already tracks `fork/main`, so a bare
  `git push` goes to the fork; upstream syncs come from `origin/main`.
- `script/` (singular) is fork-local tooling. `scripts/` (plural) is upstream's. They
  are different directories; do not conflate them.
- `tools/` is gitignored local-only scratch work. Nothing there ships.
- `AGENTS.md` and `CLAUDE.md` are **upstream-tracked paths** that the fork has replaced.
  Every upstream merge will conflict on both; resolve by keeping the fork's version, and
  re-sync `AGENTS.upstream.md` from `origin/main:AGENTS.md` at the same time — nothing
  enforces that copy staying current.
- [CHANGELOG-RS.md](CHANGELOG-RS.md) tracks fork-local versions and, more usefully,
  which fork-local patches were **dropped** because upstream shipped an equivalent.
  On an upstream merge, prefer upstream's implementation and delete the fork's — keep
  a fork-local patch only when it encodes behavior upstream does not have, and cover
  that behavior with a test so the next merge cannot silently erase it.

## Upstream docs that are wrong for this fork

`CRUSH.md`, `CONTRIBUTING.md`, and `AGENTS.upstream.md` are upstream's and predate the
fork's changes. Specifically:

- They tell you to `curl` the Silero VAD model. **Don't** — `src-tauri/resources/models/silero_vad_v4.onnx`
  is committed to the repo (1.8 MB, not LFS). The download step is stale upstream
  boilerplate.
- `CRUSH.md` says to run `bun run tauri build` for a production build. On macOS that is
  exactly the command the fork's identity guard rejects — see below.

## macOS bundle identity

The fork builds as `Handy Local` / `com.rsanheim.handy` so it does not fight the
released Handy over macOS permissions, application data, logs, and the single-instance
namespace. That split is applied by `script/local-signing` via `--config`, **not** by
`tauri.conf.json` — which still carries upstream's `com.pais.handy`.

`guard_upstream_bundle_identity()` in `src-tauri/build.rs` fails any **macOS release**
build that resolves to `com.pais.handy`. It returns early — and so does not fire — for
non-macOS targets, any debug profile (including `tauri build --debug`, which will happily
bundle an upstream-identity app), when the `CI` env var is set, or when
`HANDY_ALLOW_UPSTREAM_IDENTITY=1` is set for a deliberate one.

If you hit that build error, you almost certainly wanted `script/local-signing build`.

## Build setup

```bash
bun install
bun run tauri dev
```

`bun run format` runs Prettier **and** `cargo fmt` — it covers both halves of the repo.
Run it plus `bun run lint` before committing; CI enforces both, plus `format:check` and
`check:translations`. See [BUILD.md](BUILD.md) for platform-specific setup.

Upstream docs mention a `CMAKE_POLICY_VERSION_MINIMUM=3.5` prefix for macOS cmake
errors. No current dependency appears to need it and nothing in CI sets it — treat it as
an inherited workaround with no known trigger, not a required step.

Debug features: `Cmd+Shift+D` (macOS), `Ctrl+Shift+D` elsewhere.

## Behavior worth knowing before you change it

**Single instance.** Launching while already running normally forwards the CLI args to
the running app and exits, bringing the settings window forward. The remote-control flags
(`--toggle-transcription` and friends) depend on this. The exception is headless mode —
`--transcribe-file`, `--list-devices`, `--list-models` deliberately skip the plugin and
run as a standalone process, since forwarding would make them a silent no-op whenever the
app is already open. Break single-instance and you break the remote-control flags with
it.

**CLI flags are runtime-only overrides.** They never write to persisted settings. Run
`--help` for the current list; `cli.rs` is the definition and `signal_handle.rs` holds
`send_transcription_input()`, shared between the signal handlers and the CLI.

**Linux overlay.** Uses GTK layer shell; disable with `HANDY_NO_GTK_LAYER_SHELL=1`.
Wayland support is limited.

**macOS needs accessibility permissions** for global shortcuts. A fresh local build is a
new bundle identity, so it asks again.

## i18n

Every user-facing string goes through i18next. `i18next/no-literal-string` is an ESLint
error on JSX (`eslint.config.js`), so hardcoded strings fail `bun run lint` and the
`code-quality` CI job — note `bun run build` is `tsc && vite build` and does _not_ run
ESLint. English is the reference locale: add keys to
`src/i18n/locales/en/translation.json` first. Other locales follow
[CONTRIBUTING_TRANSLATIONS.md](CONTRIBUTING_TRANSLATIONS.md).

## Contributing upstream

**Read the template before opening anything.** Upstream requires every section,
including the ones that look ceremonial — "Human Written Description" (the template says
outright that a PR may be rejected without one), "Community Feedback", and "AI
Assistance". A generic Summary/Test-plan layout gets rejected. Where a section needs a
human-written paragraph, leave a clear TODO and ask Rob to fill it in — do not write in
his voice.

- PRs: [`.github/PULL_REQUEST_TEMPLATE.md`](.github/PULL_REQUEST_TEMPLATE.md)
- Issues: [`.github/ISSUE_TEMPLATE/`](.github/ISSUE_TEMPLATE/) — blank issues are
  disabled, and feature requests belong in Discussions, not issues
- Handy is under a **feature freeze** (`CONTRIBUTING.md`). A new feature the community
  has not asked for will be rejected; gather support in
  [Discussions](https://github.com/cjpais/Handy/discussions) before opening a PR.

Commits use conventional prefixes (`feat:`, `fix:`, `docs:`, `refactor:`, `chore:`).
Say _why_ in the message, not _what_.
