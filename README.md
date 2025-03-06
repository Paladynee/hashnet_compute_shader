# 🌀 Hashnet Compute Shader

![License](https://img.shields.io/badge/license-MIT-blue)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![GPU](https://img.shields.io/badge/GPU-WebGPU-purple)

> A blazingly fast 2D particle simulator written in Rust, capable of handling up to 89,478,485 particles simultaneously on modern GPUs.

## ✨ Features

-   🚀 **High Performance**: Leverages GPU compute shaders for massive parallelism
-   🔢 **Scale**: Handles tens of millions of particles with stable frame rates
-   🖱️ **Interactive**: Particles respond to mouse movements in real-time
-   🎮 **Multiple Modes**: Switch between different particle behaviors
-   🔧 **Configurable**: Easy customization via JSON configuration

## 📋 Requirements

-   🦀 Rust 1.70 or newer
-   🖥️ A GPU with WebGPU support (most modern GPUs)
-   🐧 Windows/Linux/macOS

## 🛠️ Build from Source

1. **Clone the repository**

```bash
git clone https://github.com/yourusername/hashnet_compute_shader.git
cd hashnet_compute_shader
```

2. **Build with Cargo**

```bash
cargo build --release
```

3. **Run the application**

```bash
cargo run --release
```

⚠️ **Note**: Make sure to use the release build for optimal performance when running with millions of particles!

## 🎮 Controls

-   **Mouse Movement**: Particles gravitate toward cursor
-   **R key**: Switch to Roam mode (particles gravitate around the cursor)
-   **S key**: Switch to Shuffle mode (particles are randomly offset)

## ⚙️ Configuration

Customize the simulation by modifying the `config.json` file:

```json
{
    "num_particles": 89478485,
    "quad_size": 0.001
}
```

-   **num_particles**: Number of particles to simulate
-   **quad_size**: Size of each particle on screen

## 🔬 How It Works

Hashnet Compute Shader uses WebGPU through the `wgpu` Rust library to run highly parallelized compute shaders. The simulation follows these steps:

1. **Initialization**: Particles are created with random positions and velocities
2. **Compute Phase**: GPU updates particle positions and velocities in parallel
3. **Render Phase**: Particles are rendered as small quads with velocity-based coloring

The application dispatches compute workgroups in a 2D grid to overcome the 65535 limit per dimension, allowing it to handle many millions of particles.

## 📈 Performance Tips

-   Adjust particle count based on your GPU capabilities
-   For integrated GPUs, try starting with 1-5 million particles
-   For high-end dedicated GPUs, you can push well beyond 50 million particles

## 🤝 Contributing

Contributions are welcome! Feel free to:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

-   The [wgpu](https://github.com/gfx-rs/wgpu) team for their excellent WebGPU implementation
-   The Rust community for building an amazing ecosystem for systems programming
