# ALS Rescue Desktop

Local Tauri 2 + React/TypeScript adapter for the ALS Rescue Rust workspace.

## Development

```text
npm install
npm run check
npm run tauri dev
```

## Local macOS Alpha Bundle

```text
npm run tauri build -- --bundles app
```

The resulting unsigned local application is written to the workspace target
directory under `target/release/bundle/macos/ALS Rescue.app`.

## Architecture Boundary

The frontend selects input and renders results. It must not parse ALS files,
inspect dependency paths or make matching, copy or rewrite decisions. Tauri
delegates the read-only flow to `rescue_application::analyze_project`.
