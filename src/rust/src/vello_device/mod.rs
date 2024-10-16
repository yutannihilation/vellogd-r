#[cfg(not(target_os = "macos"))]
mod default;
#[cfg(not(target_os = "macos"))]
pub use default::VelloGraphicsDevice;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::VelloGraphicsDevice;

mod with_server;
pub use with_server::VelloGraphicsDeviceWithServer;
