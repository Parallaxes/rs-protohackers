use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct Request {
    pub method: String,
    pub number: serde_json::Number,
}

#[derive(Serialize, Debug)]
pub struct Response {
    pub method: String,
    pub prime: bool,
}

#[derive(Serialize, Debug)]
pub struct MalformedResponse {
    pub method: String,
    pub error: String,
}

impl Request {
    pub fn is_valid(&self) -> bool {
        self.method == "isPrime"
    }

    pub fn get_number(&self) -> Option<f64> {
        self.number.as_f64()
    }
}

impl Response {
    pub fn new(is_prime: bool) -> Self {
        Self {
            method: "isPrime".to_string(),
            prime: is_prime,
        }
    }
}

impl MalformedResponse {
    pub fn new() -> Self {
        Self {
            method: "isPrime".to_string(),
            error: "malformed request".to_string()
        }
    }
}

pub fn parse_request(line: &str) -> Result<Request, serde_json::Error> {
    serde_json::from_str(line.trim())
}

pub fn serialize_response<T: Serialize>(response: &T) -> Result<String, serde_json::Error> {
    let mut json = serde_json::to_string(response)?;
    json.push('\n');
    Ok(json)
}