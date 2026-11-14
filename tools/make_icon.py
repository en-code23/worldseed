"""Draws the Worldseed pixel-art icon (a sprouting seed over a strip of earth) as a 1024px PNG."""
import zlib, struct, sys
G = 16  # 16x16 pixel grid
art = [
"................",
"................",
"........LL......",
".......LLLL.....",
"..LL...L..L.....",
".LLLL..LS.......",
".L..LL.S........",
".....LLS........",
"......SS........",
"......S.........",
"....BBBBB.......",
"...BBBBBBB......",
"GGGGGGGGGGGGGGGG",
"DDDDDDDDDDDDDDDD",
"DDDdDDDDDDdDDDDD",
"DDDDDDDDDDDDDDDD",
]
pal = {".": (13, 47, 33), "L": (212, 236, 74), "S": (150, 190, 60), "B": (196, 150, 92),
       "G": (88, 160, 64), "D": (92, 64, 44), "d": (70, 48, 34)}
N = 1024; cell = N // G
rows = []
for y in range(N):
    row = bytearray([0])
    for x in range(N):
        gx, gy = x // cell, y // cell
        c = pal[art[gy][gx]]
        # rounded-square mask (macOS style)
        r = 180; cx = min(x, N - 1 - x); cy = min(y, N - 1 - y)
        a = 255
        if len(sys.argv) < 3 and cx < r and cy < r and (r - cx) ** 2 + (r - cy) ** 2 > r * r: a = 0
        row += bytes(c) + (bytes([a]) if len(sys.argv) < 3 else b"")
    rows.append(bytes(row))
raw = b"".join(rows)
def chunk(t, d): return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xffffffff)
png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", N, N, 8, 6 if len(sys.argv) < 3 else 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")
open(sys.argv[1], "wb").write(png)
