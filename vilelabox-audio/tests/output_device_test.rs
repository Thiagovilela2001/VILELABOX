use cpal::traits::{DeviceTrait, HostTrait};

#[test]
fn print_default_audio_device() {
    let host = cpal::default_host();
    println!("Host: {:?}", host.id());
    if let Some(device) = host.default_output_device() {
        println!("Default output device: {:?}", device.name());
        if let Ok(config) = device.default_output_config() {
            println!("Default config: {:?}", config);
            println!("Sample format: {:?}", config.sample_format());
            println!("Channels: {:?}", config.channels());
            println!("Sample rate: {:?}", config.sample_rate());
        } else {
            println!("Could not get default output config!");
        }
    } else {
        println!("No default output device found!");
    }
}
