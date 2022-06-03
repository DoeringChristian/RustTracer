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

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PushConstants{
    pub width: u32,
    pub height: u32,
    pub num_paths: u32,
}

fn main() {
    let model = Model::load_obj("src/assets/suzanne.obj");

    //let bvh = model.create_bvh_glsl();

    /*
    println!("len: {}", bvh.nodes().len());
    println!("root node: {:#?}", bvh.nodes()[0]);
    println!("l: {:#?}", bvh.nodes()[1]);
    println!("r: {:#?}", bvh.nodes()[bvh.nodes()[0].right as usize]);
    */

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
    let mut index_buffer = model.upload_indices(&mut cache);
    let mut vertex_buffer = model.upload_verts(&mut cache);
    let mut bvh_buffer = model.upload_bvh(&mut cache);

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
                    .read_descriptor((0, 1), vertex_node)
                    .read_descriptor((0, 2), index_node);
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
