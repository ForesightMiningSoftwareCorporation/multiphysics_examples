# multiphysics_examples

## Features

- A barebone [`editor_map`][./crates/editor_map] to help with level creation.
  - [x] A topography mesh (floor). A parry Heightfield, the  crate is here to help with level editing.
  - [x] Initialize a pile of cubes close to the wall for the rock.
  - [x] A point-cloud (or block-model) to represent pile of rock particles.
- A barebone [`editor_vehicle`][./crates/editor_map] to help with vehicle customization.
  - [x] Loads a .ron file to tweak control settings.
- a [sandbox](crates/sandbox/README.md) to help with understanding how to wire things together.
  - [x] A model for a bulldozer to push particles into a wall to make the rock pile steeper.
  - [x] A model of a shovel to pick up scoops of rock (for simplicity, once the scoop is picked, the rock particles are just removed and teleported into the truck).
    - [x] Excavator arm initialized with a model. See [accessory_controls](crates/shared_vehicle/src/accessory_controls).
  - [x] Once the truck is full, the user drives it to a muck pile and dumps the material.
  - [x] ui
    - [x] switch between vehicles
    - [ ] see how many rocks are in an area (muck piles ; truck ; excavator)
      - ~~See `muck_pile.rs` and `stats_rocks.rs`~~ ; <- This is a CPU only implementation.
      - You should export particles positions from GPU if you need that feature. 

The project is set up with right-handed Z-up, to the extent possible:
as both parry and bevy sometimes expect Y-Up, comments are here to guide you.
Models are also Y-up, but children in the hierarchy and rotated locally.

## Assets

You're responsible for providing assets in `assets/private/` folder.

![private folder organization](/docs/private_folder.png)

For practicity, These assets are shared for all projects, through your `.env`'s `BEVY_ASSET_ROOT`: be sure to adapt `.env.example`.

Those can't be checked in because of their licences:

- Bulldozer: https://sketchfab.com/3d-models/bulldozer-b06a715d23a7450babac383b8bb7fb0a (glb 1k)
- Excavator: https://www.artstation.com/marketplace/p/A7W15/hydraulic-mining-shovel
  - [Bevy doesn't support more than 256 bones](https://github.com/bevyengine/bevy/issues/10522)
  so you'll need to remove some (flavor chains are good candidates.)
  - Re-export it to gltf without animations (it reduces its size).
- Truck: undisclosed
  - provided model didn't have a great origin for the bucket to rotate it without mesh interlacing so I modified it.

## Opinionated decisions

The project is set up with [bevy's recommended optimizations](https://bevyengine.org/learn/quick-start/getting-started/setup/#compile-with-performance-optimizations), multi_threaded feature and hot reloading.

A minimal CI from <https://github.com/TheBevyFlock/bevy_new_2d>, check out their other release scripts!

## How to navigate this project ?

[Sandbox](#Sandbox) is the main final project. Run it with `cargo run --release -p sandbox`.

Let's go through important modules:

### Map

[shared_map](crates/shared_map) is dedicated to loading a map. You'll probably want to change `MapDef` to adapt to your requirements.

It supports hot reloading, and you can isolate its behaviour by using [editor_map](crates/editor_map).

### sim_data_loader

To load more complex maps, you can use this module, which transforms block models into a digestible format for the `shared_map` module.

For example, you can invoke a data "migration" with:

```sh
cargo run --bin sim_to_mapdef assets/private/Sim\ data/Unbroken\ rock.csv  assets/private/Sim\ data/Broken\ rock.csv assets/private/Sim\ data/transformed/imported_cubes.mapdef.ron`
```

There is also a `visualize` example to help with understanding data loading in isolation.

Because of performance issues, you may want to not load all rocks, and group them somehow, check out [this rock spawn logic](https://github.com/ForesightMiningSoftwareCorporation/multiphysics_examples/blob/67023c3023c571da4206404c57376bf9993d4050/crates/shared_map/src/map_def.rs#L202-L211) for example.

### Vehicles

#### Shared

[shared_vehicle](crates/shared_vehicle) is dedicated to loading and using a vehicle.

[shared_vehicle::vehicle_spawner](crates/shared_vehicle/vehicle_spawner) is dedicated to loading the vehicle and setting up their minimal requirements.

[shared_vehicle::rapier_vehicle_controller](crates/shared_vehicle/rapier_vehicle_controller)
is a thin wrapper around [rapier's raycast vehicle controller](https://github.com/dimforge/rapier/blob/master/examples3d/vehicle_controller3.rs).

[shared_vehicle::accessory_controls](crates/shared_vehicle/accessory_controls) is for real-time control of excavator arm or truck dump. "Control" is isolated from the input, so it's easier to create bots or automated behaviour if needed.

The vehicle accessories work by finding the "control" bone and the mesh of the accessory you want to control, then:

- Compute its collider mesh
- Unparent it, but make it follow its "control" bone through KinematicPositionBased
- the accessory controls update the "control" bone rotation.

#### Editor

[editor_vehicle](crates/editor_vehicle) only loads a vehicle, it can be helpful to verify a model loads correctly and inspect it with bevy_editor_egui.

### bevy_wgsparkl

Integrates <https://github.com/dimforge/wgsparkl> into bevy.

Known limitations:

- Doesn't support scene queries (you'd have to reach for gpu data)
- Only 1 upload supported: dynamic shape addition is not yet supported.
- debug mode not supported: launch the app with `--release`

### Sandbox

The sandbox uses the "shared" modules to create a scenario: load a map, load vehicles.

Additionally, more gameplay features are added:

- input is integrated to loaded vehicles
- a loading screen

## Collision groups

- `Group_2` is used for ground.
- `Group_3` is used for excavator parts
- `Group_4` is used for truck parts

### Limitations

Check out the `TODO`, `FIXME` and `HACK` for limitations, hacks or surprising behaviours.
