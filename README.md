# tody

> **tiny tidy todos = tody**

A fast, single-binary task manager that lives in your terminal. Keep global todos alongside project-local ones — tody knows where you are.

- **Path-aware** — tasks can be scoped to a project (git root) so they only appear when you're there
- **Zero config** — works out of the box, stores everything in a local SQLite database
- **Interactive** — run `tody` with no arguments for a fuzzy picker
- **Portable** — single static binary, no runtime dependencies

## Install

### Homebrew (macOS / Linux)

```sh
brew install treyorr/tap/tody     # coming soon
```

### Pre-built binaries

Grab the latest release for your platform from [**Releases**](https://github.com/treyorr/tody/releases) and place it somewhere on your `PATH`.

### From source

Requires [Rust](https://rustup.rs/) 1.85+:

```sh
cargo install --git https://github.com/treyorr/tody.git
```

Or clone and build locally:

```sh
git clone https://github.com/treyorr/tody.git
cd tody
cargo install --path .
```

## Quick start

```sh
tody add "Buy groceries"              # add a global task
tody add "Fix login bug" --local      # add a task scoped to this project
tody                                  # interactive picker — select a task to complete
tody list                             # view pending tasks
```

## Commands

| Command | Description |
|---------|-------------|
| `tody` | Interactive picker — select a task to mark done |
| `tody add <title>` | Add a task (`-l` / `--local` to scope to current project) |
| `tody list` | Show tasks (`-g` global, `-l` local, `-a` include completed) |
| `tody done <id>` | Mark a task as completed |
| `tody rm <id>` | Delete a task (with confirmation) |
| `tody log` | Show recently completed tasks |
| `tody prune` | Find & remove tasks for deleted project folders |
| `tody config set <key> <value>` | Set a config value |
| `tody config get <key>` | Read a config value |
| `tody update` | Self-update from GitHub releases |
| `tody uninstall` | Remove database, config, and binary |

### Adding tasks

```sh
tody add "Review PR #42"              # global — shows up everywhere
tody add "Write migration" --local    # local — tied to current project root
```

### Listing tasks

```sh
tody list                # default view (merged)
tody list --local        # only tasks for this project
tody list --global       # only global tasks
tody list --all          # include completed tasks
```

### Completing & removing

```sh
tody done 3              # mark task #3 as completed
tody rm 7                # permanently delete task #7
```

### Activity log

```sh
tody log                 # recently completed tasks with relative timestamps
```

### Pruning orphans

```sh
tody prune               # detect project folders that no longer exist, clean up
```

## How scoping works

Every task is either **global** or **local**:

| Scope | Behavior |
|-------|----------|
| **Global** | No folder association — visible everywhere |
| **Local** | Tied to a project path (nearest git root, or `cwd` as fallback) |

The interactive picker and `list` command respect your `default_view` config, so you can choose to see only local tasks, only global, or both.

## Configuration

Config lives at `~/.config/tody/config.toml` (macOS/Linux):

```toml
default_view = "merged"          # merged | local | global
color_local = "bright_magenta"   # ANSI color name
color_global = "bright_cyan"     # ANSI color name
```

```sh
tody config set default_view local
tody config get color_local
```

## Data storage

| Item | Path (macOS) |
|------|-------------|
| Database | `~/Library/Application Support/tody/tody.db` |
| Config | `~/.config/tody/config.toml` |

Paths are resolved via [`dirs`](https://docs.rs/dirs) so they follow platform conventions on Linux and Windows too.

## Development

Requires [mise](https://mise.jdx.dev/) (or Rust 1.85+ installed manually).

```sh
git clone https://github.com/treyorr/tody.git
cd tody
mise install              # install toolchain
mise run ci               # fmt-check → check → clippy → test
```

Individual tasks:

```sh
mise run fmt              # format code
mise run lint             # clippy with -D warnings
mise run test             # run unit tests
cargo run -- add "test"   # run locally
```

## Versioning

tody uses [CalVer](https://calver.org/) — **`YYYY.MM.COUNTER`**

- `YYYY` — full year
- `MM` — month (no zero-padding)
- `COUNTER` — release number within that month, starting at 1

Example: `2026.2.1` is the first release of February 2026.

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

[MIT](LICENSE)
