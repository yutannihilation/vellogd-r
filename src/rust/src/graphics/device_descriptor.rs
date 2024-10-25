#![allow(unused_variables)]
#![allow(dead_code)]

// From R internals[^1]:
//
// > There should be a ‘pointsize’ argument which defaults to 12, and it should
// > give the pointsize in big points (1/72 inch). How exactly this is
// > interpreted is font-specific, but it should use a font which works with
// > lines packed 1/6 inch apart, and looks good with lines 1/5 inch apart (that
// > is with 2pt leading).
//
// [^1]: https://cran.r-project.org/doc/manuals/r-release/R-ints.html#Conventions
const POINTSIZE: f64 = 12.0;

const PT: f64 = 1.0 / 72.0;
const PT_PER_INCH: f64 = 72.0;

// From R internals[^1]:
//
// > where ‘fnsize’ is the ‘size’ of the standard font (cex=1) on the device, in
// > device units.
//
// and it seems the Postscript device chooses `pointsize` as this.
//
// [^1]: https://cran.r-project.org/doc/manuals/r-release/R-ints.html#Handling-text
const FONTSIZE: f64 = POINTSIZE;

// From R internals[^1]:
//
// > The default size of a device should be 7 inches square.
//
// [^1]: https://cran.r-project.org/doc/manuals/r-release/R-ints.html#Conventions
const WIDTH_INCH: f64 = 7.0;
const HEIGH_INCH: f64 = 7.0;

/// A builder of [libR_sys::DevDesc].
///
// # Design notes (which feels a bit too internal to be exposed as an official document)
//
// Compared to the original [DevDesc], `DeviceDescriptor` omits several fields
// that seem not very useful. For example,
//
// - `clipLeft`, `clipRight`, `clipBottom`, and `clipTop`: In most of the cases,
//   this should match the device size at first.
// - `xCharOffset`, `yCharOffset`, and `yLineBias`: Because I get [the
//   hatred](https://github.com/wch/r-source/blob/9f284035b7e503aebe4a804579e9e80a541311bb/src/include/R_ext/GraphicsDevice.h#L101-L103).
//   They are rarely used.
// - `gamma`, and `canChangeGamma`: These fields are now ignored because gamma
//   support has been removed.
// - `deviceSpecific`: This can be provided later when we actually create a
//   [Device].
// - `canGenMouseDown`, `canGenMouseMove`, `canGenMouseUp`, `canGenKeybd`, and
//   `canGenIdle`: These fields are currently not used by R and preserved only
//   for backward-compatibility.
// - `gettingEvent`, `getEvent`: This is set true when getGraphicsEvent is
//   actively looking for events. Reading the description on ["6.1.6 Graphics
//   events" of R
//   Internals](https://cran.r-project.org/doc/manuals/r-release/R-ints.html#Graphics-events),
//   it seems this flag is not what is controlled by a graphic device.
// - `canHAdj`: it seems this parameter is used only for tweaking the `hadj`
//   before passing it to the `text()` function. This tweak probably can be done
//   inside `text()` easily, so let's pretend to be able to handle any
//   adjustments... c.f.
//   <https://github.com/wch/r-source/blob/9f284035b7e503aebe4a804579e9e80a541311bb/src/main/engine.c#L1995-L2000>
#[allow(non_snake_case)]
pub struct DeviceDescriptor {
    pub(crate) left: f64,
    pub(crate) right: f64,
    pub(crate) bottom: f64,
    pub(crate) top: f64,

    // Note: the header file questions about `ipr` and `cra` [1]. Actually,
    // svglite and ragg have `pointsize` and `scaling` parameters instead. But,
    // I couldn't be sure if it's enough as an framework (I mean, as a package,
    // abstracting these parameters to `pointsize` and `scaling` is definitely a
    // good idea), so I chose to expose these parameters as they are.
    //
    // [1]:
    //     https://github.com/wch/r-source/blob/9f284035b7e503aebe4a804579e9e80a541311bb/src/include/R_ext/GraphicsDevice.h#L75-L81
    pub(crate) ipr: [f64; 2],
    pub(crate) cra: [f64; 2],

    pub(crate) startps: f64,
    pub(crate) startcol: u32,
    pub(crate) startfill: u32,
    pub(crate) startlty: i32,
    pub(crate) startfont: i32,
}

impl DeviceDescriptor {
    /// Create a new DeviceDescriptor with the specified sizes (unit: point).
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            // The From R internals [1] " The default size of a device should be 7
            // inches square."
            left: 0.0,
            right: width,
            bottom: 0.0,
            top: height,

            ipr: [PT, PT],

            // Font size. Not sure why these 0.9 and 1.2 are chosen, but R
            // internals says this is "a good choice."
            cra: [0.9 * FONTSIZE, 1.2 * FONTSIZE],

            startps: POINTSIZE,
            startcol: 0xff000000,
            startfill: 0xffffffff,
            startlty: vellogd_shared::ffi::LTY_SOLID, // Solid
            startfont: 1,                             // Plain
        }
    }
}
