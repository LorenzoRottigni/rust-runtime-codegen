use libloading::{Library, Symbol};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

/// signature of function which should be called from plugin
type AddFunc = unsafe fn(isize, isize) -> isize;

/// Create a plugin file at runtime which will be converted to shared library
fn write_file() -> std::io::Result<()> {
    let mut file = File::create("gen/plugin.rs")?;
    file.write_all(b"fn main() {\n")?;
    file.write_all(b"\t#[no_mangle]\n")?;
    file.write_all(b"\tpub extern \"C\" fn add(a: isize, b: isize) -> isize {\n")?;
    file.write_all(b"\t\ta + b\n")?;
    file.write_all(b"\t}\n")?;
    file.write_all(b"}\n")?;
    Ok(())
}

/// Compile plugin code file to shared library
fn compile_file() {
    let mut compile_file = Command::new("sh");
    compile_file
        .arg("-c")
        .arg("rustc --crate-type cdylib gen/plugin.rs -o gen/plugin")
        .status()
        .expect("process failed to execute");
}

/// Call function from shared library
fn call_plugin(a: isize, b: isize) -> isize {
    // Detect the platform and load the library accordingly
    let lib_path = match std::env::consts::OS {
        "linux" => "gen/plugin.so",    // Linux
        "macos" => "gen/plugin.dylib", // macOS
        "windows" => "gen/plugin.dll", // Windows
        _ => panic!("Unsupported OS"),
    };

    let lib = Library::new(lib_path).unwrap(); // Use the appropriate path for the current OS
    unsafe {
        let func: Symbol<AddFunc> = lib.get(b"add").unwrap();
        let answer = func(a, b);
        answer
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 {
        match write_file() {
            Ok(_) => println!("Emitted bundle"),
            Err(e) => println!("Error: {}", e),
        }
        compile_file();
        let a: isize = args[1].trim().parse().expect("number required");
        let b: isize = args[2].trim().parse().expect("number required");
        println!("{}+{}:{}", a, b, call_plugin(a, b));
    } else {
        println!("USAGE: main.exe NUM NUM");
    }
}
