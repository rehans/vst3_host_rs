// Copyright(c) 2026-present Hansen Audio.

use std::{error::Error, ffi::c_void, path::PathBuf};
use libloading::{Library, Symbol};
use vst3_rust_api::*;

use crate::hosting::plugin_factory::*;

type GetPluginFactoryFn = unsafe extern "C" fn() -> *mut Steinberg_IPluginFactory;
type ModuleEntryFn = unsafe extern "C" fn(module: *mut c_void) -> bool;
type ModuleExitFn = unsafe extern "C" fn() -> bool;

pub struct Module {
    library: Option<Library>,
    plugin_factory: Option<Box<dyn IPluginFactory>>,
}

impl Module {
    pub fn new() -> Self {
        Self {
            library: Option::None,
            plugin_factory: Option::None,
        }
    }

    pub fn get_plugin_factory(&self) -> &dyn IPluginFactory {
        self.plugin_factory.as_ref().unwrap().as_ref()
    }

    pub fn load(&mut self, path: &PathBuf) -> Result<(), Box<dyn Error>> {
        unsafe {
            self.library = Library::new(path).ok();
            let lib = self.library.as_ref().unwrap();

            let mut available = lib.get::<ModuleEntryFn>(b"ModuleEntry\0").is_ok();
            if !available {
                return Err("ModuleEntry symbol does not exist!".into());
            }

            available = lib.get::<ModuleExitFn>(b"ModuleExit\0").is_ok();
            if !available {
                return Err("ModuleExit symbol does not exist!".into());
            }

            available = lib.get::<GetPluginFactoryFn>(b"GetPluginFactory\0").is_ok();
            if !available {
                return Err("GetPluginFactory symbol does not exist!".into());
            }

            let module_entry: Symbol<ModuleEntryFn> = lib.get(b"ModuleEntry\0")?;
            let result = module_entry(std::ptr::null_mut());
            if !result {
                return Err("ModuleEntry symbol does not exist!".into());
            }

            let get_plugin_factory: Symbol<GetPluginFactoryFn> = lib.get(b"GetPluginFactory\0")?;
            let plugin_factory_ptr = get_plugin_factory();
            if plugin_factory_ptr.is_null() {
                return Err("GetPluginFactory returned null".into());
            }

            self.plugin_factory = PluginFactory::new(plugin_factory_ptr)
                .map(|factory| Box::new(factory) as Box<dyn IPluginFactory>);

            Ok(())
        }
    }

    pub fn unload(&mut self) -> Result<(), Box<dyn Error>> {
        unsafe {
            match &self.library {
                Some(lib) => {
                    let module_exit: Symbol<ModuleExitFn> = lib.get(b"ModuleExit\0")?;
                    let result = module_exit();
                    if !result {
                        return Err("ModuleExit successful!".into());
                    }
                }
                None => return Err("No library available!".into()),
            }

            Ok(())
        }
    }

    pub fn read_factory_info(&self) -> Result<Steinberg_PFactoryInfo, String> {
        unsafe {
            let mut info = Steinberg_PFactoryInfo {
                vendor: [0; 64],
                url: [0; 256],
                email: [0; 128],
                flags: 0,
            };

            let result = self.get_plugin_factory().get_factory_info(&mut info);

            if result != 0 {
                return Err(format!("get_factory_info failed with result {result}"));
            }

            Ok(info)
        }
    }
}
