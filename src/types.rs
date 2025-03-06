use bytemuck::{Pod, Zeroable};

// Particle structure to store in the GPU buffer
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Particle {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub acceleration: [f32; 2],
}

// Time uniform to pass deltaTime to the compute shader
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct TimeUniform {
    pub delta_time: f32,
    pub particle_count: u32,
    pub _padding1: [f32; 2], // Adjust padding to keep 16-byte alignment
    pub _padding2: [f32; 4], // Second padding to 32 bytes total
}

// Mouse position uniform to pass mouse coordinates to the compute shader
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct MouseUniform {
    pub mouse_position: [f32; 2],
}

// Resolution
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ResolutionUniform {
    pub width: f32,
    pub height: f32,
}

// Command uniform to pass commands that are shared between all particles
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CommandUniform {
    pub command: u32,
}

impl CommandUniform {
    pub fn from_command(command: Command) -> Self {
        let val = match command {
            Command::Roam => 0,
            Command::Shuffle => 1,
        };

        Self { command: val }
    }
}

// Human readable command names
#[derive(Copy, Clone, Debug)]
pub enum Command {
    Roam,    // particles gravitate around the cursor
    Shuffle, // particles are randomly offset by an amount
}
