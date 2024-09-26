#![recursion_limit = "256"]

use std::env;
use std::fs::read_dir;
use std::path::Path;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    // Update and init submodule
    if let Err(error) = Command::new("git")
        .arg("submodule")
        .arg("update")
        .arg("--init")
        .current_dir(src_dir)
        .status()
    {
        eprintln!("{error}");
        return ExitCode::FAILURE;
    }

    // find & apply patches to XML definitions to avoid crashes
    let patch_dir = src_dir.join("build/patches");
    let mavlink_dir = src_dir.join("mavlink");

    if let Ok(dir) = read_dir(patch_dir) {
        for entry in dir.flatten() {
            if let Err(error) = Command::new("git")
                .arg("apply")
                .arg(entry.path().as_os_str())
                .current_dir(&mavlink_dir)
                .status()
            {
                eprintln!("{error}");
                return ExitCode::FAILURE;
            }
        }
    }

    let definitions_dir = src_dir.join("mavlink/message_definitions/v1.0");

    let out_dir = env::var("OUT_DIR").unwrap();


    let mut file_result = vec![];
    let ls = std::fs::read_dir(&definitions_dir).unwrap();
    for entry in ls {
        let path = entry.unwrap().path();
        if path.is_file() {
            file_result.push(path);
        }
    }

    let result = match mavgen::generate_dir(&file_result, Path::new(&out_dir)) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e:?}");
            return ExitCode::FAILURE;
        }
    };

    #[cfg(feature = "format-generated-code")]
    mavlink_bindgen::format_generated_code(&result);

    //mavlink_bindgen::emit_cargo_build_messages(&result);

    ExitCode::SUCCESS
}
