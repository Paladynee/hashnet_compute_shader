struct Particle {
    position: vec2<f32>,
    velocity: vec2<f32>,
    acceleration: vec2<f32>,
};


struct Resolution {
    width: f32,
    height: f32,
};

@group(0) @binding(1) var<storage, read> particles: array<Particle>;
@group(0) @binding(2) var<uniform> resolution: Resolution;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

// Define the size of our quads
// $RUST_REPLACEME
const QUAD_SIZE: f32 = 0.001;
// $RUST_REPLACEMEEND

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Determine which particle this vertex belongs to
    let particle_index = vertex_index / 6u;
    let particle = particles[particle_index];
    
    // Determine which vertex of the quad we're drawing
    let vertex_in_quad = vertex_index % 6u;
    
    // Define offsets for each vertex of the quad
    // We need 6 vertices to form 2 triangles:
    // 0, 1, 2 for first triangle and 2, 3, 0 for second triangle
    // (or any similar arrangement)
    var offset = vec2<f32>(0.0, 0.0);

    switch vertex_in_quad {
        case 0u: { offset = vec2<f32>(-QUAD_SIZE, -QUAD_SIZE); } // Bottom-left
        case 1u: { offset = vec2<f32>(QUAD_SIZE, -QUAD_SIZE); }  // Bottom-right
        case 2u: { offset = vec2<f32>(QUAD_SIZE, QUAD_SIZE); }   // Top-right
        case 3u: { offset = vec2<f32>(QUAD_SIZE, QUAD_SIZE); }   // Top-right (duplicate)
        case 4u: { offset = vec2<f32>(-QUAD_SIZE, QUAD_SIZE); }  // Top-left
        case 5u: { offset = vec2<f32>(-QUAD_SIZE, -QUAD_SIZE); } // Bottom-left (duplicate)
        default: { offset = vec2<f32>(0.0, 0.0); }
    }

    // Calculate the aspect ratio using the resolution uniform
    let aspect_ratio = resolution.height / resolution.width;
    // Scale x-offset based on the aspect ratio
    offset.x = offset.x * aspect_ratio;

    var output: VertexOutput;
    // Add offset to particle position to form the quad
    output.position = vec4<f32>(particle.position + offset, 0.0, 1.0);
    
    // Color based on velocity (red/blue for horizontal, green for vertical)
    let speed = length(particle.velocity);
    output.color = vec3<f32>(
        0.5 + particle.velocity.x,
        0.5 + particle.velocity.y,
        1.0 - speed
    );

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.color, 1.0);
}
