vellogd: A GPU-powered Interactive Graphics Device for R
========================================================

[![R-CMD-check.yaml](https://github.com/yutannihilation/vellogd-r/actions/workflows/R-CMD-check.yaml/badge.svg)](https://github.com/yutannihilation/vellogd-r/actions/workflows/R-CMD-check.yaml)
[![vellogd status badge](https://yutannihilation.r-universe.dev/badges/vellogd)](https://yutannihilation.r-universe.dev/vellogd)

Vellogd is an experimental graphics device for R. This is powered by vello, a
cross-platform 2D rendering engine with GPU.

[vello]: https://github.com/linebender/vello

# Why yet another graphics device?

You might wonder why do we need vellogd while there are already high-quality,
cross-platform graphics devices like [ragg] and [svglite]. Behind vellogd, I
have two motivations.

[ragg]: https://ragg.r-lib.org/
[svglite]: https://svglite.r-lib.org/

## "Interactive" graphics device with extra features

As far as I know, there's no cross-platform and **interactive** graphics device.
R provides interactive graphics devices, but they are not cross-platform. They
are implemented differently, which makes some features missing on some platform
(e.g. [X11 on Windows doesn't provide `onIdle` event][coolbutuseless]).

[coolbutuseless]: https://coolbutuseless.github.io/2022/05/06/introducing-eventloop-realtime-interactive-rendering-in-r/

What's more exciting about crafting a new graphics device is, we might be able
to implement extra features beyond R's graphics API. What do you want an
interactive graphics device to be? Here's my wishlist, for example.

* Zoom in and out by mousewheel
* Support touch device
* Provide an API for [Lottie animation](https://airbnb.io/lottie/)
* Provide an API to draw a bezier curve (why R provides no API...?)

## A showcase of the power of WebGPU

One more thing I feel a bit frustrating about R's graphics ecosystem is, it's
too CPU-centric. I'm not an expert of GPU, but I think R can utilize the power
of GPU a lot more.

I understand it's difficult to use GPU in an R package because different
platforms and different vendors require different implementation. However, now
we have [**the WebGPU standard**](https://gpuweb.github.io/gpuweb/) and the
cross-platform implemntations of the API in C++ ([dawn], used by Chrome) and
Rust ([wgpu], used by Firefox and Deno).

[dawn]: https://dawn.googlesource.com/dawn/+/refs/heads/chromium-gpu-experimental/README.md
[wgpu]: https://wgpu.rs/

Unlike the name indicates, WebGPU is not only for web. It's a GPU-powered
graphics API, which is so **portable and safe** that it can be used on web
browsers. So, if you want to create a cross-platform R package using GPU, WebGPU
can be the best choice. I hope vellogd servers as a showcase of such an attempt.

(Personally, I'm hoping rayshader will use WebGPU!)

# Installation

> [!CAUTION]
> vellogd is at the verrry early stage of the development. This might crash not only your R sesson, but also your GPU. Please try at your own risk!

The vellogd package can be installed via R-universe.

```r
install.packages("vellogd", repos = c("https://yutannihilation.r-universe.dev", "https://cloud.r-project.org"))
```

## Usages

Vellogd provides two functions to open the graphics device. You can use
`vellogd()` if you are on Windows or on Linux, otherwise (i.e. on macOS) use
`vellogd_with_server()`.

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

# Supported R Graphics Device API

cf. <https://github.com/r-devel/r-svn/blob/main/src/include/R_ext/GraphicsDevice.h>


| API               | supported? | Note |
|:------------------|:---|:-----------|
| `activate`        | ✅ |  |
| `deactivate`      | ✅ |  |
| `close`           | ✅ |  |
| `newPage`         | ✅ |  |
| `size`            | ✅ |  |
| `mode`            | ✅ | TODO: server version |
| `newFrameConfirm` | ✅ | Do nothing |
| `holdflush`       |    | |
| `locator`         |    | |
| `onExit`          |    | |
| `line`            | ✅ | Draw [`kurbo::Line`] |
| `circle`          | ✅ | Draw [`kurbo::Circle`] |
| `rect`            | ✅ | Draw [`kurbo::Rect`] |
| `polygon`         | ✅ | Draw [`kurbo::BezPath`]. |
| `path`            | ✅ | Draw [`kurbo::BezPath`]. |
| `polyline`        | ✅ | Draw [`kurbo::BezPath`]. |
| `raster`          | ✅ | TODO: server version, non-interpolated version |
| `metricInfo`      | ✅ | |
| `strWidth`        | ✅ | |
| `text`            | ✅ | |
| `textUTF8`        | ✅ | |
| `glyph`           | ✅ | TODO: server version |
| `clip`            | ✅ | TODO: server version, can I hide the clipping rectangle? |
| `cap`             |    | |
| `eventHelper`     |    | |
| `setPattern`      | ✅  | TODO: tiling, fix #24 |
| `releasePattern`  | ✅  | TODO: tiling, fix #24 |
| `setClipPath`     |    | |
| `releaseClipPath` |    | |
| `setMask`         |    | |
| `releaseMask`     |    | |
| `defineGroup`     |    | |
| `useGroup`        |    | |
| `releaseGroup`    |    | |
| `stroke`          |    | |
| `fill`            |    | |
| `fillStroke`      |    | |
| `capabilities`    | ✅ | |

[`kurbo::Line`]: https://docs.rs/kurbo/latest/kurbo/struct.Line.html
[`kurbo::Circle`]: https://docs.rs/kurbo/latest/kurbo/struct.Circle.html
[`kurbo::Rect`]: https://docs.rs/kurbo/latest/kurbo/struct.Rect.html
[`kurbo::BezPath`]: https://docs.rs/kurbo/latest/kurbo/struct.BezPath.html

# Special note

The code related to R Graphics Device API is based on [extendr's code][extendr].
While most part of it is done by me when I was a member of extendr org, vello
would not exist if there were no extendr.

[extendr]: https://github.com/extendr/extendr/tree/master/extendr-api/src/graphics
