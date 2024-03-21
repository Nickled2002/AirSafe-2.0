use image::GenericImageView;
use image::io::Reader as ImageReader;
use anyhow::*;

pub struct ITexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl ITexture {   
    pub fn create_texture_data(
        device:&wgpu::Device, 
        queue: &wgpu::Queue, 
        img_file: &str, 
        u_mode:wgpu::AddressMode, 
        v_mode:wgpu::AddressMode
    ) -> Result<Self> {

        let img = ImageReader::open(img_file)?.decode()?;
        let rgba = img.as_rgba8().unwrap();
        let dimensions = img.dimensions();
        
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label: Some("Image Texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }
        );

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4*dimensions.0),
                rows_per_image:Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: u_mode,
                address_mode_v: v_mode,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        );
        
        Ok(Self { texture, view, sampler })
    }


}