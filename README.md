# SimTem

Diagnose your driving. SimTem is a telemetry overlay for Assetto Corsa Competizione (and future simulators), built natively for Linux using Rust, `egui`, and `wgpu`. 

Designed for sim racers running via Proton/Wine, this application reads directly from shared memory mapped to `/dev/shm/` to provide real-time telemetry natively on Linux with virtually zero overhead.

## Features

* **Telemetry Polling:** Reads physics data at 333Hz (3ms) to strictly sync with Assetto Corsa's internal physics engine. 
* **Live Graphing:** 15-second rolling historical graph for Throttle, Brake, ABS, and Traction Control actuation.
* **Auto Car Detection:** Detects the current car via `acpmf_static` to load specific profiles (Shift RPM, Brake Bias offsets).
* **Dynamic Rev Strip:** Visual RPM bar that triggers yellow and strobe warnings based on the exact shift-point of the current car.

## Tech Stack

* **Language:** Rust
* **GUI Framework:** `egui` (Immediate mode GUI)
* **Renderer:** `wgpu` (Vulkan)
* **Memory Management:** `mimalloc` (Allocator) & `memmap2` (Shared memory reading)

## Linux / Proton Architecture & SHM Bridge

Assetto Corsa is a Windows game, meaning its internal telemetry is written to Windows Shared Memory. To access this on Linux, SimTem reads from the Linux `tmpfs` (`/dev/shm/`).

To map the Windows memory out of the Proton prefix and into Linux, SimTem relies on an SHM bridge. **Please refer to [Datalink](https://github.com/LukasLichten/Datalink) for instructions on how to set up and run the SHM bridge alongside your game.**

Once Datalink is running, it will populate the following required files:
* `/dev/shm/acpmf_physics`
* `/dev/shm/acpmf_graphics`
* `/dev/shm/acpmf_static`


## Installation & Usage

### Prerequisites
1. **Rust Toolchain:** Ensure you have [Rust installed](https://rustup.rs/).
2. **System Dependencies:** You will need pkg-config, Wayland, and Vulkan development libraries. Install them using your system's package manager:

**Ubuntu / Debian / Pop!_OS:**
```bash
sudo apt update
sudo apt install build-essential pkg-config libwayland-dev libxkbcommon-dev libvulkan-dev
```

**Fedora:**
```bash
sudo dnf install gcc pkgconf-pkg-config wayland-devel libxkbcommon-devel vulkan-loader-devel
```

**Arch Linux:**
```bash
sudo pacman -S base-devel pkgconf wayland libxkbcommon vulkan-icd-loader
```

*(Note for Nix users: A flake.nix is provided. Simply run nix develop to enter a ready-to-use development shell, or nix run to build and execute.)*

### Build and Run

1. **Clone the repository:**
   ```bash
   git clone https://github.com/faizm55212/SimTem
   cd SimTem
   ```
2. **Build for Release:**
   ```bash
   cargo build --release
   ```
3. **Run:**
   ```bash
   ./target/release/simtem
   ```

## Configuration (`car_data.rs`)

The overlay uses a local struct to define shift lights and brake bias offsets. Add or modify cars by editing the `CAR_MODELS` array in `car_data.rs`.

```rust
CarModelData {
    car_id: 53,
    shift_rpm: 6700,
    bb_offset: -22.0,
    brake_pressure_co: [7.2886, 10.0],
    max_steering_angle: 246,
    car_model: "bmw_m4_gt4",
}
```
