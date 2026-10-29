# Synths

**Currently being reorganized from logic in other repos - working toward a structure something like:**

- `/dsp-core`: `no_std` Rust DSP logic for writing Eurorack modules and other synthesizers
- `/modules`: firmware for various Eurorack modules that use the above
- `/vcv-rack`: roughly parallel to `/modules`, bringing that logic to [VCVRack](https://vcvrack.com) (mostly used for development)
  - `/dsp`: bindings for Rust types/methods/functions so they're available in C++
  - `/res`: svgs for module layouts
  - `/src`: VCVRack module setup (this is mostly setting up the layout and I/O, calling the Rust bindings for DSP logic)

**More to come...**
