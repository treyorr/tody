# tody

> **tiny tidy todos = tody**

Path-aware terminal task manager for global and project-local todos.

![tody demo](demo.gif)

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
tody add "Buy groceries"           # global task (when not in a project)
cd ~/projects/my-app
tody add "Fix login bug"            # auto-detected as local (you're in a git repo)
tody add "Read docs" --global       # explicitly global, even inside a project
tody                                 # interactive picker scoped to current project
tody list                            # pending tasks for current project
tody list --global                   # pending global tasks only
tody list --all                      # everything across all projects
tody list --done                     # completed tasks
```

## Command quick guide

| If you want to... | Run | Result |
|-------------------|-----|--------|
| complete a task from a picker | `tody` | interactive prompt scoped to current project |
| add a task (auto-scoped) | `tody add "Title"` | local when in a git repo, global otherwise |
| add an explicitly global task | `tody add "Title" -g` | task is visible everywhere |
| add an explicitly local task | `tody add "Title" -l` | task is tied to the current project |
| see this project's tasks | `tody list` | local tasks for the current project only |
| see global tasks | `tody list -g` | global tasks only |
| see everything | `tody list -a` | all tasks across all projects |
| see completed tasks here | `tody list -d` | completed tasks scoped to current context |
| see all completed tasks | `tody list -a -d` | all completed tasks across projects |
| mark a task completed by id | `tody done <id>` | marks one task as done |
| edit a task's title | `tody edit <id> "New title"` | updates the task title |
| undo the last completion | `tody undo` | restores the last completed task to pending |
| permanently delete a task | `tody rm <id>` | removes one task after confirmation |

Other useful commands:

- `tody log`: recently completed activity (project-scoped by default)
- `tody log --all`: completed activity across all projects
- `tody prune`: remove tasks tied to deleted folders
- `tody update`: self-update from GitHub releases

## What it does

- **Project-scoped by default**: Inside a git repo, tasks are automatically local to that project.
- **Smart auto-detection**: No flags needed — `tody add` and `tody list` just work in context.
- **Zero config**: Works immediately with a local SQLite database.
- **Single binary**: No runtime dependencies.
- **Fast CLI flow**: Interactive picker plus explicit commands.
- **Developer-friendly**: `edit`, `undo`, project-scoped `log`.

## Scope model

Every task is either:

- **Global**: visible from any folder. Created with `-g` flag, or auto-detected outside a git repo.
- **Local**: tied to a project path (nearest git root). Auto-detected when inside a git repo.

When you `cd` into a project and run `tody`, you only see that project's tasks.
Use `tody list -g` for global tasks, or `tody list -a` to see everything.

## Configuration

Config file (macOS/Linux): `~/.config/tody/config.toml`

```toml
default_view = "auto"            # auto | merged | local | global
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
