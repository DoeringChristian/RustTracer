use archery::*;
use screen_13::prelude_arc::*;
use super::*;

pub struct TracerExtent {
    pub width: u32,
    pub height: u32,
    pub ppp: u32,
}

pub struct Tracer {
    pub cppl: SharedPointer<ComputePipeline, ArcK>,
    pub cache: HashPool,
    pub index_buffer: Option<BufferLeaseBinding<ArcK>>,
    pub vertex_buffer: Option<BufferLeaseBinding<ArcK>>,
    pub bvh_buffer: Option<BufferLeaseBinding<ArcK>>,
}

impl Tracer {
    pub fn new(screen_13: &mut EventLoop) -> Self {
        let cppl = screen_13.new_compute_pipeline(ComputePipelineInfo::new(
            inline_spirv::include_spirv!("src/shaders/trace.glsl", comp).as_slice(),
        ));

        let cache = HashPool::new(&screen_13.device);

        Self { 
            cppl ,
            cache,
            index_buffer: None,
            vertex_buffer: None,
            bvh_buffer: None,
        }
    }

    pub fn fill_buffers(
        &mut self,
        indices: &[u32],
        vertices: &[Vert],
        bvh: &[GlslBVHNode],
    ){
        self.index_buffer = Some(BufferLeaseBinding({
            let mut buf = self.cache.lease(BufferInfo::new_mappable(
                    (std::mem::size_of::<u32>() * indices.len()) as u64,
                    vk::BufferUsageFlags::STORAGE_BUFFER,
            )).expect("Could not create Index Buffer");
            Buffer::copy_from_slice(
                buf.get_mut().expect("Could not get Index Buffer"),
                0,
                bytemuck::cast_slice(indices),
            );
            buf
        }));
        self.vertex_buffer = Some(BufferLeaseBinding({
            let mut buf = self.cache.lease(BufferInfo::new_mappable(
                    (std::mem::size_of::<Vert>() * vertices.len()) as u64,
                    vk::BufferUsageFlags::STORAGE_BUFFER,
            )).unwrap();
            Buffer::copy_from_slice(
                buf.get_mut().unwrap(),
                0,
                bytemuck::cast_slice(vertices),
            );
            buf
        }));
        self.bvh_buffer = Some(BufferLeaseBinding({
            let mut buf = self.cache.lease(BufferInfo::new_mappable(
                    (std::mem::size_of::<GlslBVHNode>() * bvh.len()) as u64,
                    vk::BufferUsageFlags::STORAGE_BUFFER,
            )).unwrap();
            Buffer::copy_from_slice(
                buf.get_mut().unwrap(),
                0,
                bytemuck::cast_slice(bvh),
            );
            buf
        }));
    }

    pub fn record(
        &mut self, 
        graph: &mut RenderGraph, 
        dst: impl Into<AnyImageNode>,
        extent: TracerExtent,
    ) {
        //println!("{:?}", self.index_buffer.take().unwrap());
        let index_node = graph.bind_node(self.index_buffer.take().unwrap());
        let vertex_node = graph.bind_node(self.vertex_buffer.take().unwrap());
        let bvh_node = graph.bind_node(self.bvh_buffer.take().unwrap());

        graph
            .begin_pass("Tracer Pass")
            .bind_pipeline(&self.cppl)
            //.access_descriptor((0, [0, 1, 2]), &[bvh_buffer.into(), vertex_buffer.into(), index_buffer.into()], AccessType::ComputeShaderReadOther);
            .access_descriptor((0, 0), bvh_node, AccessType::ComputeShaderReadOther)
            .access_descriptor((0, 1), vertex_node, AccessType::ComputeShaderReadOther)
            .access_descriptor((0, 2), index_node, AccessType::ComputeShaderReadOther)
            .access_descriptor((1, 0), dst.into(), AccessType::ComputeShaderWrite)
            .record_compute(move |c| {
                c.dispatch(extent.width, extent.height, extent.ppp);
            });

        graph.unbind_node(index_node);
        graph.unbind_node(vertex_node);
        graph.unbind_node(bvh_node);
    }
}
