#!/usr/bin/env python3
"""Idempotently seed 50 unclaimed Spaces and one guide per Space for every
Chinese province-level region.

The source points come from the repository's geo_places table. These are
editorial bootstrap Spaces, not invented attraction listings: every row is
labelled "Host wanted" and asks a local curator to claim and improve it.

Usage:
  DATABASE_URL=postgres://... python3 scripts/seed/seed_china_provinces.py --dry-run
  DATABASE_URL=postgres://... python3 scripts/seed/seed_china_provinces.py --apply
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import tempfile
import uuid
from collections import defaultdict
from dataclasses import dataclass

NAMESPACE = uuid.UUID("145f4d4e-2333-4b23-a4c3-53e6784be94f")
AUTHOR_ID = "00000000-0000-0000-0000-000000000001"
AUTHOR_NAME = "inspace 编辑部"
SEED_HASH = "$argon2id$v=19$m=19456,t=2,p=1$3e128l181V8VNw9ws9WIpQ$uOVumKruus0OXbjbuH3hJpvM4olLhVXTQ3iRUYahp7U"
PER_PROVINCE = 50


@dataclass(frozen=True)
class Province:
    key: str
    zh: str
    admin1: tuple[str, ...]


PROVINCES = [
    Province("anhui", "安徽省", ("Anhui",)),
    Province("beijing", "北京市", ("Beijing",)),
    Province("chongqing", "重庆市", ("Chongqing",)),
    Province("fujian", "福建省", ("Fujian",)),
    Province("gansu", "甘肃省", ("Gansu",)),
    Province("guangdong", "广东省", ("Guangdong",)),
    Province("guangxi", "广西壮族自治区", ("Guangxi",)),
    Province("guizhou", "贵州省", ("Guizhou",)),
    Province("hainan", "海南省", ("Hainan",)),
    Province("hebei", "河北省", ("Hebei",)),
    Province("heilongjiang", "黑龙江省", ("Heilongjiang",)),
    Province("henan", "河南省", ("Henan",)),
    Province("hong-kong", "香港特别行政区", ("Hong Kong",)),
    Province("hubei", "湖北省", ("Hubei",)),
    Province("hunan", "湖南省", ("Hunan",)),
    Province("inner-mongolia", "内蒙古自治区", ("Inner Mongolia",)),
    Province("jiangsu", "江苏省", ("Jiangsu",)),
    Province("jiangxi", "江西省", ("Jiangxi",)),
    Province("jilin", "吉林省", ("Jilin",)),
    Province("liaoning", "辽宁省", ("Liaoning",)),
    Province("macao", "澳门特别行政区", ("Macao", "Macau")),
    Province("ningxia", "宁夏回族自治区", ("Ningxia",)),
    Province("qinghai", "青海省", ("Qinghai",)),
    Province("shaanxi", "陕西省", ("Shaanxi",)),
    Province("shandong", "山东省", ("Shandong",)),
    Province("shanghai", "上海市", ("Shanghai",)),
    Province("shanxi", "山西省", ("Shanxi",)),
    Province("sichuan", "四川省", ("Sichuan",)),
    Province("taiwan", "台湾省", ("Taiwan", "Taipei")),
    Province("tianjin", "天津市", ("Tianjin",)),
    Province("tibet", "西藏自治区", ("Tibet",)),
    Province("xinjiang", "新疆维吾尔自治区", ("Xinjiang",)),
    Province("yunnan", "云南省", ("Yunnan",)),
    Province("zhejiang", "浙江省", ("Zhejiang",)),
]

THEMES = [
    ("抵达", "Arrival", "transit"),
    ("漫游", "Wandering", "scenic"),
    ("城市散步", "City walk", "park"),
    ("夜行", "Night walk", "event"),
    ("在场问答", "On-site questions", "custom"),
    ("地方味道", "Local flavour", "food"),
    ("旧城记忆", "Local memory", "scenic"),
    ("周末路线", "Weekend route", "custom"),
    ("山水入口", "Landscape gateway", "park"),
    ("旅行互助", "Travel help", "custom"),
]


def sql(value: str | None) -> str:
    if value is None:
        return "NULL"
    return "'" + value.replace("'", "''") + "'"


def database_url() -> str:
    value = os.environ.get("DATABASE_URL", "").strip()
    if not value:
        raise SystemExit("DATABASE_URL is required")
    return value


def fetch_points(url: str) -> dict[str, list[dict]]:
    query = """
      SELECT coalesce(admin1_name,''), place_name, coalesce(admin2_name,''),
             lat::text, lng::text, population::text
      FROM geo_places
      WHERE country_code IN ('CN','HK','MO','TW')
        AND admin1_name IS NOT NULL
        AND btrim(admin1_name) <> ''
      ORDER BY admin1_name, population DESC, place_name
    """
    result = subprocess.run(
        ["psql", url, "-X", "-A", "-t", "-F", "\t", "-v", "ON_ERROR_STOP=1", "-c", query],
        check=True,
        capture_output=True,
        text=True,
    )
    by_admin: dict[str, list[dict]] = defaultdict(list)
    seen: dict[str, set[tuple[str, str, str]]] = defaultdict(set)
    for line in result.stdout.splitlines():
        parts = line.split("\t")
        if len(parts) != 6:
            continue
        admin1, place, admin2, lat, lng, population = parts
        marker = (place, lat, lng)
        if marker in seen[admin1]:
            continue
        seen[admin1].add(marker)
        by_admin[admin1].append(
            {
                "place": place.strip(),
                "admin2": admin2.strip() or None,
                "lat": float(lat),
                "lng": float(lng),
                "population": int(population or 0),
            }
        )
    return by_admin


def tiny_offset(seed: str, repeated: bool) -> tuple[float, float]:
    if not repeated:
        return (0.0, 0.0)
    digest = hashlib.sha256(seed.encode()).digest()
    # Keep repeated editorial nodes within roughly 700 m of the source town.
    return ((digest[0] / 255 - 0.5) * 0.010, (digest[1] / 255 - 0.5) * 0.010)


def sections(province: Province, place: str, theme_zh: str, theme_en: str) -> list[dict]:
    return [
        {
            "id": "before-you-go",
            "type": "text",
            "title_zh": "出发前先知道",
            "title_en": "Before you go",
            "content_zh": f"这是 {province.zh}{place} 的{theme_zh}基础页。当前由 inspace 编辑部先点亮，开放时间、交通与现场变化请在出发前再次核实。",
            "content_en": f"This is a starter {theme_en.lower()} page for {place}, {province.zh}. It was first lit by the inspace editorial team; verify opening, access and current conditions before travelling.",
            "images": [],
        },
        {
            "id": "ask-on-site",
            "type": "text",
            "title_zh": "到达之后问什么",
            "title_en": "What to ask on arrival",
            "content_zh": "路线是否临时调整、哪里正在排队、哪些入口关闭，这些只适合问此刻在场的人。进入讨论页，留下一个具体问题。",
            "content_en": "Temporary route changes, queues and closed entrances are best answered by people there now. Open the discussion page and ask one concrete question.",
            "images": [],
        },
        {
            "id": "host-wanted",
            "type": "text",
            "title_zh": "这个空间正在等主理人",
            "title_en": "This Space needs a local host",
            "content_zh": f"如果你熟悉 {place}，可以把真实路线、时段、避坑和地方故事补进来。系统能点亮坐标，只有当地人能让空间长期可信。",
            "content_en": f"If you know {place}, add the routes, timing, pitfalls and local stories that make this useful. A system can light a coordinate; only a local host can keep it trustworthy.",
            "images": [],
        },
    ]


def build_sql(points_by_admin: dict[str, list[dict]]) -> tuple[str, list[tuple[str, int, int]]]:
    statements = ["BEGIN;", "SET LOCAL statement_timeout = '10min';"]
    report = []
    for province in PROVINCES:
        points = []
        for admin in province.admin1:
            points.extend(points_by_admin.get(admin, []))
        points.sort(key=lambda row: (-row["population"], row["place"]))
        if not points:
            raise SystemExit(f"no geo points for {province.zh}: {province.admin1}")

        for index in range(PER_PROVINCE):
            point = points[index % len(points)]
            repeated = index >= len(points)
            theme_zh, theme_en, space_type = THEMES[(index // max(1, len(points))) % len(THEMES)]
            ordinal = index + 1
            suffix = f" {ordinal:02d}" if repeated else ""
            name_zh = f"{point['place']} · {theme_zh}{suffix}"
            name_en = f"{point['place']} · {theme_en}{suffix}"
            seed_key = f"china-province-v1:{province.key}:{ordinal:02d}"
            space_id = uuid.uuid5(NAMESPACE, seed_key + ":space")
            guide_id = uuid.uuid5(NAMESPACE, seed_key + ":guide")
            dlat, dlng = tiny_offset(seed_key, repeated)
            lat, lng = point["lat"] + dlat, point["lng"] + dlng
            description_zh = (
                f"{province.zh}{point['place']}的{theme_zh}空间，由 inspace 编辑部先点亮。"
                "目前正在招募熟悉这里的空间主理人，补充真实攻略、回答现场问题并保存地方故事。"
            )
            description_en = (
                f"A {theme_en.lower()} Space for {point['place']} in {province.zh}, first lit by the inspace editorial team. "
                "A local host is wanted to add field-tested guidance, answer on-site questions and keep local stories."
            )
            summary_zh = f"{point['place']}的{theme_zh}基础攻略；当前为待认领页，等待当地主理人补充真实路线与现场变化。"
            summary_en = f"A starter {theme_en.lower()} guide for {point['place']}; waiting for a local host to add tested routes and current conditions."
            content_zh = (
                f"这里是 {province.zh}{point['place']} 的{theme_zh}入口。\n\n"
                "这份攻略只提供进入空间所需的基础结构，不冒充当地人的亲历经验。出发前请再次核实交通、开放时间与天气。\n\n"
                "到达后，可以在空间讨论页询问当天路线、排队和临时关闭；有价值的现场回答会逐步整理回攻略。\n\n"
                f"如果你长期生活在 {point['place']}、经常来这里，或愿意持续维护这处地点，欢迎成为空间主理人。"
            )
            content_en = (
                f"This is the {theme_en.lower()} entry for {point['place']} in {province.zh}.\n\n"
                "This starter guide provides structure without pretending to be local first-hand knowledge. Recheck transport, opening and weather before travelling.\n\n"
                "Once there, use the Space discussion for today's route, queues and temporary closures. Useful field answers can be folded back into the guide.\n\n"
                f"If you live in, regularly visit or are willing to maintain {point['place']}, become its local Space host."
            )
            section_json = json.dumps(sections(province, point["place"], theme_zh, theme_en), ensure_ascii=False)

            statements.append(
                "INSERT INTO spaces "
                "(id,name_zh,name_en,space_type,country,province,city,district,spot_name,address_line,lat,lng,is_public,password_hash,duration_hours,status,resident,resident_days,tag_zh,tag_en,description_zh,description_en,creator_id,host_user_id) VALUES ("
                f"{sql(str(space_id))},{sql(name_zh)},{sql(name_en)},{sql(space_type)}::space_type,'China',{sql(province.zh)},{sql(point['place'])},{sql(point['admin2'])},{sql(name_zh)},{sql(province.zh + ' · ' + point['place'])},{lat:.7f},{lng:.7f},TRUE,{sql(SEED_HASH)},24,'active',TRUE,3650,'主理人招募中','Host wanted',{sql(description_zh)},{sql(description_en)},{sql(AUTHOR_ID)},NULL) "
                "ON CONFLICT (id) DO UPDATE SET name_zh=EXCLUDED.name_zh,name_en=EXCLUDED.name_en,space_type=EXCLUDED.space_type,country='China',province=EXCLUDED.province,city=EXCLUDED.city,district=EXCLUDED.district,spot_name=EXCLUDED.spot_name,address_line=EXCLUDED.address_line,lat=EXCLUDED.lat,lng=EXCLUDED.lng,is_public=TRUE,status='active',resident=TRUE,resident_days=3650,tag_zh='主理人招募中',tag_en='Host wanted',description_zh=EXCLUDED.description_zh,description_en=EXCLUDED.description_en,host_user_id=NULL,updated_at=now();"
            )
            statements.append(
                "INSERT INTO guides "
                "(id,title_zh,title_en,summary_zh,summary_en,content_zh,content_en,guide_type,category,province,city,district,spot_name,status,featured,author_id,author_name,space_id,sections) VALUES ("
                f"{sql(str(guide_id))},{sql(name_zh + ' · 基础攻略')},{sql(name_en + ' · starter guide')},{sql(summary_zh)},{sql(summary_en)},{sql(content_zh)},{sql(content_en)},'attraction','host-wanted',{sql(province.zh)},{sql(point['place'])},{sql(point['admin2'])},{sql(name_zh)},'published',FALSE,{sql(AUTHOR_ID)},{sql(AUTHOR_NAME)},{sql(str(space_id))},{sql(section_json)}::jsonb) "
                "ON CONFLICT (id) DO UPDATE SET title_zh=EXCLUDED.title_zh,title_en=EXCLUDED.title_en,summary_zh=EXCLUDED.summary_zh,summary_en=EXCLUDED.summary_en,content_zh=EXCLUDED.content_zh,content_en=EXCLUDED.content_en,category='host-wanted',province=EXCLUDED.province,city=EXCLUDED.city,district=EXCLUDED.district,spot_name=EXCLUDED.spot_name,status='published',author_id=EXCLUDED.author_id,author_name=EXCLUDED.author_name,space_id=EXCLUDED.space_id,sections=EXCLUDED.sections,updated_at=now();"
            )
        report.append((province.zh, len(points), PER_PROVINCE))
    statements.append("COMMIT;")
    return "\n".join(statements) + "\n", report


def main() -> None:
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--dry-run", action="store_true")
    mode.add_argument("--apply", action="store_true")
    args = parser.parse_args()

    url = database_url()
    points = fetch_points(url)
    payload, report = build_sql(points)
    print(f"province-level regions: {len(report)}")
    print(f"spaces: {len(report) * PER_PROVINCE}; guides: {len(report) * PER_PROVINCE}")
    for province, available, target in report:
        print(f"  {province:10s} geo points={available:4d} seed={target}")

    if args.dry_run:
        print(f"dry run SQL bytes: {len(payload.encode('utf-8'))}")
        return

    with tempfile.NamedTemporaryFile("w", encoding="utf-8", suffix=".sql", delete=False) as handle:
        handle.write(payload)
        path = handle.name
    try:
        subprocess.run(["psql", url, "-X", "-q", "-v", "ON_ERROR_STOP=1", "-f", path], check=True)
    finally:
        os.unlink(path)
    print("seed applied")


if __name__ == "__main__":
    main()
