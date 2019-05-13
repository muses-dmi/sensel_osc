# Sensel OSC Driver for SVG Interfaces

SVG interfaces (```.mi``` files) are a simple approach to specifiying control
interfaces for Digitial Musical Instruments or DMIs. SVG Interfaces generate OSC 
control messages and as such as indepdendent of any particular audio engine and were
designed within the context of the [Muses](https://muses-dmi.github.io/) project.

#  <span style="color:#F3B73B">Dependencies</span> 

The application is written in [Rust](https://www.rust-lang.org/) and tested with
1.34.1 (and nightly). To install Rust go you need simply to install
[Rustup](https://rustup.rs/) and if you already have Rust installed, then you can update
with the command ```rustup update```.

#  <span style="color:#F3B73B">Building</span>

To build run the command:

```
cargo build --release
```

#  <span style="color:#F3B73B">Using it</span>

To start the driver simple pass it as valid SVG JSON IR file:

```
cargo run --release -- <filename.json>
```

If no Sensel Morph is attached the program will say as much an exit.

#  <span style="color:#F3B73B">More Information</span>

Parent project

   - [Muses](https://muses-dmi.github.io/).

Tool and documentation for specification of interfaces as SVGs:

   - [SVG Creator tool](https://github.com/muses-dmi/svg-creator).
   - [SVG Interface Documentation](https://github.com/muses-dmi/svg-creator/blob/master/docs/interfaces.md).

Tools for translating SVG Interfaces to the JSON intermidiate representation and different backends:

   - [SVG Interface to IR tool](https://github.com/muses-dmi/svg_interface). (This repo.)
   - [Interface IR to Littlefoot tool](https://github.com/muses-dmi/svg-littlefoot).
   - [SVG Sensel Driver](https://github.com/muses-dmi/sensel_osc).


#  <span style="color:#F3B73B">License</span>

The source in this repo is licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
 * [Mozilla Public License 2.0](https://www.mozilla.org/en-US/MPL/2.0/)

at your option.

Dual MIT/Apache2 is strictly more permissive