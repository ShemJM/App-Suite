# App Suite

A macOS desktop app that bundles personal productivity tools into a single launcher. Built with [Tauri](https://tauri.app), [SvelteKit](https://kit.svelte.dev), and TypeScript.

## Download

Pre-built macOS DMGs are attached to each [GitHub Release](https://github.com/ShemJM/App-Suite/releases).

> **Note:** The app is not code-signed. After dragging it to Applications, remove the quarantine flag before opening:
> ```
> xattr -cr /Applications/App\ Suite.app
> ```

## Apps

### PhotoVault

Browse and manage a photo library from any local folder.

- **Timeline** — photos grouped by month, sorted by date or file size
- **Burst groups** — automatically detects photos taken within 20 seconds of each other
- **Lightbox** — full-size view with prev/next navigation and move-to-trash
- Keyboard shortcuts: `←` / `→` to navigate, `Delete` to trash, `Escape` to close

## Development

**Requirements:** Node 20+, Rust (stable)

```bash
npm install
npm run tauri dev
```

## Building

```bash
npm run tauri build
```

The DMG will be at `src-tauri/target/release/bundle/dmg/`.

## Releases

Releases are created automatically by GitHub Actions when you push a version tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```
