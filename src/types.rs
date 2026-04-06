use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum RankingMode {
    Embedding,
    Rerank,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SearchFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speaker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchRequest {
    pub query: String,
    pub max_results: u32,
    pub ranking_mode: RankingMode,
    pub include_answer: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<SearchFilters>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
    pub credits_used: u64,
    pub credits_remaining: u64,
    pub request_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f64,
    pub rerank_score: Option<f64>,
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub transcript: Option<String>,
    pub thumbnail_url: Option<String>,
    pub keyframe_url: Option<String>,
    pub duration: u64,
    pub source: String,
    pub speaker: Option<String>,
    pub published_at: Option<String>,
    pub language: Option<String>,
    pub timestamp_start: Option<f64>,
    pub timestamp_end: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UsageResponse {
    pub tier: String,
    pub plan_code: PlanCode,
    pub period_start: String,
    pub period_end: String,
    pub credits_limit: u64,
    pub credits_used: u64,
    pub credits_remaining: u64,
    pub wallet_balance: u64,
    pub credit_breakdown: CreditBreakdown,
    pub expiring_credits: Vec<ExpiringCredit>,
    pub rate_limit_per_sec: u64,
    pub api_keys_active: u64,
    pub billing_hold: bool,
    pub daily_free_remaining: u64,
    pub daily_free_limit: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PlanCode {
    Free,
    Pro,
    Enterprise,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreditBreakdown {
    pub included_remaining: u64,
    pub bonus_remaining: u64,
    pub paid_remaining: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExpiringCredit {
    pub grant_type: String,
    pub credits: u64,
    pub expires_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}
