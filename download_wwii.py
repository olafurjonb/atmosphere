import urllib.request
import os
import subprocess

def download_file(url, filepath):
    print(f"Downloading {url} -> {filepath}...")
    try:
        req = urllib.request.Request(
            url, 
            headers={'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)'}
        )
        with urllib.request.urlopen(req) as response:
            with open(filepath, 'wb') as f:
                f.write(response.read())
        print("Success!")
        return True
    except Exception as e:
        print(f"Failed to download {url}: {e}")
        return False

def convert_to_wav(input_path, output_path):
    print(f"Converting {input_path} -> {output_path} (48kHz Mono WAV)...")
    try:
        subprocess.run([
            "ffmpeg", "-y", "-i", input_path, 
            "-ac", "1", "-ar", "48000", output_path
        ], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        print("Conversion Success!")
        return True
    except Exception as e:
        print(f"Failed to convert {input_path}: {e}")
        return False

def main():
    os.makedirs("assets", exist_ok=True)
    
    # Define files to download and convert
    downloads = {
        "stuka_temp.mp3": {
            "url": "https://raw.githubusercontent.com/mattwright324/hll-sound-guide/main/sounds/ger%20precision.mp3",
            "output": "assets/wwii_stuka_dive.wav"
        },
        "spitfire_temp.ogg": {
            "url": "https://upload.wikimedia.org/wikipedia/commons/3/3c/Spitfire_fly_past_at_RAF_Halton.ogg",
            "output": "assets/wwii_spitfire_flyby.wav"
        },
        "mgun_temp.wav": {
            "url": "https://raw.githubusercontent.com/kdahlhaus/Pedal-Plane-Avionics/master/machguns.wav",
            "output": "assets/wwii_machine_gun.wav"
        }
    }
    
    for temp_name, data in downloads.items():
        temp_filepath = os.path.join("assets", temp_name)
        if download_file(data["url"], temp_filepath):
            convert_to_wav(temp_filepath, data["output"])
            # Clean up temporary downloaded file
            try:
                os.remove(temp_filepath)
            except OSError:
                pass

    # Reuse the explosion sound as our bomb explosion
    if os.path.exists("assets/tie_explosion.wav"):
        print("Duplicating explosion sound for WWII bomb...")
        try:
            import shutil
            shutil.copy("assets/tie_explosion.wav", "assets/wwii_bomb_explosion.wav")
            print("Explosion cloned successfully!")
        except Exception as e:
            print(f"Failed to copy explosion sound: {e}")

if __name__ == "__main__":
    main()
