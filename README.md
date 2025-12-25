# 🐍 WovenSnake

A high-performance Python package manager built with **Rust**.

## ✨ Features

- **⚡ Blazing Fast**: Parallel dependency resolution and installation.
- **🔒 Lockfile Support**: Reproducible builds with `wovenpkg.lock` (including SHA256 verification).
- **🌪️ Self-Healing Environments**: Automatically prunes unused packages and repairs broken environments.
- **📦 Smart Extraction**: Handles Wheels and Source Distributions (`.tar.gz`) seamlessly.
- **🛠️ Zero-Config Venv**: Automatically creates and manages virtual environments.

## 🚀 Quick Start

### Initialize a project
```bash
wovensnake init
```

### Install dependencies
```bash
wovensnake install
```

### Update to latest versions
```bash
wovensnake update
```

## 📄 Configuration (`wovenpkg.json`)

```json
{
  "name": "my-python-project",
  "version": "1.0.0",
  "dependencies": {
    "requests": "2.26.0",
    "pytest": "7.0.0"
  },
  "virtualEnvironment": ".venv"
}
```
