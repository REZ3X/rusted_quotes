use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    console,
    window,
    HtmlElement,
    HtmlInputElement,
    HtmlTextAreaElement,
    Request,
    RequestInit,
    RequestMode,
    Headers,
    Response,
    Event,
};
use serde::{ Deserialize, Serialize };

macro_rules! log {
    ($($t:tt)*) => {
        console::log_1(&format!( $( $t )* ).into());
    };
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Quote {
    pub id: String,
    pub quote: String,
    pub author: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateQuote {
    pub quote: String,
    pub author: Option<String>,
}

#[wasm_bindgen]
pub struct QuoteManager {
    api_base: String,
}

#[wasm_bindgen]
impl QuoteManager {
    #[wasm_bindgen(constructor)]
    pub fn new() -> QuoteManager {
        QuoteManager {
            api_base: "http://localhost:3000/api".to_string(),
        }
    }

    #[wasm_bindgen]
    pub async fn fetch_quotes(&self) -> Result<JsValue, JsValue> {
        let url = format!("{}/quotes", self.api_base);

        let opts = RequestInit::new();
        opts.set_method("GET");
        opts.set_mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(&url, &opts)?;

        let window = window().unwrap();
        let resp_value = window.fetch_with_request(&request);
        let resp: Response = JsFuture::from(resp_value).await?.dyn_into()?;

        let json = JsFuture::from(resp.json()?).await?;
        Ok(json)
    }

    #[wasm_bindgen]
    pub async fn create_quote(
        &self,
        quote: &str,
        author: Option<String>
    ) -> Result<JsValue, JsValue> {
        let url = format!("{}/quotes", self.api_base);

        let create_quote = CreateQuote {
            quote: quote.to_string(),
            author,
        };

        let body_json = serde_json
            ::to_string(&create_quote)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let opts = RequestInit::new();
        opts.set_method("POST");
        opts.set_mode(RequestMode::Cors);

        let body_js = JsValue::from_str(&body_json);
        opts.set_body(&body_js);

        let headers = Headers::new()?;
        headers.set("Content-Type", "application/json")?;
        opts.set_headers(&headers);

        let request = Request::new_with_str_and_init(&url, &opts)?;

        let window = window().unwrap();
        let resp_value = window.fetch_with_request(&request);
        let resp: Response = JsFuture::from(resp_value).await?.dyn_into()?;

        let json = JsFuture::from(resp.json()?).await?;
        Ok(json)
    }

    #[wasm_bindgen]
    pub async fn get_random_quote(&self) -> Result<JsValue, JsValue> {
        let url = format!("{}/quotes/random", self.api_base);

        let opts = RequestInit::new();
        opts.set_method("GET");
        opts.set_mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(&url, &opts)?;

        let window = window().unwrap();
        let resp_value = window.fetch_with_request(&request);
        let resp: Response = JsFuture::from(resp_value).await?.dyn_into()?;

        let json = JsFuture::from(resp.json()?).await?;
        Ok(json)
    }

    #[wasm_bindgen]
pub async fn get_quote_by_id(&self, id: &str) -> Result<JsValue, JsValue> {
    let url = format!("{}/quotes/{}", self.api_base, id);

    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(&url, &opts)?;

    let window = window().unwrap();
    let resp_value = window.fetch_with_request(&request);
    let resp: Response = JsFuture::from(resp_value).await?.dyn_into()?;

    if resp.ok() {
        let json = JsFuture::from(resp.json()?).await?;
        Ok(json)
    } else {
        Err(JsValue::from_str(&format!("HTTP {}: Quote not found", resp.status())))
    }
}
}

#[wasm_bindgen(start)]
pub fn main() {
    log!("Rusted Quotes WASM module loaded!");
    init_app();
}

#[wasm_bindgen]
pub fn init_app() {
    let win = window().unwrap();
    let document = win.document().unwrap();

    if let Some(count_element) = document.get_element_by_id("quote-count") {
        count_element.set_text_content(Some("Loading..."));
    }

    setup_modal();
    setup_form_handlers();
    load_quotes();
}

fn setup_modal() {
    let win = window().unwrap();
    let document = win.document().unwrap();

    if let Some(modal_close) = document.get_element_by_id("modal-close") {
        let modal_close: HtmlElement = modal_close.dyn_into().unwrap();
        let closure = Closure::wrap(
            Box::new(move || {
                close_modal();
            }) as Box<dyn Fn()>
        );

        modal_close.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    if let Some(modal_bg) = document.get_element_by_id("quote-modal") {
        let modal_bg: HtmlElement = modal_bg.dyn_into().unwrap();
        let closure = Closure::wrap(
            Box::new(move |event: Event| {
                if let Some(target) = event.target() {
                    if target == event.current_target().unwrap() {
                        close_modal();
                    }
                }
            }) as Box<dyn Fn(Event)>
        );

        modal_bg.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }
}

fn setup_form_handlers() {
    let win = window().unwrap();
    let document = win.document().unwrap();

    if let Some(add_btn) = document.get_element_by_id("add-quote-btn") {
        let add_btn: HtmlElement = add_btn.dyn_into().unwrap();
        let closure = Closure::wrap(
            Box::new(move || {
                show_add_quote_form();
            }) as Box<dyn Fn()>
        );

        add_btn.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    if let Some(random_btn) = document.get_element_by_id("random-quote-btn") {
        let random_btn: HtmlElement = random_btn.dyn_into().unwrap();
        let closure = Closure::wrap(
            Box::new(move || {
                get_random_quote();
            }) as Box<dyn Fn()>
        );

        random_btn.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }
}

fn show_add_quote_form() {
    let win = window().unwrap();
    let document = win.document().unwrap();

    if let Some(modal) = document.get_element_by_id("quote-modal") {
        if let Some(modal_content) = document.get_element_by_id("modal-content") {
            modal_content.set_inner_html(
                r#"
                <div class="bg-white rounded-2xl p-8 max-w-md w-full mx-4 transform transition-all">
                    <div class="flex justify-between items-center mb-6">
                        <h3 class="text-2xl font-bold text-gray-800">Add New Quote</h3>
                        <button id="modal-close" class="text-gray-400 hover:text-gray-600 transition-colors">
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                            </svg>
                        </button>
                    </div>
                    <form id="quote-form" class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">Quote</label>
                            <textarea id="quote-input" rows="4" class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:ring-2 focus:ring-purple-500 focus:border-transparent resize-none" placeholder="Enter your inspiring quote..." required></textarea>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">Author (Optional)</label>
                            <input type="text" id="author-input" class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:ring-2 focus:ring-purple-500 focus:border-transparent" placeholder="Author name">
                        </div>
                        <div class="flex gap-3 pt-4">
                            <button type="submit" class="flex-1 bg-gradient-to-r from-purple-600 to-pink-600 text-white font-semibold py-3 px-6 rounded-xl hover:from-purple-700 hover:to-pink-700 transition-all transform hover:scale-105 shadow-lg">
                                Add Quote
                            </button>
                            <button type="button" id="cancel-btn" class="flex-1 bg-gray-200 text-gray-700 font-semibold py-3 px-6 rounded-xl hover:bg-gray-300 transition-colors">
                                Cancel
                            </button>
                        </div>
                    </form>
                </div>
            "#
            );

            let modal: HtmlElement = modal.dyn_into().unwrap();
            modal.class_list().remove_1("hidden").unwrap();

            setup_add_quote_form();
        }
    }
}

fn setup_add_quote_form() {
    let win = window().unwrap();
    let document = win.document().unwrap();

    if let Some(form) = document.get_element_by_id("quote-form") {
        let form: HtmlElement = form.dyn_into().unwrap();
        let closure = Closure::wrap(
            Box::new(move |event: Event| {
                event.prevent_default();
                submit_quote_form();
            }) as Box<dyn Fn(Event)>
        );

        form.set_onsubmit(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    if let Some(cancel_btn) = document.get_element_by_id("cancel-btn") {
        let cancel_btn: HtmlElement = cancel_btn.dyn_into().unwrap();
        let closure = Closure::wrap(
            Box::new(move || {
                close_modal();
            }) as Box<dyn Fn()>
        );

        cancel_btn.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    if let Some(modal_close) = document.get_element_by_id("modal-close") {
        let modal_close: HtmlElement = modal_close.dyn_into().unwrap();
        let closure = Closure::wrap(
            Box::new(move || {
                close_modal();
            }) as Box<dyn Fn()>
        );

        modal_close.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }
}

fn submit_quote_form() {
    let win = window().unwrap();
    let document = win.document().unwrap();

    let quote_input = document.get_element_by_id("quote-input").unwrap();
    let author_input = document.get_element_by_id("author-input").unwrap();

    let quote = quote_input.dyn_ref::<HtmlTextAreaElement>().unwrap().value();
    let author = author_input.dyn_ref::<HtmlInputElement>().unwrap().value();

    if quote.trim().is_empty() {
        return;
    }

    let author_opt = if author.trim().is_empty() { None } else { Some(author) };

    wasm_bindgen_futures::spawn_local(async move {
        let manager = QuoteManager::new();
        match manager.create_quote(&quote, author_opt).await {
            Ok(_) => {
                close_modal();
                load_quotes();
            }
            Err(e) => {
                log!("Error creating quote: {:?}", e);
            }
        }
    });
}

fn close_modal() {
    let win = window().unwrap();
    let document = win.document().unwrap();

    if let Some(modal) = document.get_element_by_id("quote-modal") {
        let modal: HtmlElement = modal.dyn_into().unwrap();
        modal.class_list().add_1("hidden").unwrap();
    }
}

fn load_quotes() {
    wasm_bindgen_futures::spawn_local(async move {
        let manager = QuoteManager::new();
        match manager.fetch_quotes().await {
            Ok(quotes_js) => {
                let quotes: Vec<Quote> = serde_wasm_bindgen
                    ::from_value(quotes_js)
                    .unwrap_or_default();
                let quote_count = quotes.len();
                display_quotes(quotes);

                let win = window().unwrap();
                let document = win.document().unwrap();
                if let Some(count_element) = document.get_element_by_id("quote-count") {
                    count_element.set_text_content(Some(&quote_count.to_string()));
                }
            }
            Err(e) => {
                log!("Error fetching quotes: {:?}", e);
                let win = window().unwrap();
                let document = win.document().unwrap();
                if let Some(count_element) = document.get_element_by_id("quote-count") {
                    count_element.set_text_content(Some("0"));
                }
            }
        }
    });
}

fn get_random_quote() {
    wasm_bindgen_futures::spawn_local(async move {
        let manager = QuoteManager::new();
        match manager.get_random_quote().await {
            Ok(quote_js) => {
                let quote: Quote = serde_wasm_bindgen::from_value(quote_js).unwrap();
                show_quote_modal(&quote);
            }
            Err(e) => {
                log!("Error fetching random quote: {:?}", e);
                show_error_modal("No quotes available. Add some quotes first!");
            }
        }
    });
}

fn show_error_modal(message: &str) {
    let win = window().unwrap();
    let document = win.document().unwrap();

    if let Some(modal) = document.get_element_by_id("quote-modal") {
        if let Some(modal_content) = document.get_element_by_id("modal-content") {
            modal_content.set_inner_html(
                &format!(r#"
                <div class="bg-white rounded-2xl p-8 max-w-md w-full mx-4 transform transition-all">
                    <div class="flex justify-between items-center mb-6">
                        <h3 class="text-2xl font-bold text-gray-800">Oops!</h3>
                        <button id="modal-close" class="text-gray-400 hover:text-gray-600 transition-colors">
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                            </svg>
                        </button>
                    </div>
                    <div class="text-center">
                        <p class="text-gray-600 mb-4">{}</p>
                        <button id="close-error" class="bg-purple-600 text-white font-semibold py-2 px-4 rounded-lg hover:bg-purple-700 transition-colors">
                            OK
                        </button>
                    </div>
                </div>
                "#, message)
            );

            let modal: HtmlElement = modal.dyn_into().unwrap();
            modal.class_list().remove_1("hidden").unwrap();

            if let Some(modal_close) = document.get_element_by_id("modal-close") {
                let modal_close: HtmlElement = modal_close.dyn_into().unwrap();
                let closure = Closure::wrap(
                    Box::new(move || {
                        close_modal();
                    }) as Box<dyn Fn()>
                );

                modal_close.set_onclick(Some(closure.as_ref().unchecked_ref()));
                closure.forget();
            }

            if let Some(close_error) = document.get_element_by_id("close-error") {
                let close_error: HtmlElement = close_error.dyn_into().unwrap();
                let closure = Closure::wrap(
                    Box::new(move || {
                        close_modal();
                    }) as Box<dyn Fn()>
                );

                close_error.set_onclick(Some(closure.as_ref().unchecked_ref()));
                closure.forget();
            }
        }
    }
}

fn display_quotes(quotes: Vec<Quote>) {
    let win = window().unwrap();
    let document = win.document().unwrap();

    if let Some(quotes_container) = document.get_element_by_id("quotes-container") {
        let mut html = String::new();

        for quote in quotes {
            let author = quote.author.unwrap_or_else(|| "Unknown".to_string());
            let preview = if quote.quote.len() > 120 {
                format!("{}...", &quote.quote[..120])
            } else {
                quote.quote.clone()
            };

            html.push_str(
                &format!(
                    r#"
                <div class="bg-white rounded-2xl p-6 shadow-lg hover:shadow-xl transition-all duration-300 transform hover:scale-105 cursor-pointer quote-card" data-id="{}">
                    <blockquote class="text-gray-800 font-medium leading-relaxed mb-4">
                        "{}"
                    </blockquote>
                    <div class="flex justify-between items-center">
                        <cite class="text-purple-600 font-semibold">— {}</cite>
                        <div class="text-xs text-gray-400">{}</div>
                    </div>
                </div>
                "#,
                    quote.id,
                    preview,
                    author,
                    quote.created_at.split('T').next().unwrap_or("")
                )
            );
        }

        if html.is_empty() {
            html =
                r#"
                <div class="col-span-full text-center py-12">
                    <div class="text-white text-opacity-70 text-lg">No quotes found. Be the first to add one!</div>
                </div>
            "#.to_string();
        }

        quotes_container.set_inner_html(&html);

        setup_quote_card_handlers();
    }
}

fn setup_quote_card_handlers() {
    let win = window().unwrap();
    let document = win.document().unwrap();

    if let Ok(quote_cards) = document.query_selector_all(".quote-card") {
        for i in 0..quote_cards.length() {
            if let Some(card) = quote_cards.get(i) {
                let card: HtmlElement = card.dyn_into().unwrap();
                if let Some(quote_id) = card.get_attribute("data-id") {
                    let quote_id_clone = quote_id.clone();
                    let closure = Closure::wrap(
                        Box::new(move || {
                            show_quote_detail(&quote_id_clone);
                        }) as Box<dyn Fn()>
                    );

                    card.set_onclick(Some(closure.as_ref().unchecked_ref()));
                    closure.forget();
                }
            }
        }
    }
}

fn show_quote_detail(quote_id: &str) {
    let quote_id_owned = quote_id.to_string();
    wasm_bindgen_futures::spawn_local(async move {
        let manager = QuoteManager::new();
        match manager.get_quote_by_id(&quote_id_owned).await {
            Ok(quote_js) => {
                let quote: Quote = serde_wasm_bindgen::from_value(quote_js).unwrap();
                show_quote_modal(&quote);
            }
            Err(e) => {
                log!("Error fetching quote: {:?}", e);
                show_error_modal("Could not load quote details.");
            }
        }
    });
}

fn show_detail_modal(quote_id: &str) {
    let win = window().unwrap();
    let document = win.document().unwrap();

    if let Some(modal) = document.get_element_by_id("quote-modal") {
        if let Some(modal_content) = document.get_element_by_id("modal-content") {
            modal_content.set_inner_html(
                &format!(r#"
                <div class="bg-white rounded-2xl p-8 max-w-2xl w-full mx-4 transform transition-all">
                    <div class="flex justify-between items-center mb-6">
                        <h3 class="text-2xl font-bold text-gray-800">Quote Details</h3>
                        <button id="modal-close" class="text-gray-400 hover:text-gray-600 transition-colors">
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                            </svg>
                        </button>
                    </div>
                    <div class="space-y-4">
                        <div class="text-center">
                            <div class="text-lg text-gray-600 mb-4">Loading quote details...</div>
                            <div class="text-sm text-gray-400">Quote ID: {}</div>
                        </div>
                    </div>
                </div>
                "#, quote_id)
            );

            let modal: HtmlElement = modal.dyn_into().unwrap();
            modal.class_list().remove_1("hidden").unwrap();

            if let Some(modal_close) = document.get_element_by_id("modal-close") {
                let modal_close: HtmlElement = modal_close.dyn_into().unwrap();
                let closure = Closure::wrap(
                    Box::new(move || {
                        close_modal();
                    }) as Box<dyn Fn()>
                );

                modal_close.set_onclick(Some(closure.as_ref().unchecked_ref()));
                closure.forget();
            }
        }
    }
}

fn show_quote_modal(quote: &Quote) {
    let win = window().unwrap();
    let document = win.document().unwrap();

    if let Some(modal) = document.get_element_by_id("quote-modal") {
        if let Some(modal_content) = document.get_element_by_id("modal-content") {
            let author = quote.author.clone().unwrap_or_else(|| "Unknown".to_string());

            modal_content.set_inner_html(
                &format!(
                    r#"
                <div class="bg-white rounded-2xl p-8 max-w-2xl w-full mx-4 transform transition-all">
                    <div class="flex justify-between items-center mb-6">
                        <h3 class="text-2xl font-bold text-gray-800">Random Quote</h3>
                        <button id="modal-close" class="text-gray-400 hover:text-gray-600 transition-colors">
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                            </svg>
                        </button>
                    </div>
                    <div class="text-center space-y-6">
                        <blockquote class="text-2xl font-medium text-gray-800 leading-relaxed">
                            "{}"
                        </blockquote>
                        <cite class="text-xl text-purple-600 font-semibold">— {}</cite>
                        <div class="text-sm text-gray-400">Added on {}</div>
                        <button id="another-random" class="bg-gradient-to-r from-purple-600 to-pink-600 text-white font-semibold py-3 px-6 rounded-xl hover:from-purple-700 hover:to-pink-700 transition-all transform hover:scale-105 shadow-lg">
                            Get Another Quote
                        </button>
                    </div>
                </div>
                "#,
                    quote.quote,
                    author,
                    quote.created_at.split('T').next().unwrap_or("")
                )
            );

            let modal: HtmlElement = modal.dyn_into().unwrap();
            modal.class_list().remove_1("hidden").unwrap();

            if let Some(modal_close) = document.get_element_by_id("modal-close") {
                let modal_close: HtmlElement = modal_close.dyn_into().unwrap();
                let closure = Closure::wrap(
                    Box::new(move || {
                        close_modal();
                    }) as Box<dyn Fn()>
                );

                modal_close.set_onclick(Some(closure.as_ref().unchecked_ref()));
                closure.forget();
            }

            if let Some(another_btn) = document.get_element_by_id("another-random") {
                let another_btn: HtmlElement = another_btn.dyn_into().unwrap();
                let closure = Closure::wrap(
                    Box::new(move || {
                        get_random_quote();
                    }) as Box<dyn Fn()>
                );

                another_btn.set_onclick(Some(closure.as_ref().unchecked_ref()));
                closure.forget();
            }
        }
    }
}
