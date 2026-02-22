use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"));
    let web_dir = PathBuf::from("web");
    if !web_dir.exists() {
        panic!("missing `web/` directory; web assets are required for builds");
    }

    emit_rerun_for_paths(&[
        PathBuf::from("pnpm-workspace.yaml"),
        PathBuf::from("pnpm-lock.yaml"),
        PathBuf::from("package.json"),
        web_dir.join("src"),
        web_dir.join("public"),
        web_dir.join("index.html"),
        web_dir.join("package.json"),
        web_dir.join("components.json"),
        web_dir.join("postcss.config.js"),
        web_dir.join("vite.config.ts"),
        web_dir.join("tsconfig.json"),
        web_dir.join("tsconfig.node.json"),
    ]);

    // Build inside OUT_DIR so that node_modules and dist are not created in
    // the source tree.  This is required for `cargo publish` which rejects
    // build scripts that modify the source directory.
    let build_root = out_dir.join("web-build");
    copy_dir(&web_dir, &build_root.join("web"));
    for file in &["pnpm-workspace.yaml", "pnpm-lock.yaml", "package.json"] {
        let src = PathBuf::from(file);
        if src.exists() {
            fs::copy(&src, build_root.join(file)).unwrap_or_else(|err| {
                panic!("failed to copy {file} to OUT_DIR: {err}");
            });
        }
    }

    run_pnpm(
        &build_root,
        &["install", "--frozen-lockfile", "--filter", "./web..."],
    );
    run_pnpm(&build_root, &["--filter", "./web", "build"]);
}

fn emit_rerun_for_paths(paths: &[PathBuf]) {
    for path in paths {
        if path.is_dir() {
            emit_rerun_for_dir(path);
        } else {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}

fn emit_rerun_for_dir(dir: &Path) {
    println!("cargo:rerun-if-changed={}", dir.display());

    let mut stack = vec![dir.to_path_buf()];
    while let Some(next) = stack.pop() {
        let entries = match fs::read_dir(&next) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if should_skip_path(&path) {
                continue;
            }

            if path.is_dir() {
                stack.push(path.clone());
            }
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}

fn should_skip_path(path: &Path) -> bool {
    path.file_name()
        .and_then(OsStr::to_str)
        .map(|name| matches!(name, "node_modules" | "dist"))
        .unwrap_or(false)
}

fn copy_dir(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap_or_else(|err| {
        panic!("failed to create {}: {err}", dst.display());
    });

    let mut stack = vec![(src.to_path_buf(), dst.to_path_buf())];
    while let Some((s, d)) = stack.pop() {
        let entries = match fs::read_dir(&s) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let dest = d.join(entry.file_name());

            if should_skip_path(&path) {
                continue;
            }

            if path.is_dir() {
                fs::create_dir_all(&dest).unwrap_or_else(|err| {
                    panic!("failed to create {}: {err}", dest.display());
                });
                stack.push((path, dest));
            } else {
                fs::copy(&path, &dest).unwrap_or_else(|err| {
                    panic!(
                        "failed to copy {} -> {}: {err}",
                        path.display(),
                        dest.display()
                    );
                });
            }
        }
    }
}

fn run_pnpm(cwd: &Path, args: &[&str]) {
    let status = Command::new("pnpm")
        .env("CI", "true")
        .current_dir(cwd)
        .args(args)
        .status()
        .unwrap_or_else(|err| {
            panic!(
                "failed to run `pnpm {}`: {err}. Install Node.js + pnpm to build Earl web assets.",
                args.join(" ")
            )
        });

    if !status.success() {
        panic!(
            "`pnpm {}` failed with status {status}. Fix the web build before running cargo commands.",
            args.join(" ")
        );
    }
}
