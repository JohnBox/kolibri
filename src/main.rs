mod writer;

use chrono::Utc;
use pprof::protos::Message;
use reqwest::blocking::Client;
use serde_json::json;
use std::env::{args, var};
use std::fs::{create_dir_all, File};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn start_record(id: &String, link: &String, record_path: String) {
    let record_cmd = format!("-rtsp_transport tcp -i {} -acodec copy -vcodec copy -f ssegment -segment_list_flags +live -segment_time 2 -strftime 1 -flush_packets 1 %s.ts", link);
    let mut ffmpeg = Command::new("ffmpeg")
        .args(record_cmd.split(" ").collect::<Vec<&str>>())
        .stderr(Stdio::piped())
        .current_dir(&record_path)
        .spawn()
        .expect(format!("cannot start record {}", record_path).as_str());

    let mut prev = String::new();
    let once_per = 60;
    let mut counter = -2;
    let guard = pprof::ProfilerGuard::new(100).unwrap();
    if let Some(ref mut stdout) = ffmpeg.stderr {
        for line in BufReader::new(stdout).lines() {
            if let Ok(line) = line {
                let ts_index = line.find("'");

                if let Some(index) = ts_index {
                    let filename: String = line.chars().skip(index + 1).take(13).collect();
                    if !filename.ends_with(".ts") {
                        continue;
                    }

                    println!("{}", &filename);

                    if !prev.is_empty() && counter % once_per == 0 {
                        create_preview(prev, &record_path);
                    }

                    prev = filename.clone();
                    send_to_playlister(&id, &record_path, &filename);
                    counter += 1;

                    if counter > 10 {
                        if let Ok(report) = guard.report().build() {
                            let mut file = File::create("profile.pb").unwrap();
                            let profile = report.pprof().unwrap();

                            let mut content = Vec::new();
                            profile.encode(&mut content).unwrap();
                            file.write_all(&content).unwrap();

                            println!("report: {:?}", &report);
                        }
                        return;
                    }
                };
            };
        }
    }
}

fn send_to_playlister(id: &String, record_path: &String, filename: &String) {
    let playlister_url: String = format!(
        "{}/records",
        var("PLAYLISTER_URL").unwrap_or(String::from("http://195.201.193.177:32502"))
    );
    let node_url: String = var("SERVER_SOCKET").unwrap_or(String::from("127.0.0.1:1234"));

    let body = json!({
        "camera_id": id,
        "node_url":  node_url,
        "date_time": record_path.replace(id, "").strip_prefix("mnt//"),
        "segment":   filename.strip_suffix(".ts"),
        "duration":  "2.000",
    });

    println!("{:?}", body);
    let response = Client::new()
        .post(&playlister_url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&body)
        .send();

    println!("{:?}", response);
    match response {
        Ok(response) => println!("segment saved {}", response.status()),
        Err(e) => println!("segment error {}", e),
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
    let mut args = args().skip(1);
    if args.len() < 2 {
        return;
    }
    let link = args.next().unwrap();
    let id = args.next().unwrap();
    println!("runs with args {} {}", &link, &id);

    let camera_path = format!("{}/{}", ROOT_PATH, id);
    create_dir_all(&camera_path).expect("cannot create camera dir");


    loop {
        let date_hour_path = Utc::now().format(DATE_HOUR_FORMAT);
        let current_record = format!("{}/{}", &camera_path, date_hour_path);
        println!("recording now in {:}", current_record);
        create_dir_all(&current_record).expect("cannot create record dir");
        start_record(&id, &link, current_record);

    }
}
