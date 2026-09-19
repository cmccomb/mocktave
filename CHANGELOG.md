# Changelog

## 0.1.6

- Add `native` and `bundled` Cargo features alongside the existing Docker default.
  `bundled` embeds a verified private Octave runtime and its numerical libraries;
  `native` uses an installed executable. Existing `eval`, `wrap`, and result APIs
  follow the selected feature without application changes.
- Add fallible `try_eval` and an explicit backend selector for applications that
  enable multiple backends. Native evaluations use separate workspace files and
  report launch, execution, and workspace errors with diagnostics.
- Publish pinned bundles with corresponding sources for macOS ARM64/Intel and
  Linux ARM64/x86-64. Windows bundling remains pending source packaging.
- Fix diagonal-matrix parsing and propagate Docker execution failures.
- Verify bundled consumers after relocation with an empty PATH, and qualify
  native and bundled backends in platform CI.
