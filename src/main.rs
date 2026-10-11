// Copyright(c) 2026-present Hansen Audio.

use vst3_rust_api::*;
use walkdir::WalkDir;

pub mod hosting;
use std::{error::Error, ffi::CStr, path::PathBuf};

use crate::hosting::module::Module;

unsafe fn print_factory_info(info: &Steinberg_PFactoryInfo) {
    unsafe {
        let vendor = CStr::from_ptr(info.vendor.as_ptr());
        let url = CStr::from_ptr(info.url.as_ptr());
        let email = CStr::from_ptr(info.email.as_ptr());

        println!(
            "vendor: {}, url: {}, email: {}, flags: {}",
            vendor.to_string_lossy(),
            url.to_string_lossy(),
            email.to_string_lossy(),
            info.flags,
        );
    }
}

unsafe fn print_class_info(info: &Steinberg_PClassInfo) {
    unsafe {
        let cat = CStr::from_ptr(info.category.as_ptr());
        let name = CStr::from_ptr(info.name.as_ptr());

        let hex: String = info
            .cid
            .iter()
            .map(|&byte| format!("{:02X}", byte as u8))
            .collect();

        println!(
            "cid: {}, category: {}, name: {}, cardinality: {}",
            hex,
            cat.to_string_lossy(),
            name.to_string_lossy(),
            info.cardinality,
        );
    }
}

fn collect_plugin_paths(vst3_dir: PathBuf) -> Vec<PathBuf> {
    let paths: Vec<PathBuf> = WalkDir::new(vst3_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "so"))
        .filter(|entry| {
            entry
                .path()
                .components()
                .any(|component| component.as_os_str().to_string_lossy().ends_with(".vst3"))
        })
        .map(|entry| entry.into_path())
        .collect();
    paths
}

fn main() -> Result<(), Box<dyn Error>> {
    let vst3_dir: PathBuf = std::env::var_os("HOME")
        .map(PathBuf::from)
        .expect("HOME is not set")
        .join(".vst3");

    let paths = collect_plugin_paths(vst3_dir);

    for path in &paths {
        let module_path = path;
        let mut module = Module::new();
        module.load(&module_path)?;

        unsafe {
            let class_count = module.get_plugin_factory().count_classes();
            for i in 0..class_count {
                let mut info = Steinberg_PClassInfo {
                    cid: [0; std::mem::size_of::<Steinberg_TUID>()],
                    cardinality: 0,
                    category: [0; 32],
                    name: [0; 64],
                };
                module.get_plugin_factory().get_class_info(i, &mut info);
                print_class_info(&info);
            }
        };

        let info = module.read_factory_info()?;
        unsafe { print_factory_info(&info) };

        module.unload();
    }

    Ok(())
}
