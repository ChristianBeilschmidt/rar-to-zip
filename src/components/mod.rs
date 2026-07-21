use crate::converter::ZipFile;
use crate::converter::convert_file;
use crate::utils::JsResultExt;
use anyhow::{Context, Result};
use js_sys::Uint8Array;
use leptos::component;
use leptos::html::*;
use leptos::prelude::*;
use std::error::Error;
use std::fmt;
use wasm_bindgen::JsCast;
use web_sys::{Blob, File};

#[derive(Clone)]
pub struct AppError(String);

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for AppError {}

#[component]
pub fn App() -> impl IntoView {
    let (file, set_file) = signal::<Option<File>>(None);

    // LocalResource handles non-Send futures (JS interop)
    let zip_resource = LocalResource::new(move || {
        let file_val = file.get();
        async move {
            let file = match file_val {
                Some(file) => file,
                None => return Err(AppError("No file selected".to_string())),
            };

            if !file.name().to_lowercase().ends_with(".rar") {
                return Err(AppError("Only .rar files are accepted".to_string()));
            }

            convert_file(&file)
                .await
                .map_err(|e| AppError(e.to_string()))
        }
    });

    let _reset_file = move || {
        set_file.set(None);
        zip_resource.refetch();
    };

    let handle_file_input = move |ev: web_sys::Event| {
        ev.prevent_default();
        ev.stop_propagation();
        let target = ev.target().unwrap();
        let input = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();
        set_file.set(input.files().and_then(|files| files.get(0)));
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

        let file = ev
            .data_transfer()
            .and_then(|data_transfer| data_transfer.files())
            .and_then(|files| files.get(0));

        set_file.set(file);
    };

    let handle_download = move |_| {
        let Some(Ok(data)) = zip_resource.get() else {
            return;
        };
        if let Err(e) = download_file(&data) {
            web_sys::console::error_1(&format!("Download failed: {}", e).into());
        }
    };

    view! {
        <div class="min-h-screen bg-gradient-to-br from-slate-900 to-slate-800 flex items-center justify-center p-4">
            <ErrorBoundary fallback=|errors| view! {
                <div class="w-full max-w-md">
                    <div class="bg-white rounded-lg shadow-2xl p-8">
                        <h1 class="text-3xl font-bold text-slate-900 mb-2">"Application Error"</h1>
                        <div class="p-4 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm">
                            {move || {
                                errors.get().into_iter().map(|(_, e)| {
                                    let error_msg = e.to_string();
                                    view! { <p>{error_msg}</p> }
                                }).collect_view()
                            }}
                        </div>
                    </div>
                </div>
            }>
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
                                <Show when=move || file.get().is_some()>
                                    <p class="mt-3 text-sm text-green-600 font-medium">
                                        "✓ " {move || file.get().map(|f| f.name()).unwrap_or_default()}
                                    </p>
                                </Show>
                            </div>

                            {/* Conversion Status with Suspense and Error Handling */}
                            <Show when=move || file.get().is_some()>
                                <Suspense
                                    fallback=move || view! {
                                        <div class="p-4 bg-blue-50 border border-blue-200 rounded-lg text-blue-700 text-sm flex items-center justify-center gap-2">
                                            <span class="inline-block animate-spin">
                                                "⚙️"
                                            </span>
                                            "Converting..."
                                        </div>
                                    }
                                >
                                    <ErrorBoundary fallback=|errors| view! {
                                        <div class="p-4 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm">
                                            {move || {
                                                errors.get().into_iter().map(|(_, e)| {
                                                    view! { <p>{e.to_string()}</p> }
                                                }).collect_view()
                                            }}
                                        </div>
                                    }>
                                        {move || {
                                            zip_resource.get().map(|result| {
                                                result.map(|_zip_file| view! {
                                                    <div class="p-4 bg-green-50 border border-green-200 rounded-lg text-green-700 text-sm">
                                                        {"✓ Conversion successful"}
                                                    </div>
                                                })
                                            })
                                        }}
                                    </ErrorBoundary>
                                </Suspense>
                            </Show>
                            <Show when=move || {
                                zip_resource.get().map(|r| r.is_ok()).unwrap_or(false)
                            }>
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
            </ErrorBoundary>
        </div>
    }
}

/// Download a file
fn download_file(file: &ZipFile) -> Result<()> {
    let window = web_sys::window().context("No window available")?;

    let array = js_sys::Array::new();
    array.push(&Uint8Array::from(file.data.as_slice()));

    let blob = Blob::new_with_u8_array_sequence(&array)
        .map_js_err()
        .context("Failed to create Blob from ZIP data")?;

    let url = web_sys::Url::create_object_url_with_blob(&blob)
        .map_js_err()
        .context("Failed to create object URL")?;

    let document = window.document().context("No document available")?;
    let link = document
        .create_element("a")
        .map_js_err()
        .context("Failed to create download link element")?;
    link.set_attribute("href", &url)
        .map_js_err()
        .context("Failed to set href attribute on download link")?;
    link.set_attribute("download", &file.name)
        .map_js_err()
        .context("Failed to set download attribute on download link")?;

    // Append to body, click, then remove
    let body = document.body().unwrap();
    body.append_child(&link)
        .map_js_err()
        .context("Failed to append download link to document")?;

    if let Some(html_element) = link.dyn_ref::<web_sys::HtmlElement>() {
        html_element.click();
    }

    body.remove_child(&link)
        .map_js_err()
        .context("Failed to remove download link from document")?;
    web_sys::Url::revoke_object_url(&url)
        .map_js_err()
        .context("Failed to revoke object URL")
}
