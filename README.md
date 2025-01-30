# multiphysics_examples

## Features

- A barebone [`editor`][./crates/editor] to help with level creation.
  - [x] A topography mesh (floor). A parry Heightfield, the  crate is here to help with level editing.
  - [x] Initialize a pile of cubes close to the wall for the rock.
  - [ ] A point-cloud (or block-model) to represent pile of rock particles.

- a [sandbox](crates/sandbox/README.md) to help with understanding how to wire things together.
  - [x] A model for a bulldozer to push particles into a wall to make the rock pile steeper.
  - [ ] A model of a shovel to pick up scoops of rock (for simplicity, once the scoop is picked, the rock particles are just removed and teleported into the truck).
  - [ ] Once the truck is full, the user drives it to a muck pile and dumps the material.
TODO: vehicle module

The project is set up with right-handed Z-up, to the extent possible:
as both parry and bevy sometimes expect Y-Up, comments are here to guide you.

## Assets

You're responsible for providing assets in `assets/private/` folder.
For practicity, These assets are shared for all projects, through `.env`'s `BEVY_ASSET_ROOT`.

Those can't be checked in because of their licences:

- Bulldozer: https://sketchfab.com/3d-models/bulldozer-b06a715d23a7450babac383b8bb7fb0a (glb 1k)
- Excavator: undisclosed
- Truck: undisclosed

## Opinionated decisions

The project is set up with [bevy's recommended optimizations](https://bevyengine.org/learn/quick-start/getting-started/setup/#compile-with-performance-optimizations), multi_threaded feature and hot reloading.

A minimal CI from https://github.com/TheBevyFlock/bevy_new_2d, check out their other release scripts!