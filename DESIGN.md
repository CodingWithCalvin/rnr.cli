# rnr - Cross-Platform Task Runner

## Vision

**Clone a repo and tasks just work** - no dependency installs, no global tools, no configuration. A truly zero-setup task runner for Windows, macOS, and Linux.

## Core Principles

1. **Zero setup for contributors** - Clone the repo, run tasks immediately
2. **Cross-platform** - First-class support for Windows, macOS, and Linux
3. **Self-contained** - All binaries and wrappers checked into the repo
4. **Simple** - Easy to understand, easy to use
5. **Composable** - Tasks can delegate to other tasks, enabling monorepo workflows

---

## Project Details

| Attribute | Value |
|-----------|-------|
| Name | `rnr` (pronounced "runner") |
| Language | Rust |
| Task file | `rnr.yaml` |
| License | TBD |

---

## Repository Structure

When a repo is initialized with rnr, it contains:

```
project/
├── .rnr/
│   ├── config.yaml            # Platform configuration
│   └── bin/
│       ├── rnr-linux-amd64    # Linux x86_64 binary
│       ├── rnr-macos-amd64    # macOS x86_64 binary
│       ├── rnr-macos-arm64    # macOS ARM64 binary
│       ├── rnr-windows-amd64.exe   # Windows x86_64 binary
│       └── rnr-windows-arm64.exe   # Windows ARM64 binary
├── rnr                        # Shell wrapper script (Unix)
├── rnr.cmd                    # Batch wrapper script (Windows)
├── rnr.yaml                   # Task definitions
└── ... (rest of project)
```

**Note:** Only the selected platforms are included. Most projects only need 2-3 platforms.

### Platform Configuration

`.rnr/config.yaml` tracks which platforms are configured:

```yaml
version: "0.1.0"
platforms:
  - linux-amd64
  - macos-arm64
  - windows-amd64
```

### Wrapper Scripts

**`rnr` (Unix shell script):**
```bash
#!/bin/sh
set -e

# Detect OS
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
EXT=""
case "$OS" in
  linux*) OS="linux" ;;
  darwin*) OS="macos" ;;
  mingw*|msys*|cygwin*) OS="windows"; EXT=".exe" ;;
  *) echo "Error: Unsupported OS: $OS" >&2; exit 1 ;;
esac

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
  x86_64|amd64) ARCH="amd64" ;;
  arm64|aarch64) ARCH="arm64" ;;
  *) echo "Error: Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

BINARY="$(dirname "$0")/.rnr/bin/rnr-${OS}-${ARCH}${EXT}"

if [ ! -f "$BINARY" ]; then
  echo "Error: rnr is not configured for ${OS}-${ARCH}." >&2
  echo "Run 'rnr init --add-platform ${OS}-${ARCH}' to add support." >&2
  exit 1
fi

exec "$BINARY" "$@"
```

**`rnr.cmd` (Windows batch script):**
```batch
@echo off
setlocal

:: Detect architecture
if "%PROCESSOR_ARCHITECTURE%"=="ARM64" (
  set "ARCH=arm64"
) else (
  set "ARCH=amd64"
)

set "BINARY=%~dp0.rnr\bin\rnr-windows-%ARCH%.exe"

if not exist "%BINARY%" (
  echo Error: rnr is not configured for windows-%ARCH%. >&2
  echo Run 'rnr init --add-platform windows-%ARCH%' to add support. >&2
  exit /b 1
)

"%BINARY%" %*
```

---

## User Experience

### Initialization (one-time, by repo maintainer)

```bash
# macOS/Linux
curl -sSL https://rnr.dev/rnr -o rnr && chmod +x rnr && ./rnr init

# Windows (PowerShell)
irm https://rnr.dev/rnr.exe -OutFile rnr.exe; .\rnr.exe init
```

#### Interactive Platform Selection

When running `rnr init`, an interactive prompt lets you choose which platforms to support:

```
Initializing rnr...

Which platforms should this project support?
(Current platform is pre-selected)

  [x] linux-amd64      (760 KB)
  [ ] macos-amd64      (662 KB)
  [x] macos-arm64      (608 KB)  <- current
  [x] windows-amd64    (584 KB)
  [ ] windows-arm64    (528 KB)

  Selected: 1.95 MB total

  [Enter] Confirm  [Space] Toggle  [a] All  [n] None  [Esc] Cancel
```

#### Non-Interactive Mode

For CI/CD or scripting, use flags:

```bash
# Specify exact platforms
rnr init --platforms linux-amd64,macos-arm64,windows-amd64

# Include all platforms
rnr init --all-platforms

# Current platform only
rnr init --current-platform-only
```

#### Init Steps

The `init` command:
1. Detects current platform and pre-selects it
2. Shows interactive platform selection (unless flags provided)
3. Creates `.rnr/bin/` directory
4. Downloads selected platform binaries from GitHub Releases
5. Creates `.rnr/config.yaml` with selected platforms
6. Creates wrapper scripts (`rnr`, `rnr.cmd`)
7. Creates starter `rnr.yaml` if one doesn't exist
8. Cleans up the initial downloaded binary

#### Adding/Removing Platforms Later

```bash
# Add a platform
rnr init --add-platform windows-arm64

# Remove a platform
rnr init --remove-platform linux-amd64

# Show current platforms
rnr init --show-platforms
```

### Running Tasks (contributors)

```bash
git clone <repo>
./rnr build        # macOS/Linux
rnr build          # Windows
```

### Upgrading rnr

```bash
./rnr upgrade      # Updates binaries in .rnr/bin/
```

---

## Invocation Model

**Always run from project root.** Tasks can operate on subdirectories via the `dir:` property or namespaced task names.

```bash
./rnr build              # Run 'build' task
./rnr web:build          # Run namespaced 'web:build' task
./rnr api:test           # Run namespaced 'api:test' task
```

---

## Task File Schema

### Shorthand Syntax

For simple commands, use the shorthand form:

```yaml
build: cargo build --release
lint: npm run lint
test: cargo test
```

### Full Syntax

```yaml
task-name:
  description: Human-readable description for --list
  dir: <working directory, relative to project root>
  env:
    KEY: value
  cmd: <shell command>
  # OR
  task: <another rnr task name>
  # OR
  steps:
    - cmd: <command>
    - task: <task>
    - parallel:
        - cmd: <command>
        - task: <task>
```

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `description` | string | Human-readable description shown in `--list` |
| `dir` | string | Working directory (relative to project root) |
| `env` | map | Environment variables for this task |
| `cmd` | string | Shell command to execute |
| `task` | string | Another rnr task to run (can be in a subdirectory's rnr.yaml) |
| `steps` | array | Sequential list of commands/tasks |
| `parallel` | array | Steps to run in parallel (used within `steps`) |

### Example: Complete rnr.yaml

```yaml
# Simple commands (shorthand)
lint: npm run lint
format: npm run format

# Task with description
test:
  description: Run all tests
  cmd: cargo test

# Task with working directory
build-api:
  description: Build the API service
  dir: services/api
  cmd: cargo build --release

# Task with environment variables
build-web:
  description: Build web frontend for production
  dir: services/web
  env:
    NODE_ENV: production
  cmd: npm run build

# Delegate to nested task file
api:test:
  dir: services/api
  task: test    # runs 'test' from services/api/rnr.yaml

# Sequential steps
ci:
  description: Run full CI pipeline
  steps:
    - task: lint
    - task: test
    - task: build-all

# Parallel execution
build-all:
  description: Build all services in parallel
  steps:
    - cmd: echo "Starting builds..."
    - parallel:
        - task: build-api
        - task: build-web
    - cmd: echo "All builds complete"

# Complex workflow
deploy:
  description: Deploy to production
  steps:
    - task: ci
    - parallel:
        - dir: services/api
          cmd: ./deploy.sh
        - dir: services/web
          cmd: ./deploy.sh
    - cmd: echo "Deployment complete"
```

### Nested Task Files

Subdirectories can have their own `rnr.yaml` for organization:

```
project/
├── rnr.yaml                    # Root task file
└── services/
    ├── api/
    │   └── rnr.yaml            # API-specific tasks
    └── web/
        └── rnr.yaml            # Web-specific tasks
```

**services/api/rnr.yaml:**
```yaml
build: cargo build --release
test: cargo test
clean: cargo clean
```

**Root rnr.yaml delegates:**
```yaml
api:build:
  dir: services/api
  task: build

api:test:
  dir: services/api
  task: test
```

---

## Built-in Commands

| Command | Description |
|---------|-------------|
| `rnr init` | Initialize the current directory with rnr |
| `rnr upgrade` | Update rnr binaries to latest version |
| `rnr --list` | List all available tasks with descriptions |
| `rnr --help` | Show help information |
| `rnr --version` | Show rnr version |
| `rnr <task>` | Run the specified task |

### rnr init

Creates the full rnr setup in the current directory:

```
rnr init [OPTIONS]

Options:
  --platforms <list>       Comma-separated list of platforms (non-interactive)
  --all-platforms          Include all available platforms
  --current-platform-only  Only include the current platform
  --add-platform <name>    Add a platform to existing setup
  --remove-platform <name> Remove a platform from existing setup
  --show-platforms         Show currently configured platforms
```

Creates:
- `.rnr/bin/` with selected platform binaries
- `.rnr/config.yaml` tracking configured platforms
- `rnr` and `rnr.cmd` wrapper scripts
- Starter `rnr.yaml` with example tasks (if not exists)

### rnr upgrade

Downloads the latest rnr binaries for configured platforms and replaces those in `.rnr/bin/`. Reads `.rnr/config.yaml` to know which platforms to update.

### rnr --list

Displays all available tasks with their descriptions:

```
$ ./rnr --list

Available tasks:
  build-all     Build all services in parallel
  build-api     Build the API service
  build-web     Build web frontend for production
  ci            Run full CI pipeline
  deploy        Deploy to production
  format
  lint
  test          Run all tests
```

---

## MVP Features (v0.1)

### Core Functionality
- [x] Parse `rnr.yaml` task files
- [x] Run shell commands (`cmd`)
- [x] Set working directory (`dir`)
- [x] Environment variables (`env`)
- [x] Task descriptions (`description`)
- [x] Sequential steps (`steps`)
- [x] Parallel execution (`parallel`)
- [x] Delegate to other tasks (`task`)
- [x] Delegate to nested task files (`dir` + `task`)
- [x] Shorthand syntax (`task: command`)

### Built-in Commands
- [ ] `rnr init` - Initialize repo with platform selection
- [ ] `rnr upgrade` - Update binaries for configured platforms
- [x] `rnr --list` - List tasks
- [x] `rnr --help` - Show help
- [x] `rnr --version` - Show version

### Cross-Platform
- [x] Build for Linux (x86_64)
- [x] Build for macOS (x86_64, arm64)
- [x] Build for Windows (x86_64, arm64)
- [ ] Shell wrapper script generation
- [ ] Batch wrapper script generation

### Distribution
- [ ] Host binaries on GitHub Releases
- [ ] Init downloads selected platform binaries
- [ ] Upgrade fetches latest binaries for configured platforms
- [ ] Platform selection (interactive and non-interactive)

---

## Future Features

### Task Dependencies
Run prerequisite tasks before a task executes:

```yaml
deploy:
  depends: [build, test]
  cmd: ./deploy.sh
```

### Conditional Execution
Run tasks conditionally based on environment or conditions:

```yaml
deploy:
  if: ${{ env.CI == 'true' }}
  cmd: ./deploy.sh

test:
  when:
    - file_exists: package.json
  cmd: npm test
```

### Watch Mode
Automatically re-run tasks when files change:

```yaml
dev:
  watch:
    paths: [src/**/*.rs]
    ignore: [target/]
  cmd: cargo build
```

### Variable Interpolation
Use variables in task definitions:

```yaml
vars:
  version: "1.0.0"

build:
  cmd: cargo build --version ${{ vars.version }}
```

### Includes
Import tasks from other files:

```yaml
include:
  - .rnr/common.yaml
  - .rnr/deploy.yaml
```

### Dotenv Loading
Automatically load `.env` files:

```yaml
build:
  dotenv: .env.production
  cmd: npm run build
```

Or global setting:

```yaml
config:
  dotenv: true    # Auto-load .env if present

tasks:
  build:
    cmd: npm run build
```

### Caching / Incremental Runs
Skip tasks if inputs haven't changed:

```yaml
build:
  inputs:
    - src/**/*.rs
    - Cargo.toml
  outputs:
    - target/release/myapp
  cmd: cargo build --release
```

### Task Arguments
Pass arguments to tasks:

```yaml
greet:
  args:
    - name: name
      required: true
  cmd: echo "Hello, ${{ args.name }}"
```

```bash
./rnr greet --name=World
```

### Hooks
Run commands before/after tasks:

```yaml
build:
  before: echo "Starting build"
  cmd: cargo build
  after: echo "Build complete"
```

### Default Task
Run a default task when no task is specified:

```yaml
default: build

build:
  cmd: cargo build
```

```bash
./rnr           # Runs 'build'
```

### Interactive Task Selection
When run without arguments, show interactive task picker:

```bash
./rnr
# Shows interactive list of tasks to choose from
```

---

## Technical Decisions

### Why Rust?
- Compiles to small, static binaries
- Cross-compilation is well-supported
- No runtime dependencies
- Memory safe
- Strong ecosystem for CLI tools (clap, serde, etc.)

### Binary Sizes

Current binary sizes per platform:

| Platform | Size |
|----------|------|
| Linux x86_64 | ~760 KB |
| macOS x86_64 | ~662 KB |
| macOS ARM64 | ~608 KB |
| Windows x86_64 | ~584 KB |
| Windows ARM64 | ~528 KB |
| **All platforms** | **~3.1 MB** |

Size optimizations applied:
- `opt-level = "z"` for size optimization
- LTO (Link-Time Optimization)
- Strip symbols
- `panic = "abort"`

Future size reduction options:
- Replace `reqwest` with `ureq` (smaller HTTP client)
- Remove `tokio` if not needed for async
- Use smaller YAML parser

### Shell Execution
- **Unix**: Execute commands via `sh -c "<command>"`
- **Windows**: Execute commands via `cmd /c "<command>"` or PowerShell

### Parallel Execution
- Use Rust async or threads for parallel task execution
- Capture and interleave output appropriately
- Handle failures (fail-fast vs. continue)

### Error Handling
- Clear error messages with context
- Exit codes: 0 for success, non-zero for failure
- Propagate exit codes from executed commands

---

## Open Questions

1. **Task name restrictions** - What characters are allowed in task names? Alphanumeric + hyphen + colon?

2. **Parallel failure behavior** - Fail fast or wait for all? Configurable?

3. **Output handling** - How to handle output from parallel tasks? Interleave, buffer, or prefix?

4. **Shell selection** - Allow users to specify shell? (bash, zsh, fish, PowerShell)

5. **Binary hosting** - Where to host binaries? GitHub Releases? Custom CDN?

6. **Versioning** - How to handle version pinning? Lock file?

---

## Appendix: Comparison with Existing Tools

| Tool | Requires Install | Cross-Platform | Checked into Repo |
|------|------------------|----------------|-------------------|
| npm scripts | Node.js | Yes | No (needs Node) |
| Make | Make | Partial (Windows pain) | No |
| Just | Just | Yes | No |
| Task (taskfile.dev) | Task | Yes | No |
| **rnr** | **Nothing** | **Yes** | **Yes** |
