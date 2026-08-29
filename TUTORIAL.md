# Tutorial: Writing JAFF (JSON Atmosphere File Format) Files 📐🎚️

Welcome to **Atmosphere**! This tutorial will teach you how to write and customize **JAFF** (`.jaff`) files. 

JAFF is a declarative, human-readable JSON format that acts as a "3D spatial choreography script." By writing simple mathematical coordinate equations, you can define exactly where sounds move, rise, fall, and accelerate in a 3D theater space over time.

---

## 1. The Structure of a `.jaff` File

At its core, a JAFF file is a JSON object with a `title` and an array of `objects`. Each object represents a single audio track (stem) that is panned in 3D space:

```json
{
  "title": "My First 3D Scene",
  "objects": [
    {
      "metadata": {
        "title": "Helicopter",
        "source_sound_file": "assets/rotor.wav"
      },
      "temporal": {
        "start_offset": 0.0,
        "end_offset": 10.0,
        "loop": true
      },
      "spatial": {
        "startX": 0.0,
        "startY": 0.0,
        "startZ": 0.5,
        "xformula": "0.8 * cos(2 * pi * t * 0.2)",
        "yformula": "0.8 * sin(2 * pi * t * 0.2)",
        "zformula": "0.1 * sin(pi * t * 0.1)",
        "volume": "1.0",
        "proximity_bass_boost": false
      }
    }
  ]
}
```

---

## 2. Understanding the 3D Coordinate Grid 📐

Atmosphere uses a Cartesian grid normalized between `-1.0` and `1.0` representing a listening room. The listener is always sitting at the absolute origin `(0.0, 0.0, 0.0)` facing forward.

```text
               FRONT (TV / Stage)
                   Y = +1.0
                      ▲
                      │
                      │
   LEFT  ◄────────────┼────────────►  RIGHT
 X = -1.0             │ (0,0,0)       X = +1.0
                      │ Listener
                      │
                      ▼
                   Y = -1.0
                 BACK (Couch)
```

*   **X Axis (Left/Right):**
    *   `-1.0` is the far **Left** speaker wall.
    *   `0.0` is the **Center** of the room.
    *   `1.0` is the far **Right** speaker wall.
*   **Y Axis (Back/Front):**
    *   `-1.0` is the far **Back** wall (behind your couch).
    *   `0.0` is the **Center** of the room (where you are sitting).
    *   `1.0` is the far **Front** wall (where your TV/Screen is).
*   **Z Axis (Floor/Ceiling Height):**
    *   `0.0` is **Ear level** (ground plane / flat surround speakers).
    *   `1.0` is the **Ceiling** (Top height speakers).

---

## 3. Timeline & Timing (`temporal`) ⏱️

The `temporal` block controls when your sound file starts, stops, and whether it loops:

*   **`start_offset`**: The exact second in your scene's timeline when this sound starts playing. (e.g. `start_offset: 2.5` means the sound stays silent until 2.5 seconds into the demo).
*   **`end_offset`** *(Optional)*: The exact second when the sound stops playing. If omitted, the compiler will play the source WAV file until it naturally ends.
*   **`loop`**: If `true`, and the duration between `start_offset` and `end_offset` is longer than the raw WAV file, the compiler will loop the audio seamlessly.

---

## 4. Choreographing Trajectories with Math 🧮

This is where the magic happens! Instead of setting static coordinates, you can use mathematical formulas to make sounds fly around the room dynamically. 

The compiler calculates the position at any active moment using:
$$\text{Calculated Position} = \text{Start Coordinate} + \text{Formula Offset}$$

*   `startX`, `startY`, `startZ`: The starting anchor point of the object.
*   `xformula`, `yformula`, `zformula`: The mathematical equation representing the movement offset.
*   `volume` *(Optional)*: An equation controlling the volume envelope over time. (Defaults to `"1.0"`).

### 🔑 The Global Variable: `t`
Every formula is evaluated relative to **`t`**, which represents the **active duration** (in seconds) that the sound has been playing.
*   `t` starts at `0.0` exactly when the object enters the timeline (`start_offset`).
*   If your sound starts at `start_offset: 5.0`, when the master clock hits 6.5 seconds, the variable `t` inside your formula evaluates to `1.5` seconds.

### Available Constants & Math Functions
You can use standard mathematical operations and a full suite of trigonometric and exponential functions:

*   **Constants:** `pi` ($\approx 3.14159$), `e` ($\approx 2.71828$)
*   **Operators:** `+`, `-`, `*`, `/`, `%` (modulo), `^` (power)
*   **Basic Trig:** `sin(x)`, `cos(x)`, `tan(x)` *(Angles are in radians!)*
*   **Inverse Trig:** `asin(x)`, `acos(x)`, `atan(x)`
*   **Exponents/Logs:** `exp(x)`, `ln(x)`
*   **Modifiers:** `sqrt(x)` (square root), `abs(x)` (absolute value)
*   **Rounding:** `floor(x)`, `ceil(x)`, `round(x)`

---

## 5. Walkthrough: Writing Custom Trajectories

Here are four classic panned movements you can copy-paste and customize:

### A. The Perfect Ceiling Orbit (Circle)
To make a sound circle perfectly overhead, we use standard sine and cosine math.
*   **X Formula (Cosine):** `radius * cos(2 * pi * t * rate)`
*   **Y Formula (Sine):** `radius * sin(2 * pi * t * rate)`
*   **Z Anchor:** `startZ: 1.0` (ceiling level)

```json
"spatial": {
  "startX": 0.0,
  "startY": 0.0,
  "startZ": 1.0,
  "xformula": "0.9 * cos(2 * pi * t * 0.2)",
  "yformula": "0.9 * sin(2 * pi * t * 0.2)",
  "zformula": "0.0"
}
```
*💡 Explanation:* This sound orbits at a radius of `0.9` (extremely wide near the walls) at a rate of `0.2` Hz (meaning it takes exactly 5 seconds to complete one full circle: $1 / 0.2 = 5$).

---

### B. The 3D Roller Coaster (Infinity Loop)
By doubling the frequency on one axis, you create a **Figure-Eight (Lissajous) curve**. We will also make it rise and fall as it loops!
```json
"spatial": {
  "startX": 0.0,
  "startY": 0.0,
  "startZ": 0.5,
  "xformula": "0.85 * sin(2 * pi * t * 0.1)",
  "yformula": "0.5 * sin(4 * pi * t * 0.1)",
  "zformula": "0.3 * cos(2 * pi * t * 0.1)"
}
```
*💡 Explanation:*
*   The X coordinate swings left and right once every 10 seconds.
*   The Y coordinate swings back and forth twice as fast (once every 5 seconds).
*   The Z coordinate starts high (`0.5 + 0.3 = 0.8`), drops low (`0.5 - 0.3 = 0.2`), and rises back up, tracing a beautiful vertical wave!

---

### C. The High-Speed Flyby (Linear Sweep)
To simulate a jet plane or spaceship screaming past the couch, we use a simple linear progression:
```json
"spatial": {
  "startX": -2.0,
  "startY": 2.0,
  "startZ": 0.8,
  "xformula": "0.8 * t",
  "yformula": "-0.8 * t",
  "zformula": "0.0"
}
```
*💡 Explanation:*
*   **StartX/Y:** Starts far off-screen in the front-left (`x=-2.0, y=2.0`).
*   **Offset:** For every second that passes, it travels `0.8` units to the right (`+X`) and `0.8` units toward the back (`-Y`). 
*   **Crossing Point:** At exactly `t = 2.5` seconds, the offsets evaluate to `+2.0` and `-2.0`, placing the sound at exactly `(0.0, 0.0, 0.8)`—directly overhead. Our **Doppler Engine** will automatically pitch-bend the sound perfectly at this split-second crossing!

---

### D. The Dynamic Volume Envelope (Fade-In/Fade-Out)
You can use mathematical curves to slowly fade sounds in or create rhythmic pulsing:
```json
"spatial": {
  "startX": 0.0,
  "startY": 0.0,
  "startZ": 0.0,
  "volume": "0.5 + 0.5 * sin(2 * pi * t * 2.0)"
}
```
*💡 Explanation:* The volume swings smoothly between `0.0` (silent) and `1.0` (maximum) twice per second (2.0Hz rate), creating a pulsing acoustic strobe effect!

---

## 6. Advanced Features 💥

### Proximity Bass Boost (`proximity_bass_boost`)
Set this to `true` to enable an immersive physical growl or chest slam. 
*   **How it works:** When your moving virtual object approaches the front-center TV boundary closer than `0.2` units, the compiler engages an acoustic crossover filter that boosts low frequencies (<120Hz) by up to **`+6.0 dB`**. 
*   This sends a physical impact shockwave directly to your subwoofer (LFE channel) only when the object is extremely close to the screen, heightening the realism of close-up explosions, engine revs, or growls!

```json
"spatial": {
  "startX": 0.0,
  "startY": 1.1,
  "startZ": 0.0,
  "proximity_bass_boost": true
}
```

---

## 7. Compiling Your Creation

Once you have written your `.jaff` file, compile it into a fully immersive 7.1.4 WAV file in your terminal:

```bash
./target/release/atmosphere my_scene.jaff output_multichannel.wav
```

Now grab your remote, set your AV Receiver to **Dolby Surround**, and experience your mathematical audio choreography come to life! 🚀🔊
