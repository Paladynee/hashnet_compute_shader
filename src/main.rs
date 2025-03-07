use std::{fs, io, path::Path};

use serde::{Deserialize, Serialize};
use state::State;
use types::ResolutionUniform;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

mod state;
mod types;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameConfiguration {
    pub num_particles: u32,
    pub quad_size: f32,
}

impl Default for GameConfiguration {
    fn default() -> Self {
        Self {
            num_particles: 1000,
            quad_size: 0.001,
        }
    }
}

impl GameConfiguration {
    pub fn from_path(path: &Path) -> io::Result<Self> {
        // read from the path, or create it if it doesnt exist with default.
        if path.exists() {
            let file = fs::File::open(path)?;
            let config: GameConfiguration = serde_json::from_reader(file)?;
            Ok(config)
        } else {
            let default_config = GameConfiguration::default();
            let file = fs::File::create(path)?;
            serde_json::to_writer_pretty(file, &default_config)?;
            Ok(default_config)
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Red Triangle")
        .build(&event_loop)
        .unwrap();

    let config = GameConfiguration::from_path(Path::new("config.json")).unwrap();

    let mut state = pollster::block_on(State::new(&window, config));
    state.current_resolution = ResolutionUniform {
        width: window.inner_size().width as f32,
        height: window.inner_size().height as f32,
    };
    state.resize(state.size);

    event_loop
        .run(|event, elwt| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                            state.current_resolution = ResolutionUniform {
                                width: physical_size.width as f32,
                                height: physical_size.height as f32,
                            };
                        }

                        WindowEvent::CursorMoved {
                            device_id,
                            position,
                        } => {
                            state.mouse_moved(*device_id, *position);
                        }

                        WindowEvent::KeyboardInput {
                            device_id,
                            event,
                            is_synthetic,
                        } => {
                            state.keyboard_input(*device_id, event, *is_synthetic, &window);
                        }

                        WindowEvent::RedrawRequested => {
                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    state.resize(state.size)
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                Err(wgpu::SurfaceError::Timeout) => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        })
        .unwrap();
}
