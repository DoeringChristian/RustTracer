use archery::*;
use inline_spirv::*;
use screen_13::prelude_arc::*;

pub struct Presenter {
    rppl: SharedPointer<GraphicPipeline, ArcK>,
}
impl Presenter {
    pub fn new(device: &SharedPointer<Device, ArcK>) -> Self {
        let rppl = SharedPointer::new(
            GraphicPipeline::create(
                device,
                GraphicPipelineInfo::new(),
                [
                    Shader::new_vertex(
                        inline_spirv!(
                            r#"
                            #version 450

                            const vec2 UV[6] = {
                                vec2(0.0, 0.0),
                                vec2(0.0, 1.0),
                                vec2(1.0, 0.0),
                                vec2(1.0, 1.0),
                                vec2(0.0, 1.0),
                                vec2(1.0, 0.0),
                            };
                            const vec4 POS[6] = {
                                vec4(-1.0, -1.0, 0.0, 1.0),
                                vec4(-1.0, 1.0, 0.0, 1.0),
                                vec4(1.0, -1.0, 0.0, 1.0),
                                vec4(1.0, 1.0, 0.0, 1.0),
                                vec4(1.0, -1.0, 0.0, 1.0),
                                vec4(-1.0, 1.0, 0.0, 1.0),
                            };

                            layout(location = 0) out vec2 o_uv;

                            void main(){
                                o_uv = UV[gl_VertexIndex];
                                gl_Position = POS[gl_VertexIndex];
                            }
                            "#,
                            vert
                        )
                        .as_slice(),
                    ),
                    Shader::new_fragment(
                        inline_spirv!(
                            r#"
                            #version 450
                            layout(location = 0) out vec4 o_color;

                            layout(location = 0) in vec2 i_uv;

                            layout(set = 0, binding = 0) uniform sampler2D tex_s;

                            void main(){
                                o_color = texture(tex_s, i_uv);
                            }
                            "#,
                            frag
                        )
                        .as_slice(),
                    ),
                ],
            )
            .unwrap(),
        );

        Self { rppl }
    }

    pub fn present(
        &self,
        graph: &mut RenderGraph,
        image: impl Into<AnyImageNode>,
        swapchain: SwapchainImageNode<ArcK>,
        size: [u32; 2],
    ) {
        let image = image.into();

        let mut render_graph = graph
            .begin_pass("Present Pass")
            .bind_pipeline(&self.rppl)
            .access_descriptor((0, 0), image, AccessType::FragmentShaderReadOther)
            .clear_color(0)
            .store_color(0, swapchain);
        render_graph.set_render_area(0, 0, size[0], size[1]);
        render_graph.record_subpass(move |subpass| {
            subpass.set_scissor(0, 0, size[0], size[1]);
            subpass.set_viewport(0.0, 0.0, size[0] as f32, size[1] as f32, 0.0..1.);
            subpass.draw(6, 1, 0, 0);
        });
    }
}
