use rand::seq::IndexedRandom;
use regex::Regex;
use mavlink::common;
use mavlink::Message;
use std::fs::File;
use std::io::Write;

use rand::Rng;
use std::process::Command;

pub fn main() {
    for id in common::MavMessage::all_ids() {
        test_message(*id);
    }
}

fn test_message(id: u32) {
    let mut buf = vec![];
    let mut rng = rand::rng();
    let random_msg = common::MavMessage::random_message_from_id(id, &mut rng).unwrap();
    let header = mavlink::MavHeader {
        sequence: rng.random(),
        system_id: rng.random(),
        component_id: rng.random(),
    };
    mavlink::write_v2_msg(&mut buf, header, &random_msg).unwrap();

    let mut f = File::create("filename.bin").unwrap();
    f.write_all(&buf).unwrap();
    write_c_asserts(&random_msg, &header);

    println!("Compiling C");
    assert!(Command::new("gcc")
        .arg("main.c")
        .arg("-o")
        .arg("main")
        .arg("-Wno-address-of-packed-member")
        .status()
        .unwrap()
        .success());
    println!("Running C");
    assert!(Command::new("./main").status().unwrap().success());
    println!("Done");
    println!();
}

fn write_c_asserts(msg: &common::MavMessage, header: &mavlink::MavHeader) {
    let mut str = String::new();
    str += &format!("assert(msg.seq == {});\n", header.sequence);
    str += &format!("assert(msg.sysid == {});\n", header.system_id);
    str += &format!("assert(msg.compid == {});\n", header.component_id);
    str += &format!("assert(msg.msgid == {});\n", msg.message_id());
    let lower_name = msg.message_name().to_ascii_lowercase();
    str += &format!("mavlink_{}_t decode;\n", lower_name);
    str += &format!("mavlink_msg_{}_decode(&msg, &decode);\n", lower_name);
    
    let json = serde_json::to_string(msg).unwrap();
    for part in json.split(",") {
        if part.contains("[") {
            continue;
        }
        let re_number = Regex::new("\"([a-z_]+)\":(\\d+)$").unwrap();
        let re_enum = Regex::new("\"([a-z_]+)\":\\{\"type\":\"([A-Z_]+)\"\\}$").unwrap();
        if let Some(caps) = re_number.captures(part) {
            let mut name = caps.get(1).unwrap().as_str();
            if name == "mavtype" {
                name = "type";
            }
            let value: f64 = caps.get(2).unwrap().as_str().parse().unwrap();
            if value.fract() == 0.0 && value >= 0.0 {
                let value: u64 = caps.get(2).unwrap().as_str().parse().unwrap();
                //println!("U64  {name}: {value}");
                str += &format!("assert(decode.{name} == {value}ULL);\n");
            } else if value.fract() == 0.0 {
                let value: i64 = caps.get(2).unwrap().as_str().parse().unwrap();
                //println!("I64  {name}: {value}");
                str += &format!("assert(decode.{name} == {value});\n");
            } else {
                //println!("DBL  {name}: {value}");
                str += &format!("assert(decode.{name} == {value});\n");
            }

        } else if let Some(caps) = re_enum.captures(part) {
            let mut name = caps.get(1).unwrap().as_str();
            if name == "mavtype" {
                name = "type";
            }
            let value = caps.get(2).unwrap().as_str();
            //println!("ENUM {name}: {value}");
            str += &format!("assert(decode.{name} == {value});\n");
        } else {
            //println!("{part}");
        }
    }
    std::fs::write("c_msg_asserts.c", &str).unwrap();
    print!("{str}");
}
