use screen_13::prelude_arc::*;
use std::ops::Range;

mod aabb;
mod bvh;
mod glsl_bvh;
mod presenter;
mod trace_ppl;
mod mesh;
mod model;
mod world;

use aabb::*;
use bvh::*;
use mesh::*;
use glsl_bvh::*;
use presenter::*;
use screen_13_fx::prelude_arc::*;
use trace_ppl::*;
use model::*;
use world::*;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PushConstants{
    pub width: u32,
    pub height: u32,
    pub num_paths: u32,
}

fn main() {
    let mut world = World::new();
    world.append_obj("src/assets/test_multi.obj");

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

    let mut world_binding = Some(world.upload(&mut cache));
    println!("test");

    let trace_extent = [200, 200, 1];

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

                let world_node = world_binding.take().unwrap().bind(&mut render_graph);

                let image_buffer_node = render_graph.bind_node(image_buffer.take().unwrap());
                let image_node = 
                    render_graph.bind_node(cache.lease(ImageInfo::new_2d(
                                vk::Format::R8G8B8A8_UNORM,
                                trace_extent[0],
                                trace_extent[1],
                                vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::SAMPLED,
                    )).unwrap());

                let mut tracer_pass = render_graph
                    .begin_pass("Tracer Pass")
                    .bind_pipeline(&cppl);
                tracer_pass = world_node.record_descriptors(tracer_pass);
                tracer_pass = tracer_pass.write_descriptor((1, 0), image_node);

                tracer_pass
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

                world_binding = Some(world_node.unbind(&mut render_graph));
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
