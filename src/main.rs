use chrono::Utc;
use std::env::args;
use std::fs::create_dir_all;
use std::io::{BufReader, BufRead};
use std::process::{Command, Stdio};

fn start_record(link: &str, record_path: &str) {
    let record_cmd = format!("-rtsp_transport tcp -i {} -acodec copy -vcodec copy -f ssegment -segment_list_flags +live -segment_time 2 -strftime 1 -flush_packets 1 %s.ts", link);
    let mut ffmpeg = Command::new("ffmpeg")
        .args(record_cmd.split(" ").collect::<Vec<&str>>())
        .stderr(Stdio::piped())
        .current_dir(record_path)
        .spawn()
        .expect(format!("cannot start record {}", record_path).as_str());

    let mut prev = String::new();
    let once_per = 2;
    let mut counter = -1;
    if let Some(ref mut stdout) = ffmpeg.stderr {

        for line in BufReader::new(stdout).lines() {
            println!("{:?}", line);
            let line = match line {
                Ok(v) => v,
                Err(e) => String::from("lol"),
            };

            let ts_index = line.find("'");

            let ts = match ts_index {
                Some(index) => {
                    let maybe_ts: String = line.chars().skip(index + 1).take(13).collect();
                    if maybe_ts.ends_with(".ts") {
                        Some(maybe_ts)
                    } else {
                        continue;
                    }
                }
                None => continue,
            };

            let filename = ts.unwrap();
            println!("{}", filename);

            if !prev.is_empty() && counter % once_per == 0 {
                create_preview(prev, &record_path);
            }

            prev = filename;
            counter += 1;
        }
    }
}

fn create_preview(filename: String, record_path: &str) {
    let preview_cmd: String = format!("-i {} -y -frames:v 1 ../../main_image.jpg", filename);

    let preview = Command::new("ffmpeg")
        .current_dir(record_path)
        .args(preview_cmd.split(" ").collect::<Vec<&str>>())
        .output();

    match preview {
        Ok(_) => println!("preview generated"),
        Err(v) => panic!("err {}", v),
    }
}

const DATE_HOUR_FORMAT: &str = "%Y_%m_%d/%-H";
const ROOT_PATH: &str = "mnt";

fn main() {
    let args: Vec<String> = args().skip(1).collect();
    if args.len() < 2 {
        return;
    }
    let link = &args[0];
    let id = &args[1];
    let camera_path = format!("{}/{}", ROOT_PATH, id);
    println!("{:}", link);
    println!("{:}", id);

    create_dir_all(&camera_path).expect("cannot create camera dir");

    if true {
        let date_hour_path = Utc::now().format(DATE_HOUR_FORMAT);
        println!("{:}", date_hour_path);
        let current_record = format!("{}/{}", &camera_path, date_hour_path);
        println!("{:}", current_record);
        create_dir_all(&current_record).expect("cannot create record dir");
        start_record(link, &*current_record);
    }
}
