# Citra Engine
A Nintendo 3DS first Game Engine with cross-platform support

## Licensing
The entire project as the sum of its parts and all code in all folders besides the [n3ds_target](n3ds_target/) folder is licensed under GPLv3. I have licensed the contents of [n3ds_target](n3ds_target/) folder as MIT because writing code for the 3DS is hard due to poor documentation (Or maybe I didn't look hard enough and Phind AI is dumb). This is so anyone who wishes to make their own 3ds game can use my code as foundation to work off of.

## Cross-Platform Design
I'm making the game engine ([citra_engine](citra_engine/)) and platform-specific routines (any folder that ends in "_target") seperate so that the game engine can be as portable as possible,
right now I plan on making a Nintendo 3DS version and PC Version (Win/Mac/Linux) of the game.
</br>
If you want to see the game engine target another platform, feel free to make a feature request for it. Or if you're feeling adventurous you can check out [PORTING.md](PORTING.md) to see how you could make a target for the game engine yourself.

## Getting Started
### Building
System Dependencies:
- devkitpro
- devkitarm (arm-none-eabi)
- libctru
- citro3d
- citro2d
- cargo-3ds

To build your game for n3ds, run:
```bash
cargo 3ds build -p n3ds_target
```
To build for pc, run:
```bash
cargo build -p pc_target
```
### Project Structure
[assets](./assets/) contains READ ONLY data. For now, any read-write data must be created at runtime manually with a script.
</br>
[game](./game/) is where you will write your game code, every other package can be ignored.
</br>
You can set your game version, description, and authors in [Cargo.toml](./Cargo.toml) at the `[workspace.package]` section
</br>
Due to a limitation of the Cargo.toml system, you have to go to each *_target/Cargo.toml and manually set name at the `[[bin]]` section. Do not set name at the `[package]` section, it'll break the game engine.
#### Caveats
3ds textures need to be handled in a special way. By default all png files will be dealt with automatically but if you wish to override this with your own behaviour, make a t3s file with the same name as the png file.
