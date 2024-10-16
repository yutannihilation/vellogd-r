vellogd: A Vello Graphics Device for R
======================================

[![R-CMD-check.yaml](https://github.com/yutannihilation/vellogd-r/actions/workflows/R-CMD-check.yaml/badge.svg)](https://github.com/yutannihilation/vellogd-r/actions/workflows/R-CMD-check.yaml)

This is a prototype for Rust-implemented R graphics device. This relies on these crates:

* vello: 2D rendering engine with GPU
* parley: text-layout
* winit: window management

# R Graphics Device API

The code related to R Graphics Device API is based on [extendr's code][extendr].
Some part of it is my code when I was a member of extendr org.

[extendr]: https://github.com/extendr/extendr/tree/master/extendr-api/src/graphics

cf. <https://github.com/r-devel/r-svn/blob/main/src/include/R_ext/GraphicsDevice.h>

* `activate`: Do nothing
* `circle`: Draw [`kurbo::Circle`](https://docs.rs/kurbo/latest/kurbo/struct.Circle.html)
* `clip`: [`vello::Scene::push_layer()`](https://docs.rs/vello/latest/vello/struct.Scene.html#method.push_layer) seems to handle this.
* `close`: `event_loop.exit()`
* `deactivate`: Do nothing
* `locator`: TBD
* `line`: Draw [`kurbo::Line`](https://docs.rs/kurbo/latest/kurbo/struct.Line.html)
* `metricInfo`: Use `vello::skrifa`. [the official example](https://github.com/linebender/vello/blob/7647a14838a9bfe86c6f93abe62c8a7c2e6a7115/examples/scenes/src/simple_text.rs#L8)
* `mode`: Do nothing.
* `newPage`: Currently, just use `scene.reset()`. For non-interactive usage, this needs to handle filenames.
* `polygon`: Draw `kurbo::BezPath`.
* `polyline`: Draw `kurbo::BezPath`.
* `rect`:
* `path`:
* `raster`:
* `cap`:
* `size`: Do nothing.
* `strWidth`:
* `text`:
* `onExit`:
* `newFrameConfirm`:
* `textUTF8`:
* `eventHelper`:
* `holdflush`:
* `setPattern`:
* `releasePattern`:
* `setClipPath`:
* `releaseClipPath`:
* `setMask`:
* `releaseMask`:
* `defineGroup`:
* `useGroup`:
* `releaseGroup`:
* `stroke`:
* `fill`:
* `fillStroke`:
* `capabilities`:
* `glyph`:
