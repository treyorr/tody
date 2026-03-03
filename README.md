# tody

> **tiny tidy todos = tody**

Path-aware terminal task manager for global and project-local todos.

## Install and run

```sh
curl -fsSL https://tlo3.com/tody-install.sh | sh
tody --version
```

If `tody` is not found, add this and restart your shell:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

Start using it:

```sh
tody add "Buy groceries"         # global task
tody add "Fix login bug" --local # task scoped to this project
tody                              # interactive picker
tody list                         # print pending tasks
```

## Most-used commands

| Command | Description |
|---------|-------------|
| `tody` | Interactive picker to mark a task done |
| `tody add <title>` | Add a task (`-l` / `--local` for project scope) |
| `tody list` | List tasks (`-g`, `-l`, `-a`) |
| `tody done <id>` | Mark task as completed |
| `tody rm <id>` | Delete task (with confirmation) |
| `tody log` | Show recently completed tasks |
| `tody prune` | Remove tasks tied to deleted folders |
| `tody update` | Self-update from GitHub releases |

## What it does

- **Path-aware**: Local tasks are tied to your current project root.
- **Zero config**: Works immediately with a local SQLite database.
- **Single binary**: No runtime dependencies.
- **Fast CLI flow**: Interactive picker plus explicit commands.

## Scope model

Every task is either:

- **Global**: visible from any folder.
- **Local**: tied to a project path (nearest git root, or `cwd` fallback).

## Configuration

Config file (macOS/Linux): `~/.config/tody/config.toml`

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

Paths are resolved via [`dirs`](https://docs.rs/dirs), so Linux and Windows use their platform conventions too.

## Other install options

Install a specific release:

```sh
curl -fsSL https://tlo3.com/tody-install.sh | TODY_VERSION=2026.2.1 sh
```

Install to a custom location:

```sh
curl -fsSL https://tlo3.com/tody-install.sh | BINDIR=/usr/local/bin sh
```

Install directly from source (Rust 1.85+):

```sh
cargo install --git https://github.com/treyorr/tody.git
```

Or build from a clone:

```sh
git clone https://github.com/treyorr/tody.git
cd tody
cargo install --path .
```

## Development

Requires [mise](https://mise.jdx.dev/) or Rust 1.85+.

```sh
git clone https://github.com/treyorr/tody.git
cd tody
mise install
mise run ci
```

```sh
mise run fmt
mise run lint
mise run test
cargo run -- add "test"
```

## Versioning

tody uses [CalVer](https://calver.org/): `YYYY.MM.COUNTER`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
