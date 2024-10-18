vellogd: A Vello Graphics Device for R
======================================

[![R-CMD-check.yaml](https://github.com/yutannihilation/vellogd-r/actions/workflows/R-CMD-check.yaml/badge.svg)](https://github.com/yutannihilation/vellogd-r/actions/workflows/R-CMD-check.yaml)
[![vellogd status badge](https://yutannihilation.r-universe.dev/badges/vellogd)](https://yutannihilation.r-universe.dev/vellogd)

vellogd is an experimental graphics device for R. It relies on these crates:

* [vello]: Cross-platform 2D rendering engine with GPU
* [parley]: Rich text-layout
* [winit]: Cross-platform window management

[vello]: https://github.com/linebender/vello
[parley]: https://github.com/linebender/parley
[winit]: https://docs.rs/winit/latest/winit/

# Installation

> [!CAUTION]
> vellogd is at the verrry early stage of the development. This might crash not only your R sesson, but also your GPU. Please try at your own risk!

vellogd can be installed via R-universe.

```r
install.packages("vellogd", repos = c("https://yutannihilation.r-universe.dev", "https://cloud.r-project.org"))
```

## Usages

vellogd provides two functions to open the graphics device.

## `vellogd()` (Windows, Linux)

If you are on macOS, this isn't available to you! If you are curious about the reason, my write-up might help: [How To Use Winit With R (Or How To Run Winit On A Non-Main Thread)](https://yutani.rbind.io/post/winit-and-r/).

If you are on Windows or on Linux, this method should preferable. As this runs a window on the same process as the R session, less data copy is needed.

```r
# Open a device
vellogd()

library(ggplot2)

ggplot(mpg, aes(displ, hwy, colour = class)) + 
  geom_point() +
  theme(text = element_text(size = 25))

dev.off()
```

## `vellogd_with_server()` (macOS, Windows, Linux)

This is available to all macOS, Windows and Linux.
This launches a server as a separate process, so drawing heavy data (e.g. raster) might take longer time.

```r
# Open a device
vellogd_with_server()

library(ggplot2)

ggplot(mpg, aes(displ, hwy, colour = class)) + 
  geom_point() +
  theme(text = element_text(size = 25))

dev.off()
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
