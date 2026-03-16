use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

#[doc(hidden)]
#[proc_macro_attribute]
pub fn script_exports(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input: ItemStruct = parse_macro_input!(item as ItemStruct);
    let name: &Ident = &input.ident;

    let expanded = quote! {
        #input

        impl #name {
            fn assert_script<S: Script>(_: &S) {}
        }

        use std::ffi::{c_void, c_char, CStr, CString};
        use osbot_api::eframe::egui;
        use osbot_api::eframe::egui::Memory;
        use osbot_api::c_vec::CVec;
        use osbot_api::api::ui::chatbox::{ChatMessageType, ChatMessageListener};
        use osbot_api::api::domain::chat_message::RSChatMessage;
        use osbot_api::api::script::script_metadata::ScriptMetadataFFI;
        use osbot_api::log;
        use osbot_api::log::{Log, Level, Record, Metadata};

        static mut SCRIPT: Option<#name> = None;
        static mut STOPPED: bool = false;
        static mut COMPLETED: bool = false;

        static mut CHAT_MESSAGE_LISTENERS: Vec<Box<dyn ChatMessageListener + 'static>> = Vec::new();

        static mut LOG_FN: Option<unsafe extern "C" fn(*const Record)> = None;

        static mut UI_MEMORY: Option<Memory> = None;
        static mut UI_DEBUG_MEMORY: Option<Memory> = None;

        struct Logger { }
        impl Log for Logger {
            fn enabled(&self, metadata: &Metadata) -> bool {
                true
            }

            fn log(&self, record: &Record) {
                unsafe {
                    if let Some(log_fn) = LOG_FN {
                        log_fn(record as *const Record);
                    }
                }
            }

            fn flush(&self) { }
        }

        pub fn script_add_chat_message_listener<F>(listener: F)
        where
            F: ChatMessageListener + 'static
        {
            unsafe {
                CHAT_MESSAGE_LISTENERS.push(Box::new(listener));
            }
        }

        pub fn script_stop() {
            unsafe { STOPPED = true; }
        }

        pub fn script_complete() {
            unsafe { COMPLETED = true; }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_metadata() -> ScriptMetadataFFI {
            use osbot_api::api::script::script_metadata::ScriptMetadata;
            let metadata: ScriptMetadata = metadata();

            let name: CString = CString::new(metadata.name).unwrap();
            let author: CString = CString::new(metadata.author).unwrap();
            let info: CString = CString::new(metadata.info).unwrap();
            let logo: CString = CString::new(metadata.logo).unwrap();

            ScriptMetadataFFI {
                name: name.into_raw(),
                author: author.into_raw(),
                version: metadata.version,
                info: info.into_raw(),
                logo: logo.into_raw(),
                category: metadata.category,
            }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_metadata_free(meta: ScriptMetadataFFI) {
            unsafe {
                if !meta.name.is_null() { let _ = CString::from_raw(meta.name as *mut c_char); }
                if !meta.author.is_null() { let _ = CString::from_raw(meta.author as *mut c_char); }
                if !meta.info.is_null() { let _ = CString::from_raw(meta.info as *mut c_char); }
                if !meta.logo.is_null() { let _ = CString::from_raw(meta.logo as *mut c_char); }
            }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_initialize(log_fn: *mut c_void) -> bool {
            unsafe {
                if SCRIPT.is_some() {
                    return false;
                }

                SCRIPT = Some(<#name as Script>::new());

                LOG_FN = Some(std::mem::transmute(log_fn));

                log::set_boxed_logger(Box::new(Logger {})).unwrap();
                log::set_max_level(log::LevelFilter::Info);

                true
            }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_terminate() -> bool {
            unsafe {
                if SCRIPT.is_none() {
                    return false;
                }

                let _ = SCRIPT.take();

                CHAT_MESSAGE_LISTENERS.clear();
                CHAT_MESSAGE_LISTENERS.shrink_to_fit();
                LOG_FN = None;
                UI_MEMORY = None;
                UI_DEBUG_MEMORY = None;

                true
            }
        }

        fn _script_report_error(name: &str, error: Box<dyn std::any::Any + core::marker::Send>) {
            use osbot_api::log::error;

            match error.downcast::<String>() {
                Ok(s) => {
                    error!("Panic in script.{}: {}", name, s);
                }
                Err(error) => match error.downcast::<&'static str>() {
                    Ok(s) => {
                        error!("Panic in script.{}: {}", name, s);
                    }
                    Err(_) => {
                        error!("Panic in script.{}: <unknown panic payload>", name);
                    }
                },
            }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_on_start(params: *const c_char) {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                unsafe {
                    if let Some(script) = SCRIPT.as_mut() {
                        let params = if !params.is_null() {
                            Some(CStr::from_ptr(params).to_string_lossy().into_owned())
                        } else {
                            None
                        };

                        script.on_start(params);
                    }
                }
            })) {
                Ok(_) => {},
                Err(err) => {
                    _script_report_error("on_start", err);
                }
            }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_on_loop() -> i32 {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                unsafe {
                    if let Some(script) = SCRIPT.as_mut() {
                        return script.on_loop();
                    }
                }

                0
            })) {
                Ok(value) => value,
                Err(err) => {
                    _script_report_error("on_loop", err);
                    1000
                }
            }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_can_start() -> bool {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                unsafe {
                    if let Some(script) = SCRIPT.as_ref() {
                        return script.can_start();
                    }
                }

                true
            })) {
                Ok(value) => value,
                Err(err) => {
                    _script_report_error("can_start", err);
                    false
                }
            }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_can_break() -> bool {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                unsafe {
                    if let Some(script) = SCRIPT.as_ref() {
                        return script.can_break();
                    }
                }

                true
            })) {
                Ok(value) => value,
                Err(err) => {
                    _script_report_error("can_break", err);
                    false
                }
            }
        }

         #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_can_login() -> bool {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                unsafe {
                    if let Some(script) = SCRIPT.as_ref() {
                        return script.can_login();
                    }
                }

                true
            })) {
                Ok(value) => value,
                Err(err) => {
                    _script_report_error("can_login", err);
                    false
                }
            }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_should_stop() -> bool {
            unsafe { STOPPED }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_should_complete() -> bool {
            unsafe { COMPLETED }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_on_stop() {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                unsafe {
                    if let Some(script) = SCRIPT.as_mut() {
                        script.on_stop();
                    }
                }
            })) {
                Ok(_) => {},
                Err(err) => {
                    _script_report_error("on_stop", err);
                }
            }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_on_pause() {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                unsafe {
                    if let Some(script) = SCRIPT.as_mut() {
                        script.on_pause();
                    }
                }
            })) {
                Ok(_) => {},
                Err(err) => {
                    _script_report_error("on_pause", err);
                }
            }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_on_resume() {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                unsafe {
                    if let Some(script) = SCRIPT.as_mut() {
                        script.on_resume();
                    }
                }
            })) {
                Ok(_) => {},
                Err(err) => {
                    _script_report_error("on_resume", err);
                }
            }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_on_render(ui_ptr: *mut c_void) {
            if ui_ptr.is_null() {
                return;
            }

            unsafe {
                let ui = &mut *(ui_ptr as *mut egui::Ui);

                use crate::egui::util::IdTypeMap;
                ui.ctx().memory_mut(|m| {
                    let plugin_data = UI_MEMORY.get_or_insert_with(Memory::default);
                    std::mem::swap(&mut m.data, &mut plugin_data.data);
                });

                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    unsafe {
                        if let Some(script) = SCRIPT.as_ref() {
                            let ui = &mut *(ui_ptr as *mut egui::Ui);

                            script.on_render(ui);
                        }
                    }
                })) {
                    Ok(_) => {},
                    Err(err) => {
                        _script_report_error("on_render", err);
                    }
                }

                ui.ctx().memory_mut(|m| {
                    std::mem::swap(&mut m.data, &mut UI_MEMORY.as_mut().unwrap().data);
                });
            }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_on_debug_render(ui_ptr: *mut c_void) {
            if ui_ptr.is_null() {
                return;
            }

            unsafe {
                let ui = &mut *(ui_ptr as *mut egui::Ui);

                use crate::egui::util::IdTypeMap;
                ui.ctx().memory_mut(|m| {
                    let plugin_data = UI_DEBUG_MEMORY.get_or_insert_with(Memory::default);
                    std::mem::swap(&mut m.data, &mut plugin_data.data);
                });

                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    unsafe {
                        if let Some(script) = SCRIPT.as_ref() {
                            let ui = &mut *(ui_ptr as *mut egui::Ui);

                            script.on_debug_render(ui);
                        }
                    }
                })) {
                    Ok(_) => {},
                    Err(err) => {
                        _script_report_error("on_debug_render", err);
                    }
                }

                ui.ctx().memory_mut(|m| {
                    std::mem::swap(&mut m.data, &mut UI_DEBUG_MEMORY.as_mut().unwrap().data);
                });
            }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_get_disabled_random_events() -> CVec {
            unsafe {
                if let Some(script) = SCRIPT.as_ref() {
                    let mut events = script.get_disabled_random_events();

                    let len = events.len();
                    let capacity = events.capacity();
                    let ptr = events.as_mut_ptr() as *mut c_void;

                    std::mem::forget(events);
                    return CVec::new(ptr, len, capacity);
                }
            }

            CVec::new(std::ptr::null_mut(), 0, 0)
        }
        
        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_get_chat_message_types() -> CVec {
            unsafe {
                if let Some(script) = SCRIPT.as_ref() {
                    let mut types = script.get_chat_message_types();

                    let len = types.len();
                    let capacity = types.capacity();
                    let ptr = types.as_mut_ptr() as *mut c_void;

                    std::mem::forget(types);
                    return CVec::new(ptr, len, capacity);
                }
            }

            CVec::new(std::ptr::null_mut(), 0, 0)
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_on_chat_message(chat_message_type: ChatMessageType, chat_message_ptr: *mut c_void) {
            if chat_message_ptr.is_null() {
                return;
            }
            unsafe {
                if let Some(script) = SCRIPT.as_mut() {
                    let chat_message = &mut *(chat_message_ptr as *mut RSChatMessage);
                    script.on_chat_message(chat_message_type, &chat_message);

                    for listener in &mut CHAT_MESSAGE_LISTENERS {
                        listener.on_chat_message(chat_message_type, &chat_message);
                    }
                }
            }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_free_vec(vec: CVec) {
            unsafe {
                if !vec.get_ptr().is_null() {
                    let _ = Vec::from_raw_parts(vec.get_ptr(), vec.get_len(), vec.get_capacity());
                }
            }
        }
    };

    TokenStream::from(expanded)
}