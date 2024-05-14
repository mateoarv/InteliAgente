use std::fs::File;
use std::io::Read;
use std::path;
use async_openai::{
    types::{AudioResponseFormat, CreateTranscriptionRequestArgs, TimestampGranularity},
    Client,
};
use async_openai::config::OpenAIConfig;
use async_openai::types::{ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs};

const PROMPT: &str = "Eres el asistente de un médico y tu tarea es extraer la información importante de una consulta médica para que esta
información sea usada para llenar la historia clínica. Vas a recibir dos cosas: Un ejemplo del formato en el que se
desea que entregues la información, y la transcripción de la consulta médica. En la transcripción no se indica quién
está hablando (médico o paciente) y por lo tanto es algo que debes intuir. Al final debes entregar la información
importante desde un punto de vista clínico en el mismo formato del ejemplo que se te ha proporcionado.";

pub async fn transcribe_file<P: AsRef<path::Path>>(path: P) -> String {
    let client = Client::with_config(OpenAIConfig::new().with_api_key(get_key()));
    // let client = Client::new();
    let request = CreateTranscriptionRequestArgs::default()
        .file(path)
        .model("whisper-1")
        .language("es")
        .response_format(AudioResponseFormat::Json)
        .build()
        .unwrap();

    let response = client.audio().transcribe(request).await.unwrap();
    return response.text;
}

pub async fn process_text(text: String, format: String) -> String {
    let client = Client::with_config(OpenAIConfig::new().with_api_key(get_key()));
    // let client = Client::new();
    let request = CreateChatCompletionRequestArgs::default()
        //.max_tokens(512u16)
        .model("gpt-3.5-turbo")
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(PROMPT)
                .build()
                .unwrap()
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(format!("Formato de ejemplo:\n{}", format))
                .build()
                .unwrap()
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(format!("Transcripción:\n{}", text))
                .build()
                .unwrap()
                .into(),
        ])
        .build().unwrap();

    let mut response = client.chat().create(request).await.unwrap();
    response.choices.remove(0).message.content.unwrap()
}

fn get_key() -> String {
    let mut file = File::open("private.txt").unwrap();
    let mut key = String::new();
    file.read_to_string(&mut key).unwrap();
    return key;
}