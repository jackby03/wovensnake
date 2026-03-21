<div align="center">
  <img src="../.github/assets/logo.png" alt="WovenSnake Cozy Logo" width="500">
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
*   **🌍 Truly Cross-Platform**: Native binaries for macOS (arm64 & x86_64), Linux, and Windows — no Rosetta 2 penalty on Apple Silicon.
*   **🧶 Self-Mending**: Automatically removes loose threads (unused packages) to keep your project clean.
*   **🏠 Zero-Config Home**: Creates virtual environments automatically, so your packages have a safe place to live.

---

## 💻 Platform Support

| Operating System | Architecture | Status |
| :--- | :--- | :--- |
| **macOS** | Apple Silicon (arm64) | ✅ Supported |
| **macOS** | Intel (x86_64) | ✅ Supported |
| **Linux** | x86_64 | ✅ Supported |
| **Linux** | aarch64 | ✅ Supported |
| **Windows** | x86_64 | ✅ Supported |

---

## 💿 Installation

### 🚀 Automatic (Recommended)

**macOS / Linux**:
```bash
curl -fsSL https://raw.githubusercontent.com/jackby03/wovensnake/main/scripts/install.sh | bash
```

The installer auto-detects your architecture (arm64 or x86_64), downloads the right binary, and optionally adds `woven` to your PATH. Pass `--yes` to skip all prompts.

**Windows (PowerShell)**:
```powershell
iwr -useb https://raw.githubusercontent.com/jackby03/wovensnake/main/scripts/install.ps1 | iex
```

### 📦 Pre-built Binaries

Download the binary for your platform from the [latest release](https://github.com/jackby03/wovensnake/releases/latest):

| Platform | Binary |
| :--- | :--- |
| macOS (Apple Silicon) | `woven-macos-arm64` |
| macOS (Intel) | `woven-macos-amd64` |
| Linux (x86_64) | `woven-linux-amd64` |
| Windows (x86_64) | `woven-windows-amd64.exe` |

### 🦀 From Source (Rustaceans)
```bash
cargo install --path .
```

### ♻️ Updating

Re-run the installer — it will overwrite the existing binary:

```bash
curl -fsSL https://raw.githubusercontent.com/jackby03/wovensnake/main/scripts/install.sh | bash
```

### 🗑️ Uninstalling

**macOS / Linux** — run the uninstall script (works even if `woven` is broken or not in PATH):
```bash
curl -fsSL https://raw.githubusercontent.com/jackby03/wovensnake/main/scripts/uninstall.sh | bash
```

Or if `woven` is already working:
```bash
woven self-uninstall
```

Both options remove the binary, the global cache (`~/.wovensnake`), and clean the `PATH` entry from your shell rc files. Add `--yes` to skip the confirmation prompt.

---

## 🎮 How to Use

### 1. Start a New Pattern (`init`)
Prepares `wovenpkg.json` for your project.
```bash
woven init
```

### 2. Knit Dependencies (`install`)
Install all dependencies from `wovenpkg.json`, or add and install specific packages in one step:
```bash
woven install                    # install everything in wovenpkg.json
woven install requests           # add requests and install
woven install requests flask     # add multiple packages
woven install flask==3.0.0       # add a specific version
```

### 3. Run in the Nest (`run`)
Execute any command within the context of your virtual environment.
```bash
woven run python main.py
```

### 4. Tidy Up (`remove`)
Gently removes a package and its unused threads.
```bash
woven remove flask
```

### 5. View the Tapestry (`list`)
Admire the packages currently woven into your project.
```bash
woven list
```

### 6. Fresh Start (`clean`)
Clears the virtual environment and local packages. Use `--all` to clear the global cache too.
```bash
woven clean
```

---

## 🧸 Support the Nest

Building such a cozy tool takes care and patience. If WovenSnake brought you comfort, consider supporting:

<p align="center">
  <a href="https://ko-fi.com/jackby03"><img src="https://img.shields.io/badge/Ko--fi-Support_on_Ko--fi-FF5E5B?style=for-the-badge&logo=ko-fi&logoColor=white" alt="Ko-fi" /></a>
  <a href="https://publishers.basicattentiontoken.org/en/c/jackby03"><img src="https://img.shields.io/badge/BAT-Brave_Rewards-FB542B?style=for-the-badge&logo=brave&logoColor=white" alt="BAT" /></a>
</p>

---

## 🤝 Contributing to the Nest

We follow a **Trunk-Based Development** workflow with strict CI verification. Check out **[CONTRIBUTING.md](CONTRIBUTING.md)** to get started.

## 📜 Licenssse

This project is licensed under the **MIT License**. Ssssee [LICENSE](LICENSE) for details.
