# multiphysics_examples

## What's in this?

- [ ] A topography mesh (floor). Maybe you make a parry Heightfield, bump some cells to create a vertical wall (something like in the drawing below. Or work something out in blender).
- [ ] Initialize a pile of cubes close to the wall for the rock (red stuff in the second drawing). They can be grid-aligned if you want.
- [ ] A point-cloud (or block-model) to represent pile of rock particles.
- [ ] A model for a bulldozer to push particles into a wall to make the rock pile steeper.
- [ ] A model of a shovel to pick up scoops of rock (for simplicity, once the scoop is picked, the rock particles are just removed and teleported into the truck).
- [ ] Once the truck is full, the user drives it to a muck pile and dumps the material.

Everything must be Z-up.

## Assets

We're expecting you to provide assets in `assets/private/` folder.
Those can't be checked in because of their licences:

- Bulldozer: https://sketchfab.com/3d-models/bulldozer-b06a715d23a7450babac383b8bb7fb0a
- Excavator: undisclosed
- Truck: undisclosed
