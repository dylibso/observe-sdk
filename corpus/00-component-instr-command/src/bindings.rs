// Generated by `wit-bindgen` 0.16.0. DO NOT EDIT!
pub mod dylibso {
  pub mod observe {
    
    #[allow(clippy::all)]
    pub mod api {
      #[used]
      #[doc(hidden)]
      #[cfg(target_arch = "wasm32")]
      static __FORCE_SECTION_REF: fn() = super::super::super::__link_section;
      #[repr(u8)]
      #[derive(Clone, Copy, Eq, PartialEq)]
      pub enum LogLevel {
        Error,
        Warn,
        Info,
        Debug,
        Trace,
      }
      impl ::core::fmt::Debug for LogLevel {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
          match self {
            LogLevel::Error => {
              f.debug_tuple("LogLevel::Error").finish()
            }
            LogLevel::Warn => {
              f.debug_tuple("LogLevel::Warn").finish()
            }
            LogLevel::Info => {
              f.debug_tuple("LogLevel::Info").finish()
            }
            LogLevel::Debug => {
              f.debug_tuple("LogLevel::Debug").finish()
            }
            LogLevel::Trace => {
              f.debug_tuple("LogLevel::Trace").finish()
            }
          }
        }
      }
      
      impl LogLevel{
        pub(crate) unsafe fn _lift(val: u8) -> LogLevel{
          if !cfg!(debug_assertions) {
            return ::core::mem::transmute(val);
          }
          
          match val {
            0 => LogLevel::Error,
            1 => LogLevel::Warn,
            2 => LogLevel::Info,
            3 => LogLevel::Debug,
            4 => LogLevel::Trace,
            
            _ => panic!("invalid enum discriminant"),
          }
        }
      }
      
      #[repr(u8)]
      #[derive(Clone, Copy, Eq, PartialEq)]
      pub enum MetricFormat {
        Statsd,
      }
      impl ::core::fmt::Debug for MetricFormat {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
          match self {
            MetricFormat::Statsd => {
              f.debug_tuple("MetricFormat::Statsd").finish()
            }
          }
        }
      }
      
      impl MetricFormat{
        pub(crate) unsafe fn _lift(val: u8) -> MetricFormat{
          if !cfg!(debug_assertions) {
            return ::core::mem::transmute(val);
          }
          
          match val {
            0 => MetricFormat::Statsd,
            
            _ => panic!("invalid enum discriminant"),
          }
        }
      }
      
      #[allow(unused_unsafe, clippy::all)]
      pub fn metric(format: MetricFormat,name: &[u8],){
        
        #[allow(unused_imports)]
        use wit_bindgen::rt::{alloc, vec::Vec, string::String};
        unsafe {
          let vec0 = name;
          let ptr0 = vec0.as_ptr() as i32;
          let len0 = vec0.len() as i32;
          
          #[cfg(target_arch = "wasm32")]
          #[link(wasm_import_module = "dylibso:observe/api")]
          extern "C" {
            #[link_name = "metric"]
            fn wit_import(_: i32, _: i32, _: i32, );
          }
          
          #[cfg(not(target_arch = "wasm32"))]
          fn wit_import(_: i32, _: i32, _: i32, ){ unreachable!() }
          wit_import(format.clone() as i32, ptr0, len0);
        }
      }
      #[allow(unused_unsafe, clippy::all)]
      pub fn log(level: LogLevel,msg: &[u8],){
        
        #[allow(unused_imports)]
        use wit_bindgen::rt::{alloc, vec::Vec, string::String};
        unsafe {
          let vec0 = msg;
          let ptr0 = vec0.as_ptr() as i32;
          let len0 = vec0.len() as i32;
          
          #[cfg(target_arch = "wasm32")]
          #[link(wasm_import_module = "dylibso:observe/api")]
          extern "C" {
            #[link_name = "log"]
            fn wit_import(_: i32, _: i32, _: i32, );
          }
          
          #[cfg(not(target_arch = "wasm32"))]
          fn wit_import(_: i32, _: i32, _: i32, ){ unreachable!() }
          wit_import(level.clone() as i32, ptr0, len0);
        }
      }
      #[allow(unused_unsafe, clippy::all)]
      pub fn span_enter(name: &str,){
        
        #[allow(unused_imports)]
        use wit_bindgen::rt::{alloc, vec::Vec, string::String};
        unsafe {
          let vec0 = name;
          let ptr0 = vec0.as_ptr() as i32;
          let len0 = vec0.len() as i32;
          
          #[cfg(target_arch = "wasm32")]
          #[link(wasm_import_module = "dylibso:observe/api")]
          extern "C" {
            #[link_name = "span-enter"]
            fn wit_import(_: i32, _: i32, );
          }
          
          #[cfg(not(target_arch = "wasm32"))]
          fn wit_import(_: i32, _: i32, ){ unreachable!() }
          wit_import(ptr0, len0);
        }
      }
      #[allow(unused_unsafe, clippy::all)]
      pub fn span_tags(tags: &str,){
        
        #[allow(unused_imports)]
        use wit_bindgen::rt::{alloc, vec::Vec, string::String};
        unsafe {
          let vec0 = tags;
          let ptr0 = vec0.as_ptr() as i32;
          let len0 = vec0.len() as i32;
          
          #[cfg(target_arch = "wasm32")]
          #[link(wasm_import_module = "dylibso:observe/api")]
          extern "C" {
            #[link_name = "span-tags"]
            fn wit_import(_: i32, _: i32, );
          }
          
          #[cfg(not(target_arch = "wasm32"))]
          fn wit_import(_: i32, _: i32, ){ unreachable!() }
          wit_import(ptr0, len0);
        }
      }
      #[allow(unused_unsafe, clippy::all)]
      pub fn span_exit(){
        
        #[allow(unused_imports)]
        use wit_bindgen::rt::{alloc, vec::Vec, string::String};
        unsafe {
          
          #[cfg(target_arch = "wasm32")]
          #[link(wasm_import_module = "dylibso:observe/api")]
          extern "C" {
            #[link_name = "span-exit"]
            fn wit_import();
          }
          
          #[cfg(not(target_arch = "wasm32"))]
          fn wit_import(){ unreachable!() }
          wit_import();
        }
      }
      
    }
    
  }
}

#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:component-instr-command"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 454] = [3, 0, 23, 99, 111, 109, 112, 111, 110, 101, 110, 116, 45, 105, 110, 115, 116, 114, 45, 99, 111, 109, 109, 97, 110, 100, 0, 97, 115, 109, 13, 0, 1, 0, 7, 168, 2, 1, 65, 2, 1, 65, 2, 1, 66, 15, 1, 109, 5, 5, 101, 114, 114, 111, 114, 4, 119, 97, 114, 110, 4, 105, 110, 102, 111, 5, 100, 101, 98, 117, 103, 5, 116, 114, 97, 99, 101, 4, 0, 9, 108, 111, 103, 45, 108, 101, 118, 101, 108, 3, 0, 0, 1, 109, 1, 6, 115, 116, 97, 116, 115, 100, 4, 0, 13, 109, 101, 116, 114, 105, 99, 45, 102, 111, 114, 109, 97, 116, 3, 0, 2, 1, 112, 125, 1, 64, 2, 6, 102, 111, 114, 109, 97, 116, 3, 4, 110, 97, 109, 101, 4, 1, 0, 4, 0, 6, 109, 101, 116, 114, 105, 99, 1, 5, 1, 64, 2, 5, 108, 101, 118, 101, 108, 1, 3, 109, 115, 103, 4, 1, 0, 4, 0, 3, 108, 111, 103, 1, 6, 1, 64, 1, 4, 110, 97, 109, 101, 115, 1, 0, 4, 0, 10, 115, 112, 97, 110, 45, 101, 110, 116, 101, 114, 1, 7, 1, 64, 1, 4, 116, 97, 103, 115, 115, 1, 0, 4, 0, 9, 115, 112, 97, 110, 45, 116, 97, 103, 115, 1, 8, 1, 64, 0, 1, 0, 4, 0, 9, 115, 112, 97, 110, 45, 101, 120, 105, 116, 1, 9, 3, 1, 19, 100, 121, 108, 105, 98, 115, 111, 58, 111, 98, 115, 101, 114, 118, 101, 47, 97, 112, 105, 5, 0, 4, 1, 55, 101, 120, 97, 109, 112, 108, 101, 58, 99, 111, 109, 112, 111, 110, 101, 110, 116, 45, 105, 110, 115, 116, 114, 45, 99, 111, 109, 109, 97, 110, 100, 47, 99, 111, 109, 112, 111, 110, 101, 110, 116, 45, 105, 110, 115, 116, 114, 45, 99, 111, 109, 109, 97, 110, 100, 4, 0, 11, 29, 1, 0, 23, 99, 111, 109, 112, 111, 110, 101, 110, 116, 45, 105, 110, 115, 116, 114, 45, 99, 111, 109, 109, 97, 110, 100, 3, 0, 0, 0, 16, 12, 112, 97, 99, 107, 97, 103, 101, 45, 100, 111, 99, 115, 0, 123, 125, 0, 70, 9, 112, 114, 111, 100, 117, 99, 101, 114, 115, 1, 12, 112, 114, 111, 99, 101, 115, 115, 101, 100, 45, 98, 121, 2, 13, 119, 105, 116, 45, 99, 111, 109, 112, 111, 110, 101, 110, 116, 6, 48, 46, 49, 56, 46, 50, 16, 119, 105, 116, 45, 98, 105, 110, 100, 103, 101, 110, 45, 114, 117, 115, 116, 6, 48, 46, 49, 54, 46, 48];

#[inline(never)]
#[doc(hidden)]
#[cfg(target_arch = "wasm32")]
pub fn __link_section() {}
