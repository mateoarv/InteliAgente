use std::fs::File;
use std::io::BufWriter;
use std::{path, thread};
use std::time::{Duration, Instant};
use cpal::{BufferSize, Device, FrameCount, SampleFormat, SampleRate, Stream, StreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound;
use hound::{WavSpec, WavWriter};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

const BASE_RATE: u32 = 48000;

pub struct Recorder {
    device: Device,
    stream_config: StreamConfig,
    wav_spec: WavSpec,
}



impl Recorder {
    pub fn new(device: Device) -> Self {
        println!("{}", device.name().unwrap());
        let stream_config = StreamConfig {
            channels: 2,
            sample_rate: SampleRate(48000),
            buffer_size: BufferSize::Default,
        };

        let wav_spec = WavSpec {
            channels: 1,
            sample_rate: 48000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        Self {
            device,
            stream_config,
            wav_spec,
        }
    }

    pub fn start<P: AsRef<path::Path>>(&self, path: P) -> Recording {
        let mut writer = hound::WavWriter::create(path, self.wav_spec).unwrap();
        let stream = self.device.build_input_stream(&self.stream_config, move |data: &[i16], info| {
            println!("Data: {}", data.len());
            let mut writer16 = writer.get_i16_writer((data.len() / 2) as u32);
            let mut it = data.iter();

            while let Some(s) = it.next() {
                let mono = (*s + *it.next().unwrap()) / 2;
                //writer16.write_sample(*s);
                writer16.write_sample(mono);
            }

            writer16.flush().unwrap();
        }, |err| {}, None).unwrap();
        stream.play().unwrap();
        Recording {
            stream,
        }
    }
}

pub fn start_recording<P: AsRef<path::Path>>(device: Device, path: P) -> Stream {
    let configs = device.supported_input_configs().unwrap().chain(device.supported_output_configs().unwrap());

    let mut channels = 0;
    for c in configs {
        if c.sample_format() == SampleFormat::I16 {
            assert!(c.min_sample_rate().0 <= BASE_RATE && c.max_sample_rate().0 >= BASE_RATE);
            channels = c.channels();
            break;
        }
    }
    assert!(channels == 1 || channels == 2);

    let stream_config = StreamConfig {
        channels,
        sample_rate: SampleRate(BASE_RATE),
        buffer_size: BufferSize::Default,
    };

    let wav_spec = WavSpec {
        channels: 1,
        sample_rate: BASE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, wav_spec).unwrap();
    let stream = device.build_input_stream(&stream_config, move |data: &[i16], info| {
        println!("Data: {}", data.len());
        let mut writer16 = writer.get_i16_writer((data.len() / (channels as usize)) as u32);
        let mut it = data.iter();

        while let Some(s) = it.next() {
            if channels == 1 {
                writer16.write_sample(*s);
            } else {
                let mono = (*s + *it.next().unwrap()) / 2;
                writer16.write_sample(mono);
            }
        }

        writer16.flush().unwrap();
    }, |err| {}, None).unwrap();
    stream.play().unwrap();
    stream
}

pub fn start_recording2<P: AsRef<path::Path>>(device: Device, path: P) -> (Stream, Receiver<(Duration, Vec<i16>)>) {
    let configs = device.supported_input_configs().unwrap().chain(device.supported_output_configs().unwrap());

    let mut channels = 0;
    for c in configs {
        if c.sample_format() == SampleFormat::I16 {
            assert!(c.min_sample_rate().0 <= BASE_RATE && c.max_sample_rate().0 >= BASE_RATE);
            channels = c.channels();
            break;
        }
    }
    assert!(channels == 1 || channels == 2);

    let stream_config = StreamConfig {
        channels,
        sample_rate: SampleRate(BASE_RATE),
        buffer_size: BufferSize::Default,
    };


    let (tx, rx): (Sender<(Duration, Vec<i16>)>, Receiver<(Duration, Vec<i16>)>) = mpsc::channel();

    let mut t0 = Instant::now();

    let stream = device.build_input_stream(&stream_config, move |data: &[i16], info| {
        let mut sample_vec: Vec<i16> = Vec::new();
        sample_vec.extend_from_slice(data);
        tx.send((t0.elapsed(), sample_vec)).unwrap();
        t0 = Instant::now();
    }, |err| {}, None).unwrap();
    stream.play().unwrap();
    (stream, rx)
}

pub struct Recording {
    stream: Stream,
    //writer: WavWriter<BufWriter<File>>,
}

impl Recording {
    pub fn stop(self) {}
}