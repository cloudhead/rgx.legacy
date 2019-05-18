rgx
===

*A mid-level graphics library for rust*

Introduction
------------
**rgx** is a Rust library built on top of [wgpu], a [WebGPU] implementation in Rust. This library
aims to be "mid-level" in that exposes an API one level above [wgpu], while still allowing
the user to work with concepts such as *pipelines* and *buffers*. See the `examples` directory
to get a feel.

At this stage, the focus is on *bitmap* graphics and *sprite rendering*. In the future, support
for 3D pipelines may be added.

The library is split into two modules, `kit`, and `core`. The latter provides **rgx**'s core
API with no assumption on what kind of graphics will be rendered, while the former exposes some
useful building blocks for various use-cases.

[wgpu]: https://crates.io/crates/wgpu
[WebGPU]: https://www.w3.org/community/gpu/

Usage
-----
See [examples/helloworld.rs](examples/helloworld.rs) for a simple usage example.

Copyright
---------
(c) 2019 Alexis Sellier\
Licensed under the MIT license.
