# TD-104 Boundary Correction V2

Date: `2026-08-30`

V1 implementation preflight:
`391955442b0eaa72a43701888bd6ca43e9518942acb7373b49b0447575bcdd33`

## Correction

Remove `src/nanda_wave/context_phase/surface_field.rs` from the closed cold
set. The live `context_phase::online` route stores and reads
`SurfaceMutationField`; feature-gating that module would change the default
runtime type graph.

The effective V2 candidate keeps `surface_field` and its re-export
unconditional. All other V1 route, measurement, release, proof, and
no-deployment constraints remain unchanged.

V1 is retained as a superseded design receipt. No source implementation had
started when this dependency was found.
