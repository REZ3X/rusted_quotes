use wasm_bindgen::prelude::*;
use web_sys::{console, window, HtmlElement};

macro_rules! log {
    ( $( $t:tt )* ) => {
        console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
pub struct QuoteGenerator {
    quotes: Vec<&'static str>,
}

#[wasm_bindgen]
impl QuoteGenerator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> QuoteGenerator {
        QuoteGenerator {
            quotes: vec![
                "The only way to do great work is to love what you do. - Steve Jobs",
                "Innovation distinguishes between a leader and a follower. - Steve Jobs",
                "Life is what happens to you while you're busy making other plans. - John Lennon",
                "The future belongs to those who believe in the beauty of their dreams. - Eleanor Roosevelt",
                "It is during our darkest moments that we must focus to see the light. - Aristotle",
                "Success is not final, failure is not fatal: it is the courage to continue that counts. - Winston Churchill",
            ],
        }
    }

    #[wasm_bindgen]
    pub fn get_random_quote(&self) -> String {
        use js_sys::Math;
        let index = (Math::random() * self.quotes.len() as f64).floor() as usize;
        self.quotes[index].to_string()
    }

    #[wasm_bindgen]
    pub fn get_quote_count(&self) -> usize {
        self.quotes.len()
    }

    #[wasm_bindgen]
    pub fn get_all_quotes(&self) -> Vec<JsValue> {
        self.quotes.iter().map(|quote| JsValue::from_str(quote)).collect()
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
    
    let generator = QuoteGenerator::new();

    if let Some(count_element) = document.get_element_by_id("quote-count") {
        count_element.set_text_content(Some(&generator.get_quote_count().to_string()));
    }

    if let Some(button) = document.get_element_by_id("new-quote-btn") {
        let button: HtmlElement = button.dyn_into().unwrap();
        let closure = Closure::wrap(Box::new(move || {
            let generator = QuoteGenerator::new();
            let quote = generator.get_random_quote();

            let win = window().unwrap();
            let document = win.document().unwrap();
            
            if let Some(display) = document.get_element_by_id("quote-display") {
                display.set_inner_html(&format!(
                    r#"<blockquote class="text-2xl md:text-3xl font-medium text-gray-800 mb-6 italic">"{}"</blockquote>"#,
                    quote
                ));
            }
        }) as Box<dyn Fn()>);
        
        button.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    let quote = generator.get_random_quote();
    if let Some(display) = document.get_element_by_id("quote-display") {
        display.set_inner_html(&format!(
            r#"<blockquote class="text-2xl md:text-3xl font-medium text-gray-800 mb-6 italic">"{}"</blockquote>"#,
            quote
        ));
    }
}