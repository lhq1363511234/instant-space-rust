#!/usr/bin/env python3
"""Collect real, well-known places for the space catalogue.

Wikidata's SPARQL endpoint times out on "everything notable in country X", so
this walks the problem from the other direction: for a curated list of cities
per country, ask Wikipedia's GeoSearch for articles near the centre, then hydrate
each candidate from Wikidata (labels in zh + en, instance-of, heritage status)
and from Wikipedia (30-day pageviews) to rank and classify them.

Output: places.json  — one record per place, ready for the SQL generator.
"""
import json
import os
import sys
import time
import urllib.error
import urllib.parse
import urllib.request

UA = {"User-Agent": "inspace-seed/1.0 (admin@opctoai.com)"}
HERE = os.path.dirname(os.path.abspath(__file__))


def get(url, tries=4):
    for attempt in range(tries):
        try:
            with urllib.request.urlopen(urllib.request.Request(url, headers=UA), timeout=60) as r:
                return json.load(r)
        except Exception as exc:  # noqa: BLE001 - network is the failure mode here
            if attempt == tries - 1:
                print("  ! give up:", exc, file=sys.stderr)
                return None
            time.sleep(2 + attempt * 3)
    return None


def wp(lang, params):
    params.update({"format": "json", "action": "query", "formatversion": "2"})
    return get(f"https://{lang}.wikipedia.org/w/api.php?" + urllib.parse.urlencode(params))


# --- what counts as a place worth a space -----------------------------------
# Wikidata P31 (instance of) values, mapped to our space_type enum.
TYPE_MAP = {
    # scenic / landmark / heritage
    "Q570116": "scenic",   # tourist attraction
    "Q839954": "scenic",   # archaeological site
    "Q23413": "scenic",    # castle
    "Q751876": "scenic",   # château
    "Q16560": "scenic",    # palace
    "Q41176": "scenic",    # building
    "Q811979": "scenic",   # architectural structure
    "Q2319498": "scenic",  # landmark
    "Q1440300": "scenic",  # observation tower
    "Q12518": "scenic",    # tower
    "Q44613": "scenic",    # monastery
    "Q16970": "scenic",    # church building
    "Q34627": "scenic",    # synagogue
    "Q32815": "scenic",    # mosque
    "Q44539": "scenic",    # temple
    "Q11707": "food",      # restaurant
    "Q207694": "scenic",   # art museum
    "Q33506": "scenic",    # museum
    "Q22698": "park",      # park
    "Q46169": "park",      # botanical garden
    "Q1107656": "park",    # garden
    "Q46124": "park",      # national park (generic)
    "Q1370598": "scenic",  # place of worship
    "Q8514": "scenic",     # desert
    "Q8502": "scenic",     # mountain
    "Q23397": "park",      # lake
    "Q40080": "park",      # beach
    "Q4022": "park",       # river
    "Q34763": "park",      # peninsula
    "Q23442": "park",      # island
    "Q179700": "scenic",   # statue
    "Q1348006": "scenic",  # memorial
    "Q55488": "transit",   # railway station
    "Q1248784": "transit",  # airport
    "Q11315": "scenic",    # shopping mall
    "Q41253": "event",     # movie theatre
    "Q24354": "event",     # theatre
    "Q483110": "event",    # stadium
    "Q1329623": "event",   # cultural centre
    "Q207320": "event",    # concert hall
    "Q2087181": "scenic",  # historic site
    "Q4989906": "scenic",  # monument
    "Q3947": "scenic",     # house
    "Q12280": "scenic",    # bridge
    "Q174782": "scenic",   # town square
    "Q79007": "scenic",    # street
    "Q1497375": "scenic",  # architectural ensemble
    "Q3947226": "scenic",  # residence
    "Q35112127": "scenic",  # archaeological find
    "Q2065736": "scenic",  # cultural property
    "Q15243209": "scenic",  # historic district
}

# Things that are technically near the centre but are not a place you visit.
REJECT_P31 = {
    "Q5",           # human
    "Q515", "Q1549591", "Q486972", "Q3957",   # city / town / settlement
    "Q6256", "Q3624078",   # country
    "Q4167410",     # disambiguation
    "Q13406463",    # list article
    "Q178561",      # battle
    "Q198",         # war
    "Q1656682",     # event
    "Q43229",       # organization
    "Q4830453",     # business
    "Q3918",        # university
    "Q3914",        # school
    "Q16917",       # hospital
    "Q11424",       # film
    "Q7889",        # video game
    "Q571",         # book
    "Q101352",      # family name
    "Q11266439",    # Wikimedia template
    "Q4167836",     # Wikimedia category
}

BAD_TITLE_BITS = (
    "list of", "timeline", "history of", "siege of", "battle of", "election",
    "census", "massacre", "bombing", "attack", "riot", "disambiguation",
    "(company)", "(band)", "(film)", "(album)", "university", "school",
    "hospital", "airport terminal", "population", "demographics", "economy of",
    "government of", "politics of",
)


def looks_like_a_place(title):
    low = title.lower()
    return not any(bit in low for bit in BAD_TITLE_BITS)


# GeoSearch caps radius at 10km and 500 results per call, so a large city is
# covered by sampling a ring of offset centres around the given point.
MAX_RADIUS = 10000


def _one_geosearch(lang, lat, lon, radius, limit):
    d = wp(lang, {
        "list": "geosearch",
        "gscoord": f"{lat}|{lon}",
        "gsradius": str(min(radius, MAX_RADIUS)),
        "gslimit": str(min(limit, 500)),
    })
    if not d:
        return []
    return [
        {"title": g["title"], "lat": g["lat"], "lng": g["lon"], "dist": g["dist"]}
        for g in d.get("query", {}).get("geosearch", [])
    ]


def geosearch(lang, lat, lon, radius, limit=500):
    out = {}
    for hit in _one_geosearch(lang, lat, lon, radius, limit):
        out[hit["title"]] = hit
    if radius > MAX_RADIUS:
        # Ring of six satellites at the requested radius, each covering 10km.
        import math

        for i in range(6):
            ang = math.pi * 2 * i / 6
            dlat = (radius / 111_320.0) * math.cos(ang)
            dlon = (radius / (111_320.0 * max(0.2, math.cos(math.radians(lat))))) * math.sin(ang)
            for hit in _one_geosearch(lang, lat + dlat, lon + dlon, MAX_RADIUS, limit):
                out.setdefault(hit["title"], hit)
            time.sleep(0.15)
    return list(out.values())


def hydrate_wikipedia(lang, titles):
    """Pageviews + Wikidata id, 50 titles at a time."""
    info = {}
    for i in range(0, len(titles), 50):
        chunk = titles[i:i + 50]
        d = wp(lang, {
            "prop": "pageviews|pageprops",
            "titles": "|".join(chunk),
            "pvipdays": "30",
        })
        if not d:
            continue
        for p in d.get("query", {}).get("pages", []):
            if p.get("missing"):
                continue
            views = sum(v for v in (p.get("pageviews") or {}).values() if v)
            info[p["title"]] = {
                "views": views,
                "qid": (p.get("pageprops") or {}).get("wikibase_item"),
            }
        time.sleep(0.2)
    return info


def hydrate_wikidata(qids):
    out = {}
    for i in range(0, len(qids), 50):
        chunk = qids[i:i + 50]
        u = "https://www.wikidata.org/w/api.php?" + urllib.parse.urlencode({
            "action": "wbgetentities",
            "ids": "|".join(chunk),
            "props": "labels|claims|sitelinks",
            "languages": "zh|zh-hans|zh-hant|en",
            "format": "json",
            "formatversion": "2",
        })
        d = get(u)
        if not d:
            continue
        for qid, e in (d.get("entities") or {}).items():
            if "missing" in e:
                continue
            labels = e.get("labels", {})
            claims = e.get("claims", {})
            p31 = [
                c["mainsnak"]["datavalue"]["value"]["id"]
                for c in claims.get("P31", [])
                if c["mainsnak"].get("datavalue")
            ]
            zh = None
            for key in ("zh", "zh-hans", "zh-hant"):
                if key in labels:
                    zh = labels[key]["value"]
                    break
            out[qid] = {
                "zh": zh,
                "en": labels.get("en", {}).get("value"),
                "p31": p31,
                "heritage": len(claims.get("P1435", [])) > 0,
                "sitelinks": len(e.get("sitelinks") or {}),
            }
        time.sleep(0.2)
    return out


def classify(p31, heritage):
    for q in p31:
        if q in REJECT_P31:
            return None
    for q in p31:
        if q in TYPE_MAP:
            return TYPE_MAP[q]
    # Heritage listing alone is a strong enough signal to keep it as scenic.
    return "scenic" if heritage else None


def collect_city(country, city, langs, want):
    """Return ranked, de-duplicated places around one city."""
    lat, lng, radius = city["lat"], city["lng"], city.get("radius", 9000)
    primary = langs[0]
    cands = {}
    for hit in geosearch(primary, lat, lng, radius, 300):
        if looks_like_a_place(hit["title"]):
            cands[hit["title"]] = hit
    if not cands:
        return []

    info = hydrate_wikipedia(primary, list(cands))
    qids = [v["qid"] for v in info.values() if v.get("qid")]
    wd = hydrate_wikidata(qids)

    rows = []
    for title, hit in cands.items():
        meta = info.get(title)
        if not meta or not meta.get("qid"):
            continue
        ent = wd.get(meta["qid"])
        if not ent:
            continue
        space_type = classify(ent["p31"], ent["heritage"])
        if not space_type:
            continue
        name_en = ent["en"] or title
        name_zh = ent["zh"] or name_en
        # Score: how well-known is it, really.
        score = meta["views"] + ent["sitelinks"] * 400 + (3000 if ent["heritage"] else 0)
        rows.append({
            "qid": meta["qid"],
            "name_zh": name_zh,
            "name_en": name_en,
            "lat": hit["lat"],
            "lng": hit["lng"],
            "space_type": space_type,
            "heritage": ent["heritage"],
            "sitelinks": ent["sitelinks"],
            "views": meta["views"],
            "score": score,
            "country": country["name_en"],
            "country_zh": country["name_zh"],
            "province": city["province_zh"],
            "province_en": city["province_en"],
            "city": city["name_zh"],
            "city_en": city["name_en"],
        })
    rows.sort(key=lambda r: -r["score"])
    return rows[:want]


def main():
    plan = json.load(open(os.path.join(HERE, "plan.json")))
    all_rows = []
    seen_qids = set()
    for country in plan["countries"]:
        got = []
        per_city = country["per_city"]
        langs = country.get("langs", ["en"])
        for city in country["cities"]:
            rows = collect_city(country, city, langs, per_city)
            fresh = [r for r in rows if r["qid"] not in seen_qids]
            for r in fresh:
                seen_qids.add(r["qid"])
            got.extend(fresh)
            print(f"  {country['name_en']:15s} {city['name_en']:22s} +{len(fresh):3d} (total {len(got)})", flush=True)
            if len(got) >= country["target"]:
                break
        got.sort(key=lambda r: -r["score"])
        got = got[:country["target"]]
        print(f"== {country['name_en']}: {len(got)}", flush=True)
        all_rows.extend(got)

    out = os.path.join(HERE, "places.json")
    json.dump(all_rows, open(out, "w"), ensure_ascii=False, indent=1)
    print("wrote", out, len(all_rows))


if __name__ == "__main__":
    main()
