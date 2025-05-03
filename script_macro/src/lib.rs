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

        static mut SCRIPT: Option<#name> = None;
        static mut STOPPED: bool = false;
        static mut COMPLETED: bool = false;

        use std::ffi::{c_void, c_char, CStr};
        use osbot_api::eframe::egui;
        use osbot_api::c_vec::CVec;
        use osbot_api::api::ui::chatbox::ChatMessageType;
        use osbot_api::api::domain::chat_message::RSChatMessage;

        #[no_mangle]
        pub extern "C" fn script_stop() {
            unsafe { STOPPED = true; }
        }

        #[no_mangle]
        pub extern "C" fn script_complete() {
            unsafe { COMPLETED = true; }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_initialize() -> bool {
            unsafe {
                if SCRIPT.is_some() {
                    return false;
                }

                SCRIPT = Some(<#name as Script>::new());
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

                SCRIPT.take();
                true
            }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_on_start(params: *const c_char) {
            unsafe {
                if let Some(script) = SCRIPT.as_ref() {
                    let params = if !params.is_null() {
                        Some(CStr::from_ptr(params).to_string_lossy().into_owned())
                    } else {
                        None
                    };

                    script.on_start(params);
                }
            }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_on_loop() -> i32 {
            unsafe {
                if let Some(script) = SCRIPT.as_ref() {
                    return script.on_loop();
                }
            }
            0
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_can_start() -> bool {
            unsafe {
                if let Some(script) = SCRIPT.as_ref() {
                    return script.can_start();
                }
            }
            true
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_can_break() -> bool {
            unsafe {
                if let Some(script) = SCRIPT.as_ref() {
                    return script.can_break();
                }
            }
            true
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
            unsafe {
                if let Some(script) = SCRIPT.as_ref() {
                    script.on_stop();
                }
            }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_on_pause() {
            unsafe {
                if let Some(script) = SCRIPT.as_ref() {
                    script.on_pause();
                }
            }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_on_resume() {
            unsafe {
                if let Some(script) = SCRIPT.as_ref() {
                    script.on_resume();
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
                if let Some(script) = SCRIPT.as_mut() {
                    let ui = &mut *(ui_ptr as *mut egui::Ui);
                    script.on_render(ui);
                }
            }
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn _script_on_debug_render(ui_ptr: *mut c_void) {
            if ui_ptr.is_null() {
                return;
            }
            unsafe {
                if let Some(script) = SCRIPT.as_mut() {
                    let ui = &mut *(ui_ptr as *mut egui::Ui);
                    script.on_debug_render(ui);
                }
            }
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
                if let Some(script) = SCRIPT.as_ref() {
                    let chat_message = &mut *(chat_message_ptr as *mut RSChatMessage);
                    script.on_chat_message(chat_message_type, &chat_message);
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