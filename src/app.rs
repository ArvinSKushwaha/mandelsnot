use std::sync::Arc;

use eframe::{
    egui::{CentralPanel, Sense, Ui, Vec2, Key, Modifiers, Frame},
    egui_wgpu::CallbackFn,
    epaint::PaintCallback,
    wgpu::{
        self, include_wgsl,
        util::{BufferInitDescriptor, DeviceExt},
        BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
        Buffer, BufferUsages, PipelineLayoutDescriptor, RenderPipeline,
        RenderPipelineDescriptor,
    },
    App, CreationContext,
};

pub struct Mandelsnot {
    bounds: [Vec2; 2],
}

pub struct MandelsnotRenderResources {
    pipeline: RenderPipeline,
    uniform: Buffer,
    bind_group: wgpu::BindGroup,
}

impl Mandelsnot {
    pub const APP_NAME: &str = "Mandelsnot";

    pub fn new(cc: &CreationContext<'_>) -> Box<Self> {
        let Some(render_state) = cc.wgpu_render_state.as_ref() else {
            panic!("Could not WGPU render state");
        };

        let device = &render_state.device;

        let shader = device.create_shader_module(include_wgsl!("mandelsnot.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    has_dynamic_offset: false,
                    min_binding_size: None,
                    ty: wgpu::BufferBindingType::Uniform,
                },
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    blend: Some(wgpu::BlendState {
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::Zero,
                            operation: wgpu::BlendOperation::Add,
                        },
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::Zero,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    format: render_state.target_format,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: Some(wgpu::IndexFormat::Uint32),
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let uniform = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[0.; 4]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });

        render_state
            .renderer
            .write()
            .paint_callback_resources
            .insert(MandelsnotRenderResources {
                pipeline,
                uniform,
                bind_group,
            });

        Box::new(Mandelsnot {
            bounds: [[-2., -2.].into(), [2., 2.].into()],
        })
    }

    pub fn mandelbrot_painting(&mut self, ui: &mut Ui) {
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(680., 680.), Sense::drag());

        let mut disp = resp.drag_delta() / 680. * (self.bounds[1] - self.bounds[0]);
        disp.x *= -1.;

        let mut center = (self.bounds[0] + self.bounds[1]) / 2.;
        let mut extent = (self.bounds[1] - self.bounds[0]) / 2.;

        center += disp;

        if ui.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::X)) {
            extent.x *= 2.;
        } else if ui.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::Y)) {
            extent.y *= 2.;
        } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::X)) {
            extent.x /= 2.;
        } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Y)) {
            extent.y /= 2.;
        } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::O)) {
            extent = Vec2::new(2., 2.);
            center = Vec2::new(0., 0.);
        }

        self.bounds = [center - extent, center + extent];
        let bounds = self.bounds.clone();

        let callback = CallbackFn::new()
            .prepare(move |_, queue, _, typemap| {
                let resources: &MandelsnotRenderResources = typemap.get().unwrap();

                queue.write_buffer(&resources.uniform, 0, bytemuck::cast_slice(&bounds));

                vec![]
            })
            .paint(|_, render_pass, typemap| {
                let resources: &MandelsnotRenderResources = typemap.get().unwrap();

                render_pass.set_pipeline(&resources.pipeline);
                render_pass.set_bind_group(0, &resources.bind_group, &[]);
                render_pass.draw(0..4, 0..1);
            });

        let callback = Arc::new(callback);

        ui.painter_at(rect).add(PaintCallback { callback, rect });
    }
}

impl App for Mandelsnot {
    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("X: {}", (self.bounds[0].x + self.bounds[1].x) / 2.));
            ui.label(format!("Y: {}", (self.bounds[0].y + self.bounds[1].y) / 2.));

            Frame::canvas(ui.style()).show(ui, |ui| {
                self.mandelbrot_painting(ui);
            });
        });
    }
}
