// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cmd_channel;
mod recorder;
mod openai;

use std::fmt::Formatter;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};
use cpal::Stream;
use cpal::traits::HostTrait;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tauri::{AppHandle, Manager, State};
use cmd_channel::{CmdSender, CmdReceiver};
use crate::recorder::Recorder;
use std::fs::File;
use serde::de::{SeqAccess, Visitor};
use serde::ser::{SerializeMap, SerializeSeq};
use async_openai::{
    types::{AudioResponseFormat, CreateTranscriptionRequestArgs, TimestampGranularity},
    Client,
};
use tauri::async_runtime::block_on;

/*
Necesito un sistema para pasar comandos del thread del ui al thread principal. Para eso necesito:
    -Un tipo comando
        -Debe tener un tipo de entrada y salida
    -Un queue de comandos
    -Una forma de responder a un comando específico
*/

#[derive(Debug)]
enum Cmd {
    StartRecording,
    StopRecording,
}

#[derive(Serialize, Deserialize)]
pub struct Chunk {
    t: Duration,
    data: Vec<i16>,
}

#[derive(Serialize, Deserialize)]
pub struct RecFile {
    chunks: Vec<Chunk>,
}

#[tokio::main]
async fn main() {
    // test_ai().await;
    // return;
    let (tx, rx) = cmd_channel::channel::<Cmd>();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![start_recording, stop_recording])
        .manage(tx)
        .setup(|app| {
            let handle = app.handle();

            thread::spawn(move || {
                main_thread(handle, rx);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

struct MyVisitor;

impl<'de> Visitor<'de> for MyVisitor {
    type Value = u8;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("A u8")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
        let el: u8 = seq.next_element()?.unwrap();
        Ok(el)
    }
}

fn test_ser() {
    {
        let f = File::create("test.bin").unwrap();

        let mut ser = rmp_serde::encode::Serializer::new(f);
        let mut seq = ser.serialize_seq(None).unwrap();
        let n1 = 5u8;
        seq.serialize_element(&n1).unwrap();
        //seq.serialize_element(&n1).unwrap();
        SerializeSeq::end(seq).unwrap();
    }

    let f = File::open("test.bin").unwrap();
    let mut deser = rmp_serde::decode::Deserializer::new(f);
    //let n0 = deser.deserialize_u8(MyVisitor).unwrap();
    let n1 = deser.deserialize_seq(MyVisitor).unwrap();
    //deser.de
    //let n2 = deser.deserialize_seq(MyVisitor).unwrap();
    println!("{n1}");
}

async fn test_ai() {
    let client = Client::new();
    let request = CreateTranscriptionRequestArgs::default()
        .file("../audio_test.wav")
        .model("whisper-1")
        .response_format(AudioResponseFormat::Json)
        .build()
        .unwrap();

    let response = client.audio().transcribe(request).await.unwrap();
    println!("{}", response.text);
}

#[tauri::command]
async fn start_recording(tx: State<'_, CmdSender<Cmd>>) -> Result<(), ()> {
    if tx.send_o::<Result<(), ()>>(Cmd::StartRecording).await.is_ok() {
        println!("Recording started");
    }
    Ok(())
}

#[tauri::command]
async fn stop_recording(tx: State<'_, CmdSender<Cmd>>, format_text: String) -> Result<(), ()> {
    if tx.send_io::<String, Result<(), ()>>(Cmd::StopRecording, Some(format_text)).await.is_ok() {
        println!("Recording stopped");
    }
    Ok(())
}

fn main_thread(app: AppHandle, mut rx: CmdReceiver<Cmd>) {
    enum States {
        Idle,
        Recording,
    }
    let mut state = States::Idle;

    struct Context {
        start_t: Option<Instant>,
        stream_1: Option<Stream>,
        stream_2: Option<Stream>,
        rx: Option<Receiver<(Duration, Vec<i16>)>>,
        file_1: Option<RecFile>,
    }

    let mut context = Context {
        start_t: None,
        stream_1: None,
        stream_2: None,
        rx: None,
        file_1: None,
    };

    let mut last_sec = 0u32;

    loop {
        rx.get(|id, data| {
            //Procesar comandos
            match id {
                Cmd::StartRecording => {
                    println!("Recording");

                    context.start_t = Some(Instant::now());
                    last_sec = 0u32;
                    context.stream_1 = Some(recorder::start_recording(cpal::default_host().default_input_device().unwrap(), "../rec.wav"));
                    // context.stream_2 = Some(recorder::start_recording(cpal::default_host().default_output_device().unwrap(), "../test_o.wav"));
                    // let (stream, rx) = recorder::start_recording2(cpal::default_host().default_output_device().unwrap(), "../test_o.wav");
                    // context.stream_1 = Some(stream);
                    // context.rx = Some(rx);
                    // context.file_1 = Some(RecFile {
                    //     chunks: Vec::new(),
                    // });
                    state = States::Recording;


                    Some(Box::new(Ok::<(), ()>(())))
                }

                Cmd::StopRecording => {
                    println!("Stopped");
                    context.stream_1 = None;
                    context.stream_2 = None;
                    state = States::Idle;
                    println!("Transcribing...");
                    let text = block_on(openai::transcribe_file("F:/Dropbox/Dropbox/Proyectos/InteliAgente/Audios/Seg1.mp3"));
                    println!("Transcribed: {text}");
                    app.emit_all("trans_text", text.clone()).unwrap();
                    let format = *data.unwrap().downcast::<String>().unwrap();
                    println!("Format:\n{format}");
                    let result = block_on(openai::process_text(text, format));
                    println!("Result:\n{result}");
                    app.emit_all("result_text", result).unwrap();
                    Some(Box::new(Ok::<(), ()>(())))
                }
            }
        });

        match state {
            States::Recording => {
                let secs = context.start_t.unwrap().elapsed().as_secs() as u32;
                if secs != last_sec {
                    let h = secs / 3600;
                    let m = (secs / 60) % 60;
                    let s = secs % 60;
                    println!("{secs}");
                    let s = format!("{:02}:{:02}:{:02}", h, m, s);
                    app.emit_all("rec_time", s).unwrap();
                    last_sec = secs;
                }

                // if let Ok(chunk) = context.rx.as_ref().unwrap().try_recv() {
                //     let mut avg = 0;
                //     for s in &chunk.1 {
                //         avg += *s as i32;
                //     }
                //     avg /= chunk.1.len() as i32;
                //
                //     context.file_1.as_mut().unwrap().chunks.push(Chunk {
                //         t: chunk.0,
                //         data: chunk.1,
                //     });
                //
                //     //println!("{} {} {}", chunk.0.as_millis(), chunk.1.len(), avg);
                // }
            }
            _ => {}
        };
    }
}