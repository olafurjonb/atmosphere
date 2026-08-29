# ASSC: Atmosphere Spatial Audio Compiler 🔊🛰️

**ASSC** (Atmosphere Spatial Audio Compiler) is a high-performance spatial audio compiler written in **Rust**, designed to compile declarative 3D audio object definitions into immersive, cinema-grade spatial audio scenes. 

The system parses a scene definition file (**.jaff** — *JSON Atmosphere File Format*), evaluates mathematical trajectory expressions on-the-fly, simulates real-world physical acoustic propagation, and renders the final scene into a high-fidelity, fully compliant **12-channel 7.1.4 multi-channel WAV** file (`WAVEFORMATEXTENSIBLE`).

This project bridges the gap between open-source mathematical sound design and high-end consumer cinema hardware (like **Denon**, **Marantz**, or **Yamaha** AV Receivers) by outputting exact speaker channel masks and conforming to physical acoustic laws.

---

## Key Architectural Features 📐🌌

### 1. Phase-Coherent Acoustic Propagation (Time-of-Flight & Doppler)
Unlike standard panning tools that merely adjust volume, ASSC implements a **real-time physical model of sound wave propagation**:
*   **Time-of-Flight Delays:** Sound travels through air at approximately $343\text{ m/s}$. ASSC calculates the physical distance from each moving 3D object to the listener at the origin `(0.0, 0.0, 0.0)` for every single audio frame, dynamically delaying the signal to match real-world arrival times.
*   **Acoustic Doppler Effect:** As a virtual object speeds toward the listener, the propagation delay decreases, naturally compressing the waveform and raising the pitch. As it sweeps past and retreats, the waveform stretches, lowering the pitch. This creates organic, visceral pitch-bends (like a screaming jet plane) without any pre-baked effects.
*   **Absolute Phase Coherence:** Delays are calculated directly relative to the listener, eliminating destructive comb-filtering and ensuring razor-sharp acoustic localization.

### 2. Microsoft `WAVEFORMATEXTENSIBLE` Hardware Compliance
Standard multi-channel WAV files often fail to play on physical home theater systems over HDMI, causing receivers to drop channels or force a stereo downmix.
*   ASSC bypasses this by utilizing a custom-engineered binary WAV exporter that writes a fully compliant `WAVEFORMATEXTENSIBLE` header.
*   It injects the exact speaker channel layout mask **`0x2D63F`** (`SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT | SPEAKER_FRONT_CENTER | SPEAKER_LOW_FREQUENCY | SPEAKER_BACK_LEFT | SPEAKER_BACK_RIGHT | SPEAKER_SIDE_LEFT | SPEAKER_SIDE_RIGHT | SPEAKER_TOP_FRONT_LEFT | SPEAKER_TOP_FRONT_RIGHT | SPEAKER_TOP_BACK_LEFT | SPEAKER_TOP_BACK_RIGHT`).
*   This tells operating systems (Windows, Linux/PipeWire) and AV Receivers over **HDMI eARC** exactly how to route all 12 channels natively.

### 3. Constant-Power 3D Panning & Proximity Bass Boost
*   **Constant-Power Panning:** Employs a custom 3D trigonometric panning algorithm that maps sound energy across the flat 2D surround plane and the ceiling height plane, preserving equal perceived loudness (constant acoustic power) no matter where an object moves.
*   **Proximity Bass Boost:** Simulates close-up cabin pressure and tactile growls. When a sound source approaches the TV/listener boundary closer than a threshold distance, an acoustic low-shelf filter automatically amplifies low frequencies (<120Hz) by up to `6.0 dB`, sending a visceral power wave directly to your subwoofer.
*   **Automatic Peak Normalization:** If multiple screaming engines or explosions overlap, the combined amplitude can exceed `1.0` (causing digital clipping). ASSC analyzes the final mix in real-time, automatically normalizing the output down to `0.95` only if clipping is detected, preserving the absolute maximum dynamic range.

---

## 7.1.4 Output Channel Layout

The compiled multi-channel WAV file conforms to the standard 7.1.4 channel mapping order:

| Channel Index | Channel Name | Mask Bit | Description |
|:---:|:---:|:---:|---|
| `0` | **L** | `0x00001` | Front Left |
| `1` | **R** | `0x00002` | Front Right |
| `2` | **C** | `0x00004` | Center Front |
| `3` | **LFE** | `0x00008` | Low-Frequency Effects (Subwoofer / Bass Boost) |
| `4` | **Lb** | `0x00010` | Rear Left (Surround Back Left) |
| `5` | **Rb** | `0x00020` | Rear Right (Surround Back Right) |
| `6` | **Ls** | `0x00100` | Side Left (Surround Left) |
| `7` | **Rs** | `0x00200` | Side Right (Surround Right) |
| `8` | **Ltf** | `0x04000` | Left Top Front (Ceiling Height Front-Left) |
| `9` | **Rtf** | `0x08000` | Right Top Front (Ceiling Height Front-Right) |
| `10` | **Ltr** | `0x10000` | Left Top Rear (Ceiling Height Back-Left) |
| `11` | **Rtr** | `0x20000` | Right Top Rear (Ceiling Height Back-Right) |

---

## Installation & Setup

1.  **Prerequisites**: Ensure you have the Rust toolchain installed (Rust 1.80+ / 2024 edition).
2.  **Clone & Build**:
    ```bash
    git clone https://github.com/yourusername/atmosphere.git
    cd atmosphere
    cargo build --release
    ```
    The compiled binary will be located at `target/release/atmosphere`.

---

## Usage

Run the compiler by passing the path to the scene definition file (`.jaff`) and the desired output path (`.wav`):

```bash
./target/release/atmosphere <input.jaff> <output.wav>
```

---

## 🚀 Showroom Demos (Try These!)

To keep the repository **100% legal, clean, and lightweight**, ASSC does not bundle heavy copyrighted audio files. Instead, it includes a **Procedural Audio Synthesizer** that generates high-quality sound assets from pure mathematical equations directly on your machine!

Before compiling the demos, generate the synthetic assets:
```bash
python3 generate_assets.py
```
This will instantly synthesize laser zaps, helicopter rotors, Spitfire V12 engines, and Stuka wind whistles in your `assets/` directory.

### Demo 1: The WWII Battle of Britain Dogfight 🛩️💥
A historic aerial battle featuring engine takeoffs, close climbs, high-speed flybys, and a sudden ground-level rifle blast.
*   **The Scene:** A Stuka dive-bomber screams down from the ceiling. A bomb detonates on the floor, shaking the subwoofer. A Spitfire Merlin V12 chases it overhead, firing machine guns. 
*   **The Jump Scare:** After a brief silence, an authentic **M1 Garand Rifle Shot** blasts at ground-level directly behind your head (`startX: 0, startY: -1.0, startZ: 0`), panned 100% to the floor surrounds to make it sound like someone is standing in the room behind your couch!
*   **Compile:**
    ```bash
    ./target/release/atmosphere wwii_dogfight.jaff wwii_dogfight_output.wav
    ```
*   **Play:**
    ```bash
    mpv --audio-channels=7.1 wwii_dogfight_output.wav
    ```

### Demo 2: The 90s Sci-Fi Demoscene 🎹👾
An homage to the 1990s tracker-music demoscene culture, utilizing mathematical analog synth sweeps and orbiting objects.
*   **The Scene:** An analog synth arpeggio circles your head. A spaceship rumbles diagonally overhead, laser zaps shoot straight out of the ceiling height speakers, and a synthetic chopper blade hovers in a tight, fast horizontal **figure-eight (infinity loop)** on the heights.
*   **Compile:**
    ```bash
    ./target/release/atmosphere demo_scene.jaff sci_fi_demoscene.wav
    ```
*   **Play:**
    ```bash
    mpv --audio-channels=7.1 sci_fi_demoscene.wav
    ```

### Demo 3: The African Safari Ambient Showcase 🦁🐘
A gorgeous, slow, organic 3D soundstage designed to test low-frequency crossovers and perimeter surrounds.
*   **The Scene:** Lion growls deep in the subwoofer on the left, elephants blast on the right, crickets loop on the floor surround plane to envelop the room, and insects spin in a tight overhead circle on the ceiling.
*   *(Note: This demo downloads public-domain Creative Commons files from GitHub)*
*   **Fetch & Compile:**
    ```bash
    python3 download_safari.py
    ./target/release/atmosphere safari_scene.jaff safari_output.wav
    ```
*   **Play:**
    ```bash
    mpv --audio-channels=7.1 safari_output.wav
    ```

---

## Home Theater Playback & HDMI Bottlenecks 🔌🔊

If you are playing these files on a PC connected to an AV Receiver (like a Denon) over HDMI, you may run into physical hardware limits. Here is how to handle them:

1.  **The HDMI LPCM Limit:** The HDMI specification limits uncompressed, raw multi-channel audio (LPCM) to a maximum of **8 channels (7.1)**. 
2.  **Upmixing (Easiest Method):** Play the file in `mpv` as a 7.1 stream:
    ```bash
    mpv --audio-channels=7.1 wwii_dogfight_output.wav
    ```
    Grab your receiver's remote and set the Sound Mode to **"Dolby Surround"** or **"DTS Neural:X"**. The receiver's internal upmixer will analyze the mathematically panned surrounds and automatically route the appropriate 3D height data up to your 4 ceiling speakers!
3.  **True Dolby Atmos (Bitstreaming):** To achieve discrete 1:1 heights without upmixing, download the free **Dolby Atmos Conversion Tool** provided by Dolby. Convert ASSC's multi-channel WAV (which acts as a raw spatial template) into a Dolby Atmos bitstream, then use FFmpeg to mux it into an `.mp4` file. Play the MP4 with **Audio Bitstream Passthrough** enabled, and your receiver's screen will light up with **"Dolby Atmos"**, bypassing all HDMI uncompressed channel limits!

---

## Scene Definition Schema (`.jaff`)

Each scene is defined in JSON and contains an array of `objects`:

*   **`metadata`**:
    *   `title` *(string)*: Name of the audio object.
    *   `source_sound_file` *(string)*: Path to the mono source WAV file.
*   **`temporal`**:
    *   `start_offset` *(number)*: Start time in seconds.
    *   `end_offset` *(number, optional)*: End time in seconds (defaults to full duration).
    *   `loop` *(boolean)*: If true, loops the sound if the play window is longer than the file.
*   **`spatial`**:
    *   `startX`, `startY` *(number)*: Initial horizontal coordinate (`[-1.0, 1.0]`).
    *   `startZ` *(number)*: Initial height coordinate (`[0.0, 1.0]`).
    *   `xformula`, `yformula`, `zformula` *(string, optional)*: Mathematical expressions calculating trajectory offset over timeline time `t`. (e.g. `1.0 * cos(2 * pi * t * 0.2)`).
    *   `volume` *(string, optional)*: Expression calculating volume envelope over time `t`.
    *   `proximity_bass_boost` *(boolean, optional)*: Enables low-frequency proximity amplification.

### Expression Capabilities
The custom formula parser supports:
*   **Operators:** `+`, `-`, `*`, `/`, `%`, `^` (power)
*   **Constants:** `pi`, `e`
*   **Variables:** `t` (current active time)
*   **Functions:** `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sqrt`, `abs`, `exp`, `ln`, `floor`, `ceil`, `round`

---

## License 📄

This project is released under the **MIT License**. Feel free to use, modify, distribute, and integrate ASSC into your own commercial or open-source products. Let's make spatial audio accessible to everyone! 🚀

---

*Formulas, code, and acoustics panned with care. Built for the next-generation spatial computing web.*
