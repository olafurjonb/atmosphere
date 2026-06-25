# ASSC: Atmosphere Spatial Audio Compiler

**ASSC** is a high-performance spatial audio compiler written in **Rust**, designed to transform declarative 3D audio object definitions into immersive spatial audio scenes. The system processes a scene definition file (**.jaff** — *JSON Atmosphere File Format*) and renders the final scene into a high-fidelity **7.1.4 multi-channel WAV** file.

---

## Features

- 🚀 **High-Performance Rendering**: Written in Rust for low-level audio efficiency and safety.
- 📐 **Dynamic 3D Trajectories**: Compile and evaluate mathematical expressions (e.g., `sin(t)`, `cos(t)`, `pi`) on-the-fly to compute precise positions `(x(t), y(t), z(t))` and volume envelopes `volume(t)`.
- 🎚️ **7.1.4 Constant-Power Panning**: Performs seamless 3D trigonometric panning mapping to Front (L/C/R), Side (Ls/Rs), Rear (Lb/Rb), and Height channels (Ltf/Rtf/Ltr/Rtr) with automatic power preservation.
- 🔇 **Spatial Culling Layer**: Gradually attenuates signals using distance-based linear fading if an object drifts beyond the physical spatial boundaries.
- 🔄 **On-the-fly Resampling**: Built-in high-quality linear resampler allows using source WAVs with any sample rate.
- 🎛️ **Automatic Peak Normalization**: Automatically scales down the entire mix to prevent digital clipping if multiple concurrent sources exceed `1.0` amplitude.
- ⚡ **Multi-channel Downmixing**: Automatically downmixes multi-channel source files into mono on-the-fly to represent them as precise spatial point-sources.

---

## 7.1.4 Output Channel Layout

The compiled multi-channel WAV file conforms to the standard 7.1.4 channel mapping order:

| Channel Index | Channel Name | Description |
|---|---|---|
| `0` | **L** | Left Front |
| `1` | **R** | Right Front |
| `2` | **C** | Center Front |
| `3` | **LFE** | Low-Frequency Effects (Subwoofer - Silent during panning) |
| `4` | **Ls** | Left Surround (Side) |
| `5` | **Rs** | Right Surround (Side) |
| `6` | **Lb** | Left Back (Rear) |
| `7` | **Rb** | Right Back (Rear) |
| `8` | **Ltf** | Left Top Front (Height) |
| `9` | **Rtf** | Right Top Front (Height) |
| `10` | **Ltr** | Left Top Rear (Height) |
| `11` | **Rtr** | Right Top Rear (Height) |

---

## Installation & Setup

1. **Prerequisites**: Ensure you have the Rust toolchain installed (Rust 1.80+ / 2024 edition).
2. **Clone & Build**:
   ```bash
   cargo build --release
   ```
   The compiled executable will be located at `target/release/atmosphere`.

---

## Usage

Run the compiler by passing the path to the scene definition file (`.jaff`) and the desired output path (`.wav`):

```bash
cargo run --release -- <input.jaff> <output.wav>
```

Or run the compiled binary directly:

```bash
./target/release/atmosphere <input.jaff> <output.wav>
```

---

## Scene Definition Schema (`.jaff`)

Each scene contains a list of audio objects. The schema supports both a top-level `objects` array wrapping or a raw flat array of objects.

### Fields Guide
- **`metadata`**:
  - `title` *(string)*: Descriptive title of the object.
  - `source_sound_file` *(string)*: Path to the source WAV file (can be absolute or relative to the directory of the `.jaff` file).
- **`temporal`**:
  - `start_offset` *(number)*: Timeline offset in seconds when the object starts playing.
  - `end_offset` *(number, optional)*: Timeline offset in seconds when the object stops playing. If omitted, the compiler plays the full duration of the source sound file.
  - `loop` *(boolean)*: If true, loops the source sound file if the duration between `start_offset` and `end_offset` exceeds the file's natural duration.
- **`spatial`**:
  - `startX`, `startY` *(number)*: Start coordinate on the horizontal plane (range: `[-1.0, 1.0]`).
  - `startZ` *(number)*: Start coordinate on the height plane (range: `[0.0, 1.0]`).
  - `xformula`, `yformula`, `zformula` *(string, optional)*: Math formula evaluating trajectory offsets relative to time `t`. Positions are computed as `x(t) = startX + xformula(t)`.
  - `volume` *(string, optional)*: Math formula evaluating the volume envelope based on time `t` (defaults to `1.0` if omitted).

### Formula Expression Capabilities
The expression evaluator supports:
- **Operators**: `+`, `-`, `*`, `/`, `%`, `^` (exponentiation)
- **Constants**: `pi`, `e`
- **Variables**: `t` (current timeline time in seconds)
- **Trig & Math Functions**: `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sqrt`, `abs`, `exp`, `ln`, `floor`, `ceil`, `round`

### Complete Example (`scene.jaff`)
```json
{
  "title": "Forest Meadow Scene",
  "objects": [
    {
      "metadata": {
        "title": "Circling Bird",
        "source_sound_file": "assets/bird_chirp.wav"
      },
      "temporal": {
        "start_offset": 2.0,
        "end_offset": 10.0,
        "loop": true
      },
      "spatial": {
        "startX": 0.0,
        "startY": 0.0,
        "startZ": 0.5,
        "xformula": "0.8 * sin(2 * pi * t * 0.1)",
        "yformula": "0.8 * cos(2 * pi * t * 0.1)",
        "zformula": "0.2 * sin(pi * t * 0.05)",
        "volume": "1.0 - (t * 0.02)"
      }
    }
  ]
}
```

---

## Coordinate & Culling System

- **Coordinate Limits**:
  - **Left / Right (X)**: `[-1.0, 1.0]` (Left is `-1.0`, Center is `0.0`, Right is `1.0`)
  - **Back / Front (Y)**: `[-1.0, 1.0]` (Back is `-1.0`, Center is `0.0`, Front is `1.0`)
  - **Height (Z)**: `[0.0, 1.0]` (Ear level is `0.0`, Ceiling is `1.0`)
- **Spatial Culling**: If an object's calculated position `(x, y, z)` drifts outside the bounds, it calculates the Euclidean distance to the nearest point on the bounding box. The compiler applies a linear fade-out attenuation, dropping the gain to `0.0` over $1.0$ unit of out-of-bounds distance. This guarantees click-free entry and exit from the listening space.

---

## Verification & Testing

ASSC is backed by a robust and comprehensive test suite validating schema deserialization, 3D math trajectory compiler accuracy, culling attenuation calculations, constant power speaker routing, and an end-to-end multi-channel rendering simulation.

To run all unit and integration tests:
```bash
cargo test
```
