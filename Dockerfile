# Stage 1: Build high-performance Rust Spatial Audio Compiler
FROM rust:1.80-slim-bookworm AS rust-builder
WORKDIR /usr/src/atmosphere
COPY . .
RUN cargo build --release --bin atmosphere

# Stage 2: Build Python FastAPI Web Service & Media Runtime
FROM python:3.11-slim-bookworm
WORKDIR /app

# Install FFmpeg for real-time stereo downmixing, curl for health checks
RUN apt-get update && apt-get install -y ffmpeg curl && rm -rf /var/lib/apt/lists/*

# Copy the compiled Rust spatial compiler binary from Stage 1
COPY --from=rust-builder /usr/src/atmosphere/target/release/atmosphere /app/atmosphere

# Copy requirements and install python backend stack
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy all application code, models, static files, and presets
COPY . .

# Generate all procedural WWII and sci-fi demo sound effects at container build time
# This ensures that our 7.1.4 demos work out-of-the-box immediately!
RUN python3 generate_assets.py

# Expose the backend API port
EXPOSE 8000

# Run uvicorn on all interfaces so it's accessible from host and network
CMD ["uvicorn", "app:app", "--host", "0.0.0.0", "--port", "8000"]
