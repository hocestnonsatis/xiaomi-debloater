# xiaomi-debloater

Terminal UI (TUI) for cleaning **pre-installed apps** on **Xiaomi / MIUI / HyperOS** phones over **ADB**. It cross-checks a curated catalog with `pm list packages` on your device, lets you multi-select entries, and runs:

```text
adb shell pm uninstall --user 0 <package>
```

That removes the app **for the current user (user 0)**. The package often remains on the **system partition**; a factory reset typically brings OEM apps back. Some apps can be reinstalled from an app store or recovered with advanced `adb` commands if you know what you are doing.

**Disclaimer:** Removing the wrong packages can break features (cloud, find-device, updates, assistant, etc.). Nothing here is a guarantee for every ROM or region. Review each package, use backups, and proceed at your own risk.

## Requirements

- [Android Platform Tools](https://developer.android.com/tools/releases/platform-tools) (`adb` on your `PATH`)
- USB debugging enabled on the phone, authorized for this computer

## Install

1. Open [Releases](https://github.com/hocestnonsatis/xiaomi-debloater/releases) and download the archive for your OS (Linux, Windows, or macOS—use the Apple Silicon or Intel macOS build as appropriate).
2. Extract it and run `xiaomi-debloater` (or `xiaomi-debloater.exe` on Windows). Put the binary on your `PATH` if you want to run it from any directory.

**Build from source** (requires [Rust](https://rustup.rs/)):

```bash
cargo install --path .
# or
cargo build --release
# binary: target/release/xiaomi-debloater
```

## Usage

1. Connect the phone with USB (or use wireless debugging if you already paired `adb`).
2. Run:

   ```bash
   xiaomi-debloater
   ```

3. If several devices show up in `adb devices`, set:

   ```bash
   export ANDROID_SERIAL=<your-serial>
   xiaomi-debloater
   ```

### Keys

| Key | Action |
|-----|--------|
| ↑ / ↓ | Move selection |
| Space | Toggle selected package |
| a | Select all **visible** rows |
| c | Clear selection (visible) |
| A | Show / hide **Advanced** risk packages (hidden by default) |
| r | Refresh installed list from device |
| x | Remove selected (confirmation) |
| q | Quit |

Risk labels: **safe** (green), **caution** (yellow), **advanced** (red). Read descriptions before removing **advanced** entries.

## License

MIT. See [`LICENSE`](LICENSE).
