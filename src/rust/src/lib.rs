mod graphics;
mod vello_device;

use savvy::savvy;

use graphics::DeviceDescriptor;
use graphics::DeviceDriver;
use vello_device::VelloGraphicsDevice;
use vello_device::VelloGraphicsDeviceWithServer;

#[cfg(debug_assertions)]
mod debug_device;

#[cfg(feature = "fastrace")]
mod tracing;

#[macro_export]
macro_rules! add_tracing_point {
    () => {
        add_tracing_point!(fastrace::func_path!())
    };
    ($nm:expr) => {
        #[cfg(feature = "fastrace")]
        {
            let __guard__ = fastrace::local::LocalSpan::enter_with_local_parent($nm);
        }
    };
}

#[savvy]
fn vellogd_impl(filename: &str, width: f64, height: f64) -> savvy::Result<()> {
    let device_driver = VelloGraphicsDevice::new(filename, width, height)?;

    // TODO: the actual width and height is kept on the server's side.
    let device_descriptor = DeviceDescriptor::new(width, height);

    device_driver.create_device::<VelloGraphicsDevice>(device_descriptor, "vellogd")?;

    Ok(())
}

// Currently, this is just for debugging purposes. But, in future, this can be
// used for headless usages.
#[savvy]
fn save_as_png(filename: &str) -> savvy::Result<()> {
    #[cfg(feature = "use_winit")]
    {
        use vellogd_shared::winit_app::VELLO_APP_PROXY;

        VELLO_APP_PROXY
            .tx
            .send_event(vellogd_shared::protocol::Request::SaveAsPng {
                filename: filename.into(),
            })?;
    }

    Ok(())
}

#[savvy]
fn add_lottie_animation(filename: &str) -> savvy::Result<()> {
    #[cfg(feature = "use_winit")]
    {
        use vellogd_shared::winit_app::VELLO_APP_PROXY;

        VELLO_APP_PROXY
            .tx
            .send_event(vellogd_shared::protocol::Request::AddLottieAnimation {
                filename: filename.into(),
            })?;
    }

    Ok(())
}

// #[savvy]
// fn dump_patterns() -> savvy::Result<()> {
//     use vellogd_shared::winit_app::VELLO_APP_PROXY;

//     let patterns = VELLO_APP_PROXY.scene.patterns.lock().unwrap();
//     for (i, ptn) in patterns.iter().enumerate() {
//         match ptn {
//             vellogd_shared::winit_app::FillPattern::Gradient(gradient) => {
//                 savvy::r_eprintln!(
//                     "{i}: gradient
// {gradient:?}"
//                 );
//             }
//             vellogd_shared::winit_app::FillPattern::Tiling(image) => {
//                 let data_len = image.data.data().len();
//                 savvy::r_eprintln!(
//                     "{i}: image
// data_len: {data_len}
// {image:?}",
//                 );
//             }
//         }
//     }

//     Ok(())
// }

#[savvy]
fn vellogd_with_server_impl(
    filename: &str,
    width: f64,
    height: f64,
    server: Option<&str>,
) -> savvy::Result<()> {
    let device_driver = VelloGraphicsDeviceWithServer::new(filename, server, width, height)?;

    // TODO: the actual width and height is kept on the server's side.
    let device_descriptor = DeviceDescriptor::new(width, height);

    device_driver.create_device::<VelloGraphicsDeviceWithServer>(device_descriptor, "vellogd")?;

    Ok(())
}

#[savvy]
fn debuggd() -> savvy::Result<()> {
    #[cfg(debug_assertions)]
    {
        let device_driver = debug_device::DebugGraphicsDevice::new();

        // TODO: the actual width and height is kept on the server's side.
        let device_descriptor = DeviceDescriptor::new(480.0, 480.0);

        device_driver
            .create_device::<debug_device::DebugGraphicsDevice>(device_descriptor, "debug")
            .unwrap();
    }

    Ok(())
}

#[savvy]
fn do_tracing(expr: &str) -> savvy::Result<()> {
    #[cfg(feature = "fastrace")]
    {
        use fastrace::collector::Config;
        use tracing::RConsoleReporter;

        fastrace::set_reporter(RConsoleReporter, Config::default());

        {
            let root = fastrace::Span::root("root", fastrace::prelude::SpanContext::random());
            let _guard = root.set_local_parent();

            savvy::eval::eval_parse_text(expr)?;
        }

        fastrace::flush();
    }
    Ok(())
}
