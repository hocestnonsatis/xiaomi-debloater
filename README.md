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
- Rust toolchain **only if you build from source**

## Install (from source)

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

## Publish the repo (first time)

From the project directory (folder name can be anything; GitHub repo should be `xiaomi-debloater`):

```bash
git branch -M main
git add -A
git commit -m "Initial commit: xiaomi-debloater"
gh repo create xiaomi-debloater --public --source=. --remote=origin --push
```

If the empty repo already exists on GitHub:

```bash
git remote add origin https://github.com/YOUR_USER/xiaomi-debloater.git
git branch -M main
git push -u origin main
```

Update `repository` in `Cargo.toml` if your username is not `hocestnonsatis`.

## Releases (GitHub Actions)

Pushing a **version tag** builds **Linux x86_64**, **Windows x86_64**, **macOS aarch64**, and **macOS x86_64** in CI and attaches archives to a **GitHub Release** (`.tar.gz` for Unix, `.zip` for Windows).

1. Bump `version` in `Cargo.toml` on `main` and merge/push as usual.
2. Tag and push:

   ```bash
   git tag v0.1.0   # must match Cargo.toml version SemVer; prefix v is required for the workflow
   git push origin v0.1.0
   ```

Workflow file: [`.github/workflows/release.yml`](.github/workflows/release.yml).

## Extending the catalog

Packages live in [`data/packages.json`](data/packages.json). Each entry has:

- `id`: application id (`com….`)
- `category`: grouping label
- `description`: short English explanation
- `risk`: `safe` \| `caution` \| `advanced`

**Do not** add packages that are required for boot, encryption, telephony, or core MIUI security unless you clearly mark them `advanced` and document the impact.

Pull requests that add well-described regional variants are welcome.

## License

MIT. See [`LICENSE`](LICENSE).
