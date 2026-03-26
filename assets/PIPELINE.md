# Asset Pipeline

Phase 3 standardizes the art ingest path around `glTF 2.0` plus catalog metadata in `assets/catalog/asset_catalog.ron`.

## Directory Layout

- `assets/models/buildings/`
- `assets/models/roads/`
- `assets/models/props/`
- `assets/materials/`
- `assets/textures/`
- `assets/decals/`

## Naming Conventions

- Scene files: `<family>_<variant>_<set>.glb`
- Scene roots: `#Scene0` for the primary import scene
- LOD meshes inside a file: `lod0`, `lod1`, `lod2`
- Collision proxies: suffix `_col`
- Low-quality fallback-only meshes: suffix `_lq`
- Material variants: `surface_variant_finish`
- Zone tags in metadata: `Residential`, `MixedUse`, `Industrial`, `Utility`, `Civic`, `Streetscape`, `Nature`

## Metadata Contract

Each catalog entry describes:

- footprint
- height class
- zone class
- prop anchors
- LOD distances
- low-quality fallback mesh/material
- optional scene path

The current starter library is metadata-first. Entries without a `scene` field intentionally fall back to the procedural blockout renderer until authored assets are dropped into the matching folders.
