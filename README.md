# Agrivision

Software for mushroom growing system using gantry robot. The system is running on any 
Linux controller exposing at least 5 gpios for stepper motors and 1 gpio for watering motor. Additionally, a camera interface is needed to connect to the camera.

## Build

The software is developed using the `Rust` programming language. Toolchains needed to build including: 
- Rust toolchain installed via `rustup`([https://rustup.rs/](https://rustup.rs/))
- C toolchain for the host or cross toolchain for cross compile (gcc/clang)

Build: `cargo build --release [--target=<target machine>]`

Run: executable located in `./target/release/agrvision` for native build 
or in `./target/<target machine>/release/agrvision` for cross build.


## Run

Run: `./target/release/agrivision --help`

The software require config file to run. To create sample config file:

``` bash
./agrivision --template > config.toml
```

- In the config file edit the `chip` and `line` attritures for the gpio with `chip` is the 
gpio bus in the Linux system and line is the gpio number in the bus. Example, `CP2112` 
USP to SMBus ic support 8 gpio that can be used in the Linux system natively. After 
connect the `CP2112` to the machine, the kernel recognize it and expose an gpio chip in 
`/dev/gpiochipX` (`X` is newly added entry). Use this `chip` value and `line` from 0 to 7 to fill in the config file. \
For test system without gpio chip connected, use value `stub` for `chip` attribute to make it work without the robot.

- Like the gpio chip, a camera or webcam connected to the Linux system is exposing its interface in `/dev/videoX`, use this value to configure the video source.

- Addition component is needed to modified is the computer vision model in
`[detector.yolo_v8]` entry.
- Create a file for sqlite database using command `touch data.db` then set a environment variable `DATABASE_URL` as `sqlite://<path to data.db>`,
- Run with command `./agrivition --config-file config.toml`