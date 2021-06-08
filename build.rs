use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;

const STD_CFG: &str = 
r#"# Config File, generated at build time.
# true, if this runs as a CORS API only. If true, ZK will host no static files and will allow CORS headers, as well as OPTIONS request methods.
# Also, see that X-Frame-Options is set to deny on all requests to avoid clickjacking attacks. Recommended are also gzip compression & cache-control & CSRF practices.
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
admin_password = """#;

const ROCKET_CFG: &str = 
r#"# Config File, generated at build time.
[development]
address = "localhost"
port = 8000
workers = 12
keep_alive = 5
read_timeout = 5
write_timeout = 5
log = "normal"
limits = { forms = 32768 }

[staging]
address = "0.0.0.0"
port = 8000
workers = 12
keep_alive = 5
read_timeout = 5
write_timeout = 5
log = "normal"
limits = { forms = 32768 }

[production]
address = "localhost"
port = 8000
workers = 12
keep_alive = 5
read_timeout = 5
write_timeout = 5
log = "critical"
limits = { forms = 32768 }"#;

fn main() {
    // println!("cargo:rerun-if-changed=config.json");
    println!("cargo:warning=Hello from build.rs");
    println!("cargo:warning=CWD is {:?}", env::current_dir().unwrap());
    println!("cargo:warning=OUT_DIR is {:?}", env::var("OUT_DIR").unwrap());
    println!("cargo:warning=CARGO_MANIFEST_DIR is {:?}", env::var("CARGO_MANIFEST_DIR").unwrap());
    println!("cargo:warning=PROFILE is {:?}", env::var("PROFILE").unwrap());

    let output_path = get_output_path();
    println!("cargo:warning=Calculated build path: {}", output_path.to_str().unwrap());

    // Export ZK.toml
    let mut destfile = File::create(Path::new(&output_path).join("ZK.toml")).expect("Error writing config file!");
    let mut devfile = File::create(Path::new(&env::current_dir().unwrap()).join("ZK.toml")).unwrap();
    let res = destfile.write_all(STD_CFG.as_bytes());
    let res2 = devfile.write_all(STD_CFG.as_bytes());
    print!("cargo:warning={:?}",res);
    print!("cargo:warning={:?}",res2);

    
    // Export Rocket.toml
    let mut destfile = File::create(Path::new(&output_path).join("Rocket.toml")).expect("Error writing config file!");
    let mut devfile = File::create(Path::new(&env::current_dir().unwrap()).join("Rocket.toml")).unwrap();
    let res = destfile.write_all(ROCKET_CFG.as_bytes());
    let res2 = devfile.write_all(ROCKET_CFG.as_bytes());
    print!("cargo:warning={:?}",res);
    print!("cargo:warning={:?}",res2);

}

fn get_output_path() -> PathBuf {
    //<root or manifest path>/target/<profile>/
    let manifest_dir_string = env::var("CARGO_MANIFEST_DIR").unwrap();
    let build_type = env::var("PROFILE").unwrap();
    let path = Path::new(&manifest_dir_string).join("target").join(build_type);
    return PathBuf::from(path);
}
