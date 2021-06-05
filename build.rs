use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;

const std_cfg: &str = 
r#"# true, if this runs as a CORS API only. If true, ZK will host no static files and will allow CORS headers, as well as OPTIONS request methods.
cors = false
# must be set, if cors = true. URL where the application is hosted
cors_origin = "http://localhost:8080"
# must be set, if cors = false. Location of static files to serve
static_files_location = "/home/simon/repos/zk_vue/dist/"
# Location o the user repositories
repo_files_location = "/home/simon/repos/notes/"
# Hostname of the server
hostname = "localhost"
# Start password for the "admin" user
admin_password = "wolfgang""#;

fn main() {
    // println!("cargo:rerun-if-changed=config.json");
    println!("cargo:warning=Hello from build.rs");
    println!("cargo:warning=CWD is {:?}", env::current_dir().unwrap());
    println!("cargo:warning=OUT_DIR is {:?}", env::var("OUT_DIR").unwrap());
    println!("cargo:warning=CARGO_MANIFEST_DIR is {:?}", env::var("CARGO_MANIFEST_DIR").unwrap());
    println!("cargo:warning=PROFILE is {:?}", env::var("PROFILE").unwrap());

    let output_path = get_output_path();
    println!("cargo:warning=Calculated build path: {}", output_path.to_str().unwrap());

    let mut file = File::create(Path::new(&output_path).join("ZK.toml")).expect("Error writing config file!");
    // let mut file = File::create(Path::new(&env::current_dir().unwrap()).join("ZK.toml")).unwrap();
    let res = file.write_all(std_cfg.as_bytes()).unwrap();
    print!("cargo:warning={:#?}",res);

    let input_path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("Rocket.toml");
    let res = std::fs::copy(input_path, output_path);
    println!("cargo:warning={:#?}",res);

}

fn get_output_path() -> PathBuf {
    //<root or manifest path>/target/<profile>/
    let manifest_dir_string = env::var("CARGO_MANIFEST_DIR").unwrap();
    let build_type = env::var("PROFILE").unwrap();
    let path = Path::new(&manifest_dir_string).join("target").join(build_type);
    return PathBuf::from(path);
}
