use bardecoder::Decoder;
use nokhwa::Camera;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use wifi_rs::prelude::{Config, Connectivity};
use wifi_rs::WiFi;

#[derive(Debug)]
enum WifiType { WEP, WPA, Blank }

impl WifiType {
    pub fn from(s: &str) -> WifiType {
        match s {
            "WPA" => WifiType::WPA,
            "WEP" => WifiType::WEP,
            _ => WifiType::Blank
        }
    }
}

#[derive(Debug)]
struct Wifi {
    ssid_name: String,
    wifi_type: WifiType,
    password: String,
}

fn parse_qr_code_string(code: &String) -> Option<Wifi> {
    let tokens: Vec<&str> = code.split(';').filter(|s| !s.is_empty()).collect();
    if tokens.len() > 0 && tokens[0].starts_with("WIFI") {
        let wifi = Wifi {
            ssid_name: String::from(tokens[0].split(':').collect::<Vec<&str>>()[2]),
            wifi_type: WifiType::from(tokens[1].split(':').collect::<Vec<&str>>()[1]),
            password: String::from(tokens[2].split(':').collect::<Vec<&str>>()[1]),
        };
        Some(wifi)
    } else {
        None
    }
}

fn main() {
    // let img: image::DynamicImage = image::open("/home/joe/Downloads/qr-code.jpg").unwrap();
    // let decoder = bardecoder::default_decoder();
    //
    // let results = decoder.decode(&img);

    let index = CameraIndex::Index(0);
    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::HighestFrameRate(30));
    let mut camera = Camera::new(index, requested).unwrap();

    camera.open_stream().unwrap();
    let frame = camera.frame().unwrap();
    println!("Captured Single Frame of {}", frame.buffer().len());
    let decoded = frame.decode_image::<RgbFormat>().unwrap();
    decoded.save("/home/joe/Downloads/test-capture.png").unwrap();
    println!("Decoded Frame of {} length", decoded.len());
    return;
    // for result in results {
    //     match result {
    //         Ok(code) => {
    //             if let Some(wifi) = parse_qr_code_string(&code) {
    //                 let conf = Config {
    //                     interface: Some("wlp2s0")
    //                 };
    //                 let mut conn = WiFi::new(Some(conf));
    //                 match conn.connect(&wifi.ssid_name, &wifi.password) {
    //                     Ok(result) => println!("{}", if result { "Connected!" } else { "Invalid Password!" }),
    //                     Err(err) => eprintln!("Error occurred: {:?}", err),
    //                 }
    //             }
    //         }
    //         Err(_) => {}
    //     }
    // }
}
