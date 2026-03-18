# gha

A live GitHub Actions dashboard for the terminal. Never leave your shell to check CI status.

## Install

### Homebrew (macOS / Linux)

```sh
brew tap LockeAG/tap
brew install gha
```

### Cargo (from source)

```sh
cargo install --git https://github.com/LockeAG/gha
```

### Prebuilt binaries

Download from [Releases](https://github.com/LockeAG/gha/releases) for macOS (arm64/x86_64) and Linux (x86_64/arm64).

## Why

Context switching kills flow. Opening a browser tab to check if CI passed is a small interruption that adds up across dozens of pushes a day. `gha` keeps that information where you already are: the terminal.

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

# Color theme
gha --theme tokyo-night
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

### Themes

Three built-in themes via `--theme`:

| Flag | Alias |
|------|-------|
| `catppuccin-mocha` (default) | `mocha` |
| `tokyo-night` | `tn` |
| `tokyo-night-storm` | `tns` |

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

```sh
# Pick a run and open in browser
gha fzf runs --org MyOrg

# Pick a run, drill into job/step detail, then open
gha fzf runs --org MyOrg --action detail

# Output URL for piping
gha fzf runs --org MyOrg --action url

# Pick a repo
gha fzf repos --org MyOrg
```

In detail mode, Esc goes back to the run list.

### tmux keybindings

Add to `.tmux.conf`:

```sh
# Prefix + g: pick a run with detail drill-down
bind-key g display-popup -E -w 80% -h 60% "gha fzf runs --org MyOrg --action detail"

# Prefix + G: pick a repo, then browse its runs
bind-key G display-popup -E -w 80% -h 60% \
  "gha fzf repos --org MyOrg | xargs -I{} gha fzf runs --repo {} --action detail"
```

Requires `fzf` installed. Colors match the selected `--theme`.

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

Rust + [ratatui](https://ratatui.rs) + crossterm + tokio + reqwest (rustls). Single binary, no OpenSSL dependency. ~3.4MB.

## Roadmap

- [x] Live TUI dashboard with polling
- [x] Smart repo filtering (activity-based, `--days`)
- [x] Repo picker (toggle org repos at runtime)
- [x] Detail view with tree-drawn job/step hierarchy
- [x] fzf integration for tmux popups with detail drill-down
- [x] Themes (Catppuccin Mocha, Tokyo Night, Tokyo Night Storm)
- [x] Homebrew tap (`brew install LockeAG/tap/gha`)
- [x] Prebuilt binaries (macOS arm64/x86_64, Linux x86_64/arm64)
- [ ] Workflow re-run from TUI
- [ ] Log streaming for in-progress steps
- [ ] `~/.config/gha/config.toml` for default orgs/repos

## License

MIT
