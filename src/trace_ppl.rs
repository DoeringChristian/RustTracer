use std::{borrow::Cow, collections::HashMap};

use ewgpu::*;
use crate::*;

#[derive(DerefMut)]
pub struct TracePipeline(wgpu::ComputePipeline);

impl PipelineLayout for TracePipeline{
    fn layout(device: &wgpu::Device) -> Option<wgpu::PipelineLayout> {
        Some(device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("TracePipeline Layout"),
            bind_group_layouts: &[
                &TraceMesh::bind_group_layout(device),
                &DstImage::bind_group_layout(device),
            ],
            push_constant_ranges: &[]
        }))
    }
}

impl TracePipeline{
    pub fn load(device: &wgpu::Device) -> Self{
        //let shader = Shader::load(device, &std::path::Path::new("src/shaders/trace.glsl"), wgpu::ShaderStages::COMPUTE, None).unwrap();
        
        let shader = ComputeShader::from_src_glsl(device, include_str!("shaders/trace.glsl"), None).unwrap();
        /*
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor{
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::from(include_str!("shaders/trace.wgsl"))),
        });
        */
        let cppl = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor{
            label: Some("TracePipeline"),
            layout: None,
            module: &shader,
            entry_point: "main",
            //..shader.compute_pipeline_desc()
        });

        Self(cppl)
    }
}

#[derive(BindGroupContent)]
pub struct TraceMesh{
    nodes: Buffer<GlslBVHNode>,
    verts: Buffer<Vert>,
    indices: Buffer<u32>,
}

impl TraceMesh{
    pub fn new(device: &wgpu::Device, nodes: &[GlslBVHNode], verts: &[Vert], indices: &[u32]) -> Self{
        let nodes = BufferBuilder::new()
            .storage()
            .build(device, nodes);
        let verts = BufferBuilder::new()
            .storage()
            .build(device, verts);
        let indices = BufferBuilder::new()
            .storage()
            .build(device, indices);

        Self{
            nodes,
            verts,
            indices,
        }
    }
}

impl BindGroupLayout for TraceMesh{
    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            label: Some("TraceMesh BindGroupLayout"),
            entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    ..glsl::buffer_entry(true)
                },
                wgpu::BindGroupLayoutEntry{
                    binding: 1,
                    ..glsl::buffer_entry(true)
                },
                wgpu::BindGroupLayoutEntry{
                    binding: 2,
                    ..glsl::buffer_entry(true)
                },
            ]
        })
    }
}

#[derive(BindGroupContent)]
pub struct DstImage{
    pub view: wgpu::TextureView,
}

impl BindGroupLayout for DstImage{
    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            label: Some("Trace DstImage BindGroupLayout"),
            entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    visibility: wgpu::ShaderStages::all(),
                    count: None,
                    ty: wgpu::BindingType::StorageTexture{
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    }
                    //..glsl::image2D_entry(wgpu::TextureFormat::Rgba8Unorm, wgpu::StorageTextureAccess::WriteOnly)
                }
            ]
        })
    }
}
