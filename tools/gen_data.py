"""Builds core/data/*.json from TEdit's MS-PL data files (tiles/walls/items).
Usage: python3 -I tools/gen_data.py <tedit-data-dir> <out-dir>"""
import json, sys, os
src, out = sys.argv[1], sys.argv[2]
L = lambda n: json.load(open(os.path.join(src, n), encoding="utf8"))
def col(c):
    c = (c or "#000000FF").lstrip("#")
    return int(c[0:6], 16)
tiles = L("tiles.json")
tmax = max(t["id"] for t in tiles)
T = [["?", 0xFF00FF, 0] for _ in range(tmax + 1)]
frames = {}
for t in tiles:
    T[t["id"]] = [t.get("name", "?"), col(t.get("color")), 1 if t.get("isFramed") else 0]
    if t["id"] in (21, 467, 26, 31, 187, 186, 237, 12, 231, 88):
        fr = []
        for f in t.get("frames", []):
            if "uv" in f:
                fr.append([f["uv"][0], f["uv"][1], f["name"] + (" (" + f["variety"] + ")" if f.get("variety") else "")])
        frames[t["id"]] = fr
# TEdit marks a few tiles/walls with a magenta placeholder; give them plausible map colours.
OVERRIDE = {178: 0xB070C8, 186: 0x7A6E5E, 187: 0x7A6E5E, 233: 0x5FA02E}
for k, v in OVERRIDE.items():
    if k < len(T): T[k][1] = v
walls = L("walls.json")
wmax = max(w["id"] for w in walls)
W = [0] * (wmax + 1)
WN = [""] * (wmax + 1)
for w in walls:
    c = col(w.get("color"))
    W[w["id"]] = 0x3E3A36 if c == 0xFF00FF else c; WN[w["id"]] = w.get("name", "")
items = L("items.json")
imax = max(i["id"] for i in items)
I = [""] * (imax + 1)
for i in items:
    if i["id"] >= 0: I[i["id"]] = i.get("name", "")
json.dump({"tiles": T, "frames": frames, "walls": W, "wallNames": WN}, open(os.path.join(out, "tiles.json"), "w"), separators=(",", ":"))
json.dump(I, open(os.path.join(out, "items.json"), "w"), separators=(",", ":"))
print("tiles", len(T), "walls", len(W), "items", len(I))
