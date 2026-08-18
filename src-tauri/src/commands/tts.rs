use edge_tts_rust::{EdgeTtsClient, SpeakOptions, Boundary};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TtsResult {
    pub ok: bool,
    pub data: Option<Vec<u8>>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn tts_speak(text: String, voice: Option<String>) -> Result<TtsResult, String> {
    if text.trim().is_empty() {
        return Ok(TtsResult { ok: false, data: None, error: Some("空文本".into()) });
    }

    let voice_name = voice.unwrap_or_else(|| "zh-CN-XiaoxiaoNeural".into());

    let client = EdgeTtsClient::new().map_err(|e| format!("TTS 初始化失败: {}", e))?;

    let result = client
        .synthesize(
            &text,
            SpeakOptions {
                voice: voice_name,
                boundary: Boundary::Sentence,
                ..SpeakOptions::default()
            },
        )
        .await
        .map_err(|e| format!("TTS 合成失败: {}", e))?;

    if result.audio.is_empty() {
        return Ok(TtsResult { ok: false, data: None, error: Some("合成结果为空".into()) });
    }

    Ok(TtsResult { ok: true, data: Some(result.audio), error: None })
}
