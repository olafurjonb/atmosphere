import os
import json
import subprocess
import tempfile
import uuid
from fastapi import FastAPI, Request, HTTPException
from fastapi.responses import HTMLResponse, FileResponse
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel

app = FastAPI(title="Atmosphere Spatial Audio Renderer")

# Create a directory to store compiled audio renders statically
STATIC_DIR = os.path.join(os.path.dirname(__file__), "static")
RENDERS_DIR = os.path.join(STATIC_DIR, "renders")
os.makedirs(RENDERS_DIR, exist_ok=True)

# Mount the static folder so the browser can stream the output files
app.mount("/static", StaticFiles(directory=STATIC_DIR), name="static")

# Helper to locate our compiled Rust Atmosphere binary
def get_compiler_path():
    candidates = [
        "/app/atmosphere",                     # Docker runtime path
        "./target/release/atmosphere",         # Local cargo release path
        "./atmosphere",                        # Local root path
        "atmosphere"                           # System path
    ]
    for c in candidates:
        if os.path.exists(c) and os.path.isfile(c):
            return c
    return "atmosphere" # Fallback to system command

# Root Route: Serves the visual editor in the browser
@app.get("/", response_class=HTMLResponse)
async def get_editor():
    editor_path = os.path.join(os.path.dirname(__file__), "editor.html")
    if not os.path.exists(editor_path):
        raise HTTPException(status_code=404, detail="editor.html not found")
    with open(editor_path, "r", encoding="utf-8") as f:
        return f.read()

# REST API Endpoint: Renders JAFF trajectory, compiles WAV, downmixes to Stereo
@app.post("/render")
async def render_scene(payload: dict):
    try:
        # Create a unique transaction ID for these files
        tx_id = str(uuid.uuid4())[:8]
        jaff_filename = f"scene_{tx_id}.jaff"
        output_wav_name = f"spatial_{tx_id}.wav"
        preview_wav_name = f"preview_{tx_id}.wav"

        jaff_path = os.path.join(RENDERS_DIR, jaff_filename)
        output_wav_path = os.path.join(RENDERS_DIR, output_wav_name)
        preview_wav_path = os.path.join(RENDERS_DIR, preview_wav_name)

        # 1. Write the JAFF payload to a physical file
        with open(jaff_path, "w", encoding="utf-8") as f:
            json.dump(payload, f, indent=2)

        # 2. Locate the Rust spatial compiler
        compiler_binary = get_compiler_path()

        # 3. Spawn the Rust compiler to build the 12-channel WAV
        print(f"[{tx_id}] Spawning Rust compiler: {compiler_binary} -- {jaff_path} -> {output_wav_path}")
        compile_res = subprocess.run([
            compiler_binary, jaff_path, output_wav_path
        ], capture_output=True, text=True, cwd=os.path.dirname(__file__))

        if compile_res.returncode != 0:
            print(f"[{tx_id}] Compiler failed: {compile_res.stderr}")
            raise HTTPException(status_code=500, detail=f"Rust Compilation Failed: {compile_res.stderr}")

        # 4. Spawn FFmpeg to perform an on-the-fly, equal-power stereo downmix for browser playback
        # Downmix matrix mapping for 7.1.4 (12 channels) to 2.0 Stereo (Left & Right):
        # Left Out (c0): L (c0) + 0.707*C (c2) + 0.707*Ls (c6) + 0.5*Lb (c4) + 0.707*Ltf (c8) + 0.5*Ltr (c10)
        # Right Out (c1): R (c1) + 0.707*C (c2) + 0.707*Rs (c7) + 0.5*Rb (c5) + 0.707*Rtf (c9) + 0.5*Rtr (c11)
        ffmpeg_cmd = [
            "ffmpeg", "-y", "-i", output_wav_path,
            "-filter_complex", 
            "pan=stereo|c0=c0+0.707*c2+0.707*c6+0.5*c4+0.707*c8+0.5*c10|c1=c1+0.707*c2+0.707*c7+0.5*c5+0.707*c9+0.5*c11",
            preview_wav_path
        ]
        print(f"[{tx_id}] Spawning FFmpeg stereo downmix: {' '.join(ffmpeg_cmd)}")
        ffmpeg_res = subprocess.run(ffmpeg_cmd, capture_output=True, text=True)

        if ffmpeg_res.returncode != 0:
            print(f"[{tx_id}] FFmpeg failed: {ffmpeg_res.stderr}")
            # Non-fatal: if ffmpeg fails, we can still return the master 12-channel file
            preview_url = None
        else:
            preview_url = f"/static/renders/{preview_wav_name}"

        # 5. Clean up the temporary JAFF file to save disk space
        try:
            os.remove(jaff_path)
        except OSError:
            pass

        # 6. Return the direct URLs for browser playback and downloading
        return {
            "success": True,
            "tx_id": tx_id,
            "master_url": f"/static/renders/{output_wav_name}",
            "preview_url": preview_url
        }

    except Exception as e:
        print(f"Render exception: {e}")
        raise HTTPException(status_code=500, detail=str(e))
