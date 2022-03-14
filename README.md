# TARDIS

**TARDIS** in an open source astronomy library written in Rust.
It stands for **T**he **A**stronomy **R**ust, **D**efinitely **I**ntergalactic, **S**olution.

Supported: TLE

TODO: OMM

TODO: This file

https://www.faa.gov/about/office_org/headquarters_offices/avs/offices/aam/cami/library/online_libraries/aerospace_medicine/tutorial/media/iii.4.1.4_describing_orbits.pdf

## Viewer
An experimental viewer based on [Bevy](https://bevyengine.org) is available if you include the `viewer` feature. An 
example is provided, showing the 
[Starlink constellation](https://celestrak.com/NORAD/elements/gp.php?GROUP=starlink&FORMAT=tle) in real-time.

### Running the viewer
Bevy is very good at displaying and updating lots of entities, but especially in release mode. Debug mode will
potentially be very laggy, according to the number of satellites.

To run the example, use for instance:
```bash
cargo run --package tardis --example with_bevy_viewer --release
```

### Viewer features
* Live animated (updates up to what TARDIS allows)
* Unreal-like camera control
  * **Best use**: _Hold Right mouse button to orbit the view while using WASD to navigate in the scene, using scroll 
  wheel to accelerate/decelerate_.
  * Left mouse drag: _Locomotion_.
  * Right mouse drag: _Rotate camera_.
  * Left and Right or Middle mouse drag: _Pan camera_.
  * While holding any mouse button, use A/D for panning left/right, Q/E for panning up/down.
  * While holding any mouse button, use W/S for locomotion forward/backward.
  * While holding any mouse button, use scroll wheel to increase/decrease locomotion and panning speeds.
  * While holding no mouse button, use scroll wheel for locomotion forward/backward.
* Quickly rotate the camera towards an axis by clicking this axis on the bottom-left gizmo.
* Toggle from perspective to orthographic projection by clicking the cube on the bottom-left gizmo.
