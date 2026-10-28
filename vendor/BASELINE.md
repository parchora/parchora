# Vendored Dependency Baseline

This directory holds the vendored dependency sources referenced by the root
`Cargo.toml` `[patch.crates-io]` table. It is a **pristine read-only baseline** —
do not edit the crate sources in place.

## Provenance

The sources here were originally tracked in a nested Git repository. That nested
repository was removed so that the vendored sources can be committed as ordinary
files in this repository (a nested `.git` makes Git record the whole directory as a
gitlink, which silently drops every crate that was not tracked in the nested repo).

The following record preserves the full provenance of that baseline.

| Field | Value |
|---|---|
| Nested repo baseline commit | `135a88c` — *gpui-fast 0.1.8 + gpui-fast-macos 0.1.8 pristine (vendored baseline)* |
| Files tracked in the baseline | 186 |
| Crates added after the baseline (were untracked) | `gpui-bidi` 0.1.1, `zorite-editor` 0.11.2, `zorite-markdown` 0.10.0 |
| Local deviation from pristine | [`DEC-60-park-offscreen.patch`](./DEC-60-park-offscreen.patch) |

## Crate inventory

| crate | Version | License | Origin |
|---|---|---|---|
| `gpui-fast` | 0.1.8 | Apache-2.0 | Zed Industries / Longbridge |
| `gpui-fast-macos` | 0.1.8 | Apache-2.0 | Zed Industries / Longbridge |
| `gpui-bidi` | 0.1.1 | MIT | Will Lehnertz |
| `zorite-editor` | 0.11.2 | MIT | Will Lehnertz |
| `zorite-markdown` | 0.10.0 | MIT | Will Lehnertz |

Each crate keeps its upstream `LICENSE` file alongside its source.

## The `DEC-60` local patch

`DEC-60` adds the `park_offscreen` primitive to the vendored GPUI line, plus an
`ime_diag_enabled` diagnostic switch. The patch is applied to the working tree and
is captured verbatim in `DEC-60-park-offscreen.patch`, so the pristine upstream state
remains reproducible:

```bash
# Review the deviation from the pristine baseline
git apply --stat DEC-60-park-offscreen.patch

# Reproduce the pristine state in a scratch copy
git apply --reverse DEC-60-park-offscreen.patch
```

## Rules

1. **Never edit these sources in place.** Validate a change in `spikes/` first, then
   land it here as an explicit, self-contained patch file alongside this record.
2. **Never commit build output.** Each crate's `target/` directory is ignored
   (see the root `.gitignore`); it is regenerable with `cargo build`.
3. **Never remove a crate here** without updating the root `Cargo.toml`
   `[patch.crates-io]` table in the same change.
