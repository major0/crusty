// Copyright (c) 2024 Crusty Programming Language
// Licensed under the MIT License. See LICENSE.txt in the project root.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    
    let src_dir = Path::new(&manifest_dir).join("src");
    let out_path = Path::new(&out_dir);
    
    // Discover all .crst files in src/
    let crst_files = discover_crst_files(&src_dir);
    
    if crst_files.is_empty() {
        println!("cargo:warning=No .crst files found in {}", src_dir.display());
        return;
    }
    
    println!("cargo:warning=Found {} .crst files to transpile", crst_files.len());
    
    // Transpile each .crst file
    for crst_file in &crst_files {
        transpile_file(crst_file, &src_dir, out_path);
        
        // Tell Cargo to rerun if this file changes
        println!("cargo:rerun-if-changed={}", crst_file.display());
    }
    
    // Also rerun if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
}

fn discover_crst_files(dir: &Path) -> Vec<PathBuf> {
    let mut crst_files = Vec::new();
    
    if !dir.exists() {
        return crst_files;
    }
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "crst" {
                        crst_files.push(path);
                    }
                }
            } else if path.is_dir() {
                // Recursively search subdirectories
                crst_files.extend(discover_crst_files(&path));
            }
        }
    }
    
    crst_files
}

fn transpile_file(crst_file: &Path, src_dir: &Path, out_dir: &Path) {
    // Get relative path from src_dir
    let rel_path = crst_file.strip_prefix(src_dir)
        .expect("Failed to get relative path");
    
    // Create output path with .rs extension
    let mut out_file = out_dir.join(rel_path);
    out_file.set_extension("rs");
    
    // Create parent directories if needed
    if let Some(parent) = out_file.parent() {
        fs::create_dir_all(parent).expect("Failed to create output directory");
    }
    
    println!("cargo:warning=Transpiling {} to {}", crst_file.display(), out_file.display());
    
    // Invoke crustyc to transpile
    let status = Command::new("crustyc")
        .args(&[
            crst_file.to_str().unwrap(),
            "-o",
            out_file.to_str().unwrap(),
            "--emit=rust",
        ])
        .status()
        .expect("Failed to execute crustyc");
    
    if !status.success() {
        panic!("crustyc failed to transpile {}", crst_file.display());
    }
}
