// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs, thread};
use std::fmt::Formatter;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use cpal::Stream;
use cpal::traits::{DeviceTrait, HostTrait};
use serde::{Deserialize, Serialize};
use serde::de::{SeqAccess, Visitor};
use tauri::{AppHandle, Manager, State};
use tauri::async_runtime::block_on;

use cmd_channel::{CmdReceiver, CmdSender};

mod cmd_channel;
mod recorder;
mod openai;
mod utils;

/*
Necesito un sistema para pasar comandos del thread del ui al thread principal. Para eso necesito:
    -Un tipo comando
        -Debe tener un tipo de entrada y salida
    -Un queue de comandos
    -Una forma de responder a un comando específico
*/

/*
-Si los comandos con son async, la UI bloquea hasta que el comando retorne
-Creo que tener otro thread para procesar comandos puede ser innecesario
*/

#[derive(Debug)]
enum Cmd {
    StartRecording,
    StopRecording,
    Test,
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
    fs::create_dir_all(utils::get_data_path()).unwrap();
    fs::create_dir_all(utils::get_data_path()).unwrap();

    let (tx, rx) = cmd_channel::channel::<Cmd>();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![start_recording, stop_recording, get_devices])
        .manage(tx)
        .setup(|app| {

            let resource_path = app.path_resolver()
                .resolve_resource("private.txt")
                .unwrap();

            println!("{:?}", resource_path);

            let handle = app.handle();
            thread::spawn(move || {
                main_thread(handle, rx);
            });

            let handle = app.handle();
            std::panic::set_hook(Box::new(move |info| {
                println!("Panicked: {:?}", info);
                //handle2.exit(0);
                handle.emit_all("panic", format!("{:?}", info)).unwrap();
            }));

            let _handle = app.handle();
            let _id = app.listen_global("front_ready",move |_ev| {
                println!("Front ready");

                // let mut msg = String::new();
                //
                // for host in cpal::available_hosts() {
                //     msg.push_str(format!("Host: {}\n", host.name()).as_str());
                //     let host = cpal::host_from_id(host).unwrap();
                //     for device in host.input_devices().unwrap() {
                //         msg.push_str(format!("-{}\n", device.name().unwrap()).as_str());
                //         for c in device.supported_input_configs().unwrap() {
                //             msg.push_str(format!("  -{}b, {}ch, [{},{}]\n",
                //                                  c.sample_format(), c.channels(), c.min_sample_rate().0,
                //                                  c.max_sample_rate().0).as_str());
                //         }
                //     }
                // }
                // println!("{msg}");
                // handle.app_handle().emit_all("dbg_msg", msg).unwrap();
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

#[tauri::command]
async fn get_devices() -> Result<Vec<String>, ()> {
    let def_device = cpal::default_host().default_input_device().unwrap().name().unwrap();
    let mut names = vec![def_device.clone()];
    for device in cpal::default_host().input_devices().unwrap() {
        let name = device.name().unwrap();
        println!("{}", name);
        if name != def_device {
            names.push(name);
        }
    }
    Ok(names)
}

#[tauri::command]
async fn start_recording(tx: State<'_, CmdSender<Cmd>>, device: String) -> Result<(), ()> {
    if tx.send_io::<String, Result<(), ()>>(Cmd::StartRecording, Some(device)).await.is_ok() {
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

                    //Encontrar device seleccionado
                    let selected_device = *data.unwrap().downcast::<String>().unwrap();
                    let mut found = false;
                    for device in cpal::default_host().input_devices().unwrap() {
                        let name = device.name().unwrap();
                        if name == selected_device {
                            context.start_t = Some(Instant::now());
                            last_sec = 0u32;
                            context.stream_1 = Some(recorder::start_recording(device, utils::get_rec_path()));
                            state = States::Recording;
                            found = true;
                            break;
                        }
                    }
                    assert!(found);

                    Some(Box::new(Ok::<(), ()>(())))
                }

                Cmd::StopRecording => {
                    println!("Stopped");
                    context.stream_1 = None;
                    context.stream_2 = None;
                    state = States::Idle;

                    println!("Transcribing...");
                    let n_seg = fs::read_dir(utils::get_rec_path()).unwrap().count();
                    if n_seg > 0 {
                        let paths = fs::read_dir(utils::get_rec_path()).unwrap();
                        let mut full_text = String::new();
                        for path in paths {
                            let text = block_on(openai::transcribe_file(&path.as_ref().unwrap().path()));
                            full_text.push_str(text.as_str());
                            println!("Transcribed: {text}");
                            app.emit_all("trans_text", full_text.clone()).unwrap();
                        }
                        let format = *data.unwrap().downcast::<String>().unwrap();
                        println!("Format:\n{format}");
                        let result = block_on(openai::process_text(full_text, format));
                        println!("Result:\n{result}");
                        app.emit_all("result_text", result).unwrap();
                    }

                    Some(Box::new(Ok::<(), ()>(())))
                }
                Cmd::Test => {
                    app.emit_all("result_text", "Test").unwrap();
                    assert!(false);
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