# ASSC: Atmosphere Spatial Audio Compiler

**ASSC** is a high-performance spatial audio compiler written in **Rust**, designed to transform declarative object definitions into immersive spatial audio. The system processes a project file (**.jaff** — *JSON Atmosphere File Format*) and renders the final scene into a high-fidelity **WAV** file.

### Object Definition Schema (`.jaff`)
Each audio object within a `.jaff` scene is defined by its spatial trajectory and temporal properties:

* **Metadata:** * `title` (string)
    * `source_sound_file` (path)
* **Temporal Control:** * `start_offset` (seconds)
    * `end_offset` (seconds)
    * `loop` (boolean)
* **Spatial Trajectory:** * `startX`, `startY` (range: [-1.0, 1.0])
    * `startZ` (range: [0.0, 1.0])
    * `xformula(t)`, `yformula(t)`, `zformula(t)` (dynamic position relative to time `t`)
    * `volume(t)` (gain envelope)

### Core Compilation Workflow

1.  **Scene Calculation:** The compiler parses the `.jaff` definitions and resolves all time-variant formulas to calculate the exact spatial position of every object at any given sample frame.
2.  **Spatial Culling:** To optimize output and maintain audio integrity, the compiler implements a culling layer that clips or attenuates signals that drift beyond defined spatial boundaries.
3.  **Rendering Engine:** The engine performs real-time gain-calculation and trajectory interpolation. It renders the final scene into a multi-channel WAV file, applying the necessary panning laws (e.g., Constant Power Panning) to ensure accurate spatial representation in an Atmos-compatible environment.
