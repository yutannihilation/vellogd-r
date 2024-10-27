use std::{num::NonZeroUsize, sync::atomic::Ordering};

use crate::protocol::AppResponseRelay;

use super::VelloApp;
use peniko::Color;
use vello::{
    wgpu::{
        Buffer, Device, Extent3d, ImageCopyBuffer, ImageDataLayout, Texture, TextureDescriptor,
        TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor,
    },
    RenderParams, RendererOptions, Scene,
};

pub fn create_texture(device: &Device, size: Extent3d) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some("Target texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

pub fn create_buffer(device: &Device, width: u32, height: u32) -> (Buffer, u32) {
    let padded_byte_width = (width * 4).next_multiple_of(256);
    let buffer_size = padded_byte_width as u64 * height as u64;

    let buffer = device.create_buffer(&vello::wgpu::BufferDescriptor {
        label: Some("val"),
        size: buffer_size,
        usage: vello::wgpu::BufferUsages::MAP_READ | vello::wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    (buffer, padded_byte_width)
}

impl<'a, T: AppResponseRelay> VelloApp<'a, T> {
    // This implementation is is based on
    // https://github.com/linebender/vello/blob/main/examples/headless/src/main.rs
    pub fn rasterize(
        &mut self,
        scene: &Scene,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, vello::Error> {
        let dev_id = pollster::block_on(async { self.context.device(None).await }).unwrap();
        let device_handle = &self.context.devices[dev_id];

        // TODO: move to app's field
        let mut renderer = vello::Renderer::new(
            &device_handle.device,
            RendererOptions {
                surface_format: None,
                use_cpu: false,
                antialiasing_support: vello::AaSupport::area_only(),
                num_init_threads: NonZeroUsize::new(1),
            },
        )
        .unwrap();

        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        // Note: the texture for surface is not reusable because the texture
        // usage doesn't contain COPY_SRC
        let texture = create_texture(&device_handle.device, size);
        let view = texture.create_view(&TextureViewDescriptor::default());

        let base_color = {
            let [r, g, b, a] = self.base_color.load(Ordering::Relaxed).to_ne_bytes();
            Color::rgba8(r, g, b, a)
        };

        renderer.render_to_texture(
            &device_handle.device,
            &device_handle.queue,
            scene,
            &view,
            &RenderParams {
                base_color,
                width,
                height,
                antialiasing_method: vello::AaConfig::Area,
            },
        )?;

        let (buffer, padded_byte_width) = create_buffer(&device_handle.device, width, height);
        let mut encoder =
            device_handle
                .device
                .create_command_encoder(&vello::wgpu::CommandEncoderDescriptor {
                    label: Some("Copy out buffer"),
                });

        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            ImageCopyBuffer {
                buffer: &buffer,
                layout: ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_byte_width),
                    rows_per_image: None,
                },
            },
            size,
        );
        device_handle.queue.submit([encoder.finish()]);

        let buf_slice = buffer.slice(..);

        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buf_slice.map_async(vello::wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
        if let Some(recv_result) =
            vello::util::block_on_wgpu(&device_handle.device, receiver.receive())
        {
            // TODO: handle error
            recv_result.unwrap();
        }

        let data = buf_slice.get_mapped_range();
        let mut result_unpadded =
            Vec::<u8>::with_capacity((width * height * 4).try_into().unwrap());
        for row in 0..height {
            let start = (row * padded_byte_width).try_into().unwrap();
            result_unpadded.extend(&data[start..start + (width * 4) as usize]);
        }

        Ok(result_unpadded)
    }

    pub fn save_as_png(
        &mut self,
        filename: String,
        width: u32,
        height: u32,
    ) -> Result<(), vello::Error> {
        // TODO: in theory, this doesn't need clone(). However, if I put the
        // scene directly to self.rasterize(), the borrow checker gives the
        // following error:
        //
        //     error[E0502]: cannot borrow `*self` as mutable because it is also borrowed as immutable
        //
        // Assuming writing to PNG is not called so frequently, I think this
        // won't affect the performance much. Also, probably it's a good idea
        // not to block the main rendering to screen by this.
        //
        // cf.
        // https://smallcultfollowing.com/babysteps/blog/2018/11/01/after-nll-interprocedural-conflicts/
        let scene = self.scene.scene().clone();

        let result_unpadded = self.rasterize(&scene, width, height)?;

        let mut file = std::fs::File::create(&filename).unwrap();
        let mut encoder = png::Encoder::new(&mut file, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&result_unpadded).unwrap();
        writer.finish().unwrap();

        Ok(())
    }
}
