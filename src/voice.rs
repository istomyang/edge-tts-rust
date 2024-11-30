use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{gen_sec_ms_gec, Error};

/// List all voices from Microsoft.
///
/// You can use its name to [`crate::request`] voice name.
pub fn list_voices() -> Result<Vec<Voice>, Error> {
    let client = reqwest::blocking::Client::new();
    let res = client.get(build_url())
        .header("sec-ch-ua-platform", "macOS")
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0")
        .header("sec-ch-ua", r#""Microsoft Edge";v="131", "Chromium";v="131", "Not_A Brand";v="24""#)
        .header("sec-ch-ua-mobile", "?0")
        .header("Accept", "*/*")
        .header("X-Edge-Shopping-Flag", "1")
        .header("Sec-MS-GEC", gen_sec_ms_gec())
        .header("Sec-MS-GEC-Version", "1-131.0.2903.70")
        .header("Sec-Fetch-Site", "none")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Dest", "empty")
        .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6")
        .header("Accept-Encoding", "gzip, deflate, br, zstd")
        .send().map_err(|e| Error::from(e.to_string()))?;

    if res.status() != StatusCode::OK {
        return Err(Error::from(res.text().unwrap()));
    }

    res.json().map_err(|e| Error::from(e.to_string()))
}

fn build_url() -> String {
    format!("https://speech.platform.bing.com/consumer/speech/synthesize/readaloud/voices/list?trustedclienttoken=6A5AA1D4EAFF4E9FB37E23D68491D6F4&Sec-MS-GEC={}&Sec-MS-GEC-Version=1-131.0.2903.70", gen_sec_ms_gec())
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Voice {
    pub name: String,
    pub short_name: String,
    pub gender: String,
    pub locale: String,
    pub suggested_codec: String,
    pub friendly_name: String,
    pub status: String,
    pub voice_tag: VoiceTag,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VoiceTag {
    pub content_categories: Vec<String>,
    pub voice_personalities: Vec<String>,
}

#[test]
fn test_list_voices() {
    let voice = Voice::default();
    println!("{}", serde_json::to_string(&voice).unwrap());
}

#[test]
fn run_list_voice() {
    let list = list_voices().unwrap();
    println!("{:?}", list);
}
