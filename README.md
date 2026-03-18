# gha

A live GitHub Actions dashboard for the terminal. Never leave your shell to check CI status.

> **Alpha** -- functional, polished, hardened. Not yet 1.0.

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
# Binary at ./target/release/gha (~3.4MB, no OpenSSL)
```

## Usage

```sh
# Auto-detect repo from current git directory
gha

# Watch all repos in an org (auto-filters to recently active)
gha --org MyOrg

# Watch specific repos (always polled, not filtered)
gha --repo owner/repo --repo owner/other-repo

# Custom poll interval (seconds, min 10)
gha --interval 15

# Control activity window (default: 7 days, 0 = all repos)
gha --org MyOrg --days 14

# Explicit token
gha --token ghp_xxx
```

### Smart repo filtering

When using `--org`, gha fetches all repos but only polls those with a push in the last 7 days. Archived repos are excluded. This keeps API usage sane -- an org with 50 repos but 4 active ones only costs 4 API calls per cycle.

Press `a` inside the TUI to open the repo picker and toggle any org repo on or off.

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
| `Ctrl-d` / `Ctrl-u` | Half-page scroll |
| `g` / `G` | Jump to top / bottom |
| `Enter` | Drill into job/step detail |
| `o` | Open run in browser |
| `/` | Search (repo, workflow, branch, actor) |
| `f` | Filter mode |
| `1` - `4` | Quick filter: all / failed / running / success |
| `a` | Repo picker (toggle org repos) |
| `r` | Force refresh |
| `q` / `Ctrl-C` | Quit |

### Detail view

| Key | Action |
|-----|--------|
| `j` / `k` | Scroll jobs/steps |
| `Ctrl-d` / `Ctrl-u` | Half-page scroll |
| `o` / `Enter` | Open in browser |
| `Esc` | Back to dashboard |
| `q` | Quit |

### Repo picker

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate |
| `Space` / `Enter` | Toggle repo on/off |
| `Esc` | Apply and return to dashboard |

## fzf + tmux

`gha fzf` provides composable pickers powered by fzf. Designed for tmux popup workflows.

### Pick a run (opens in browser)

```sh
gha fzf runs --org DreamsEngine
```

### Pick a run (output URL for piping)

```sh
gha fzf runs --org DreamsEngine --action url
```

### Pick a repo

```sh
gha fzf repos --org DreamsEngine
```

### tmux keybindings

Add to `.tmux.conf`:

```sh
# Prefix + g: pick a run and open in browser
bind-key g display-popup -E -w 80% -h 60% "gha fzf runs --org DreamsEngine"

# Prefix + G: pick a repo, then open its runs
bind-key G display-popup -E -w 80% -h 60% \
  "gha fzf repos --org DreamsEngine | xargs -I{} gha fzf runs --repo {}"
```

The fzf picker uses Catppuccin Mocha colors to match the TUI. Requires `fzf` installed.

## Architecture

Three async tasks feeding a single `mpsc` channel. Main thread owns all state and renders UI. No shared mutexes. Data flows one direction.

```
crossterm input ──┐
tick timer (250ms) ┤──> mpsc<AppEvent> ──> main loop (App + Terminal)
API poller ────────┘         ↑
                    watch<Vec<String>> (repo list updates from picker)
```

- Polls GitHub REST API on configurable interval (default 30s)
- Jobs fetched on-demand only (when you press Enter), not polled
- Auto-downgrades poll interval to 60s when rate limit drops below 100
- Poller reads repo list from `watch` channel -- picker changes propagate immediately
- Panic hook restores terminal state on crash
- Selection preserved by run ID across data refreshes

## Stack

Rust + [ratatui](https://ratatui.rs) + crossterm + tokio + reqwest (rustls). Single binary, no OpenSSL dependency. Catppuccin Mocha color scheme.

## Roadmap

- [x] Smart repo filtering (activity-based, `--days`)
- [x] Repo picker (toggle org repos at runtime)
- [x] Panic-safe terminal restore
- [x] Detail view with run context + tree-drawn job/step hierarchy
- [x] fzf picker for tmux popups (`gha fzf runs`, `gha fzf repos`)
- [ ] Workflow re-run from TUI
- [ ] Log streaming for in-progress steps
- [ ] Configurable color themes
- [ ] `~/.config/gha/config.toml` for default orgs/repos

## License

MIT
