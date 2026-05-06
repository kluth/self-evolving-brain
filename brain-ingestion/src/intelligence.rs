use anyhow::Result;
use serde::{Deserialize, Serialize};
use lingua::{Language, LanguageDetector, LanguageDetectorBuilder};

#[derive(Debug, Serialize, Deserialize)]
pub struct IntelligenceResult {
    pub normalized_title: String,
    pub normalized_content: String,
    pub location: Option<String>,
    pub detected_language: String,
    pub entities: Vec<String>,
}

pub struct IntelligenceEngine {
    detector: LanguageDetector,
}

impl IntelligenceEngine {
    pub fn new() -> Self {
        let languages = vec![
            Language::English,
            Language::German,
            Language::Russian,
            Language::Ukrainian,
            Language::Polish,
            Language::French,
            Language::Spanish,
            Language::Italian,
            Language::Japanese,
            Language::Chinese,
            Language::Hindi,
            Language::Arabic,
            Language::Portuguese,
            Language::Bengali,
            Language::Urdu,
        ];
        
        let detector = LanguageDetectorBuilder::from_languages(&languages).build();
        
        Self { detector }
    }

    pub async fn process(&self, title: &str, content: &str) -> Result<IntelligenceResult> {
        let lang = self.detector.detect_language_of(title);
        let detected_language = match lang {
            Some(Language::English) => "en".to_string(),
            Some(l) => format!("{:?}", l).to_lowercase(),
            None => "unknown".to_string(),
        };

        let (norm_title, norm_content) = if detected_language != "en" && detected_language != "unknown" {
            self.translate_to_english(title, content).await?
        } else {
            (title.to_string(), content.to_string())
        };

        let location = self.extract_location(&norm_title, &norm_content);
        let entities = self.extract_entities(&norm_title, &norm_content);

        eprintln!("[Intel] Processed: {} | Lang: {} | Loc: {:?}", norm_title, detected_language, location);

        Ok(IntelligenceResult {
            normalized_title: norm_title,
            normalized_content: norm_content,
            location,
            detected_language,
            entities,
        })
    }

    async fn translate_to_english(&self, title: &str, content: &str) -> Result<(String, String)> {
        let client = reqwest::Client::new();
        let translate_url = std::env::var("TRANSLATE_URL").unwrap_or_else(|_| "http://translate:5000/translate".to_string());
        let lang_code = self.detector.detect_language_of(title).map(|l| match l {
            Language::English => "en",
            Language::German => "de",
            Language::Russian => "ru",
            Language::Ukrainian => "uk",
            Language::Polish => "pl",
            Language::French => "fr",
            Language::Spanish => "es",
            Language::Italian => "it",
            Language::Japanese => "ja",
            Language::Chinese => "zh",
            Language::Hindi => "hi",
            Language::Arabic => "ar",
            Language::Portuguese => "pt",
            Language::Bengali => "bn",
            Language::Urdu => "ur",
            _ => "auto",
        }).unwrap_or("auto");

        let mut translated = (title.to_string(), content.to_string());

        for text in [&mut translated.0, &mut translated.1] {
            if text.is_empty() { continue; }
            
            let resp = client.post(&translate_url)
                .json(&serde_json::json!({
                    "q": text,
                    "source": lang_code,
                    "target": "en",
                    "format": "text"
                }))
                .send()
                .await;

            if let Ok(res) = resp {
                if let Ok(body) = res.json::<serde_json::Value>().await {
                    if let Some(t) = body["translatedText"].as_str() {
                        *text = t.to_string();
                        continue;
                    }
                }
            }

            // Fallback to Ollama if LibreTranslate fails
            let ollama_url = std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://ollama:11434/api/generate".to_string());
            let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3".to_string());
            let prompt = format!("Translate the following text to English. Respond only with the translation: {}", text);
            
            if let Ok(res) = client.post(&ollama_url)
                .json(&serde_json::json!({
                    "model": model,
                    "prompt": prompt,
                    "stream": false
                }))
                .send()
                .await {
                if let Ok(body) = res.json::<serde_json::Value>().await {
                    if let Some(t) = body["response"].as_str() {
                        *text = t.trim().to_string();
                    }
                }
            }
        }

        Ok(translated)
    }

    fn extract_location(&self, title: &str, content: &str) -> Option<String> {
        let combined = format!("{} {}", title, content).to_lowercase();
        
        // Expanded location map
        let locations = vec![
            "germany", "berlin", "duisburg", "munich", "hamburg",
            "usa", "washington", "new york", "california", "texas",
            "uk", "britain", "united kingdom", "london", "manchester", "birmingham",
            "france", "paris", "lyon", "marseille",
            "ukraine", "kyiv", "kiev", "odessa", "kharkiv", "dnipro",
            "russia", "moscow", "st petersburg",
            "israel", "tel aviv", "jerusalem", "gaza",
            "china", "beijing", "shanghai", "hong kong",
            "india", "delhi", "mumbai", "hyderabad", "bangalore",
            "brazil", "brasilia", "rio de janeiro", "sao paulo",
            "italy", "rome", "milan", "venice",
            "japan", "tokyo", "osaka", "kyoto",
            "australia", "sydney", "melbourne", "perth",
            "canada", "ottawa", "toronto", "vancouver"
        ];
        
        for loc in locations {
            if combined.contains(loc) {
                return Some(loc.to_uppercase());
            }
        }
        None
    }

    fn extract_entities(&self, title: &str, content: &str) -> Vec<String> {
        let mut entities = Vec::new();
        let combined = format!("{} {}", title, content);
        
        let entity_keywords = vec![
            "Apple", "Microsoft", "Google", "Meta", "Amazon", "OpenAI", "Nvidia",
            "Trump", "Biden", "Obama", "Zelensky", "Putin", "Netanyahu", "Musk",
            "Tesla", "SpaceX", "Starlink", "ChatGPT", "Claude", "Gemini"
        ];
        
        for entity in entity_keywords {
            if combined.contains(entity) {
                entities.push(entity.to_string());
            }
        }
        
        entities
    }
}
