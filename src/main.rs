use anyhow::Result;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::Deserialize;
use std::io::Write;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::process::Command;

const SOCK_ADDR: &str = "/tmp/luks.sock";

const KBS_ROOT_CERT: &str = "/etc/kbs-root.crt";
const PARAMS_FILE: &str = "/etc/attest-params.json";

#[derive(Clone, Deserialize)]
struct Params {
    kbs_url: String,
    key_path: String,
}

fn main() -> Result<()> {
    if Path::new(SOCK_ADDR).exists() {
        std::fs::remove_file(SOCK_ADDR)?;
    }

    let listener = UnixListener::bind(SOCK_ADDR).expect("Failed to bind Unix socket");

    let params_string = std::fs::read_to_string(PARAMS_FILE)?;
    let params = serde_json::from_str::<Params>(&params_string)?;

    loop {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let output = Command::new("kbs-client")
                    .arg("--url")
                    .arg(&params.kbs_url)
                    .arg("--cert-file")
                    .arg(KBS_ROOT_CERT)
                    .arg("get-resource")
                    .arg("--path")
                    .arg(&params.key_path)
                    .env("RUST_LOG", "off")
                    .output()
                    .expect("failed to execute process");
                let key_base64 = String::from_utf8(output.stdout)?;
                println!("{:?}", key_base64);
                let key = STANDARD.decode(&key_base64)?;
                stream.write(&key).unwrap();
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
