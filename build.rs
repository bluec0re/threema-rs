use std::env;
use std::fs;
use std::path;
use webpki::TrustAnchor;

fn main() {
    println!("cargo:rerun-if-changed=src/ca.der");
    let ca = fs::read("src/ca.der").expect("Couldn't open ca.der");
    let trust_anchor = TrustAnchor::try_from_cert_der(&ca).expect("Couldn't parse ca.der");
    let src = "static THREEMA_CA: [TrustAnchor<'static>; 1] = ".to_string()
        + &str::replace(&format!("[{:?}];\n", trust_anchor), ": [", ": &[");

    let target = path::PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not found")).join("src");
    fs::create_dir_all(&target).expect("Couldn't create target dir");
    let fname = target.join("ca.rs");
    fs::write(fname, src.as_bytes()).expect("Couldn't write ca.rs");
}
