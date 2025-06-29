use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use regex::Regex;
use base64::{Engine as _, engine::general_purpose};
use std::fs;

// --- Structs for the Request Body ---
// These are designed to create the exact JSON structure needed for this method.
#[derive(Serialize)]
struct DocumentRequest {
    #[serde(rename = "type")]
    doc_type: &'static str,
    image_url: String, // We use String because we need to build the data URI
}

#[derive(Serialize)]
struct OcrRequest {
    model: &'static str,
    document: DocumentRequest,
    include_image_base64: bool,
}

// --- Structs for the Response Body ---
#[derive(Deserialize, Debug)]
struct Page {
    markdown: String,
}

#[derive(Deserialize, Debug)]
struct OcrResponse {
    pages: Vec<Page>,
}
#[derive(Debug, PartialEq)]
pub enum OcrResult {
    Deviation,
    OccBandwidth,
    Unwanted,
    Date, // Represents an image with no other keywords
}


pub async fn process_image(image_path: &str) -> Result<(OcrResult, String)> {
    dotenvy::dotenv().ok();
    let mistral_key =
        env::var("MISTRAL_API_KEY").expect("MISTRAL_API_KEY environment variable not set");

    // let image_path = "797.png";

    println!("Reading and encoding local file: '{}'...", image_path);
    let image_bytes = fs::read(image_path)?;
    let base64_data = general_purpose::STANDARD.encode(&image_bytes);
    let data_uri = format!("data:image/png;base64,{}", base64_data);

    // This builds the JSON payload, just like the Python dictionary
    let request_body = OcrRequest {
        model: "mistral-ocr-latest",
        document: DocumentRequest {
            doc_type: "image_url",
            image_url: data_uri,
        },
        include_image_base64: false,
    };

    let client = Client::new();

    println!("Sending request to Mistral API...");

    // This sends the request and gets the response
    let response = client
        .post("https://api.mistral.ai/v1/ocr")
        .bearer_auth(&mistral_key)
        .json(&request_body)
        .send()
        .await?
        .error_for_status()?;

    let ocr_response = response.json::<OcrResponse>().await?;
    println!("test {:?}", ocr_response);
    // let mut results: Vec<OcrResult> = Vec::new();
    println!("\n--- OCR Results ---");
     if let Some(first_page) = ocr_response.pages.first() {
        let text = first_page.markdown.clone(); // Clone text to return it later

        // This logic correctly classifies the image type
        let result_type = if text.contains("Upper Limit") {
            OcrResult::Deviation
        } else if text.contains("OBW:") {
            OcrResult::OccBandwidth
        } else {
            let check_freq = |pattern: &str, expected_val: f32| -> Result<bool> {
                let re = Regex::new(pattern)?;
                if let Some(caps) = re.captures(&text) {
                    if let Some(val_str) = caps.get(1) {
                        if let Ok(val_f32) = val_str.as_str().parse::<f32>() {
                            if (val_f32 - expected_val).abs() < 0.01 { return Ok(true); }
                        }
                    }
                }
                Ok(false)
            };
            if check_freq(r"Center:?\s*(\d+\.?\d*)\s*MHz", 112.0)?
                || check_freq(r"Stop:?\s*(\d+\.?\d*)\s*MHz", 137.0)?
                || check_freq(r"Start:?\s*(\d+\.?\d*)\s*MHz", 87.0)?
            {
                OcrResult::Unwanted
            } else {
                OcrResult::Date
            }
        };

        // This correctly returns the tuple that main.rs expects
        Ok((result_type, text))
    } else {
        Ok((OcrResult::Unwanted, String::new()))
    }
  
} 



