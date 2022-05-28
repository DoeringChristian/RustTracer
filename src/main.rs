use screen_13::prelude_arc::*;
use std::ops::Range;

mod aabb;
mod bvh;
mod glsl_bvh;
mod presenter;
mod trace_ppl;

use aabb::*;
use bvh::*;
use glsl_bvh::*;
use presenter::*;
use screen_13_fx::prelude_arc::*;
use trace_ppl::*;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PushConstants{
    pub width: u32,
    pub height: u32,
    pub num_paths: u32,
}

pub trait Pos3 {
    fn pos3(&self) -> [f32; 3];
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vert {
    pub pos: [f32; 4],
    pub color: [f32; 4],
}

impl Pos3 for Vert {
    fn pos3(&self) -> [f32; 3] {
        [self.pos[0], self.pos[1], self.pos[2]]
    }
}

impl From<[Vert; 3]> for AABB {
    fn from(src: [Vert; 3]) -> Self {
        let v1 = src[0].pos3();
        let v2 = src[1].pos3();
        let v3 = src[2].pos3();
        AABB {
            min: [
                v1[0].min(v2[0]).min(v3[0]),
                v1[1].min(v2[1]).min(v3[1]),
                v1[2].min(v2[2]).min(v3[2]),
            ],
            max: [
                v1[0].max(v2[0]).max(v3[0]),
                v1[1].max(v2[1]).max(v3[1]),
                v1[2].max(v2[2]).max(v3[2]),
            ],
        }
    }
}

pub struct Mesh {
    pub verts: Vec<Vert>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn get_tri(&self, index: usize) -> [Vert; 3] {
        [
            self.verts[self.indices[index + 0] as usize],
            self.verts[self.indices[index + 1] as usize],
            self.verts[self.indices[index + 2] as usize],
        ]
    }
    pub fn get_for_tri(&self, indices: &[usize; 3]) -> [Vert; 3] {
        [
            self.verts[indices[0]],
            self.verts[indices[1]],
            self.verts[indices[2]],
        ]
    }
}

fn main() {
    let verts = vec![
        Vert {
            pos: [0., 0., 1., 1.],
            color: [0., 0., 0., 1.],
        },
        Vert {
            pos: [1., 0., 0., 1.],
            color: [0., 0., 0., 1.],
        },
        Vert {
            pos: [0., 1., 0., 1.],
            color: [0., 0., 0., 1.],
        },
    ];
    let indices = vec![0, 1, 2];

    let mesh = Mesh { verts, indices };

    let bvh =
        GlslBVH::build_buckets_16(
            (0..mesh.indices.len() / 3)
                .into_iter()
                .map(|i| {
                    IndexedAABB {
                        index: i * 3,
                        aabb: mesh.get_tri(i * 3).into(),
                    }
                }),
        );
    bvh.print_rec(0, &mut String::from(""));

    let suzanne = tobj::load_obj("src/assets/suzanne.obj", &tobj::LoadOptions::default())
        .unwrap()
        .0;

    let verts = (0..(suzanne[0].mesh.positions.len() / 3))
        .into_iter()
        .map(|i| Vert {
            pos: [
                suzanne[0].mesh.positions[i * 3],
                suzanne[0].mesh.positions[i * 3 + 1],
                suzanne[0].mesh.positions[i * 3 + 2],
                0.,
            ],
            color: [
                *suzanne[0].mesh.vertex_color.get(i * 3).unwrap_or(&0.),
                *suzanne[0].mesh.vertex_color.get(i * 3 + 1).unwrap_or(&0.),
                *suzanne[0].mesh.vertex_color.get(i * 3 + 2).unwrap_or(&0.),
                1.,
            ],
        })
        .collect();

    let indices = (0..(suzanne[0].mesh.indices.len()))
        .into_iter()
        .map(|i| suzanne[0].mesh.indices[i] as u32)
        .collect();

    let mesh = Mesh { verts, indices };

    let bvh =
        GlslBVH::build_buckets_16(
            (0..mesh.indices.len() / 3)
                .into_iter()
                .map(|i| IndexedAABB {
                    index: i * 3,
                    aabb: mesh.get_tri(i * 3).into(),
                }),
        );
    println!("len: {}", bvh.nodes().len());
    println!("aabb: {:?}", bvh.aabb());

    // ===========
    //  Rendering
    // ===========

    pretty_env_logger::init();

    let screen_13 = EventLoop::new().debug(true).build().unwrap();
    //let mut presenter = GraphicPresenter::new(&screen_13.device).unwrap();
    let presenter = Presenter::new(&screen_13.device);
    let mut cache = HashPool::new(&screen_13.device);

    let cppl = screen_13.new_compute_pipeline(ComputePipelineInfo::new(
        inline_spirv::include_spirv!("src/shaders/trace.glsl", comp, vulkan1_2).as_slice(),
    ));

    println!("test");

    let mut index_buffer = Some(BufferLeaseBinding({
        let mut buf = cache
            .lease(BufferInfo::new_mappable(
                (std::mem::size_of::<u32>() * mesh.indices.len()) as u64,
                vk::BufferUsageFlags::STORAGE_BUFFER,
            ))
            .expect("Could not create Index Buffer");
        //println!("{}", buf.as_ref().info().size);
        Buffer::copy_from_slice(
            buf.get_mut().expect("Could not get Index Buffer"),
            0,
            bytemuck::cast_slice(&mesh.indices),
        );
        buf
    }));
    let mut vertex_buffer = Some(BufferLeaseBinding({
        let mut buf = cache
            .lease(BufferInfo::new_mappable(
                (std::mem::size_of::<Vert>() * mesh.verts.len()) as u64,
                vk::BufferUsageFlags::STORAGE_BUFFER,
            ))
            .unwrap();
        Buffer::copy_from_slice(buf.get_mut().unwrap(), 0, bytemuck::cast_slice(&mesh.verts));
        buf
    }));
    let mut bvh_buffer = Some(BufferLeaseBinding({
        let mut buf = cache
            .lease(BufferInfo::new_mappable(
                (std::mem::size_of::<GlslBVHNode>() * bvh.nodes().len()) as u64,
                vk::BufferUsageFlags::STORAGE_BUFFER,
            ))
            .unwrap();
        Buffer::copy_from_slice(buf.get_mut().unwrap(), 0, bytemuck::cast_slice(bvh.nodes()));
        buf
    }));

    let trace_extent = [100, 100, 1];

    let mut image_buffer = Some(BufferLeaseBinding({
        let buf = cache
            .lease(BufferInfo::new_mappable(
                (trace_extent[0] * trace_extent[1] * 4) as u64,
                vk::BufferUsageFlags::TRANSFER_DST,
            ))
            .unwrap();
        buf
    }));

    screen_13
        .run(|mut frame| {
            let mut render_graph = &mut frame.render_graph;
            {
                println!("dt: {}", frame.dt);
                let push_constants = PushConstants{
                    width: trace_extent[0],
                    height: trace_extent[1],
                    num_paths: trace_extent[2],
                };

                let image_buffer_node = render_graph.bind_node(image_buffer.take().unwrap());
                //let image_node = render_graph.bind_node(image.take().unwrap());
                let vertex_node = render_graph.bind_node(vertex_buffer.take().unwrap());
                let index_node = render_graph.bind_node(index_buffer.take().unwrap());
                let bvh_node = render_graph.bind_node(bvh_buffer.take().unwrap());
                let image_node = 
                    render_graph.bind_node(cache.lease(ImageInfo::new_2d(
                                vk::Format::R8G8B8A8_UNORM,
                                trace_extent[0],
                                trace_extent[1],
                                vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::SAMPLED,
                    )).unwrap());

                let mut tracer_pass = render_graph
                    .begin_pass("Tracer Pass")
                    .bind_pipeline(&cppl)
                    .read_descriptor((0, 0, [0]), bvh_node)
                    //.access_descriptor((0, 0, [0]), bvh_node, AccessType::ComputeShaderReadOther)
                    .access_descriptor((0, 1), vertex_node, AccessType::ComputeShaderReadOther)
                    .access_descriptor((0, 2), index_node, AccessType::ComputeShaderReadOther);
                tracer_pass = tracer_pass.write_descriptor((1, 0), image_node);
                /*
                for (i, image_node) in images_nodes.iter().enumerate(){
                    tracer_pass = tracer_pass.write_descriptor((1, 0, [i as u32]), *image_node);
                }
                */
                tracer_pass
                    //.access_descriptor((1, 0), images_nodes[0], AccessType::ComputeShaderWrite)
                    .record_compute(move |c| {
                        c.push_constants(bytemuck::cast_slice(&[push_constants]));
                        c.dispatch(trace_extent[0], trace_extent[1], trace_extent[2]);
                    });
                render_graph.copy_image_to_buffer(image_node, image_buffer_node);

                //render_graph.clear_color_image(frame.swapchain_image);
                presenter.present(
                    &mut render_graph,
                    image_node,
                    frame.swapchain_image,
                    [
                        frame.window.inner_size().width,
                        frame.window.inner_size().height,
                    ],
                );

                //image = Some(render_graph.unbind_node(image_node));
                index_buffer = Some(render_graph.unbind_node(index_node));
                vertex_buffer = Some(render_graph.unbind_node(vertex_node));
                bvh_buffer = Some(render_graph.unbind_node(bvh_node));
                image_buffer = Some(render_graph.unbind_node(image_buffer_node));
                render_graph.unbind_node(image_node);
            }
            //frame.exit();
        })
        .unwrap();
    let image_buffer_content =
        Buffer::mapped_slice_mut(image_buffer.as_mut().unwrap().get_mut().unwrap());
    let img = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
        trace_extent[0],
        trace_extent[1],
        image_buffer_content,
    )
    .unwrap();
    img.save("out.png").unwrap();
}
