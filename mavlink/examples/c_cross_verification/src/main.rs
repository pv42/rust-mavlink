use regex::Regex;
use mavlink::all::MavMessage;
use mavlink::Message;
use std::fs::File;
use std::io::Write;

use rand::Rng;
use std::process::Command;

pub fn main() {
    for id in MavMessage::all_ids() {
        test_message(*id);
    }
}

fn test_message(id: u32) {
    let mut buf = vec![];
    let mut rng = rand::rng();
    let random_msg = MavMessage::random_message_from_id(id, &mut rng).unwrap();
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

fn get_json_key_value(data: &mut String) -> Option<(String, String)> {
    let mut index = 0usize;
    let mut level = 0;
    let mut key = String::new();
    loop {
        match data.as_bytes()[index] {
            b':' if level == 0  => {
                let tmp = data.split_off(index);
                key = data[1..data.len()-1].to_string();
                *data = tmp;
                index = 0;
            }
            b',' if level == 0 => {
                let tmp = data.split_off(index + 1);
                let value = data[1..data.len()-1].to_string();
                *data = tmp;
                return Some((key, value));
            } 
            b'{' => {
                level += 1;
            }
            b'[' => {
                level += 1;
            }
            b']' => {
                level -= 1;
            }
            b'}' => {
                if level == 0 {
                    if !key.is_empty() {
                        let tmp = data.split_off(index);
                        let value = data[1..].to_string();
                        *data = tmp;
                        return Some((key, value));
                    }
                } else {
                    level -= 1;
                }
            }
            _ => ()
        }
        index = index.wrapping_add(1);
        if index >= data.len() {
            return None;
        } 
    }
}

fn write_c_asserts(msg: &MavMessage, header: &mavlink::MavHeader) {
    let mut str = String::new();
    str += &format!("assert(msg.seq == {});\n", header.sequence);
    str += &format!("assert(msg.sysid == {});\n", header.system_id);
    str += &format!("assert(msg.compid == {});\n", header.component_id);
    str += &format!("assert(msg.msgid == {});\n", msg.message_id());
    let lower_name = msg.message_name().to_ascii_lowercase();
    str += &format!("mavlink_{}_t decode;\n", lower_name);
    str += &format!("mavlink_msg_{}_decode(&msg, &decode);\n", lower_name);
    
    let json = serde_json::to_string(msg).unwrap();
    let mut json_clone = json.clone();
    json_clone = json_clone.split_off(1);
    println!("{}", json_clone);
    while let Some((k,v)) = get_json_key_value(&mut json_clone) {
        if k == "type" {
            continue;
        }
        //println!("KV {k} {v}");
        let re_float_number = Regex::new("^(-?\\d+(.\\d+(e-?\\d+)?)?)$").unwrap();
        let re_int_number = Regex::new("^(-?\\d+)$").unwrap();
        let re_enum = Regex::new("^\\{\"type\":\"([A-Z0-9_]+)\"\\}$").unwrap();
        let re_flags = Regex::new("^\"[A-Z0-9_]+( \\| [A-Z0-9_]+)*\"$").unwrap();
        if v == "null" {
            let mut name = k.as_str();
            // NAN
            str += &format!("assert(decode.{name} != decode.{name});\n");
        } else if let Some(_) = re_int_number.captures(&v) {
            let mut name = k.as_str();
            if name == "mavtype" {
                name = "type";
            }
            let value: f64 = v.parse().unwrap();
            if value >= 0.0 {
                let value: u64 = v.parse().unwrap();
                //println!("U64  {name}: {value}");
                str += &format!("assert(decode.{name} == {value}ULL);\n");
            } else {
                let value: i64 = v.parse().unwrap();
                //println!("I64  {name}: {value}");
                str += &format!("assert(decode.{name} == {value});\n");
            } 
        } else if let Some(_) = re_float_number.captures(&v) {
            let mut name = k.as_str();
            if name == "mavtype" {
                name = "type";
            }
            let value: f64 = v.as_str().parse().unwrap();
            //println!("DBL  {name}: {value:e}");
            //str += &format!("if (decode.{name} != {value:e}) printf(\"Value: %30.30f\\n\", decode.{name});\n");
            str += &format!("if (sizeof(decode.{name}) == 4) {{ assert((float)decode.{name} == (float){value:e}); }} else {{ assert(decode.{name} == {value:e}); }}\n");
        } else if let Some(caps) = re_enum.captures(&v) {
            let mut name = k.as_str();
            if name == "mavtype" {
                name = "type";
            }
            let mut value = caps.get(1).unwrap().as_str().to_string();
            if value == "UNDER_WAY" {
                //value = "AIS_NAV_STATUS_UNDER_WAY".to_string();
            }
            str += &format!("assert(decode.{name} == {value});\n");
        } else if let Some(_) = re_flags.captures(&v) {
            let mut name = k.as_str();
            if name == "mavtype" {
                name = "type";
            }
            let value = &v[1..v.len()-1];
            str += &format!("assert(decode.{name} == {value});\n");
        } else if v.starts_with('[') {
            let name = k.as_str();
            let mut index = 0;
            for indexed_v in v[1..v.len()-1].split(",") {
                if indexed_v == "null" {
                    // NaN
                    str += &format!("assert(decode.{name}[{index}] != decode.{name}[{index}]);\n");
                } else {
                    //str += &format!("if ((uint8_t)decode.{name}[{index}] != {indexed_v}) printf(\"Value: %d\\n\", (uint8_t)(decode.{name}[{index}]));\n");
                    str += &format!("if (strcmp(typename(decode.{name}[{index}]), \"char\") == 0) {{\n  assert((uint8_t)decode.{name}[{index}] == {indexed_v});\n}} else if (strcmp(typename(decode.{name}[{index}]), \"float\") == 0) {{\n  assert(decode.{name}[{index}] == (float){indexed_v}); \n}} else {{\n  assert(decode.{name}[{index}] == {indexed_v});\n}}\n");
                }
                index += 1;
            }
            
        } else {
            println!("unknown value pattern: {v}");
            panic!();
        }
    }
    std::fs::write("c_msg_asserts.c", &str).unwrap();
    print!("{str}");
}
