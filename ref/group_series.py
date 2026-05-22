#!/usr/bin/env python3
"""
将 doc_export.csv 中的 page_title 归类为系列。

策略：
1. 规范化：剥离展会标记、翻译标签、DL版标记、章节/卷数范围、杂志年月号
2. 提取「作者/社团 + 核心标题」作为系列标识键
3. 对无法精确匹配的，使用 rapidfuzz 做近似匹配归并
4. 输出分组结果

用法：
    pip install rapidfuzz  # 可选，启用模糊合并
    python scripts/group_series.py doc_export.csv > series_groups.csv
    python scripts/group_series.py doc_export.csv --summary   # 只看系列摘要
    python scripts/group_series.py doc_export.csv --json > series_groups.json
"""

import csv
import re
import sys
from collections import defaultdict

try:
    from rapidfuzz import fuzz
    HAS_FUZZ = True
except ImportError:
    HAS_FUZZ = False


# ══════════════════════════════════════════════════════════════
# 正则规则
# ══════════════════════════════════════════════════════════════

# 展会 / 活动标记: (C95), (COMIC1☆15), (例大祭13), ...
RE_EVENT = re.compile(
    r'\((?:C\d{2,3}|コミ[^\)]*|COMIC[^\)]*|COMIC1[^\)]*|'
    r'紅楼夢\d*|例大祭\d*|サンクリ\d*|秋葉原[^\)]*|'
    r'第\d+回[^\)]*|GW超同人祭|AC\d*|FF\d+|SPARK\d+|'
    r'歌姫庭園\d*|砲雷撃戦[^\)]*|超こみっく[^\)]*|'
    r'メガMBFes[^\)]*|コミティア\d*|#にじそうさく\d*)\)',
    re.IGNORECASE,
)

# 翻译 / 版本标签
RE_TAG = re.compile(
    r'\[(?:中国翻訳|中国語|中國翻譯|中國語|中文翻譯|中文書面語版|'
    r'英訳|無修正|DL版|DLsite[^\]]*|進行中|ページ欠落|'
    r'見本|日本語、中国語|Chinese|chinese|uncensored|'
    r'流木个人汉化|无毒汉化组|鬼畜王汉化组|'
    r'デジタル特装版|FANZA特別版|'
    r'Digital|'
    r'\d{4}年\d{1,2}月\d{1,2}日)\]',
    re.IGNORECASE,
)

# 章节号: 第1話, 第1-5話, 最終話, 前編, 後編, 上巻 ...
RE_CHAPTER = re.compile(
    r'[ 　]*'
    r'(?:'
    r'第\s*(?:\d+[-~～—―]?\d*)\s*[話章节]|'
    r'Ch\.\s*\d+\s*[-~]\s*\d+|'
    r'#[#]?\d+\s*[-~]\s*\d*|'
    r'\d+\s*[-~～—]\s*\d+\s*(?:話|ページ|卷|巻|编|編)|'
    r'第\s*\d+\s*[巻卷]|'
    r'Vol\.\s*\d+|'
    r'EPISODE\s*\d+|'
    r'最終話|'
    r'[前中後后]編|'
    r'[上下]巻|'
    r'[上下](?![\u4e00-\u9fff])'
    r')',
    re.IGNORECASE,
)

# 数字范围: 1-5, 01~18 (末尾)
RE_NUMBER_RANGE = re.compile(
    r'[ 　]*[0-9０-９]+\s*[-~～—]\s*[0-9０-９]+'
    r'(?:\s*(?:話|ページ|卷|巻|编|編|Ch|Vol|Ep))?'
    r'\s*$'
)

# 末尾独立数字: 楓と鈴3, 永遠娘 6, 姉体験女学寮 5.5
RE_TRAILING_NUMBER = re.compile(r'[ 　]*\d+(?:\.\d+)?\s*$')

# 杂志年月号: 2019年4月号, 2020年10月号
RE_MAGAZINE_DATE = re.compile(r'[ 　]+\d{4}年\d{1,2}月号\s*$')

# 版本标记: (全), 総集編, 完全版, ～Standard Edition～, End Collection ...
RE_EDITION = re.compile(
    r'[ 　～~]*(?:'
    r'\((?:全|総集編|完全版|総集篇|デジタル特装版|フルカラー版)\)|'
    r'[～~]*(?:Limited|Standard|Special)\s*Edition[～~]*|'
    r'End\s*Collection|'
    r'総集編|総集篇|完全版|'
    r'デジタル特装版|'
    r'フルカラー版'
    r')\s*',
    re.IGNORECASE,
)

# 合规标记: [DL版]
RE_DL_TAG = re.compile(r'\[DL版\]', re.IGNORECASE)

RE_MULTI_SPACE = re.compile(r'[ 　]+')


# ══════════════════════════════════════════════════════════════
# 规范化
# ══════════════════════════════════════════════════════════════

def normalize(title: str) -> str:
    """返回规范化后的核心标题（用于分组比对）"""
    t = title.strip()

    # 1. 去展会标记
    t = RE_EVENT.sub('', t)

    # 2. 去翻译 / 版本标签
    t = RE_TAG.sub('', t)

    # 3. 去章节/卷数（多次迭代，处理嵌套情况）
    for _ in range(3):
        before = t
        t = RE_CHAPTER.sub('', t).strip()
        t = RE_NUMBER_RANGE.sub('', t).strip()
        t = RE_TRAILING_NUMBER.sub('', t).strip()
        if t == before:
            break

    # 4. 去杂志年月号
    t = RE_MAGAZINE_DATE.sub('', t).strip()

    # 5. 去版本标记
    t = RE_EDITION.sub(' ', t).strip()

    # 6. 去 [DL版]
    t = RE_DL_TAG.sub('', t)

    # 7. 合并多余空白
    t = RE_MULTI_SPACE.sub(' ', t).strip()

    # 8. 清理残留的括号/方括号空白
    t = re.sub(r'\(\s*\)', '', t)
    t = re.sub(r'\[\s*\]', '', t)

    # 9. 整理首尾
    t = t.strip(' ,、。')
    t = RE_MULTI_SPACE.sub(' ', t).strip()

    return t


def extract_series_key(title: str) -> str:
    """
    提取「系列标识键」。
    同一作者 + 同一核心标题 → 同一系列。
    """
    norm = normalize(title)

    author = ''
    m = re.match(r'[\[\(]([^\]\)]+)[\]\)]', norm)
    if m:
        author = m.group(1).strip()
        rest = norm[m.end():].strip()
    else:
        rest = norm

    # 如果 rest 以作者标记开头，再剥离一次
    rest = re.sub(r'^[\[\(][^\]\)]+[\]\)]\s*', '', rest)

    if not rest:
        return norm

    return f"{author}:::{rest}"


def get_core_title(title: str) -> str:
    """获取规范化后的核心标题（不含作者前缀）"""
    norm = normalize(title)
    norm = re.sub(r'^[\[\(][^\]\)]+[\]\)]\s*', '', norm)
    return norm.strip()


# ══════════════════════════════════════════════════════════════
# 分组
# ══════════════════════════════════════════════════════════════

def group_by_exact_key(rows):
    groups = defaultdict(list)
    for row in rows:
        key = extract_series_key(row[1])
        groups[key].append(row)
    return dict(groups)


def merge_fuzzy(groups, threshold=85):
    """对核心标题做近似匹配，合并相似组"""
    if not HAS_FUZZ:
        print("⚠ 未安装 rapidfuzz，跳过模糊合并。pip install rapidfuzz",
              file=sys.stderr)
        return groups

    items = list(groups.items())
    merged = {}
    used = [False] * len(items)

    for i, (key_a, rows_a) in enumerate(items):
        if used[i]:
            continue
        core_a = get_core_title(rows_a[0][1]) if rows_a else key_a
        merged[key_a] = list(rows_a)
        used[i] = True

        for j, (key_b, rows_b) in enumerate(items):
            if used[j]:
                continue
            core_b = get_core_title(rows_b[0][1]) if rows_b else key_b
            score = fuzz.token_sort_ratio(core_a, core_b)
            if score >= threshold:
                merged[key_a].extend(rows_b)
                used[j] = True

    return merged


# ══════════════════════════════════════════════════════════════
# 输出
# ══════════════════════════════════════════════════════════════

def output_csv(groups, out=sys.stdout):
    writer = csv.writer(out)
    writer.writerow(["series_id", "id", "page_title"])
    for sid, (key, rows) in enumerate(groups.items(), 1):
        for row in rows:
            writer.writerow([sid, row[0], row[1]])


def output_summary_csv(groups, out=sys.stdout):
    writer = csv.writer(out)
    writer.writerow(["series_id", "count", "representative_title"])
    for sid, (key, rows) in enumerate(groups.items(), 1):
        rep = min(rows, key=lambda r: len(r[1]))
        writer.writerow([sid, len(rows), rep[1]])


def output_json(groups, out=sys.stdout):
    import json
    result = []
    for sid, (key, rows) in enumerate(groups.items(), 1):
        result.append({
            "series_id": sid,
            "key": key,
            "count": len(rows),
            "items": [{"id": int(r[0]), "page_title": r[1]} for r in rows],
        })
    json.dump(result, out, ensure_ascii=False, indent=2)


# ══════════════════════════════════════════════════════════════
# 入口
# ══════════════════════════════════════════════════════════════

def main():
    import argparse
    parser = argparse.ArgumentParser(description="Group page_title into series")
    parser.add_argument("csv_file", help="Path to doc_export.csv")
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--summary", action="store_true")
    parser.add_argument("--threshold", type=int, default=85)
    parser.add_argument("--no-fuzzy", action="store_true")
    args = parser.parse_args()

    rows = []
    with open(args.csv_file, newline='', encoding='utf-8') as f:
        reader = csv.reader(f)
        header = next(reader, None)
        if header != ["id", "page_title"]:
            print(f"⚠ 预期标题 ['id', 'page_title']，实际: {header}", file=sys.stderr)
        for row in reader:
            if len(row) >= 2:
                rows.append((row[0], row[1]))

    print(f"读入 {len(rows)} 条", file=sys.stderr)

    groups = group_by_exact_key(rows)
    print(f"精确分组: {len(groups)} 个系列", file=sys.stderr)

    if not args.no_fuzzy:
        groups = merge_fuzzy(groups, threshold=args.threshold)
        print(f"模糊合并后: {len(groups)} 个系列", file=sys.stderr)

    if args.json:
        output_json(groups)
    elif args.summary:
        output_summary_csv(groups)
    else:
        output_csv(groups)


if __name__ == "__main__":
    main()
