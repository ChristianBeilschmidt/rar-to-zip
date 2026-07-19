use crate::converter::convert_rar_to_zip;
use js_sys::Uint8Array;
use leptos::component;
use leptos::html::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, File};

#[component]
pub fn App() -> impl IntoView {
    let (file_name, set_file_name) = signal::<String>(String::new());
    let (is_converting, set_is_converting) = signal(false);
    let (error_message, set_error_message) = signal::<String>(String::new());
    let (success_message, set_success_message) = signal::<String>(String::new());
    let (zip_data, set_zip_data) = signal::<Option<Vec<u8>>>(None);

    let handle_file_input = move |ev: web_sys::Event| {
        let target = ev.target().unwrap();
        let input = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();
        if let Some(files) = input.files()
            && files.length() > 0
            && let Some(file) = files.get(0)
        {
            set_file_name.set(file.name());
            set_error_message.set(String::new());
            set_success_message.set(String::new());
        }
    };

    let handle_drag_over = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        ev.stop_propagation();
    };

    let handle_drag_leave = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        ev.stop_propagation();
    };

    let handle_drop = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        ev.stop_propagation();

        if let Some(data_transfer) = ev.data_transfer()
            && let Some(files) = data_transfer.files()
            && files.length() > 0
            && let Some(file) = files.get(0)
        {
            let file_name_str = file.name();
            if file_name_str.to_lowercase().ends_with(".rar") {
                set_file_name.set(file_name_str.clone());
                set_error_message.set(String::new());
                set_success_message.set(String::new());

                // Store the dropped file for conversion
                set_is_converting.set(true);
                let file = file.clone();
                spawn_local(async move {
                    match read_file_as_bytes(&file).await {
                        Ok(rar_bytes) => match convert_rar_to_zip(&rar_bytes) {
                            Ok(zip_bytes) => {
                                set_zip_data.set(Some(zip_bytes));
                                set_success_message
                                    .set("✓ Conversion successful! Ready to download.".to_string());
                            }
                            Err(e) => {
                                set_error_message.set(format!("❌ Conversion failed: {}", e));
                            }
                        },
                        Err(e) => {
                            set_error_message.set(format!("❌ Failed to read file: {}", e));
                        }
                    }
                    set_is_converting.set(false);
                });
            } else {
                set_error_message.set("❌ Only .rar files are accepted".to_string());
            }
        }
    };

    let handle_download = move |_| {
        if let Some(data) = zip_data.get() {
            let original_name = file_name.get();
            let zip_filename = original_name
                .trim_end_matches(".rar")
                .trim_end_matches(".RAR")
                .to_string()
                + ".zip";
            download_file(&data, &zip_filename);
        }
    };

    view! {
        <div class="min-h-screen bg-gradient-to-br from-slate-900 to-slate-800 flex items-center justify-center p-4">
            <div class="w-full max-w-md">
                <div class="bg-white rounded-lg shadow-2xl p-8">
                    <h1 class="text-3xl font-bold text-slate-900 mb-2">
                        "RAR to ZIP"
                    </h1>
                    <p class="text-slate-600 mb-6">
                        "Convert your RAR archives to ZIP format instantly"
                    </p>

                    <div class="space-y-4">
                        {/* File Upload - Drag & Drop Area */}
                        <div>
                            <label class="block text-sm font-medium text-slate-700 mb-3">
                                "Select or Drop RAR File"
                            </label>
                            <div
                                id="drop-zone"
                                on:click=move |_| {
                                    if let Some(input) = web_sys::window()
                                        .and_then(|w| w.document())
                                        .and_then(|d| d.get_element_by_id("rar-input"))
                                        .and_then(|el| el.dyn_into::<web_sys::HtmlInputElement>().ok())
                                    {
                                        input.click();
                                    }
                                }
                                on:dragover=handle_drag_over
                                on:dragleave=handle_drag_leave
                                on:drop=handle_drop
                                class="w-full px-6 py-12 border-2 border-dashed border-slate-300 rounded-lg bg-slate-50 hover:bg-slate-100 hover:border-blue-400 transition cursor-pointer flex flex-col items-center justify-center gap-3"
                            >
                                <div class="text-4xl font-bold text-slate-400">"↓"</div>
                                <div class="text-center">
                                    <p class="text-slate-700 font-medium">"Drag RAR file here"</p>
                                    <p class="text-slate-500 text-sm">"or click to browse"</p>
                                </div>
                            </div>
                            <input
                                id="rar-input"
                                type="file"
                                accept=".rar"
                                on:change=handle_file_input
                                class="hidden"
                            />
                            <Show when=move || !file_name.get().is_empty()>
                                <p class="mt-3 text-sm text-green-600 font-medium">
                                    "✓ " {move || file_name.get()}
                                </p>
                            </Show>
                        </div>

                        {/* Error Message */}
                        <Show when=move || !error_message.get().is_empty()>
                            <div class="p-4 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm">
                                {move || error_message.get()}
                            </div>
                        </Show>

                        {/* Success Message */}
                        <Show when=move || !success_message.get().is_empty()>
                            <div class="p-4 bg-green-50 border border-green-200 rounded-lg text-green-700 text-sm">
                                {move || success_message.get()}
                            </div>
                        </Show>

                        {/* Converting Status */}
                        <Show when=move || is_converting.get()>
                            <div class="p-4 bg-blue-50 border border-blue-200 rounded-lg text-blue-700 text-sm flex items-center justify-center gap-2">
                                <span class="inline-block animate-spin">
                                    "⚙️"
                                </span>
                                "Converting..."
                            </div>
                        </Show>

                        {/* Download Button */}
                        <Show when=move || zip_data.get().is_some()>
                            <button
                                on:click=handle_download
                                class="w-full bg-green-600 hover:bg-green-700 text-white font-semibold py-3 px-4 rounded-lg transition duration-200 flex items-center justify-center gap-2"
                            >
                                "⬇️ Download ZIP"
                            </button>
                        </Show>
                    </div>

                    <p class="mt-6 text-center text-xs text-slate-500">
                        "Files are processed entirely in your browser. Nothing is uploaded to a server."
                    </p>
                </div>
            </div>
        </div>
    }
}

/// Read a File as bytes
async fn read_file_as_bytes(file: &File) -> Result<Vec<u8>, String> {
    let array_buffer = JsFuture::from(file.array_buffer())
        .await
        .map_err(|_| "Failed to read file".to_string())?;

    let array = Uint8Array::new(&array_buffer);
    Ok(array.to_vec())
}

/// Download a file
fn download_file(data: &[u8], filename: &str) {
    use web_sys::window;

    if let Some(window) = window() {
        let array = js_sys::Array::new();
        array.push(&Uint8Array::from(data));

        if let Ok(blob) = Blob::new_with_u8_array_sequence(&array) {
            let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap_or_default();

            let document = window.document().unwrap();
            let link = document.create_element("a").unwrap();
            link.set_attribute("href", &url).ok();
            link.set_attribute("download", filename).ok();

            // Append to body, click, then remove
            let body = document.body().unwrap();
            body.append_child(&link).ok();

            if let Some(html_element) = link.dyn_ref::<web_sys::HtmlElement>() {
                html_element.click();
            }

            body.remove_child(&link).ok();
            let _ = web_sys::Url::revoke_object_url(&url);
        }
    }
}
