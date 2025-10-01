# Citra Engine
A Nintendo 3DS first Game Engine with cross-platform support

## Licensing
The entire project as the sum of its parts and all code in all folders besides the [n3ds_platform](n3ds_platform/) folder is licensed under GPLv3. I have licensed the contents of [n3ds_platform](n3ds_platform/) folder as MIT because writing code for the 3DS is hard due to poor documentation (Or maybe I didn't look hard enough and Phind AI is dumb). This is so anyone who wishes to make their own 3ds game can use my code as foundation to work off of.

## Cross-Platform Design
I'm making the game engine ([citra_engine](citra_engine/)) and platform-specific routines (any folder that ends in "_platform") seperate so that the game engine can be as portable as possible,
right now I plan on making a Nintendo 3DS version and PC Version (Win/Mac/Linux) of the game.
</br>
If you want to see the game engine target another platform, feel free to make a feature request for it. Or if you're feeling adventurous you can check out [PORTING.md](PORTING.md) to see how you could make a target for the game engine yourself.

## Building (DEPRECATED!)
### Requirements
#### 3DS
- GNU Make
- DevkitPro 3DS Packages
    - devkitARM
    - libctru
    - citro3d
    - citro2d
    - 3ds-pkg-config
    - 3ds-libvorbisidec
    - 3ds-opusfile

#### PC
- GNU Make
- CMake
- SDL3 

### Building

#### Linux

targeting 3DS:
```sh
make 3dsx
```

targeting linux:
```sh
make linux
```