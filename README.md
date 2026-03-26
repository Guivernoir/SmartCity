# SmartCity Forge

Bevy-based 3D city-builder prototype focused on utility coverage, OT/security decisions, and a Steam-ready presentation path.

## Launch

Normal GPU desktop:

```bash
cargo run
```

Old Linux laptop or fallback renderer:

```bash
SMARTCITY_FORCE_SOFTWARE_RENDERER=1 cargo run
```

Manual llvmpipe / GL debug path:

```bash
LIBGL_ALWAYS_SOFTWARE=1 WGPU_BACKEND=gl WGPU_FORCE_FALLBACK_ADAPTER=1 cargo run
```

Use `F1` in-game to open the graphics/config panel and switch render tiers, prop density, and related presentation settings at runtime.

## Core Controls

- `Left click` place / inspect
- `Mouse wheel` zoom
- `MMB drag` pan
- `RMB drag` orbit / tilt
- `WASD` or arrows move camera
- `Q/E` rotate camera
- `Z/X` or `PageUp/PageDown` zoom the camera envelope
- `F2/F3/F4` camera presets
- `F1` graphics config
- `Tab` collapse / expand the left rail
- `Space` pause / resume time
- `-` / `=` adjust simulation speed

## Build / Sim Controls

- `R` regenerate city
- `1` bridge
- `2` sensor
- `3` PLC
- `4` gateway
- `5` substation
- `6` pump station
- `Esc` inspect mode
- `B` cycle bridge profile
- `L` cycle PLC logic profile
- `U` spend research toward advanced controls

## Rendering Progress

- Phase 1: world scale + city-builder camera
- Phase 2: terrain / road / river / blockout environment
- Phase 3: asset pipeline scaffolding with `glTF 2.0` catalog metadata in `assets/catalog/asset_catalog.ron`
- Phase 4: material routing, atmosphere, hourly lighting, and surface overlays
- Phase 5: props, vegetation, roadside dressing, and scalable street-life clutter
- Phase 6: district themes, skyline logic, and building massing/facade variation by context

The current starter library is metadata-first. Scene/model files can be dropped into `assets/models/**` later without changing code paths again, while the procedural fallback path still renders the same district logic on weak hardware.
