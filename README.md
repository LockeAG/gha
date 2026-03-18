# gha

A live GitHub Actions dashboard for the terminal. Never leave your shell to check CI status.

> **Alpha** -- functional but rough edges expected.

## Why

Context switching kills flow. Opening a browser tab to check if CI passed is a small interruption that adds up across dozens of pushes a day. `gha` keeps that information where you already are: the terminal.

## Install

Requires Rust 1.74+ and a GitHub token (via `gh` CLI or environment variable).

```sh
cargo install --path .
```

Or build manually:

```sh
cargo build --release
# Binary at ./target/release/gha
```

## Usage

```sh
# Auto-detect repo from current git directory
gha

# Watch all repos in an org
gha --org MyOrg

# Watch specific repos
gha --repo owner/repo --repo owner/other-repo

# Custom poll interval (seconds, min 10)
gha --interval 15

# Explicit token
gha --token ghp_xxx
```

### Token resolution

Checked in order:
1. `--token` flag
2. `GH_TOKEN` env
3. `GITHUB_TOKEN` env
4. `gh auth token` (GitHub CLI)

## Key bindings

### Dashboard

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate up/down |
| `Enter` | Drill into job/step detail |
| `o` | Open run in browser |
| `/` | Search (repo, workflow, branch, actor) |
| `f` | Filter mode |
| `1` - `4` | Quick filter: all / failed / running / success |
| `r` | Force refresh |
| `g` / `G` | Jump to top / bottom |
| `q` / `Ctrl-C` | Quit |

### Detail view

| Key | Action |
|-----|--------|
| `j` / `k` | Scroll jobs/steps |
| `o` / `Enter` | Open in browser |
| `Esc` | Back to dashboard |
| `q` | Quit |

## Architecture

Three async tasks feeding a single `mpsc` channel. Main thread owns all state and renders UI. No shared mutexes. Data flows one direction.

```
crossterm input ──┐
tick timer (250ms) ┤──> mpsc<AppEvent> ──> main loop (App + Terminal)
API poller (30s) ──┘
```

- Polls GitHub REST API on configurable interval (default 30s)
- Jobs fetched on-demand only (when you press Enter), not polled
- Auto-downgrades poll interval to 60s when rate limit drops below 100
- Rate limit displayed in header bar

## Stack

Rust + [ratatui](https://ratatui.rs) + crossterm + tokio + reqwest (rustls). Single binary, no OpenSSL dependency. ~3.4MB release build (LTO + strip).

## Roadmap

- [ ] tmux popup with fzf for quick repo/workflow selection
- [ ] Workflow re-run from TUI (`r` on a failed run)
- [ ] Log streaming for in-progress steps
- [ ] Configurable color themes
- [ ] `~/.config/gha/config.toml` for default orgs/repos

## License

MIT
