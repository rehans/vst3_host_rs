// Copyright(c) 2026-present Hansen Audio.

use std::{ffi::c_void, ptr::NonNull};

use vst3_rust_api::*;

type PFactoryInfo = Steinberg_PFactoryInfo;
type PClassInfo = Steinberg_PClassInfo;
type Tresult = Steinberg_tresult;
type IID = Steinberg_TUID;

// TODO: This is how IPluginFactory shall be used idiomatically from Rust
pub trait IPluginFactory {
    unsafe fn get_factory_info(&self, info: &mut PFactoryInfo) -> Steinberg_tresult;
    unsafe fn count_classes(&self) -> i32;
    unsafe fn get_class_info(&self, index: i32, info: &mut PClassInfo) -> Tresult;
    unsafe fn create_instance(&self, cid: &IID, _iid: &IID, obj: &mut *mut c_void) -> Tresult;
}

pub struct PluginFactory {
    ptr: NonNull<Steinberg_IPluginFactory>,
}

impl IPluginFactory for PluginFactory {
    unsafe fn get_factory_info(&self, info: &mut PFactoryInfo) -> Steinberg_tresult {
        unsafe {
            let func = self.vtbl().getFactoryInfo.expect("missing getFactoryInfo");

            func(self.ptr.as_ptr().cast(), info)
        }
    }
    unsafe fn count_classes(&self) -> i32 {
        unsafe {
            let func = self.vtbl().countClasses.expect("missing countClasses");

            func(self.ptr.as_ptr().cast())
        }
    }

    unsafe fn get_class_info(&self, index: i32, info: &mut PClassInfo) -> Steinberg_tresult {
        unsafe {
            let func = self.vtbl().getClassInfo.expect("missing getClassInfo");

            func(self.ptr.as_ptr().cast(), index, info)
        }
    }

    unsafe fn create_instance(&self, cid: &IID, iid: &IID, obj: &mut *mut c_void) -> Tresult {
        unsafe {
            let func = self.vtbl().createInstance.expect("missing createInstance");

            func(self.ptr.as_ptr().cast(), cid.as_ptr(), iid.as_ptr(), obj as *mut *mut c_void)
        }
    }
}

impl Drop for PluginFactory {
    fn drop(&mut self) {
        unsafe {
            let release = self.vtbl().release.expect("missing release");

            release(self.ptr.as_ptr().cast());
        }
    }
}

impl PluginFactory {
    pub unsafe fn new(iface: *mut Steinberg_IPluginFactory) -> Option<Self> {
        let ptr = NonNull::new(iface)?;

        Some(Self { ptr })
    }

    unsafe fn vtbl(&self) -> &Steinberg_IPluginFactoryVtbl {
        unsafe { self.ptr.as_ref().lpVtbl.as_ref().expect("null vtable") }
    }
}
