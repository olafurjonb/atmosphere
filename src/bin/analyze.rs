use hound::WavReader;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut reader = WavReader::open(&args[1]).unwrap();
    let spec = reader.spec();
    let channels = spec.channels as usize;
    println!("Channels: {}", channels);
    
    let mut peaks = vec![0_f32; channels];
    let mut current_channel = 0;
    
    let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
    
    for sample in samples {
        let abs_val = (sample as f32 / i16::MAX as f32).abs();
        if abs_val > peaks[current_channel] {
            peaks[current_channel] = abs_val;
        }
        current_channel = (current_channel + 1) % channels;
    }
    
    for (i, peak) in peaks.iter().enumerate() {
        println!("Channel {}: Peak = {:.4}", i, peak);
    }
}
