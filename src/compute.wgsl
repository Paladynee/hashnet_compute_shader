struct Particle {
    position: vec2<f32>,
    velocity: vec2<f32>,
    acceleration: vec2<f32>,
};

struct TimeUniform {
    delta_time: f32,
    particle_count: u32,
    padding1: vec2<f32>,
    padding2: vec4<f32>,
};

struct MousePosition {
    position: vec2<f32>,
};

struct Command {
    command: u32,
};

@group(0) @binding(0) var<uniform> time: TimeUniform;
@group(0) @binding(1) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(2) var<uniform> mouse_position: MousePosition;
@group(0) @binding(3) var<uniform> command: Command;


// fast pseudorandom number generation based on index
fn fast_random(seed: u32) -> u32 {
    var state = seed;
    state ^= state << 13u;
    state ^= state >> 17u;
    state ^= state << 5u;
    return state;
}

fn f32_from_u32(value: u32) -> f32 {
    return f32(value) / 4294967295.0; // Normalize to [0, 1]
}

const NUDGE_AMOUNT: f32 = 0.01;

// Increased workgroup size from 64 to 256 for better GPU utilization
@compute @workgroup_size(256)
fn update_particles(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // Calculate the actual particle index from 2D dispatch
    let index = global_id.x + global_id.y * 65535u * 256u;

    if index >= time.particle_count {
        return;
    }

    switch command.command {
        case 1u: {
            // "Shuffle" mode, randomly shift the positions of particles by a small amount
            let rng = fast_random(index);

            let small_shift = vec2<f32>(
                f32_from_u32(rng) * NUDGE_AMOUNT - NUDGE_AMOUNT * 0.5,
                f32_from_u32(fast_random(rng)) * NUDGE_AMOUNT - NUDGE_AMOUNT * 0.5
            );
            particles[index].position += small_shift;
        }

        default: {
            // this mode includes 0, which is the "Roam" mode
            // no operation
        }
    }
    
    // Get current particle data (reduces redundant memory access)
    var particle = particles[index];
    
    // Calculate direction to mouse position
    let direction = mouse_position.position - particle.position;
    
    // Early-out for particles that are too far from mouse to be affected significantly
    let dist_sq = dot(direction, direction);
    if dist_sq > 10.0 {
        // Only apply minimal updates for distant particles
        particle.position += particle.velocity * time.delta_time;
        particles[index] = particle;
        return;
    }
    
    // Optimized acceleration calculation (combined operations)
    let unit_size: f32 = 0.1;
    let scaled_dir = direction * unit_size;
    let mag_factor = 1.0 / (dot(scaled_dir, scaled_dir) + 0.1);
    
    // Directly compute normalized direction * magnitude factor
    particle.acceleration = normalize(direction) * mag_factor;
    
    // Update velocity (combine calculations)
    particle.velocity = particle.velocity * 0.99999 + particle.acceleration * time.delta_time;
    
    // Update position
    particle.position += particle.velocity * time.delta_time;
    
    // Optimized boundary collision (avoid branch divergence where possible)
    let pos_abs = abs(particle.position);
    let x_overflow = pos_abs.x > 1.0;
    let y_overflow = pos_abs.y > 1.0;

    if x_overflow || y_overflow {
        // Handle X boundary
        if x_overflow {
            let sign_x = sign(particle.position.x);
            particle.velocity.x = -particle.velocity.x * 0.8;
            particle.position.x = sign_x * 0.99;
        }
        
        // Handle Y boundary
        if y_overflow {
            let sign_y = sign(particle.position.y);
            particle.velocity.y = -particle.velocity.y * 0.8;
            particle.position.y = sign_y * 0.99;
        }
    }
    
    // Write back particle data in one operation
    particles[index] = particle;
}
