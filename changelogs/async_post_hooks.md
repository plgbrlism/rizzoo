# Async post_hooks

post_hooks now spawn detached: rizzoo renders, writes every output, spawns hook scripts in the background, and exits without waiting.

## Changes

| before | after |
|---|---|
| `render_and_write` ran `post_hook` inline via `Command::output()` and blocked until exit | `render_and_write` returns `Option<String>` (the hook); no process spawned there |
| `render_all` ran hooks interleaved with rendering | all templates render first, then hooks spawn (hooks can assume every output exists) |
| hook stdout/stderr captured, stderr logged on failure | stdout discarded (keeps output clean), stderr inherited (failures still visible) |
| hook failure checked via exit status, logged | exit status ignored; spawn failure logged |
| process waited for every hook | hook reaped by a background `std::thread`; rizzoo never waits |

Affected: `src/export/generate.rs` (`render_all`, `render_one`, `render_and_write`, new `spawn_hook`), tests in same file, `post_hook` docs in `assets/config.toml`.

## Tradeoffs

| gained | lost |
|---|---|
| slow/compiling hooks never delay rizzoo exit | hook failures no longer reported by rizzoo; only stderr text, no exit code |
| hooks run concurrently with each other | stdout of hooks discarded; hooks wanting output must redirect themselves |
| watch mode: hooks reaped in background, no zombie buildup | no cancellation: a stale hook keeps running after config changes mid-watch |
| one-shot: rizzoo exits immediately, hook orphaned like `cmd &` | hook ordering between templates is nondeterministic (never was guaranteed) |

Accepted: hooks are expected to be idempotent and cheap to re-run, so orphaned or overlapping runs are safe by design.
