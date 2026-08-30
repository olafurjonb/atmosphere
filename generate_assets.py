import wave
import struct
import math
import os
import random
import subprocess

def write_wav(filename, samples, sample_rate=48000):
    with wave.open(filename, 'wb') as w:
        w.setnchannels(1)
        w.setsampwidth(2) # 16-bit
        w.setframerate(sample_rate)
        for s in samples:
            val = max(-1.0, min(1.0, s))
            val_i16 = int(val * 32767)
            w.writeframes(struct.pack('<h', val_i16))

# Helper to run FFmpeg conversion
def convert_mp3_to_wav(mp3_path, wav_path):
    print(f"Converting {mp3_path} -> {wav_path} (48kHz Mono)...")
    try:
        subprocess.run([
            "ffmpeg", "-y", "-i", mp3_path, 
            "-ac", "1", "-ar", "48000", wav_path
        ], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        print("Success!")
        return True
    except Exception as e:
        print(f"FFmpeg conversion failed for {mp3_path}: {e}")
        return False

def main():
    os.makedirs("assets", exist_ok=True)
    sr = 48000
    random.seed(42) # Deterministic noise

    # --- PART 1: FUZZY KEYWORD MATCHING FOR USER'S ROYALTY-FREE MP3 RECORDINGS ---
    print("\n--- Processing Royalty-Free Audio Recordings (Fuzzy Keyword Matching) ---")
    rf_dir = "assets/royality-free"
    
    # Keyword matchers mapped to their expected compiler target filenames
    # This allows file names with extra numbers, suffixes, or hashes to match cleanly!
    keyword_matchers = [
        ("spitfire", "assets/real_spitfire_close.wav"),
        ("fly-past", "assets/real_ww2_flypast.wav"),
        ("fly_past", "assets/real_ww2_flypast.wav"),       # Fallback spellings
        ("flypast", "assets/real_ww2_flypast.wav"),        # Fallback spellings
        ("takeoff", "assets/real_ww2_takeoff.wav"),
        ("take-off", "assets/real_ww2_takeoff.wav"),       # Fallback spellings
        ("low-airplane-fly-by", "assets/real_low_flyby.wav"),
        ("airplane-fly-by", "assets/real_low_flyby.wav"),   # Fallback spellings
        ("garand", "assets/real_m1_garand.wav"),
        ("m1", "assets/real_m1_garand.wav")                 # Fallback spellings
    ]

    if os.path.exists(rf_dir):
        all_files = os.listdir(rf_dir)
        processed_targets = set()

        for kw, target_wav in keyword_matchers:
            # Skip if we already successfully compiled a file for this target
            if target_wav in processed_targets:
                continue

            # Look for any .mp3 file in the directory containing our keyword (case-insensitive)
            matched_file = None
            for f in all_files:
                if kw in f.lower() and f.lower().endswith(".mp3"):
                    matched_file = f
                    break
            
            if matched_file:
                mp3_path = os.path.join(rf_dir, matched_file)
                if convert_mp3_to_wav(mp3_path, target_wav):
                    processed_targets.add(target_wav)
            else:
                # Soft warning, don't crash the script
                print(f"Notice: No local MP3 found containing keyword '{kw}' for target '{target_wav}'.")
    else:
        print(f"Warning: Directory '{rf_dir}' does not exist.")

    # --- PART 2: SYNTHESIZE PROCEDURAL MATHEMATICAL WAVEFORMS ---
    print("\n--- Synthesizing Procedural Mathematical Audio ---")

    print("Generating assets/spaceship.wav (Spaceship engine flyby hum)...")
    phase = 0.0
    samples = []
    for i in range(int(6.0 * sr)):
        t = i / sr
        freq = 85.0 + 40.0 * math.sin(2.0 * math.pi * t * 0.5)
        phase += 2.0 * math.pi * freq / sr
        val = 0.5 * math.sin(phase) + 0.25 * math.sin(phase * 2.0)
        val += 0.1 * math.sin(phase * 4.5)
        val += 0.12 * random.uniform(-1.0, 1.0)
        if t < 1.0:
            val *= t
        elif t > 5.0:
            val *= (6.0 - t)
        samples.append(val * 0.5)
    write_wav("assets/spaceship.wav", samples, sr)

    print("Generating assets/laser.wav (Rapid sci-fi laser zaps)...")
    phase = 0.0
    samples = []
    for i in range(int(3.0 * sr)):
        t = i / sr
        t_cycle = t % 1.0
        if t_cycle < 0.4:
            freq = 150.0 + 2350.0 * ((1.0 - t_cycle / 0.4) ** 3)
            phase += 2.0 * math.pi * freq / sr
            val = math.sin(phase)
            val *= (1.0 - t_cycle / 0.4)
        else:
            val = 0.0
            phase = 0.0
        samples.append(val * 0.4)
    write_wav("assets/laser.wav", samples, sr)

    print("Generating assets/arpeggio.wav (Analog synthesizer arpeggio sequence)...")
    notes = [220.0, 261.63, 329.63, 392.00, 440.00, 523.25, 659.25, 783.99]
    phase = 0.0
    samples = []
    for i in range(int(10.0 * sr)):
        t = i / sr
        note_idx = int(t / 0.25) % len(notes)
        freq = notes[note_idx]
        phase += 2.0 * math.pi * freq / sr
        filter_sweep = 0.5 + 0.5 * math.sin(2.0 * math.pi * t * 0.1)
        val = 0.0
        for h in range(1, 6):
            val += (1.0 / h) * math.sin(phase * h) * math.exp(-h * (1.0 - filter_sweep))
        tremolo = 0.85 + 0.15 * math.sin(2.0 * math.pi * t * 6.0)
        val *= tremolo
        if t < 0.5:
            val *= (t / 0.5)
        elif t > 9.5:
            val *= ((10.0 - t) / 0.5)
        samples.append(val * 0.25)
    write_wav("assets/arpeggio.wav", samples, sr)

    print("Generating assets/rotor.wav (Overhead helicopter rotor blades)...")
    phase = 0.0
    samples = []
    for i in range(int(8.0 * sr)):
        t = i / sr
        phase += 2.0 * math.pi * 65.0 / sr
        engine = 0.45 * math.sin(phase) + 0.15 * math.sin(phase * 3.0)
        chopper = 0.5 + 0.5 * math.sin(2.0 * math.pi * t * 10.0)
        noise = random.uniform(-0.5, 0.5) * chopper
        val = engine + noise
        if t < 1.0:
            val *= t
        elif t > 7.0:
            val *= (8.0 - t)
        samples.append(val * 0.3)
    write_wav("assets/rotor.wav", samples, sr)

    # World War II Procedural Synthesis Assets
    print("Generating assets/wwii_stuka_dive.wav (German Ju-87 Dive Whistle)...")
    phase_eng = 0.0
    phase_siren = 0.0
    samples = []
    for i in range(int(6.0 * sr)):
        t = i / sr
        phase_eng += 2.0 * math.pi * 82.0 / sr
        engine = 0.4 * math.sin(phase_eng) + 0.2 * math.sin(phase_eng * 2.0)
        
        siren_freq = 550.0 + 800.0 * ((t / 6.0) ** 1.8)
        phase_siren += 2.0 * math.pi * siren_freq / sr
        
        chop = 0.45 + 0.55 * math.sin(2.0 * math.pi * t * 9.5)
        siren = math.sin(phase_siren) * chop * ((t / 6.0) ** 1.2)
        
        val = engine + siren * 0.65
        if t < 0.5:
            val *= (t / 0.5)
        elif t > 5.5:
            val *= ((6.0 - t) / 0.5)
        samples.append(val * 0.45)
    write_wav("assets/wwii_stuka_dive.wav", samples, sr)

    print("Generating assets/wwii_spitfire_flyby.wav (Allied Spitfire Rolls-Royce Merlin V12)...")
    phase_m1 = 0.0
    phase_m2 = 0.0
    phase_m3 = 0.0
    samples = []
    for i in range(int(8.0 * sr)):
        t = i / sr
        phase_m1 += 2.0 * math.pi * 92.0 / sr
        phase_m2 += 2.0 * math.pi * 92.8 / sr
        phase_m3 += 2.0 * math.pi * 91.2 / sr
        
        merlin = 0.35 * math.sin(phase_m1) + 0.25 * math.sin(phase_m1 * 2.0) + 0.15 * math.sin(phase_m1 * 3.0)
        merlin += 0.25 * math.sin(phase_m2) + 0.15 * math.sin(phase_m3)
        combustion = random.uniform(-0.15, 0.15) * math.sin(2.0 * math.pi * t * 45.0)
        merlin += combustion + random.uniform(-0.1, 0.1)
        
        val = merlin
        if t < 1.0:
            val *= t
        elif t > 7.0:
            val *= (8.0 - t)
        samples.append(val * 0.35)
    write_wav("assets/wwii_spitfire_flyby.wav", samples, sr)

    print("Generating assets/wwii_machine_gun.wav (Wing-Mounted .303 Browning Machine Gun Bursts)...")
    ping_phase = 0.0
    samples = []
    for i in range(int(3.0 * sr)):
        t = i / sr
        gun_cycle = t % 0.0909
        if gun_cycle < 0.045:
            burst = random.uniform(-1.0, 1.0)
            envelope = math.exp(-120.0 * gun_cycle)
            val = burst * envelope
            ping_freq = 1400.0 * math.exp(-250.0 * gun_cycle)
            ping_phase += 2.0 * math.pi * ping_freq / sr
            val += 0.35 * math.sin(ping_phase) * envelope
        else:
            val = 0.0
            ping_phase = 0.0
        samples.append(val * 0.45)
    write_wav("assets/wwii_machine_gun.wav", samples, sr)

    print("Generating assets/wwii_bomb_explosion.wav (Deep Sub-Bass Ordnance Blast)...")
    rumble_phase = 0.0
    samples = []
    for i in range(int(4.0 * sr)):
        t = i / sr
        blast = random.uniform(-1.0, 1.0) * math.exp(-18.0 * t)
        rumble_freq = 42.0 + 8.0 * math.sin(2.0 * math.pi * t * 4.5)
        rumble_phase += 2.0 * math.pi * rumble_freq / sr
        rumble = math.sin(rumble_phase)
        
        envelope = math.exp(-2.2 * t)
        val = (blast + 0.85 * rumble) * envelope
        samples.append(val * 0.6)
    write_wav("assets/wwii_bomb_explosion.wav", samples, sr)

    print("\nAll assets successfully synchronized and generated!")

if __name__ == "__main__":
    main()
