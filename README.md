vellogd: A Vello Graphics Device for R
======================================

This is a prototype for Rust-implemented R graphics device. This relies on these crates:

* vello: for drawing shapes
* winit: for managing a window
* tonic: for communicationg the server and the client with gRPC

Why is tonic needed here? This is because

* winit requires to be executed on the main thread.
* So, if I want to create an interactive device, which runs in background, it
  needs to be a seprated process.
* Since the device runs on a separated process, R needs some measure to
  communicate with the server. tonic enables this.

Note that protocol buffer is probably not the most efficient format for this purpose.

# Control flow

As described above, the main thread is used by winit (`EventLoop`). tonic also
needs to be run on the main thread (with `#[tokio::main]`), so it's spawned in
the main thread before the winit blocks.

`EventLoop` itself is `!Send` and `!Sync` while tonic requires `Send` and `Sync`
to make the resource accessible via the server. winit provides `EventLoopProxy`
for such cases. A proxy is `Send` and `Sync` and allows to send user-defined
events. The emitted event will be handled in
`ApplicationHandler<T>::user_event()`.

```
┌─────────┐             
│ device  │             
└────▲────┘             
     │                  
     │ winit & vello API
┌────┴────┐             
│eventloop│             
└───▲─────┘             
    │                   
    │ proxy             
┌───┴─────┐             
│ server  │             
└────▲────┘             
     │                  
     │ gRPC             
┌────┴────┐             
│ client  │             
└─────────┘             
```

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
