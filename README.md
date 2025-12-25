<div align="center">
  <img src=".github/assets/logo.png" alt="WovenSnake Cozy Logo" width="500">
  <h1>🧶 WovenSnake</h1>
</div>

[![Crates.io](https://img.shields.io/crates/v/wovensnake.svg?style=flat-square&color=98c379)](https://crates.io/crates/wovensnake)
[![Documentation](https://docs.rs/wovensnake/badge.svg?style=flat-square)](https://docs.rs/wovensnake)
[![CI Status](https://github.com/jackby03/wovensnake/workflows/Development%20CI/badge.svg?style=flat-square)](https://github.com/jackby03/wovensnake/actions)
[![License](https://img.shields.io/crates/l/wovensnake.svg?style=flat-square)](https://github.com/jackby03/wovensnake/blob/main/LICENSE)
[![Ko-Fi](https://img.shields.io/badge/Ko--fi-Fba8a8?style=flat-square&logo=ko-fi&logoColor=white)](https://ko-fi.com/jackby03)

> **"Dependencies, neatly woven."** 🐍🧶

**WovenSnake** is a cozy, high-performance Python package manager built with **Rust**. It knits your dependencies together securely, keeping your environment warm and tidy.

---

## 🍵 Why WovenSnake?

Managing packages shouldn't be a tangle. WovenSnake keeps things organized:
*   **⚡ Swift Knitting**: Parallel resolution and installation that finishes before your tea is ready.
*   **🔒 Secure Stitching**: Deterministic `wovenpkg.lock` ensures every install is identical.
*   **📦 Global Cache**: Shared storage in `~/.wovensnake/cache` to avoid downloading the same package twice.
*   **🧶 Self-Mending**: Automatically removes loose threads (unused packages) to keep your project clean.
*   **🏠 Zero-Config Home**: Creates virtual environments automatically, so your packages have a safe place to live.

---

## 💿 Installation

### 🚀 Automatic (Recommended)

**Linux / macOS**:
```bash
curl -fsSL https://raw.githubusercontent.com/jackby03/wovensnake/main/scripts/install.sh | sh
```

**Windows (PowerShell)**:
```powershell
iwr -useb https://raw.githubusercontent.com/jackby03/wovensnake/main/scripts/install.ps1 | iex
```

### 🦀 From Source (Rustaceans)
```bash
cargo install --path .
```

---

## 🎮 How to Use

### 1. Start a New Pattern (`init`)
Prepares `wovenpkg.json` for your project.
```bash
woven init
```

### 2. Add a New Thread (`add`)
Adds a package from PyPI to your project and installs it immediately.
```bash
woven add requests
```

### 3. Knit Dependencies (`install`)
Reads your pattern, gathers materials, and weaves the environment.
```bash
woven install
```

### 4. Run in the Nest (`run`)
Execute any command within the context of your virtual environment.
```bash
woven run python main.py
```

### 5. Tidy Up (`remove`)
Gently removes a package and its unused threads.
```bash
woven remove flask
```

### 6. View the Tapestry (`list`)
Admire the packages currently woven into your project.
```bash
woven list
```

### 7. Fresh Start (`clean`)
Clears the virtual environment and local packages. Use `--all` to clear the global cache too.
```bash
woven clean
```

---

## 🧸 Support the Nest

Building such a cozy tool takes care and patience. If WovenSnake brought you comfort, consider donating a warm coffee:

<a href='https://ko-fi.com/jackby03' target='_blank'><img height='36' style='border:0px;height:36px;' src='https://storage.ko-fi.com/cdn/kofi2.png?v=3' border='0' alt='Buy Me a Coffee at ko-fi.com' /></a>

*(Open Collective coming ssssoon...)*

---

## 🤝 Contributing to the Nest

We welcome all serpents! Whether you're fixing a bug or adding a new fang.
Check out **[CONTRIBUTING.md](CONTRIBUTING.md)** to get started.

## 📜 Licenssse

This project is licensed under the **MIT License**. Ssssee [LICENSE](LICENSE) for details.
