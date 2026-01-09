# 🏃 rnr

**Clone a repo. Run tasks. No setup required.**

[![Build & Test](https://github.com/CodingWithCalvin/rnr.cli/actions/workflows/build.yml/badge.svg)](https://github.com/CodingWithCalvin/rnr.cli/actions/workflows/build.yml)
[![Integration Tests](https://github.com/CodingWithCalvin/rnr.cli/actions/workflows/integration-test.yml/badge.svg)](https://github.com/CodingWithCalvin/rnr.cli/actions/workflows/integration-test.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## ✨ What is rnr?

**rnr** (pronounced "runner") is a cross-platform task runner that works **instantly** on any machine. No Node.js. No Python. No global installs. Just clone and go.

```bash
git clone your-repo
./rnr build    # It just works! 🎉
```

### 🤔 Why rnr?

| Tool | Requires |
|------|----------|
| npm scripts | Node.js installed |
| Makefile | Make installed (painful on Windows) |
| Just | Just installed |
| Task | Task installed |
| **rnr** | **Nothing!** ✅ |

rnr binaries live **inside your repo**. Contributors clone and run—zero friction.

---

## 🚀 Quick Start

### Initialize a Project

```bash
# Download and run init (one-time setup by maintainer)
curl -sSL https://rnr.dev/rnr -o rnr && chmod +x rnr && ./rnr init
```

This creates:
```
your-repo/
├── .rnr/bin/          # Platform binaries (Linux, macOS, Windows)
├── rnr                # Unix wrapper script
├── rnr.cmd            # Windows wrapper script
└── rnr.yaml           # Your task definitions
```

### Run Tasks

```bash
./rnr build        # Run the 'build' task
./rnr test         # Run the 'test' task
./rnr --list       # See all available tasks
```

---

## 📝 Task File Format

Tasks are defined in `rnr.yaml` at your project root.

### Simple Commands (Shorthand)

```yaml
build: cargo build --release
test: cargo test
lint: npm run lint
```

### Full Task Definition

```yaml
build:
  description: Build for production
  dir: src/backend           # Working directory
  env:
    NODE_ENV: production     # Environment variables
  cmd: npm run build
```

### Sequential Steps

```yaml
ci:
  description: Run CI pipeline
  steps:
    - task: lint
    - task: test
    - task: build
```

### Parallel Execution

```yaml
build-all:
  description: Build all services
  steps:
    - cmd: echo "Starting builds..."
    - parallel:
        - task: build-api
        - task: build-web
    - cmd: echo "✅ All done!"
```

### Nested Task Files

Subdirectories can have their own `rnr.yaml`:

```yaml
# Root rnr.yaml
api:build:
  dir: services/api
  task: build          # Runs 'build' from services/api/rnr.yaml
```

---

## 🛠️ Built-in Commands

| Command | Description |
|---------|-------------|
| `rnr <task>` | Run a task |
| `rnr --list` | List available tasks |
| `rnr --help` | Show help |
| `rnr --version` | Show version |
| `rnr init` | Initialize rnr in current directory |
| `rnr upgrade` | Update rnr binaries to latest |

---

## 📋 Complete Example

```yaml
# rnr.yaml

# Simple commands
lint: cargo clippy
format: cargo fmt

# Full tasks
build:
  description: Build release binary
  env:
    RUST_LOG: info
  cmd: cargo build --release

test:
  description: Run all tests
  cmd: cargo test --all

# Multi-step workflow
ci:
  description: Full CI pipeline
  steps:
    - task: format
    - task: lint
    - task: test
    - task: build

# Parallel builds for monorepo
build-all:
  description: Build all services
  steps:
    - parallel:
        - dir: services/api
          cmd: cargo build --release
        - dir: services/web
          cmd: npm run build
    - cmd: echo "🎉 Build complete!"

# Deploy workflow
deploy:
  description: Deploy to production
  steps:
    - task: ci
    - cmd: ./scripts/deploy.sh
```

---

## 🌍 Platform Support

| Platform | Architecture | Status |
|----------|--------------|--------|
| Linux | x86_64 | ✅ |
| macOS | x86_64 | ✅ |
| macOS | ARM64 (Apple Silicon) | ✅ |
| Windows | x86_64 | ✅ |

---

## 🔮 Roadmap

- [ ] Task dependencies (`depends: [build, test]`)
- [ ] Conditional execution (`if: ${{ env.CI }}`)
- [ ] Watch mode (`watch: [src/**/*.rs]`)
- [ ] Variable interpolation (`${{ vars.version }}`)
- [ ] Caching / incremental builds
- [ ] Interactive task picker

See [DESIGN.md](DESIGN.md) for the full roadmap.

---

## 🤝 Contributing

Contributions are welcome! Please read our contributing guidelines and submit PRs.

### Development

```bash
# Clone the repo
git clone https://github.com/CodingWithCalvin/rnr.cli
cd rnr.cli

# Build
cargo build

# Run tests
cargo test

# Run locally
cargo run -- --help
```

---

## 👥 Contributors

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->
<!-- ALL-CONTRIBUTORS-LIST:END -->

---

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

<p align="center">
  Made with ❤️ by <a href="https://github.com/CodingWithCalvin">CodingWithCalvin</a>
</p>
