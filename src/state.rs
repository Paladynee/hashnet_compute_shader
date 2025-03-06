use std::time::Instant;

use rand::Rng;
use wgpu::util::DeviceExt;
use winit::{
    event::{DeviceId, KeyEvent, WindowEvent},
    keyboard::{Key, SmolStr},
};

use crate::{
    GameConfiguration,
    types::{Command, CommandUniform, MouseUniform, Particle, ResolutionUniform, TimeUniform},
};

pub struct State<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub compute_pipeline: wgpu::ComputePipeline,
    pub particle_buffer: wgpu::Buffer,
    pub time_buffer: wgpu::Buffer,
    pub mouse_buffer: wgpu::Buffer,
    pub resolution_buffer: wgpu::Buffer,
    pub command_buffer: wgpu::Buffer,
    pub compute_bind_group: wgpu::BindGroup,
    pub render_bind_group: wgpu::BindGroup,
    pub last_update: Instant,
    pub mouse_position: [f32; 2],
    pub current_resolution: ResolutionUniform,
    pub current_command: Command,
    pub game_config: GameConfiguration,
}

impl<'a> State<'a> {
    pub async fn new(window: &'a winit::window::Window, game_config: GameConfiguration) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create a surface from the window
        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::VERTEX_WRITABLE_STORAGE,
                    required_limits: wgpu::Limits {
                        max_storage_buffer_binding_size: 2 << 30,
                        ..adapter.limits()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Initialize particles with random positions and velocities
        let mut particles = Vec::with_capacity(game_config.num_particles as usize);
        let mut rng = rand::thread_rng();

        for _ in 0..game_config.num_particles {
            particles.push(Particle {
                position: [rng.gen_range(-0.9..0.9), rng.gen_range(-0.9..0.9)],
                velocity: [rng.gen_range(-0.1..0.1), rng.gen_range(-0.1..0.1)],
                acceleration: [0.0, 0.0],
            });
        }

        // Create particle buffer
        let particle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Buffer"),
            contents: bytemuck::cast_slice(&particles),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST,
        });

        let resolution = ResolutionUniform {
            width: size.width as f32,
            height: size.height as f32,
        };

        let resolution_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Resolution Buffer"),
            contents: bytemuck::cast_slice(&[resolution]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Time uniform buffer
        let time_data = TimeUniform {
            delta_time: 0.016, // default to 16ms
            particle_count: game_config.num_particles,
            _padding1: [0.0; 2],
            _padding2: [0.0; 4],
        };

        let time_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Time Uniform Buffer"),
            contents: bytemuck::cast_slice(&[time_data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Mouse position buffer
        let mouse_position = MouseUniform {
            mouse_position: [0.0, 0.0],
        };

        let mouse_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mouse Position Buffer"),
            contents: bytemuck::cast_slice(&[mouse_position]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let command = CommandUniform { command: 0 };

        let command_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Command Buffer"),
            contents: bytemuck::cast_slice(&[command]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create compute bind group layout
        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Compute Bind Group Layout"),
                entries: &[
                    // Time uniform
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Particle buffer (read-write for compute)
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Mouse position buffer (read-only for compute)
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Command buffer (read-only for compute)
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        // Create render bind group layout
        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Render Bind Group Layout"),
                entries: &[
                    // Particle buffer (read-only for vertex)
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Resolution buffer (read-only for vertex)
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        // Create bind groups
        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: time_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: particle_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: mouse_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: command_buffer.as_entire_binding(),
                },
            ],
        });

        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render Bind Group"),
            layout: &render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: particle_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: resolution_buffer.as_entire_binding(),
                },
            ],
        });

        // Create compute shader
        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("compute.wgsl").into()),
        });

        // Create compute pipeline
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Compute Pipeline Layout"),
                    bind_group_layouts: &[&compute_bind_group_layout],
                    push_constant_ranges: &[],
                }),
            ),
            module: &compute_shader,
            entry_point: "update_particles",
        });

        // Create render shader
        let render_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Render Shader"),
            source: wgpu::ShaderSource::Wgsl(get_shader(&game_config).into()),
        });

        // Create render pipeline
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&render_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            compute_pipeline,
            particle_buffer,
            time_buffer,
            mouse_buffer,
            resolution_buffer,
            command_buffer,
            compute_bind_group,
            render_bind_group,
            last_update: Instant::now(),
            mouse_position: [0.0, 0.0],
            current_resolution: resolution,
            current_command: Command::Roam,
            game_config,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    pub fn mouse_moved(
        &mut self,
        _device_id: winit::event::DeviceId,
        position: winit::dpi::PhysicalPosition<f64>,
    ) {
        // Convert to normalized device coordinates
        let x = (position.x / self.size.width as f64) * 2.0 - 1.0;
        let y = -((position.y / self.size.height as f64) * 2.0 - 1.0);
        self.mouse_position[0] = x as f32;
        self.mouse_position[1] = y as f32;
    }

    pub fn update(&mut self) {
        // Calculate delta time
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;

        // Clamp delta time to avoid large jumps
        let delta_time = delta_time.min(0.1);

        // Update time uniform
        let time_data = TimeUniform {
            delta_time,
            particle_count: self.game_config.num_particles,
            _padding1: [0.0; 2],
            _padding2: [0.0; 4],
        };

        // update mouse position
        let mouse_data = MouseUniform {
            mouse_position: self.mouse_position,
        };

        // update command
        let command_data = CommandUniform::from_command(self.current_command);

        self.queue
            .write_buffer(&self.time_buffer, 0, bytemuck::cast_slice(&[time_data]));

        self.queue
            .write_buffer(&self.mouse_buffer, 0, bytemuck::cast_slice(&[mouse_data]));

        self.queue.write_buffer(
            &self.resolution_buffer,
            0,
            bytemuck::cast_slice(&[self.current_resolution]),
        );

        self.queue.write_buffer(
            &self.command_buffer,
            0,
            bytemuck::cast_slice(&[command_data]),
        );

        // Dispatch compute shader
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Particle Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);

            // Use 2D dispatch to avoid exceeding the 65535 limit per dimension
            let workgroups_x = 65535u32; // Maximum value for x dimension
            let workgroups_y = self.game_config.num_particles.div_ceil(workgroups_x * 256); // Calculate y dimension
            compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    #[allow(clippy::single_match)]
    pub fn keyboard_input(
        &mut self,
        device_id: DeviceId,
        key_event: &KeyEvent,
        is_synthetic: bool,
    ) {
        if key_event.state == winit::event::ElementState::Pressed && !is_synthetic {
            match &key_event.logical_key {
                Key::Character(a) => match a.as_str() {
                    "r" => {
                        self.current_command = Command::Roam;
                    }
                    "s" => {
                        self.current_command = Command::Shuffle;
                    }
                    _ => {}
                },

                _ => {}
            }
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.render_bind_group, &[]);
            // Draw 6 vertices (2 triangles) per particle
            render_pass.draw(0..self.game_config.num_particles * 6, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub fn get_shader(config: &GameConfiguration) -> String {
    let string = include_str!("shader.wgsl");
    /*
       // $RUST_REPLACEME
       const QUAD_SIZE: f32 = 0.001;
       // $RUST_REPLACEMEEND

       we need to replace "const QUAD_SIZE: f32 = 0.001;" with whatever is provided from the GameConfiguration.
       the place is marked with $RUST_REPLACEME and $RUST_REPLACEMEEND
    */

    let mut string = string.to_string();
    let start = string.find("$RUST_REPLACEME").unwrap();
    let end = string.find("$RUST_REPLACEMEEND").unwrap() + "$RUST_REPLACEMEEND".len();
    let replacement = format!("\nconst QUAD_SIZE: f32 = {};", config.quad_size);
    string.replace_range(start..end, &replacement);
    println!("Shader: {}", string);
    string
}
