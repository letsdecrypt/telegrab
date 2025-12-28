use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Refine 排序规则（单字段排序）
#[derive(Debug, Deserialize)]
pub struct RefineSort {
    pub field: String,          // 排序字段（如 "created_at"）
    pub order: RefineSortOrder, // 排序方向
}

// Refine 排序方向（asc/desc）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RefineSortOrder {
    Asc,
    Desc,
}

// Refine 过滤规则（单条件过滤）
#[derive(Debug, Deserialize)]
pub struct RefineFilter {
    pub field: String,            // 过滤字段（如 "status"）
    pub operator: RefineOperator, // 过滤操作符
    pub value: Value,             // 过滤值（动态类型）
}

// Refine 支持的过滤操作符（按需扩展）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RefineOperator {
    Eq, // 等于
    Ne, // 不等于
    Gt, // 大于
    Lt, // 小于
    Contains, // 包含（模糊查询）
        // 可扩展：Gte/Lte/In/NotIn 等
}

// 分页请求参数（解析 URL Query 后结构化）
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_start")]
    #[serde(rename = "_start")]
    pub start: u64,
    #[serde(default = "default_end")]
    #[serde(rename = "_end")]
    pub end: u64,
    #[serde(default, deserialize_with = "deserialize_json_str")]
    pub sort: Option<Vec<RefineSort>>, // 排序规则（JSON 字符串解析）
    #[serde(default, deserialize_with = "deserialize_json_str")]
    pub filters: Option<Vec<RefineFilter>>, // 过滤规则（JSON 字符串解析）
}

// 分页响应结构体（对齐 Refine 规范）
#[derive(Debug, Serialize)]
pub struct PaginationResponse<T> {
    pub data: Vec<T>, // 当前页数据
    pub total: u64,   // 总条数
}

// -------------- 辅助函数 --------------
// 默认页码：1
fn default_start() -> u64 {
    0
}

// 默认每页条数：10
fn default_end() -> u64 {
    10
}

// 解析 Refine 传递的 JSON 字符串参数（如 sort/filters）
fn deserialize_json_str<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: DeserializeOwned,
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(s) => serde_json::from_str(&s)
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

// 从 PaginationQuery 计算 offset（用于 SQL LIMIT/OFFSET）
impl PaginationQuery {
    pub fn offset(&self) -> u64 {
        self.start
    }

    pub fn limit(&self) -> u64 {
        self.end - self.start
    }
}
