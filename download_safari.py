import urllib.request
import os

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

def main():
    os.makedirs("assets", exist_ok=True)
    
    sounds = {
        "lion.wav": "https://raw.githubusercontent.com/Sheel-ui/3d-audio-playground/master/sample/lion.wav",
        "elephant.wav": "https://raw.githubusercontent.com/Sheel-ui/3d-audio-playground/master/sample/elephant.wav",
        "bird.wav": "https://raw.githubusercontent.com/Sheel-ui/3d-audio-playground/master/sample/bird.wav",
    }
    
    for filename, url in sounds.items():
        filepath = os.path.join("assets", filename)
        download_file(url, filepath)

    # Fetch research-grade cricket chirps
    cricket_urls = [
        "https://raw.githubusercontent.com/gluijk/cricket-sound-analysis/master/grillo.wav",
        "https://raw.githubusercontent.com/gluijk/cricket-sound-analysis/master/grilloalicantino.wav"
    ]
    
    filepath = os.path.join("assets", "cricket.wav")
    for url in cricket_urls:
        if download_file(url, filepath):
            break

if __name__ == "__main__":
    main()
