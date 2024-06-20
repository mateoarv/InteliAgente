use std::{fs, path};
use std::path::PathBuf;

use cpal::{BufferSize, Device, SampleFormat, SampleRate, SizedSample, Stream, StreamConfig, SupportedStreamConfigRange};
use cpal::traits::{DeviceTrait, StreamTrait};
use dasp_sample;
use dasp_sample::{ToSample};
use hound;
use hound::WavSpec;

const IDEAL_RATE: u32 = 48000;
const SEG_DUR: u32 = 30;
const PREFERRED_FORMATS: [SampleFormat; 10] = [
    SampleFormat::I16,
    SampleFormat::U16,
    SampleFormat::I32,
    SampleFormat::U32,
    SampleFormat::F32,
    SampleFormat::I64,
    SampleFormat::U64,
    SampleFormat::F64,
    SampleFormat::I8,
    SampleFormat::U8,
];

pub fn start_recording<P: AsRef<path::Path>>(device: Device, dir_path: P) -> Stream {
    let configs = device
        .supported_input_configs().unwrap()
        .chain(device.supported_output_configs().unwrap())
        .collect::<Vec<SupportedStreamConfigRange>>();

    let mut rate = 0;
    let mut channels = 0;
    let mut format = SampleFormat::I16;
    let mut found = false;

    for _format in PREFERRED_FORMATS {
        if let Some(c) = configs.iter().find(|x| { x.sample_format() == _format }) {
            format = _format;
            if c.max_sample_rate().0 <= IDEAL_RATE {
                rate = c.max_sample_rate().0;
            } else if c.min_sample_rate().0 >= IDEAL_RATE {
                rate = c.min_sample_rate().0;
            } else {
                rate = IDEAL_RATE;
            }
            channels = c.channels();
            found = true;
            break;
        }
    }

    if !found {
        let mut dbg_msg = String::new();
        for c in configs.iter() {
            dbg_msg.push_str(format!("Format: {}, Channels: {}, Min rate: {}, Max rate: {}",
                                     c.sample_format(), c.channels(), c.min_sample_rate().0,
                                     c.max_sample_rate().0).as_str());
            dbg_msg.push('\n');
        }
        panic!("Incompatible format: \n{}", dbg_msg);
    }

    println!("Device: {}, Channels: {}, Rate: {}, Fmt: {}",
             device.name().unwrap(),
             channels,
             rate,
             format
    );

    let stream_config = StreamConfig {
        channels,
        sample_rate: SampleRate(rate),
        buffer_size: BufferSize::Default,
    };

    let wav_spec = WavSpec {
        channels: 1,
        sample_rate: rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };


    if dir_path.as_ref().try_exists().unwrap() {
        fs::remove_dir_all(dir_path.as_ref()).unwrap();
    }
    fs::create_dir_all(dir_path.as_ref()).unwrap();

    let base_path_buf = PathBuf::from(dir_path.as_ref());
    let mut path_buf = base_path_buf.clone();
    path_buf.push("seg0.wav");
    let mut writer = hound::WavWriter::create(path_buf.as_path(), wav_spec).unwrap();
    let mut sample_count = 0u32;
    let mut seg_count = 0u32;
    let f = move |data: &[i16]| {
        //println!("Data: {}", data.len());

        //Crear nuevo segmento
        if sample_count >= rate * SEG_DUR {
            seg_count += 1;
            let mut path_buf = base_path_buf.clone();
            path_buf.push(format!("seg{}.wav", seg_count));
            writer = hound::WavWriter::create(path_buf.as_path(), wav_spec).unwrap();
            sample_count = 0;
        }

        let mut writer16 = writer.get_i16_writer((data.len() / (channels as usize)) as u32);
        let mut it = data.iter();

        while let Some(s) = it.next() {
            let mut mono = *s;
            for _ in 0..(channels - 1) {
                mono += *it.next().unwrap();
            }
            mono = mono / (channels as i16);
            writer16.write_sample(mono);
            sample_count += 1;
        }

        writer16.flush().unwrap();
    };

    let stream = match format {
        SampleFormat::I8 => get_stream::<i8, _>(device, stream_config, f),
        SampleFormat::I16 => get_stream::<i16, _>(device, stream_config, f),
        SampleFormat::I32 => get_stream::<i32, _>(device, stream_config, f),
        SampleFormat::I64 => get_stream::<i64, _>(device, stream_config, f),
        SampleFormat::U8 => get_stream::<u8, _>(device, stream_config, f),
        SampleFormat::U16 => get_stream::<u16, _>(device, stream_config, f),
        SampleFormat::U32 => get_stream::<u32, _>(device, stream_config, f),
        SampleFormat::U64 => get_stream::<u64, _>(device, stream_config, f),
        SampleFormat::F32 => get_stream::<f32, _>(device, stream_config, f),
        SampleFormat::F64 => get_stream::<f64, _>(device, stream_config, f),
        _ => get_stream::<i16, _>(device, stream_config, f), //TODO: err
    };

    stream.play().unwrap();
    stream
}

fn get_stream<T, F>(device: Device, stream_config: StreamConfig, mut callback: F) -> Stream
    where
        T: SizedSample + ToSample<i16>,
        F: FnMut(&[i16]) + Send + 'static
{
    device.build_input_stream(&stream_config, move |data: &[T], _info| {
        callback(data.iter().map(|x| { x.to_sample::<i16>() }).collect::<Vec<i16>>().as_slice());
    }, |_| {}, None).unwrap()
}
