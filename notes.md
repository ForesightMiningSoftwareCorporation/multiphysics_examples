# Notes

Notes taken while developing this, which may be useful to improve the ecosystem.

- changing y-up to z-up is not too obvious
  - rapier configuration could be better (changing gravity requires a setup system) (insert the whole configuration for its setup?)
- Reflect capability is unclear, suboptimal or missing, making it difficult to make editors quickly.
- no debug position visualization on wheels / car controller ; but `Wheel::raycast_info` is great.
- `TimeStepMode` max deltatime is surprising when introducing differences with Time.delta_secs ; a `Time<Physics>`  may help.
- :warning: rapier hack fix for angular interpolation: https://github.com/Vrixyz/rapier/tree/no_angular_interpolation
- substeps is not great to use with kinematic position based (the movement is done on the first step)
- gizmo toggle is great ; bevy_rapier should thrive to be compatible with this project's `ui_gizmo_toggle`.
  - maybe upstream to bevy a helper for a gizmo toggled off by default ?
- add `IntegrationParameters` to prelude?
- An attempt to use fixed update failed (see branch fixed_update_attempt):
  - using fixed update is way more laggy (this is expected because variable timestep has a concept of max_dt,
    effectively slowing the simulation, that's where [bevy_fixed_update_task](https://crates.io/crates/bevy_fixed_update_task) can help).
  - the model loading to collider doesn't work, probably a bug on this repository, where we should react to the async collider being created, or order after the relevant systems in bevy_rapier (but they are probably in fixed update then).

## TODO

- truck dump
  - make the dump slippery depending on control.
- better logic for rocks to stay in truck
  - activate invisible back wall only if truck is advancing (+ 1 second) + its dump is up.
- check performances on mac -> ~1000 max rocks at 60fps