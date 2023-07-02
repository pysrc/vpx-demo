use std::{fs::File, io::BufWriter, time::Instant, path::Path};


use screen::Cap;

mod screen;
mod convert;

fn main() {
    let mut cap = Cap::new();
    let (width, height) = cap.wh();

    // vpxencode
    let ecfg = vpx_codec::encoder::Config {
        width: width as _,
        height: height as _,
        timebase: [1, 1000],
        bitrate: 1024,
        codec: vpx_codec::encoder::VideoCodecId::VP8,
    };
    let mut enc = vpx_codec::encoder::Encoder::new(ecfg).unwrap();

    // vpxdecode
    let dcfg = vpx_codec::decoder::Config {
        width: width as _,
        height: height as _,
        timebase: [1, 1000],
        bitrate: 1024,
        codec: vpx_codec::decoder::VideoCodecId::VP8,
    };
    let mut dec = vpx_codec::decoder::Decoder::new(dcfg).unwrap();

    // yuv
    let mut yuv = Vec::<u8>::new();
    let mut endata = Vec::<u8>::new();
    let mut dedata = Vec::<u8>::new();


    let start = Instant::now();
    for k in 0..10 {
        // encode
        {
            let bgra = cap.cap();
            let now = Instant::now();
            let time = now - start;
            let ms = time.as_secs() * 1000 + time.subsec_millis() as u64;
            convert::bgra_to_i420(width, height, bgra, &mut yuv);
            endata.clear();
            for f in enc.encode(ms as _, &yuv).unwrap() {
                endata.extend_from_slice(f.data);
            }
        }

        eprintln!("encode len {}", endata.len());

        // decode
        {
            for f in dec.decode(&endata).unwrap() {
                let (width, height) = (f.width(), f.height());
                let (y, u, v) = f.data();
                if dedata.len() < width * height * 3 {
                    eprintln!("resize {} {}", width, height);
                    dedata.resize(width * height * 3, 0);
                }
                convert::i420_to_rgb(width, height, y, u, v, &mut dedata);
                {
                    let p = format!("res-{}.png", k);
                    let path = Path::new(&p);
                    let file = File::create(path).unwrap();
                    let ref mut w = BufWriter::new(file);
                    let mut enc = png::Encoder::new(w, width as _, height as _);
                    enc.set_color(png::ColorType::Rgb);
                    enc.set_depth(png::BitDepth::Eight);
                    let mut writer = enc.write_header().unwrap();
                    writer.write_image_data(&dedata).unwrap();
                    writer.finish().unwrap();
                }
            }
        }
        
    }

}