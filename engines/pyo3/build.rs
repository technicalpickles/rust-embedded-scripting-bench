// Embed an rpath to the libpython the build linked against (mise/python-build-standalone
// ships libpython3.x.dylib with an @rpath install name).
fn main() {
    let out = std::process::Command::new(std::env::var("PYO3_PYTHON").unwrap_or("python3".into()))
        .args(["-c", "import sysconfig;print(sysconfig.get_config_var('LIBDIR'))"])
        .output()
        .unwrap();
    let libdir = String::from_utf8(out.stdout).unwrap();
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", libdir.trim());
}
