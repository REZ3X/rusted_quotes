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
                <div class="glass-morphism rounded-3xl p-8 cyber-border max-w-md w-full mx-auto transform transition-all">
                    <div class="flex justify-between items-center mb-8">
                        <div>
                            <h3 class="text-3xl font-bold text-white mb-2">Add New Quote</h3>
                            <p class="text-gray-400 font-mono text-sm">{ create_wisdom() }</p>
                        </div>
                        <button id="modal-close" class="text-gray-400 hover:text-neon-cyan transition-colors p-2 rounded-lg hover:bg-dark-800">
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                            </svg>
                        </button>
                    </div>
                    
                    <form id="quote-form" class="space-y-6">
                        <div>
                            <label class="flex items-center gap-2 text-sm font-bold text-neon-cyan mb-3 font-mono">
                                <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M18 13V5a2 2 0 00-2-2H4a2 2 0 00-2 2v8a2 2 0 002 2h3l3 3 3-3h3a2 2 0 002-2zM5 7a1 1 0 011-1h8a1 1 0 110 2H6a1 1 0 01-1-1zm1 3a1 1 0 100 2h3a1 1 0 100-2H6z" clip-rule="evenodd" />
                                </svg>
                                QUOTE_TEXT
                            </label>
                            <textarea 
                                id="quote-input" 
                                rows="4" 
                                class="w-full px-4 py-4 bg-dark-900 border border-dark-700 rounded-xl focus:ring-2 focus:ring-neon-cyan focus:border-transparent resize-none text-gray-100 placeholder-gray-500 transition-all font-primary"
                                placeholder="Enter your inspiring quote..."
                                required
                            ></textarea>
                        </div>
                        
                        <div>
                            <label class="flex items-center gap-2 text-sm font-bold text-neon-purple mb-3 font-mono">
                                <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" clip-rule="evenodd" />
                                </svg>
                                AUTHOR (Optional)
                            </label>
                            <input 
                                type="text" 
                                id="author-input" 
                                class="w-full px-4 py-4 bg-dark-900 border border-dark-700 rounded-xl focus:ring-2 focus:ring-neon-purple focus:border-transparent text-gray-100 placeholder-gray-500 transition-all font-primary"
                                placeholder="Author name"
                            >
                        </div>
                        
                        <div class="flex gap-4 pt-4">
                            <button type="submit" class="flex-1 bg-gradient-to-r from-neon-purple to-neon-cyan text-dark-950 font-bold py-4 px-6 rounded-xl hover:from-neon-cyan hover:to-neon-purple transition-all transform hover:scale-105 shadow-lg hover:shadow-neon-cyan/25">
                                <span class="flex items-center justify-center gap-2">
                                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                                    </svg>
                                    Deploy Quote
                                </span>
                            </button>
                            <button type="button" id="cancel-btn" class="flex-1 bg-dark-800 border border-dark-600 text-gray-300 font-bold py-4 px-6 rounded-xl hover:bg-dark-700 hover:border-dark-500 transition-all">
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
                <div class="glass-morphism rounded-3xl p-8 cyber-border max-w-md w-full mx-auto transform transition-all">
                    <div class="flex justify-between items-center mb-6">
                        <div>
                            <h3 class="text-2xl font-bold text-red-400 mb-2">System Error</h3>
                            <p class="text-gray-400 font-mono text-sm">{{ error_handler() }}</p>
                        </div>
                        <button id="modal-close" class="text-gray-400 hover:text-red-400 transition-colors p-2 rounded-lg hover:bg-dark-800">
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                            </svg>
                        </button>
                    </div>
                    
                    <div class="text-center space-y-6">
                        <div class="text-6xl">⚠️</div>
                        <div class="space-y-3">
                            <p class="text-gray-300 text-lg">{}</p>
                            <div class="bg-dark-900 border border-red-500/20 rounded-lg p-4">
                                <p class="text-red-400 font-mono text-sm">
                                    ERROR_CODE: CONNECTION_FAILED
                                </p>
                            </div>
                        </div>
                        <button id="close-error" class="bg-red-500 hover:bg-red-600 text-white font-bold py-3 px-8 rounded-xl transition-all transform hover:scale-105 shadow-lg">
                            <span class="flex items-center gap-2">
                                <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                                </svg>
                                Acknowledge
                            </span>
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
            let author = quote.author.unwrap_or_else(|| "Anonymous".to_string());
            let preview = if quote.quote.len() > 150 {
                format!("{}...", &quote.quote[..150])
            } else {
                quote.quote.clone()
            };

            html.push_str(
                &format!(
                    r#"
                <div class="glass-morphism rounded-2xl p-6 cyber-border card-hover cursor-pointer quote-card group" data-id="{}">
                    <div class="space-y-4">
                        <div class="flex items-start justify-between">
                            <div class="flex-1">
                                <blockquote class="text-gray-100 font-medium leading-relaxed text-lg group-hover:text-white transition-colors">
                                    "{}"
                                </blockquote>
                            </div>
                            <div class="ml-4 opacity-50 group-hover:opacity-100 transition-opacity">
                                <svg class="w-5 h-5 text-neon-cyan" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-6-3a2 2 0 11-4 0 2 2 0 014 0zm-2 4a5 5 0 00-4.546 2.916A5.986 5.986 0 0010 16a5.986 5.986 0 004.546-2.084A5 5 0 0010 11z" clip-rule="evenodd" />
                                </svg>
                            </div>
                        </div>
                        
                        <div class="flex items-center justify-between pt-4 border-t border-dark-700 group-hover:border-neon-cyan/30 transition-colors">
                            <cite class="text-neon-cyan font-bold group-hover:text-white transition-colors">
                                — {}
                            </cite>
                            <div class="text-xs text-gray-500 font-mono group-hover:text-gray-400 transition-colors">
                                {}
                            </div>
                        </div>
                        
                        <div class="opacity-0 group-hover:opacity-100 transition-all duration-300 text-center">
                            <div class="text-xs text-neon-purple font-mono">
                                Click to expand
                            </div>
                        </div>
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
                <div class="col-span-full">
                    <div class="glass-morphism rounded-2xl p-12 text-center cyber-border">
                        <div class="space-y-4">
                            <div class="text-6xl">📝</div>
                            <div class="text-white text-xl font-bold">No quotes found</div>
                            <p class="text-gray-400">Be the first to share your wisdom with the world!</p>
                            <div class="text-gray-500 font-mono text-sm">
                                { quotes.is_empty() }
                            </div>
                        </div>
                    </div>
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

fn show_quote_modal(quote: &Quote) {
    let win = window().unwrap();
    let document = win.document().unwrap();

    if let Some(modal) = document.get_element_by_id("quote-modal") {
        if let Some(modal_content) = document.get_element_by_id("modal-content") {
            let author = quote.author.clone().unwrap_or_else(|| "Anonymous".to_string());
            
            modal_content.set_inner_html(
                &format!(
                    r#"
                <div class="glass-morphism rounded-3xl p-8 cyber-border max-w-4xl w-full mx-auto transform transition-all">
                    <div class="mb-8">
                        <div>
                            <h3 class="text-3xl font-bold text-white mb-2">Quote Details</h3>
                            <p class="text-gray-400 font-mono text-sm">{{ quote.render() }}</p>
                        </div>
                    </div>
                    
                    <div class="text-center space-y-8">
                        <div class="relative">
                            <div class="absolute -top-4 -left-4 text-6xl text-neon-cyan opacity-20">"</div>
                            <blockquote class="text-2xl md:text-3xl font-medium text-white leading-relaxed px-8">
                                {}
                            </blockquote>
                            <div class="absolute -bottom-4 -right-4 text-6xl text-neon-cyan opacity-20">"</div>
                        </div>
                        
                        <div class="space-y-4">
                            <cite class="text-xl text-neon-cyan font-bold">— {}</cite>
                            <div class="inline-flex items-center gap-2 text-gray-400 font-mono text-sm">
                                <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M6 2a1 1 0 00-1 1v1H4a2 2 0 00-2 2v10a2 2 0 002 2h12a2 2 0 002-2V6a2 2 0 00-2-2h-1V3a1 1 0 10-2 0v1H7V3a1 1 0 00-1-1zm0 5a1 1 0 000 2h8a1 1 0 100-2H6z" clip-rule="evenodd" />
                                </svg>
                                Added on {}
                            </div>
                        </div>
                        
                        <div class="flex flex-col sm:flex-row gap-4 justify-center pt-4">
                            <button id="another-random" class="bg-gradient-to-r from-neon-purple to-neon-cyan text-dark-950 font-bold py-3 px-8 rounded-xl hover:from-neon-cyan hover:to-neon-purple transition-all transform hover:scale-105 shadow-lg hover:shadow-neon-cyan/25">
                                <span class="flex items-center gap-2">
                                    <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0014.001 13H11a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z" clip-rule="evenodd" />
                                    </svg>
                                    Another Quote
                                </span>
                            </button>
                            
                            <button id="close-modal-btn" class="bg-dark-800 border border-dark-600 text-gray-300 font-bold py-3 px-8 rounded-xl hover:bg-dark-700 hover:border-dark-500 transition-all">
                                <span class="flex items-center gap-2">
                                    Close
                                </span>
                            </button>
                        </div>
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

            if let Some(close_btn) = document.get_element_by_id("close-modal-btn") {
                let close_btn: HtmlElement = close_btn.dyn_into().unwrap();
                let closure = Closure::wrap(
                    Box::new(move || {
                        close_modal();
                    }) as Box<dyn Fn()>
                );
                
                close_btn.set_onclick(Some(closure.as_ref().unchecked_ref()));
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
