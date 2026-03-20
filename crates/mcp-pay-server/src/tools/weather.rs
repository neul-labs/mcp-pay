//! Weather tool implementations.
//!
//! This module provides toy weather API tools for demonstrating
//! payment gating with mcp-pay.

use serde::{Deserialize, Serialize};

/// Weather data response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub city: String,
    pub temperature_c: f64,
    pub condition: String,
    pub humidity: u8,
    pub wind_speed_kmh: f64,
}

/// Weather forecast entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastEntry {
    pub date: String,
    pub high_c: f64,
    pub low_c: f64,
    pub condition: String,
    pub precipitation_chance: u8,
}

/// Get current weather for a city (FREE).
///
/// This is a mock implementation that returns simulated weather data.
pub async fn get_current(city: &str) -> String {
    // Simulated weather data based on city name hash
    let hash = city.bytes().map(|b| b as u32).sum::<u32>();
    let temp = 15.0 + (hash % 20) as f64;
    let humidity = 40 + (hash % 40) as u8;
    let wind = 5.0 + (hash % 30) as f64;

    let conditions = ["Sunny", "Cloudy", "Partly Cloudy", "Rainy", "Clear"];
    let condition = conditions[(hash as usize) % conditions.len()];

    let weather = WeatherData {
        city: city.to_string(),
        temperature_c: temp,
        condition: condition.to_string(),
        humidity,
        wind_speed_kmh: wind,
    };

    serde_json::to_string_pretty(&weather).unwrap_or_else(|_| "Error generating weather".into())
}

/// Get 7-day weather forecast for a city (PAID - $0.003 per call).
///
/// This is a mock implementation that returns simulated forecast data.
pub async fn get_forecast(city: &str) -> String {
    let hash = city.bytes().map(|b| b as u32).sum::<u32>();

    let conditions = ["Sunny", "Cloudy", "Partly Cloudy", "Rainy", "Clear", "Thunderstorm"];

    let forecast: Vec<ForecastEntry> = (0..7)
        .map(|day| {
            let day_hash = hash + day as u32;
            let high = 18.0 + (day_hash % 15) as f64;
            let low = high - 5.0 - (day_hash % 8) as f64;
            let precip = (day_hash % 100) as u8;

            ForecastEntry {
                date: format!("2025-09-{:02}", 15 + day),
                high_c: high,
                low_c: low,
                condition: conditions[(day_hash as usize) % conditions.len()].to_string(),
                precipitation_chance: precip,
            }
        })
        .collect();

    serde_json::to_string_pretty(&forecast).unwrap_or_else(|_| "Error generating forecast".into())
}

/// Get historical weather data (PAID - tiered pricing).
///
/// This demonstrates tiered pricing where bulk queries are cheaper.
pub async fn get_historical(city: &str, days: u32) -> String {
    let hash = city.bytes().map(|b| b as u32).sum::<u32>();

    let history: Vec<WeatherData> = (0..days)
        .map(|day| {
            let day_hash = hash.wrapping_add(day);
            let conditions = ["Sunny", "Cloudy", "Rainy", "Clear"];

            WeatherData {
                city: city.to_string(),
                temperature_c: 12.0 + (day_hash % 18) as f64,
                condition: conditions[(day_hash as usize) % conditions.len()].to_string(),
                humidity: 30 + (day_hash % 50) as u8,
                wind_speed_kmh: 3.0 + (day_hash % 25) as f64,
            }
        })
        .collect();

    serde_json::to_string_pretty(&history).unwrap_or_else(|_| "Error generating history".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_current() {
        let result = get_current("London").await;
        assert!(result.contains("London"));
        assert!(result.contains("temperature_c"));
    }

    #[tokio::test]
    async fn test_get_forecast() {
        let result = get_forecast("Tokyo").await;
        // Should return 7 days of forecast
        let forecast: Vec<ForecastEntry> = serde_json::from_str(&result).unwrap();
        assert_eq!(forecast.len(), 7);
    }

    #[tokio::test]
    async fn test_get_historical() {
        let result = get_historical("Paris", 5).await;
        let history: Vec<WeatherData> = serde_json::from_str(&result).unwrap();
        assert_eq!(history.len(), 5);
    }
}
