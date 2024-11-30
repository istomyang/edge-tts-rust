use std::net::TcpStream;

use chrono::{FixedOffset, Utc};
use tungstenite::{stream::MaybeTlsStream, Message, WebSocket};
use uuid::Uuid;

use crate::{gen_sec_ms_gec, Error};

pub type AudioBytes = Vec<u8>;

/// Request a text to Microsoft using Edge browser's speech.
///
/// # Examples
///
/// ```
/// use edge_tts::tts::request;
///
/// let data = request(
///     "en-US".to_owned(),
///     "Microsoft Server Speech Text to Speech Voice (en-US, JennyNeural)".to_owned(),
///     "+0Hz".to_owned(),
///     "+25%".to_owned(),
///     "+0%".to_owned(),
///     "audio-24khz-48kbitrate-mono-mp3".to_owned(),
///     "Hello, world!".to_owned(),
/// ).unwrap();
///
/// let mut file = std::fs::File::create("text.mp3").unwrap();
/// file.write_all(&data).unwrap();
/// ```
pub fn request(
    lang: &str,
    voice: &str,
    pitch: &str,
    rate: &str,
    volume: &str,
    output_format: &str,
    text: &str,
) -> Result<AudioBytes, Error> {
    // You can use a packet grabbing tool, like Charles,
    // to grab how the Edge browser handles audio read aloud
    // and you'll get the flow of operations.

    let mut socket = build_ws_connection()?;

    socket
        .send(Message::Text(request1(output_format)))
        .map_err(|e| Error::from(e.to_string()))?;

    socket
        .send(Message::Text(request2(
            lang, voice, pitch, rate, volume, text,
        )))
        .map_err(|e| Error::from(e.to_string()))?;

    // Because AI generates Audio Token continuously,
    // the client needs to accept a block of data continuously.
    let mut data = vec![];

    loop {
        match socket.read() {
            Ok(msg) => match msg {
                Message::Text(s) => {
                    if s.contains("Path:turn.end") {
                        return Ok(data);
                    }
                }
                Message::Binary(vec) => {
                    // metadata data length.
                    let len: usize = u16::from_be_bytes([vec[0], vec[1]]).into();
                    // You can use `vec[2..len].to_vec()` to get metadata.
                    data.append(&mut vec[len + 2..].to_vec());
                }
                _ => {
                    return Err(Error::from("Undefined logic"));
                }
            },
            Err(e) => match e {
                tungstenite::Error::AlreadyClosed | tungstenite::Error::ConnectionClosed => {
                    return Err(Error::from("Connection closed"));
                }
                _ => {
                    return Err(Error::from(e.to_string()));
                }
            },
        }
    }
}

fn request1(output_format: &str) -> String {
    let mut b = String::new();
    b.push_str(format!("X-Timestamp:{}\r\n", datetime_to_string()).as_str());
    b.push_str("Content-Type:application/json; charset=utf-8\r\n");
    b.push_str("Path:speech.config\r\n\r\n");
    b.push_str(
        r#"{"context":{"synthesis":{"audio":{"metadataoptions":{"sentenceBoundaryEnabled":"false","wordBoundaryEnabled":"false"},"outputFormat":"{1}"}}}}"#.replace("{1}", output_format).as_str(),
    );
    b
}

fn request2(lang: &str, voice: &str, pitch: &str, rate: &str, volume: &str, text: &str) -> String {
    let mut b = String::new();
    b.push_str(format!("X-RequestId:{}\r\n", Uuid::new_v4().simple()).as_str());
    b.push_str("Content-Type:application/ssml+xml\r\n");
    b.push_str(format!("X-Timestamp:{}\r\n", datetime_to_string()).as_str());
    b.push_str("Path:ssml\r\n\r\n");
    b.push_str(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<speak version="1.0" xmlns="http://www.w3.org/2001/10/synthesis"  xml:lang="{1}">
<voice name="{2}">
<prosody pitch="{3}" rate="{4}" volume="{5}">{6}</prosody>
</voice>
</speak>
"#
        .replace("{1}", lang)
        .replace("{2}", voice)
        .replace("{3}", pitch)
        .replace("{4}", rate)
        .replace("{5}", volume)
        .replace("{6}", text)
        .as_str(),
    );
    b
}

fn build_ws_connection() -> Result<WebSocket<MaybeTlsStream<TcpStream>>, Error> {
    let request = {
        let url = build_ws_url().parse().unwrap();
        let mut builder = tungstenite::ClientRequestBuilder::new(url);

        let headers = vec![
            ("Pragma", "no-cache"),
            ("Cache-Control", "no-cache"),
            ("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0"),
            ("Origin", "chrome-extension://jdiccldimpdaibmpdkjnbmckianbfold"),
            ("Accept-Encoding", "gzip, deflate, br, zstd"),
            ("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6"),
        ];

        for (header_name, header_value) in headers {
            builder = builder.with_header(header_name, header_value);
        }
        builder
    };

    let (socket, res) = tungstenite::connect(request).map_err(|e| Error::from(e.to_string()))?;

    println!("Response HTTP code: {}", res.status());

    Ok(socket)
}

fn build_ws_url() -> String {
    format!(
        "wss://speech.platform.bing.com/consumer/speech/synthesize/readaloud/edge/v1?TrustedClientToken={}&Sec-MS-GEC={}&Sec-MS-GEC-Version={}&ConnectionId={}",
        "6A5AA1D4EAFF4E9FB37E23D68491D6F4",
        gen_sec_ms_gec(),
        "1-131.0.2903.51",
        Uuid::new_v4().simple()
    )
}

/// datetime_to_string create like this:
/// ```text
/// Sun Feb  3 2018 12:34:56 GMT+0800 (中国标准时间)
/// ```
fn datetime_to_string() -> String {
    let now = Utc::now();
    let timezone = FixedOffset::east_opt(8 * 3600).unwrap();
    let datetime = now.with_timezone(&timezone);
    let format = "%a %b %d %Y %H:%M:%S GMT%:z (%Z)";
    datetime.format(&format).to_string()
}

#[test]
fn request_work() {
    use std::fs::File;
    use std::io::Write;

    match request(
        "en-US",
        "Microsoft Server Speech Text to Speech Voice (en-US, JennyNeural)",
        "+0Hz",
        "+25%",
        "+0%",
        "audio-24khz-48kbitrate-mono-mp3",
        "In February of 2016 I began to experience two separate realities at the same time.",
    ) {
        Ok(data) => {
            let mut file = File::create("text.mp3").unwrap();
            file.write_all(&data).unwrap();
        }
        Err(e) => println!("{}", e),
    }
}
