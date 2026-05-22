//! 从 page_title 中提取候选 tag。
//!
//! 识别维度：
//! - Author   — 作者/画师，如 [湯山チカ]
//! - Circle   — 社团，如 [DOLL PLAY (黒巣ガタリ)] → DOLL PLAY
//! - Source   — 原作系列，如 (Fate/Grand Order)、(東方Project)
//! - Magazine — 杂志，如 COMIC LO、COMIC 快楽天
//! - Event    — 展会，如 C95、COMIC1☆15
//! - Language — 语言/翻译标记，如 中国翻訳、英訳
//! - Edition  — 版本标记，如 DL版、無修正

use regex::Regex;
use std::sync::LazyLock;

// ════════════════════════════════════════════════════════════
// 正则
// ════════════════════════════════════════════════════════════

/// 作者/社团标记：[xxx] 或 [xxx (yyy)]
/// 可能不在字符串最开头（前面可能有展会标记）
static RE_AUTHOR_BRACKET: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[([^\]]+)\]").unwrap());

/// 社团+作者拆分：[Circle (Author)] → Circle
static RE_CIRCLE_AUTHOR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([^(]+?)\s*\([^)]+\)$").unwrap());

/// 展会/活动标记：(C95), (COMIC1☆15), (例大祭13), ...
static RE_EVENT_PAREN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\((?:C\d{2,3}|COMIC1[^)]*|コミ(?:ティア|ック)[^)]*|紅楼夢\d*|例大祭\d*|サンクリ[^)]*|秋葉原[^)]*|歌姫庭園\d*|砲雷撃戦[^)]*|超こみっく[^)]*|メガMBFes[^)]*|#にじそうさく\d*|第\d+回[^)]*|GW超同人祭|AC\d*|FF\d+|SPARK\d+)\)",
    )
    .unwrap()
});

/// 杂志名（独立出现于标题开头）
static RE_MAGAZINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^(?:COMIC\s*(?:LO|快楽天(?:\s*ビースト)?|BAVEL|失楽天|アンスリウム|アオハ|ExE|夢幻転生|エグゼ|快楽天\s*XTC)|コミック\s*(?:ホットミルク|ゼロス|Mate\s*legend|アンリアル|エグゼ)|永遠娘|WEEKLY快楽天|コミック\s*グレープ)\b",
    )
    .unwrap()
});

/// 杂志名出现于括号内：(COMIC 快楽天 2019年3月号)
static RE_MAGAZINE_IN_PAREN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\([^)]*(?:COMIC\s*(?:LO|快楽天(?:\s*ビースト)?|BAVEL|失楽天|アンスリウム|アオハ|ExE|夢幻転生|エグゼ|快楽天\s*XTC)|コミック\s*(?:ホットミルク|ゼロス|Mate\s*legend|アンリアル|エグゼ)|永遠娘)[^)]*\)",
    )
    .unwrap()
});

/// 语言/翻译/版本标签
static RE_TAG_BRACKET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\[(?:中国翻訳|中国語|中國翻譯|中國語|中文翻譯|中文書面語版|英訳|無修正|DL版|DLsite[^\]]*|進行中|ページ欠落|見本|uncensored|Chinese|chinese|デジタル特装版|FANZA特別版|Digital)\]",
    )
    .unwrap()
});

/// 原作来源：(Fate/Grand Order), (艦隊これくしょん -艦これ-), (東方Project), ...
/// 排除已知的杂志和展会标记
static RE_SOURCE_PAREN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\(([^)]+)\)").unwrap());

// ════════════════════════════════════════════════════════════
// 类型
// ════════════════════════════════════════════════════════════

/// Tag 类别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagCategory {
    Author,
    Circle,
    Source,
    Magazine,
    Event,
    Language,
    Edition,
}

impl TagCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            TagCategory::Author => "AUTHOR",
            TagCategory::Circle => "CIRCLE",
            TagCategory::Source => "SOURCE",
            TagCategory::Magazine => "MAGAZINE",
            TagCategory::Event => "EVENT",
            TagCategory::Language => "LANGUAGE",
            TagCategory::Edition => "EDITION",
        }
    }
}

/// 从 page_title 提取的候选 tag
#[derive(Debug, Clone)]
pub struct TagSuggestion {
    pub name: String,
    pub category: TagCategory,
}

// ════════════════════════════════════════════════════════════
// 提取逻辑
// ════════════════════════════════════════════════════════════

/// 已知杂志名的规范化映射（提取为干净的杂志名）
fn normalize_magazine_name(raw: &str) -> Option<&str> {
    let patterns: &[(&str, &str)] = &[
        ("COMIC LO", "COMIC LO"),
        ("COMIC 快楽天ビースト", "COMIC 快楽天ビースト"),
        ("COMIC 快楽天", "COMIC 快楽天"),
        ("COMIC BAVEL", "COMIC BAVEL"),
        ("COMIC 失楽天", "COMIC 失楽天"),
        ("COMIC アンスリウム", "COMIC アンスリウム"),
        ("COMIC アオハ", "COMIC アオハ"),
        ("COMIC ExE", "COMIC ExE"),
        ("COMIC 夢幻転生", "COMIC 夢幻転生"),
        ("COMIC エグゼ", "COMIC エグゼ"),
        ("コミックホットミルク", "コミックホットミルク"),
        ("コミックゼロス", "コミックゼロス"),
        ("コミックアンリアル", "コミックアンリアル"),
        ("永遠娘", "永遠娘"),
    ];
    // 比较时忽略空格差异（如 COMIC快楽天 vs COMIC 快楽天）
    let normalized_raw: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    let upper = normalized_raw.to_uppercase();
    for (key, name) in patterns {
        let key_normalized: String = key.chars().filter(|c| !c.is_whitespace()).collect();
        if upper.contains(&key_normalized.to_uppercase()) {
            return Some(name);
        }
    }
    None
}

/// 从 page_title 提取所有候选 tag（去重）
pub fn extract(page_title: &str) -> Vec<TagSuggestion> {
    let mut tags: Vec<TagSuggestion> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let mut add = |name: String, cat: TagCategory| {
        if seen.insert(name.clone()) {
            tags.push(TagSuggestion {
                name,
                category: cat,
            });
        }
    };

    // ── 1. 作者/社团 ──
    // 取第一个非标签的 [xxx] 作为作者标记
    for cap in RE_AUTHOR_BRACKET.captures_iter(page_title) {
        let content = cap[1].trim();

        // 跳过已知的标签类 bracket（语言/版本）
        if RE_TAG_BRACKET.is_match(cap.get(0).unwrap().as_str()) {
            continue;
        }

        // 尝试拆分为 [Circle (Author)]
        if let Some(cc) = RE_CIRCLE_AUTHOR.captures(content) {
            let circle = cc[1].trim();
            if !circle.is_empty() {
                add(circle.to_string(), TagCategory::Circle);
            }
        }

        // 整个 [xxx] 内容作为 Author
        if !content.chars().all(|c| c.is_ascii_digit()) && content.len() > 1 {
            add(content.to_string(), TagCategory::Author);
        }

        // 只取第一个非标签的作者标记
        break;
    }

    // ── 2. 展会 ──
    for cap in RE_EVENT_PAREN.captures_iter(page_title) {
        let event = cap[0].trim_matches(&['(', ')'] as &[_]).trim();
        if !event.is_empty() {
            add(event.to_string(), TagCategory::Event);
        }
    }

    // ── 3. 杂志（括号内）──
    for cap in RE_MAGAZINE_IN_PAREN.captures_iter(page_title) {
        let raw = cap[0].trim_matches(&['(', ')'] as &[_]);
        if let Some(name) = normalize_magazine_name(raw) {
            add(name.to_string(), TagCategory::Magazine);
        }
    }

    // ── 4. 杂志（独立）──
    if let Some(cap) = RE_MAGAZINE.captures(page_title) {
        let raw = cap[0].trim();
        if let Some(name) = normalize_magazine_name(raw) {
            add(name.to_string(), TagCategory::Magazine);
        }
    }

    // ── 5. 语言/版本 ──
    for cap in RE_TAG_BRACKET.captures_iter(page_title) {
        let raw = cap[0].trim_matches(&['[', ']'] as &[_]).trim();
        let cat = match raw.to_lowercase().as_str() {
            s if s.contains("翻訳")
                || s.contains("翻譯")
                || s.contains("翻译")
                || s.contains("chinese")
                || s == "英訳" =>
            {
                TagCategory::Language
            }
            _ => TagCategory::Edition,
        };
        add(raw.to_string(), cat);
    }

    // ── 6. 原作来源 ──
    // 提取所有括号内容，排除已知的 Event / Magazine
    for cap in RE_SOURCE_PAREN.captures_iter(page_title) {
        let raw = cap[0].trim_matches(&['(', ')'] as &[_]).trim();
        if raw.is_empty() {
            continue;
        }

        // 跳过 Event
        if RE_EVENT_PAREN.is_match(cap.get(0).unwrap().as_str()) {
            continue;
        }

        // 跳过 Magazine
        if RE_MAGAZINE_IN_PAREN.is_match(cap.get(0).unwrap().as_str()) {
            continue;
        }

        // 跳过纯数字、日期、卷期号
        if raw
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == '-' || c == '～')
        {
            continue;
        }

        // 跳过太短的
        if raw.len() < 3 {
            continue;
        }

        // 跳过已知的非原作模式
        let skip_prefixes = [
            "DL版",
            "中国翻訳",
            "中国語",
            "英訳",
            "無修正",
            "フルカラー版",
            "完全版",
            "総集編",
            "総集篇",
            "デジタル特装版",
            "FANZA特別版",
        ];
        if skip_prefixes.iter().any(|p| raw.eq_ignore_ascii_case(p)) {
            continue;
        }

        add(raw.to_string(), TagCategory::Source);
    }

    tags
}

// ════════════════════════════════════════════════════════════
// 测试
// ════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_author_and_circle() {
        let tags = extract("[湯山チカ] 先生とぼく 第1-5話 [中国翻訳]");
        let names: Vec<_> = tags.iter().map(|t| (t.name.as_str(), t.category)).collect();
        assert!(names.contains(&("湯山チカ", TagCategory::Author)));
        assert!(names.contains(&("中国翻訳", TagCategory::Language)));
    }

    #[test]
    fn test_extract_circle_with_author() {
        let tags = extract("[DOLL PLAY (黒巣ガタリ)] アカリパコパコ [DL版]");
        let names: Vec<_> = tags.iter().map(|t| (t.name.as_str(), t.category)).collect();
        assert!(names.contains(&("DOLL PLAY", TagCategory::Circle)));
        assert!(names.contains(&("DOLL PLAY (黒巣ガタリ)", TagCategory::Author)));
        assert!(names.contains(&("DL版", TagCategory::Edition)));
    }

    #[test]
    fn test_extract_source_and_event() {
        let tags = extract(
            "(C95) [P:P (おりょう)] ゆきのん限定 3 (やはり俺の青春ラブコメはまちがっている。) [中国翻訳]",
        );
        let names: Vec<_> = tags.iter().map(|t| (t.name.as_str(), t.category)).collect();
        assert!(names.contains(&("C95", TagCategory::Event)));
        assert!(names.contains(&(
            "やはり俺の青春ラブコメはまちがっている。",
            TagCategory::Source
        )));
        assert!(names.contains(&("中国翻訳", TagCategory::Language)));
    }

    #[test]
    fn test_extract_magazine() {
        let tags = extract("COMIC LO 2019年5月号 [DL版]");
        let names: Vec<_> = tags.iter().map(|t| (t.name.as_str(), t.category)).collect();
        assert!(names.contains(&("COMIC LO", TagCategory::Magazine)));
    }

    #[test]
    fn test_extract_magazine_in_paren() {
        let tags = extract("[みちきんぐ] 主従どりーみんぐ (COMIC快楽天 2019年3月号) [DL版]");
        let names: Vec<_> = tags.iter().map(|t| (t.name.as_str(), t.category)).collect();
        assert!(names.contains(&("COMIC 快楽天", TagCategory::Magazine)));
        assert!(names.contains(&("みちきんぐ", TagCategory::Author)));
    }

    #[test]
    fn test_extract_multiple_sources() {
        let tags = extract(
            "(C95) [出席番号26 (にろ)] 分身して浜風と三穴えっち (艦隊これくしょん -艦これ-)",
        );
        let names: Vec<_> = tags.iter().map(|t| (t.name.as_str(), t.category)).collect();
        assert!(names.contains(&("C95", TagCategory::Event)));
        assert!(names.contains(&("艦隊これくしょん -艦これ-", TagCategory::Source)));
        assert!(names.contains(&("出席番号26 (にろ)", TagCategory::Author)));
    }

    #[test]
    fn test_no_duplicates() {
        let tags = extract("[湯山チカ] 先生とぼく 第1-5話 [中国翻訳] [DL版]");
        // 中国翻訳 只出现一次
        let lang_count = tags.iter().filter(|t| t.name == "中国翻訳").count();
        assert_eq!(lang_count, 1);
    }
}
