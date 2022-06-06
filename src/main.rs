use screen_13::prelude::*;
use screen_13_fx::*;
use screen_13_egui::*;

mod aabb;
mod bvh;
mod glsl_bvh;
//mod presenter;
mod trace_ppl;
mod mesh;
mod model;
mod world;
mod material;

use mesh::*;
use glsl_bvh::*;
//use presenter::*;
use world::*;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PushConstants{
    pub width: u32,
    pub height: u32,
    pub num_paths: u32,
}

fn main() {
    pretty_env_logger::init();
    let mut world = World::new();
    world.load_obj("src/assets/test_multi.obj");


    // ===========
    //  Rendering
    // ===========

    let screen_13 = EventLoop::new().debug(true).build().unwrap();
    let presenter = GraphicPresenter::new(&screen_13.device).unwrap();
    let mut cache = HashPool::new(&screen_13.device);
    let mut egui = Egui::new(&screen_13.device, screen_13.window());

    let cppl = screen_13.new_compute_pipeline(ComputePipelineInfo::new(
        inline_spirv::include_spirv!("src/shaders/trace.glsl", comp, vulkan1_2).as_slice(),
    ));

    let mut world_binding = Some(world.upload(&mut cache));
    println!("test");

    let trace_extent = [800, 600, 2];

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
                        c.dispatch(trace_extent[0], trace_extent[1], 1);
                    });

                render_graph.copy_image_to_buffer(image_node, image_buffer_node);

                presenter.present_image(&mut render_graph, image_node, frame.swapchain_image);

                egui.run(frame.window, frame.events, frame.swapchain_image, &mut render_graph, |ctx|{
                    egui::Window::new("Info").show(ctx, |ui|{
                        ui.label(format!("dt: {:.4}", frame.dt));
                    });
                });

                world_binding = Some(world_node.unbind(&mut render_graph));
                image_buffer = Some(render_graph.unbind_node(image_buffer_node));
                render_graph.unbind_node(image_node);
            }
            //frame.exit();
        })
        .unwrap();
    /*
    let image_buffer_content =
        Buffer::mapped_slice_mut(image_buffer.as_mut().unwrap().get_mut().unwrap());
    let img = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
        trace_extent[0],
        trace_extent[1],
        image_buffer_content,
    )
    .unwrap();
    img.save("out.png").unwrap();
    */
}
