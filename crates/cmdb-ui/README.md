# cmdb-ui

A GPU-rendered desktop viewer for the synthetic CMDB fixture: a Makepad 2.0
(Splash DSL + Rust) single-window app.

The screen is a scaffold with a clean seam to a future data/view-model crate:

| Piece | Where | Role |
|---|---|---|
| Fixture + filtering + counts | `src/data.rs` | Plain Rust. `Data::synthetic()` is the only place the fixture is known. |
| Table widget | `src/table.rs` | `ResourceTable` owns `Data` and projects the filtered rows onto a `PortalList`. Row clicks move the selection. |
| App shell | `src/app.rs` | Routes button/input actions into `Data`, then writes the result into labels, detail fields and `mod.state`. |
| View tree | `src/ui.rs` | Splash structure only: title bar, filter rail, table, detail panel. |

Rows live only in Rust. Splash never computes a filter or a count; it renders
what the table widget and the app shell hand it. To plug in real data, build a
`Data` from the eventual source and keep the same widget/`set_*` API.

## Dependency

Makepad 2.0 is the GitHub `main` line, pinned in `Cargo.toml`:

```toml
makepad-widgets = { git = "https://github.com/makepad/makepad", rev = "506ec05105b9adb08d1d21f9879df181b5244ea7", default-features = false }
```

`default-features = false` links only the core widget family, which is all this
screen uses (window, views, labels, buttons, text input, portal list, scroll
bar). The package is `makepad-widgets` and the crate source is the `widgets/`
directory of that repository at the pinned commit.

## Run

```bash
# build
cargo build -p cmdb-ui

# run
cargo run -p cmdb-ui

# run with Splash hot reload: edits to the script_mod! blocks apply on save.
# Rust changes still require a rebuild.
cargo run -p cmdb-ui -- --hot
```

### Linux window backend

Windowing must be selected explicitly on headless or multi-compositor Linux
desktops:

```bash
# X11
cargo run -p cmdb-ui -- --linux-backend=x11

# Wayland
cargo run -p cmdb-ui -- --linux-backend=wayland

# DRM/KMS, only when forced (needs the tty, no display server)
cargo run -p cmdb-ui -- --linux-backend=direct
```

`--linux-backend=wm` targets a running makepad window manager. The same
selection can be forced with `MAKEPAD=linux_direct|wayland|x11`. Without a flag,
Makepad probes for a window-manager socket, then Wayland, then X11.

## Screen

- **Title bar** — product name, one-line fleet summary (accounts · resources ·
  edges · regions · synthetic), and a "SYNTHETIC" status chip pinned right.
- **Filter rail** — accounts, regions and kinds grouped under section labels.
  Chips toggle (click the active one again to clear it); the active selector
  shows an accent bar and its text is written by Rust. Kinds show fixture
  counts. "All" clears every selector and the search box.
- **Toolbar + table** — one row with the "Resources" title, the visible count
  and the active selectors, then the search field. Column widths are shared
  between the header and the rows (`mod.widgets.cmdb_col_*`); account, region
  and kind are fixed, id and name share the rest and ellipsize instead of
  clipping. Rows are 36px with 1px separators, separate hover tint and an
  accent bar plus tint for the selection. The header stays above the scrolling
  portal list.
- **Graph / Table toggle** — tabs in the content toolbar switch the centre pane
  in Rust. Graph is the default and renders `DagView` from `src/dag/**`
  (wheel zoom, drag pan), fed with `ResourceTable::graph()`; the live node and
  edge counts sit above the canvas. Table sits beside it and shares the same
  filter and selection state. Clicking a graph node selects the matching table
  row and updates the detail rail; clicking empty canvas clears the selection.
- **Detail rail** — uppercase muted keys, monospace values, showing account,
  region, kind, id, tags and size for the selected row. The first visible row
  is selected by default; a single sentence replaces the fields when the
  filter matches nothing.

All colours, spacing, radii and type come from `theme.*` tokens (Dark base
theme); there are no ad-hoc hex values in the Splash layer.

### DAG integration

`crates/cmdb-ui/src/dag/**` is the renderer (model caps, layout, `DagView`).
`graph_view.rs` is the wiring: it pushes `Graph` into `DagView::set_graph`
whenever the filter changes, mirrors the table's selected row with `select_id`,
and turns `DagSelectAction` into a table selection (or a clear). The DAG module
registers itself with `crate::dag::script_mod(vm)` before the root script runs.

## Current scope

- Embedded synthetic fixture: 24 rows, 3 accounts, 2 regions, 4 kinds.
- Account / region / kind filters are toggle buttons (click again to clear);
  "All resources" resets everything, including the search box.
- Search matches resource id, name and kind (case-insensitive).
- Selecting a row shows its fields in the right-hand detail panel.
- The "SYNTHETIC DATA" badge in the title bar is deliberately explicit.

No AWS calls, no SQLite, no network: this crate is the UI scaffold.
