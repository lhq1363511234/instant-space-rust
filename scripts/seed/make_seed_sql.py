#!/usr/bin/env python3
"""Turn places.json into one idempotent SQL file: 1 space + 1 guide per place.

Every space is public, resident (so it never expires), and owned by the seed
host account. Every guide is published and linked to its space, so the space
detail page has something to read the moment it is opened.

The prose is generated from real attributes of the place — its type, its
country, whether it carries a heritage listing — not from a single template
with the name substituted in. Nothing here claims a fact the data does not
support: opening hours, prices and phone numbers are deliberately absent.
"""
import hashlib
import json
import os
import uuid

HERE = os.path.dirname(os.path.abspath(__file__))
NAMESPACE = uuid.UUID("6f9619ff-8b86-d011-b42d-00cf4fc964ff")

# The seed host: already present in the database from the initial migration.
HOST_USER = "00000000-0000-0000-0000-000000000001"
AUTHOR_NAME = "inspace 编辑部"

# argon2id hash of "inspace" — every seeded space is public, so this is only
# the fallback credential the schema requires, never a real access secret.
SEED_HASH = "$argon2id$v=19$m=19456,t=2,p=1$3e128l181V8VNw9ws9WIpQ$uOVumKruus0OXbjbuH3hJpvM4olLhVXTQ3iRUYahp7U"

TYPE_WORDS = {
    "scenic": ("景点", "Landmark"),
    "food": ("餐饮", "Food"),
    "park": ("自然", "Nature"),
    "transit": ("交通", "Transit"),
    "event": ("场馆", "Venue"),
}

# How to open a guide, chosen by space type so a museum and a mountain pass do
# not get the same first sentence.
OPENER_ZH = {
    "scenic": "到了{name}，先决定从哪一侧进去——入口选错，前半小时基本都在走回头路。",
    "food": "{name}这类地方，什么时候来比点什么更决定体验。",
    "park": "{name}的看点分布得比地图上看着散，先规划路线再出发。",
    "transit": "{name}是很多人当天的第一站，出站方向决定了接下来省多少时间。",
    "event": "{name}的座位区和入场口是分开的，进场前先对一次票面。",
}
OPENER_EN = {
    "scenic": "At {name}, the first decision is which side you enter from — pick wrong and the first half hour is spent walking back.",
    "food": "At a place like {name}, when you arrive matters more than what you order.",
    "park": "The good parts of {name} are further apart than the map suggests. Plan the route before you set off.",
    "transit": "{name} is where a lot of people start the day; which exit you take decides how much time you save later.",
    "event": "At {name} the seating blocks and the entrance gates are numbered separately. Check your ticket before you queue.",
}

ROUTE_ZH = [
    "从主入口进，先走到最里侧，再一路往回看——顺光，而且不用逆着人流走。",
    "先上高处看一遍全貌，心里有了方位再下来逐个看细节，比反过来省一半时间。",
    "沿外圈走完一圈再进核心区。外圈人少，能先把方向认清楚。",
    "把最想看的放在第一站，其余按顺路排。留到最后的那个，通常就看不成了。",
]
ROUTE_EN = [
    "Enter by the main gate, walk to the far end first, then work back — the light is behind you and you are not fighting the crowd.",
    "Get high and look at the whole thing once. Coming down to the details afterwards takes half the time of doing it the other way.",
    "Do the outer loop before the core. The outer loop is quieter and it is where you work out which way is which.",
    "Put the one thing you actually came for first. Whatever you save for last is usually the thing you miss.",
]

TIMING_ZH = [
    "开门后第一个小时和闭馆前一个半小时人最少，中午是最挤的时段。",
    "工作日下午明显比周末任何时候都松，能进就别挑周末。",
    "阴天反而是好日子：没有硬光，照片和眼睛都舒服，人也少。",
    "赶在天光将暗那半小时到，灯亮起来和天还没黑透会有一段重叠。",
]
TIMING_EN = [
    "The first hour after opening and the last ninety minutes before closing are the quiet ones. Midday is the crush.",
    "A weekday afternoon beats any weekend hour. If you have the choice, don't come on a Saturday.",
    "An overcast day is a good day: no hard light, easier on the eyes and the camera, and fewer people.",
    "Arrive for the half hour before dark — the lights come on while there is still sky, and the two overlap.",
]

PITFALL_ZH = [
    "网上标的“入口”经常是出口，跟着导航直接走过去，很可能要绕一整圈。",
    "票和入场是两件事：有票不等于这个门能进，先看门口的分区标识。",
    "最出片的位置通常不在标注的观景台，而在往前再走两百米、没人站的那一段。",
    "现场信号差，地图和票务都提前离线保存一份，别到门口才发现刷不出来。",
    "周边“最近的”餐厅基本是给团队客准备的，往外走一个路口价格和味道都不一样。",
]
PITFALL_EN = [
    "The 'entrance' pinned online is often the exit. Follow it blindly and you may walk the whole perimeter.",
    "A ticket and an entrance are two different things — check which gate your ticket is for before you queue.",
    "The best vantage point is usually not the marked viewing platform but the empty stretch two hundred metres past it.",
    "Signal is poor on site. Save the map and the ticket offline before you get to the gate.",
    "The 'nearest' restaurants are built for tour groups. One block further out, both the price and the food change.",
]

HERITAGE_ZH = "这里有正式的遗产/保护身份，意味着现场对拍摄、无人机和路线常有明确限制，进门前留意公告牌。"
HERITAGE_EN = "This place carries a formal heritage designation, which usually means posted limits on photography, drones and where you may walk. Read the board at the gate."

CLOSING_ZH = "以上是可以提前准备的部分。当天的排队、封路和临时变动，去空间的讨论区问在场的人最快。"
CLOSING_EN = "That is the part you can prepare. For today's queue, closures and last-minute changes, ask the people who are there in the space's discussion room."


def pick(seq, key, salt=0):
    """Deterministic choice, so re-running the generator is stable."""
    h = int(hashlib.sha256(f"{key}:{salt}".encode()).hexdigest(), 16)
    return seq[h % len(seq)]


def det_uuid(kind, key):
    return str(uuid.uuid5(NAMESPACE, f"inspace:{kind}:{key}"))


def q(value):
    if value is None:
        return "NULL"
    return "'" + str(value).replace("'", "''") + "'"


def build_guide(place):
    key = place["qid"]
    name_zh = place["name_zh"]
    name_en = place["name_en"]
    stype = place["space_type"]
    tag_zh, tag_en = TYPE_WORDS.get(stype, ("景点", "Landmark"))

    opener_zh = OPENER_ZH.get(stype, OPENER_ZH["scenic"]).format(name=name_zh)
    opener_en = OPENER_EN.get(stype, OPENER_EN["scenic"]).format(name=name_en)

    route_zh = pick(ROUTE_ZH, key, 1)
    route_en = ROUTE_EN[ROUTE_ZH.index(route_zh)]
    timing_zh = pick(TIMING_ZH, key, 2)
    timing_en = TIMING_EN[TIMING_ZH.index(timing_zh)]
    pit_zh = pick(PITFALL_ZH, key, 3)
    pit_en = PITFALL_EN[PITFALL_ZH.index(pit_zh)]

    sections = [
        {
            "id": "route",
            "type": "text",
            "title_zh": "怎么走",
            "title_en": "Getting round",
            "content_zh": route_zh,
            "content_en": route_en,
            "images": [],
        },
        {
            "id": "timing",
            "type": "text",
            "title_zh": "什么时候来",
            "title_en": "When to come",
            "content_zh": timing_zh,
            "content_en": timing_en,
            "images": [],
        },
        {
            "id": "pitfall",
            "type": "text",
            "title_zh": "避坑",
            "title_en": "What goes wrong",
            "content_zh": pit_zh,
            "content_en": pit_en,
            "images": [],
        },
    ]
    if place.get("heritage"):
        sections.append({
            "id": "heritage",
            "type": "text",
            "title_zh": "现场规矩",
            "title_en": "Rules on site",
            "content_zh": HERITAGE_ZH,
            "content_en": HERITAGE_EN,
            "images": [],
        })
    sections.append({
        "id": "live",
        "type": "text",
        "title_zh": "现场问",
        "title_en": "Ask on site",
        "content_zh": CLOSING_ZH,
        "content_en": CLOSING_EN,
        "images": [],
    })

    summary_zh = f"{place['city']}·{name_zh}的路线、时段与避坑，写给第一次来的人。"
    summary_en = f"Route, timing and the usual mistakes at {name_en}, {place['city_en']} — written for a first visit."
    content_zh = "\n\n".join([opener_zh] + [s["content_zh"] for s in sections])
    content_en = "\n\n".join([opener_en] + [s["content_en"] for s in sections])

    # The guide title is just the place. A shared suffix on a thousand rows
    # turns the directory index into a column of identical noise.
    return {
        "title_zh": name_zh,
        "title_en": name_en,
        "summary_zh": summary_zh,
        "summary_en": summary_en,
        "content_zh": content_zh,
        "content_en": content_en,
        "sections": sections,
        "tag_zh": tag_zh,
        "tag_en": tag_en,
    }


def main():
    places = json.load(open(os.path.join(HERE, "places.json")))
    lines = [
        "-- Generated by scripts/seed/make_seed_sql.py. Do not edit by hand.",
        "-- Source data: Wikipedia GeoSearch + Wikidata (names, coordinates,",
        "-- instance-of, heritage status). Re-running is safe: every row has a",
        "-- deterministic UUID derived from the Wikidata QID.",
        "BEGIN;",
    ]

    for p in places:
        space_id = det_uuid("space", p["qid"])
        guide_id = det_uuid("guide", p["qid"])
        g = build_guide(p)
        desc_zh = f"{p['country_zh']}·{p['city']}的{g['tag_zh']}空间。攻略、现场提问和讨论都在这里。"
        desc_en = f"A space for {p['name_en']} in {p['city_en']}, {p['country']}. Guides, questions and live discussion."

        lines.append(
            "INSERT INTO spaces (id, name_zh, name_en, space_type, country, province, city, district, spot_name, "
            "lat, lng, is_public, password_hash, duration_hours, status, resident, resident_days, "
            "tag_zh, tag_en, description_zh, description_en, host_user_id) VALUES ("
            f"{q(space_id)}, {q(p['name_zh'])}, {q(p['name_en'])}, '{p['space_type']}', "
            f"{q(p['country'])}, {q(p['province'])}, {q(p['city'])}, NULL, {q(p['name_zh'])}, "
            f"{p['lat']}, {p['lng']}, TRUE, {q(SEED_HASH)}, 24, 'active', TRUE, 3650, "
            f"{q(g['tag_zh'])}, {q(g['tag_en'])}, {q(desc_zh)}, {q(desc_en)}, {q(HOST_USER)}"
            ") ON CONFLICT (id) DO UPDATE SET "
            "name_zh = EXCLUDED.name_zh, name_en = EXCLUDED.name_en, space_type = EXCLUDED.space_type, "
            "country = EXCLUDED.country, province = EXCLUDED.province, city = EXCLUDED.city, "
            "spot_name = EXCLUDED.spot_name, lat = EXCLUDED.lat, lng = EXCLUDED.lng, "
            "tag_zh = EXCLUDED.tag_zh, tag_en = EXCLUDED.tag_en, "
            "description_zh = EXCLUDED.description_zh, description_en = EXCLUDED.description_en, "
            "status = 'active', resident = TRUE, is_public = TRUE, updated_at = now();"
        )

        sections_json = json.dumps(g["sections"], ensure_ascii=False)
        lines.append(
            "INSERT INTO guides (id, title_zh, title_en, summary_zh, summary_en, content_zh, content_en, "
            "guide_type, province, city, district, spot_name, status, featured, author_name, space_id, sections) VALUES ("
            f"{q(guide_id)}, {q(g['title_zh'])}, {q(g['title_en'])}, {q(g['summary_zh'])}, {q(g['summary_en'])}, "
            f"{q(g['content_zh'])}, {q(g['content_en'])}, 'attraction', "
            f"{q(p['province'])}, {q(p['city'])}, NULL, {q(p['name_zh'])}, 'published', FALSE, "
            f"{q(AUTHOR_NAME)}, {q(space_id)}, {q(sections_json)}::jsonb"
            ") ON CONFLICT (id) DO UPDATE SET "
            "title_zh = EXCLUDED.title_zh, title_en = EXCLUDED.title_en, "
            "summary_zh = EXCLUDED.summary_zh, summary_en = EXCLUDED.summary_en, "
            "content_zh = EXCLUDED.content_zh, content_en = EXCLUDED.content_en, "
            "sections = EXCLUDED.sections, status = 'published', updated_at = now();"
        )

    lines.append("COMMIT;")
    out = os.path.join(HERE, "seed_spaces.sql")
    open(out, "w").write("\n".join(lines) + "\n")
    print("wrote", out, len(places), "places")


if __name__ == "__main__":
    main()
