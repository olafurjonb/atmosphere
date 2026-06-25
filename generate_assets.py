import wave
import struct
import math
import os
import random

def write_wav(filename, samples, sample_rate=48000):
    with wave.open(filename, 'wb') as w:
        w.setnchannels(1)
        w.setsampwidth(2) # 16-bit
        w.setframerate(sample_rate)
        for s in samples:
            val = max(-1.0, min(1.0, s))
            val_i16 = int(val * 32767)
            w.writeframes(struct.pack('<h', val_i16))

def main():
    os.makedirs("assets", exist_ok=True)
    sr = 48000
    random.seed(42) # Deterministic noise

    print("Generating assets/spaceship.wav (Spaceship engine flyby hum)...")
    phase = 0.0
    samples = []
    for i in range(int(6.0 * sr)):
        t = i / sr
        # Frequency sweeps up and down slowly
        freq = 85.0 + 40.0 * math.sin(2.0 * math.pi * t * 0.5)
        phase += 2.0 * math.pi * freq / sr
        val = 0.5 * math.sin(phase) + 0.25 * math.sin(phase * 2.0)
        # Add high-frequency engine whine
        val += 0.1 * math.sin(phase * 4.5)
        # Add white noise for thrust rumble
        val += 0.12 * random.uniform(-1.0, 1.0)
        # Fades at boundaries
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
        t_cycle = t % 1.0 # 3 zaps, 1 per second
        if t_cycle < 0.4:
            # Exponential pitch sweep
            freq = 150.0 + 2350.0 * ((1.0 - t_cycle / 0.4) ** 3)
            phase += 2.0 * math.pi * freq / sr
            val = math.sin(phase)
            # Volume envelope for each zap
            val *= (1.0 - t_cycle / 0.4)
        else:
            val = 0.0
            phase = 0.0
        samples.append(val * 0.4)
    write_wav("assets/laser.wav", samples, sr)

    print("Generating assets/arpeggio.wav (Analog synthesizer arpeggio sequence)...")
    # A-minor chord arpeggio
    notes = [220.0, 261.63, 329.63, 392.00, 440.00, 523.25, 659.25, 783.99]
    phase = 0.0
    samples = []
    for i in range(int(10.0 * sr)):
        t = i / sr
        note_idx = int(t / 0.25) % len(notes)
        freq = notes[note_idx]
        phase += 2.0 * math.pi * freq / sr
        # Add harmonics based on resonant filter sweep
        filter_sweep = 0.5 + 0.5 * math.sin(2.0 * math.pi * t * 0.1)
        val = 0.0
        for h in range(1, 6):
            val += (1.0 / h) * math.sin(phase * h) * math.exp(-h * (1.0 - filter_sweep))
        # Tremolo (volume modulation)
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
        # Low frequency engine sound
        phase += 2.0 * math.pi * 65.0 / sr
        engine = 0.45 * math.sin(phase) + 0.15 * math.sin(phase * 3.0)
        # 10 Hz rotor modulation for chopping sound
        chopper = 0.5 + 0.5 * math.sin(2.0 * math.pi * t * 10.0)
        # Modulated white noise
        noise = random.uniform(-0.5, 0.5) * chopper
        val = engine + noise
        if t < 1.0:
            val *= t
        elif t > 7.0:
            val *= (8.0 - t)
        samples.append(val * 0.3)
    write_wav("assets/rotor.wav", samples, sr)

    print("All assets successfully generated!")

if __name__ == "__main__":
    main()
