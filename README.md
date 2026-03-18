# gha

GitHub Actions in the terminal. Dashboard, drill-down, logs, re-runs — without opening a browser.

## Install

```sh
# Homebrew (macOS / Linux)
brew tap LockeAG/tap
brew install gha

# Cargo
cargo install --git https://github.com/LockeAG/gha

# Prebuilt binaries
# https://github.com/LockeAG/gha/releases
```

## Quick start

```sh
gha                          # auto-detect repo from git remote
gha --org MyOrg              # watch an org (or user account)
gha --repo owner/name        # watch specific repos
gha init                     # generate config file
```

If you have `gh` CLI authenticated, that's it. No token setup needed.

## What it does

**Dashboard** — live-updating table of workflow runs across your repos. Status icons, branch, age, run number. Polls every 30s (5s when runs are in-progress).

**Detail view** — drill into a run to see jobs and steps as a tree with durations. `Enter` from dashboard.

**Log viewer** — read job output directly in the terminal. `L` on a completed job. Error lines highlighted red, warnings yellow. Auto-tails to the end.

**Re-run** — trigger a re-run without leaving the TUI. `R` on any run. Smart: re-runs only failed jobs on failures, full workflow otherwise.

**Repo picker** — toggle which org repos to watch at runtime. `a` to open.

**fzf mode** — composable pickers for tmux popups. `gha fzf runs --action detail` gives you a two-stage fzf flow: pick a run, browse its jobs.

## Configuration

```sh
gha init  # creates ~/.config/gha/config.toml
```

```toml
theme = "tokyo-night-storm"
interval = 15
days = 7
max_repos = 5
orgs = ["DreamsEngine", "LockeAG"]
repos = ["some/pinned-repo"]
```

CLI flags override config values. Respects `XDG_CONFIG_HOME`. Stow-friendly.

### Token resolution

1. `--token` flag
2. `GH_TOKEN` env
3. `GITHUB_TOKEN` env
4. `gh auth token` (GitHub CLI)

### Themes

| Name | Alias |
|------|-------|
| `catppuccin-mocha` (default) | `mocha` |
| `tokyo-night` | `tn` |
| `tokyo-night-storm` | `tns` |

## Key bindings

### Dashboard

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate |
| `Ctrl-d` / `Ctrl-u` | Half-page scroll |
| `g` / `G` | Top / bottom |
| `Enter` | Detail view |
| `o` | Open in browser |
| `R` | Re-run workflow |
| `/` | Search |
| `f` | Filter mode |
| `1`-`4` | Quick filter: all / fail / running / pass |
| `a` | Repo picker |
| `r` | Refresh |
| `q` | Quit |

### Detail view

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate jobs/steps |
| `L` | View job logs |
| `R` | Re-run workflow |
| `o` | Open in browser |
| `Esc` | Back |

### Log viewer

| Key | Action |
|-----|--------|
| `j` / `k` | Scroll |
| `Ctrl-d` / `Ctrl-u` | Page scroll |
| `g` / `G` | Top / end |
| `Esc` | Back |

### Repo picker

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate |
| `Space` | Toggle |
| `Esc` | Apply |

## fzf + tmux

```sh
gha fzf runs --action detail     # pick run → browse jobs → open
gha fzf runs --action url        # output URL for piping
gha fzf runs --action open       # pick and open (default)
gha fzf repos                    # pick a repo name
```

### tmux keybinding

```sh
bind-key g display-popup -E -w 80% -h 80% -b rounded -S fg=#565f89 \
  "gha fzf runs --action detail"
```

Reads config for theme and orgs. Colors match the selected theme. Requires `fzf`.

## Architecture

```
crossterm input ──┐
tick timer (250ms) ┤──> mpsc<AppEvent> ──> main loop (App + Terminal)
API poller ────────┘         ↑
                    watch<Vec<String>> (repo list)
                    watch<bool> (active run detection → adaptive polling)
```

- Unidirectional data flow, no shared mutexes
- Adaptive polling: 5s when runs active, configurable interval when idle
- Jobs and logs fetched on-demand, never polled
- Rate limit auto-downgrade at <100 remaining
- Panic hook restores terminal on crash
- Selection preserved by run ID across refreshes

## API usage

Uses GitHub REST API with your authenticated token. Read-only operations only.

**Rate budget:** 5000 req/hr authenticated. Default settings (~2 req/min/repo) safe for 40+ repos. Built-in safeguards: activity filter (`--days`), repo cap (`--max-repos`), rate limit detection, on-demand fetching.

Compliant with [GitHub API Terms](https://docs.github.com/en/site-policy/github-terms/github-terms-of-service#h-api-terms).

## Stack

Rust, [ratatui](https://ratatui.rs), crossterm, tokio, reqwest (rustls). Single binary, no OpenSSL. ~3.4MB.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Issues and PRs welcome.

## License

MIT
