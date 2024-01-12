use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

fn main() {
    if !env::var("TARGET").unwrap().contains("android") {
        return;
    }

    let out_dir = env::var("OUT_DIR").unwrap();

    let slint_path = "dev/slint/android-activity";

    let out_class = format!("{out_dir}/java/{slint_path}");

    let android_home =
        PathBuf::from(env::var("ANDROID_HOME").or_else(|_| env::var("ANDROID_SDK_ROOT")).expect(
            "Please set the ANDROID_HOME environment variable to the path of the Android SDK",
        ));

    let classpath = find_latest_version(android_home.join("platforms"), "android.jar")
        .expect("No Android platforms found");

    // Try to locate javac
    let javac_path = match env::var("JAVA_HOME") {
        Ok(val) => {
            if cfg!(windows) {
                format!("{}\\bin\\javac.exe", val)
            } else {
                format!("{}/bin/javac", val)
            }
        }
        Err(_) => String::from("javac"),
    };

    // Compile the Java file into a .class file
    let o = Command::new(&javac_path)
        .arg(format!("java/HelloWorld.java"))
        .arg("-d")
        .arg(&out_class)
        .arg("-classpath").arg(&classpath)
        .arg("--release")
        .arg("8")
        .output()
        .unwrap_or_else(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                panic!("Could not locate the java compiler. Please ensure that the JAVA_HOME environment variable is set correctly.")
            } else {
                panic!("Could not run {javac_path}: {err}")
            }
        });

    if !o.status.success() {
        panic!("Java compilation failed: {}", String::from_utf8_lossy(&o.stderr));
    }

    // Convert the .class file into a .dex file
    let d8_path = find_latest_version(
        android_home.join("build-tools"),
        if cfg!(windows) { "d8.exe" } else { "d8" },
    )
    .expect("d8 tool not found");
    let o = Command::new(&d8_path)
        .args(&["--classpath", &out_class])
        .arg(format!("{out_class}/HelloWorld.class"))
        .arg("--output")
        .arg(&out_dir)
        .output()
        .unwrap_or_else(|err| panic!("Error running {d8_path:?}: {err}"));

    if !o.status.success() {
        panic!("Dex conversion failed: {}", String::from_utf8_lossy(&o.stderr));
    }

    // Tell Cargo to re-run this build script if the Java file changes
    println!("cargo:rerun-if-changed=java/HelloWorld.java");
}

fn find_latest_version(base: PathBuf, arg: &str) -> Option<PathBuf> {
    fs::read_dir(base)
        .ok()?
        .filter_map(|entry| Some(entry.ok()?.path().join(arg)))
        .filter(|path| path.exists())
        .max()
}
