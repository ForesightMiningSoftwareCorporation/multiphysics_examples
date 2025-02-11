# Notes

Notes taken while developing this, which may be useful to improve the ecosystem.

- changing y-up to z-up is not too obvious
  - rapier configuration could be better (changing gravity requires a setup system) (insert the whole configuration for its setup?)
- Reflect capability is really missing for making editors quickly.
- bad excavator model without animations or rig.
- no debug position visualization on wheels / car controller.
- TimeStepMode max deltatime is surprising when introducing differences with Time.delta_secs ; a Physics Time may help.
- substeps is not great to use with kinematic position based (the movement is done on the first step)