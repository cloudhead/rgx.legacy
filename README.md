rgx
===

*A mid-level 2D graphics library for rust*

Introduction
------------
**rgx** is a 2D graphics library built on top of [wgpu] and [Vulkan]. This
library aims to be "mid-level" in that it provides an API that is higher level
than Vulkan, but lower level than most 2D graphics libraries in the wild, by
exposing the user to concepts such as *pipelines*, *buffers* and *swap chains*.
The goal of **rgx** is to provide as simple an API as possible without
sacrificing performance or control over the rendering pipeline.  See the
`examples` directory to get a feel.

At this stage, the focus is on 2D *bitmap* graphics and *sprite rendering*. In the
future, support for 3D pipelines may be added.

[wgpu]: https://crates.io/crates/wgpu
[WebGPU]: https://www.w3.org/community/gpu/
[Vulkan]: https://www.khronos.org/vulkan/

Overview
--------
The library is split into two modules, `kit`, and `core`. The latter provides
**rgx**'s core API with no assumption on what kind of 2D graphics will be
rendered, while the former exposes some useful building blocks for various use-cases,
such as a shape-oriented pipeline and a sprite oriented pipeline. Users can construct
their own pipelines and use them with **rgx**.

### Pipelines included in the `kit`

* **shape2d**: for batched 2D shape rendering
* **sprite2d**: for batched 2D sprite rendering

### Features

* Batched texture rendering
* Batched shape rendering
* Basic primitives for sprite animation
* Off-screen rendering support
* Custom shader support
* Custom pipeline support

Usage
-----
See [examples/helloworld.rs](examples/helloworld.rs) for a simple usage example.

Support
-------
If you find this project useful, consider supporting it by sending ₿ (Bitcoin) to
`1HMfp9QFXmVUarNPmHxa1rhecZXyAPiPZd`. <3

Copyright
---------
(c) 2019 Alexis Sellier\
Licensed under the MIT license.
