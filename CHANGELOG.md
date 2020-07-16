# v0.8.0

commit 3ebcf1c3508d36e3953ba91e492e7a99c7128a7a

    Implement `From<u32>` for `Bgra8`

commit eac299b1a870be23e6c38f38b2315494dec39232

    Make `Rgba8::align` & `Bgra8::align` more generic

commit 508ca7992f77280028833c4981dc430dbcc1f420

    Remove unused `Fill::Gradient` variant

commit 096a514537be19c16404ba46b9a72d5b01711ceb

    Improve `Fill` and color parameters

commit cd87d3f33441050562ed198413a0f3a85818b226

    Add `Shape::rect` function

commit 1c618c92eb72c7fe9907a03adba98eb9491bfe08

    Add `Shape::line` function

commit df8d962985d8846e49f7f79bbeb78d5d4123f914

    Add new ways to construct certain types

commit b4ca46c1b7cb7403a85d6cf069f9b8166e1d1b6d

    Improve `shape2d::Batch` API

commit fc1c05b9bbc1fe5565ae8791dda62dd6bc4d20d5

    Animations shouldn't skip frames

commit 10d33fbf3590a85d0c8089cdb3f9ddd409a5aa46

    Improve `sprite2d::Batch` interface

    * Add `Batch::push` method that takes a `Sprite`

commit e75e477ab80ea578e73b5c5c69a863150064f1af

    Add `Default` instance for `Rect`

commit 831a21bbc6ee815417845ff1887c880737bd2de7

    Add `Default` instances to color types

# v0.7.1

commit e4d86794e3020ddf515365dfd311f37d6c7b935c

    Update dependencies

    * Remove env_logger
    * Update winit

# v0.7.0

commit 19ff9088303e8325ab8247a6e07d26500ad688d4

    Separate wgpu backend from frontend

    Adds the `renderer` feature to turn the wgpu backend on.

commit 25a22a325e326fc79cba25dbf90b96375defbbcb

    Implement some conversions for Matrix4/Vector4

# v0.6.0

commit 1f5e52bf2751be680fef6949604dfa5563cbb84c

    Add `Origin` parameter to `kit::ortho`

# v0.5.0

commit c3de863cb8e6e8d925be5418f85adb3d1009ae6b

    Implement `TransformBuffer`

    This replaces the old `Model`, and implements a transform
    buffer for use with dynamic uniform buffers.

commit 4a585ec89cd5a2a12f7f13d0f762ce3ef5b883e3

    Implement `Geometry` trait

    * `Geometry` trait for things that can be transformed
    * Change `Line` to use `Point2`
    * Implement a few extra methods in `math` module

    This enables transforming `Line` and `Rect` with a `Matrix4`.

# v0.4.1

commit d3021ba684d4fc7f131c82645cc8416212dbbc1c

    Use `Bgra8` for pixel data in `Renderer::read`

# v0.4.0

commit 14947b71417973e620d9ba027b96c174b91e19ec

    Derive `Debug` on core types

commit f152c18b831d827393838834e510d2257a8ce363

    Clear depth buffer on `Op::Clear`

commit eddbfef19bc372001227bb901a915c14146abbcc

    Use a linear color pipeline

    This change moves our pipelines to the sRGB color space, properly
    accounting for gamma correction. This means colors passed in
    need to be manually linearized. For textures, this happens automatically
    by specifying the correct format.

commit 95e6ff64791da4d4fba547c9fa97a220a27929ef

    Rename `TextureView` trait to `RenderTarget`

commit 8eb1eb0b3fccdb081a48739f5c73430b8291f078

    Add z-depth rendering functionality

    This implements depth testing and writing for all pipelines, along with
    an additional `ZDepth` parameter for shapes and sprites.

    Depth testing can significantly improve performance by minimizing
    overdraw.

commit 7edd02cd757228ff7db635130aa36cfcd7c31e00

    Make `Renderer::new` return a `Result`

    In preparation for wgpu not panicking when it doesn't
    find a suitable adapter...

commit 95d5a433a16922f5a3dc651da5b16561cb7e7b26

    Move `Rect` type to its own module
