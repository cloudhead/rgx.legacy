rgx
===

*A mid-level 2D graphics library for rust*

Introduction
------------
**rgx** is a 2D graphics library library built on top of [wgpu], a [WebGPU]
implementation in Rust. This library aims to be "mid-level" in that it exposes
an API one level above [wgpu], while still allowing the user to work with
concepts such as *pipelines* and *buffers*. The goal of **rgx** is to provide
as simple an API as possible without sacrificing performance or control over
the rendering pipeline.  See the `examples` directory to get a feel.

At this stage, the focus is on 2D *bitmap* graphics and *sprite rendering*. In the
future, support for 3D pipelines may be added.

The library is split into two modules, `kit`, and `core`. The latter provides
**rgx**'s core API with no assumption on what kind of graphics will be
rendered, while the former exposes some useful building blocks for various use-cases.

[wgpu]: https://crates.io/crates/wgpu
[WebGPU]: https://www.w3.org/community/gpu/

Features
--------
* Batched texture rendering
* Batched shape rendering
* Basic primitives for sprite animation
* Off-screen rendering support
* Custom shader support

Pipelines included
------------------
* **shape2d**: for 2D shape rendering
* **sprite2d**: for 2D sprite rendering

Usage
-----
See [examples/helloworld.rs](examples/helloworld.rs) for a simple usage example.

Support
-------
Support this project by sending ₿ (Bitcoin) to `1HMfp9QFXmVUarNPmHxa1rhecZXyAPiPZd`.

Copyright
---------
(c) 2019 Alexis Sellier\
Licensed under the MIT license.
