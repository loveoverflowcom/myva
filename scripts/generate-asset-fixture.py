#!/usr/bin/env python3
"""Sinh fixture asset D05-A: vẽ, đóng gói atlas, băm và ghi manifest (chỉ dùng thư viện chuẩn).

Toàn bộ hình được vẽ bằng hàm khoảng cách có dấu (SDF) từ mã nguồn này; không có ảnh, font hay
dữ liệu bên thứ ba. Đây là placeholder hoạt hình viền đậm, màu phẳng để kiểm chứng pipeline, không
phải art thành phẩm. Đường dẫn: vẽ frame → đóng gói (padding, extrude, loang màu biên) → PNG →
sha256 → sheet/layout JSON → manifest. `myva-assets validate` kiểm lại mọi thứ ở phía Rust.

    python3 scripts/generate-asset-fixture.py           # ghi lại assets/fixture
    python3 scripts/generate-asset-fixture.py --check   # vẽ lại, so pixel với file đã commit

Quy ước (docs/technical/asset-pipeline.md): tọa độ texel gốc trên-trái, y hướng xuống; pivot của
nhân vật/boss là điểm chân chạm nền, giống nhau ở mọi frame trong sheet; 2 texel mỗi px thiết kế
cho mặt phẳng gameplay, 1 texel cho nền xa.
"""
import argparse
import hashlib
import json
import math
import pathlib
import struct
import sys
import zlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT = ROOT / 'assets' / 'fixture'
GENERATOR = 'scripts/generate-asset-fixture.py'
CONTENT_VERSION = 'd05a-fixture-1'
GAMEPLAY_TPU = 2.0
BACKDROP_TPU = 1.0
OUTLINE = (27, 30, 43)
PROVENANCE = {
    'origin': 'procedural',
    'tool': GENERATOR,
    'author': 'MyVa — Thần Mạch contributors (sinh bằng mã, không có đầu vào bên thứ ba)',
    'license': 'LicenseRef-MyVa-Unreleased',
    'redistribution': 'Chỉ dùng trong repo MyVa; chủ dự án chưa chọn license phát hành.',
    'created': '2026-10-11',
}
# Ngân sách thiết kế cho fixture (byte RGBA8 sau decode, byte file); validator so với số thực.
BUDGETS = {
    'character.long_luu': {'texture_bytes': 8 << 20, 'file_bytes': 1 << 20},
    'boss.ke_giu_dap': {'texture_bytes': 12 << 20, 'file_bytes': 1 << 20},
    'map.dap_co': {'texture_bytes': 4 << 20, 'file_bytes': 1 << 20},
    'vfx.combat': {'texture_bytes': 2 << 20, 'file_bytes': 512 << 10},
}
LIMITS = {'max_texture_side': 2048, 'total_texture_bytes': 32 << 20, 'total_file_bytes': 4 << 20}


def hexc(value):
    return tuple(int(value[i:i + 2], 16) for i in (1, 3, 5))


# ---------------------------------------------------------------- SDF
def clamp01(v):
    return 0.0 if v < 0.0 else 1.0 if v > 1.0 else v


def circle(cx, cy, r):
    return (lambda x, y: math.hypot(x - cx, y - cy) - r), (cx - r, cy - r, cx + r, cy + r)


def ellipse(cx, cy, rx, ry, angle=0.0):
    c, s = math.cos(angle), math.sin(angle)
    m = min(rx, ry)

    def d(x, y):
        dx, dy = x - cx, y - cy
        u, v = (dx * c + dy * s) / rx, (-dx * s + dy * c) / ry
        return (math.hypot(u, v) - 1.0) * m
    e = max(rx, ry)
    return d, (cx - e, cy - e, cx + e, cy + e)


def capsule(ax, ay, bx, by, ra, rb=None):
    rb = ra if rb is None else rb
    vx, vy = bx - ax, by - ay
    ll = vx * vx + vy * vy or 1e-9

    def d(x, y):
        t = clamp01(((x - ax) * vx + (y - ay) * vy) / ll)
        return math.hypot(x - ax - vx * t, y - ay - vy * t) - (ra + (rb - ra) * t)
    r = max(ra, rb)
    return d, (min(ax, bx) - r, min(ay, by) - r, max(ax, bx) + r, max(ay, by) + r)


def rbox(cx, cy, hw, hh, r=0.0, angle=0.0):
    c, s = math.cos(angle), math.sin(angle)

    def d(x, y):
        dx, dy = x - cx, y - cy
        u, v = abs(dx * c + dy * s) - (hw - r), abs(-dx * s + dy * c) - (hh - r)
        return math.hypot(max(u, 0.0), max(v, 0.0)) + min(max(u, v), 0.0) - r
    e = math.hypot(hw, hh)
    return d, (cx - e, cy - e, cx + e, cy + e)


def poly(points):
    n = len(points)

    def d(x, y):
        best = float('inf')
        sign = 1.0
        for i in range(n):
            ax, ay = points[i]
            bx, by = points[i - 1]
            ex, ey = bx - ax, by - ay
            wx, wy = x - ax, y - ay
            t = clamp01((wx * ex + wy * ey) / (ex * ex + ey * ey or 1e-9))
            best = min(best, math.hypot(wx - ex * t, wy - ey * t))
            c1, c2, c3 = y >= ay, y < by, ex * wy > ey * wx
            if (c1 and c2 and c3) or (not c1 and not c2 and not c3):
                sign = -sign
        return sign * best
    xs, ys = [p[0] for p in points], [p[1] for p in points]
    return d, (min(xs), min(ys), max(xs), max(ys))


def ring(cx, cy, r, half):
    return (lambda x, y: abs(math.hypot(x - cx, y - cy) - r) - half), (cx - r - half, cy - r - half, cx + r + half, cy + r + half)


def crescent(cx, cy, r, ox, oy, r2):
    outer, box = circle(cx, cy, r)
    inner, _ = circle(cx + ox, cy + oy, r2)
    return (lambda x, y: max(outer(x, y), -inner(x, y))), box


def star(cx, cy, outer, inner, points, rot=0.0):
    pts = []
    for i in range(points * 2):
        a = rot + math.pi * i / points
        r = outer if i % 2 == 0 else inner
        pts.append((cx + math.cos(a) * r, cy + math.sin(a) * r))
    return poly(pts)


# ---------------------------------------------------------------- Canvas
class Canvas:
    """RGBA float premultiplied; `part` vẽ viền rồi tô, nên mọi khối đều có nét bao đậm."""

    def __init__(self, w, h):
        self.w, self.h = w, h
        self.px = [[0.0, 0.0, 0.0, 0.0] for _ in range(w * h)]

    def part(self, shape, color, outline=4.0, alpha=1.0, line=OUTLINE):
        sdf, (x0, y0, x1, y1) = shape
        pad = outline + 1.0
        xa, ya = max(0, int(x0 - pad)), max(0, int(y0 - pad))
        xb, yb = min(self.w, int(x1 + pad) + 1), min(self.h, int(y1 + pad) + 1)
        fr, fg, fb = (c / 255.0 for c in color)
        lr, lg, lb = (c / 255.0 for c in line)
        for y in range(ya, yb):
            row = y * self.w
            for x in range(xa, xb):
                d = sdf(x + 0.5, y + 0.5)
                if d > outline + 0.5:
                    continue
                p = self.px[row + x]
                if outline > 0.0:
                    a = clamp01(0.5 + outline - d) * alpha
                    if a > 0.0:
                        k = 1.0 - a
                        p[0], p[1], p[2], p[3] = lr * a + p[0] * k, lg * a + p[1] * k, lb * a + p[2] * k, a + p[3] * k
                a = clamp01(0.5 - d) * alpha
                if a > 0.0:
                    k = 1.0 - a
                    p[0], p[1], p[2], p[3] = fr * a + p[0] * k, fg * a + p[1] * k, fb * a + p[2] * k, a + p[3] * k

    def gradient(self, top, bottom):
        for y in range(self.h):
            t = y / max(1, self.h - 1)
            c = [(top[i] + (bottom[i] - top[i]) * t) / 255.0 for i in range(3)]
            for x in range(self.w):
                self.px[y * self.w + x] = [c[0], c[1], c[2], 1.0]

    def rgba(self):
        """Straight-alpha RGBA8; texel trong suốt lấy màu viền để lọc tuyến tính không bị viền sáng."""
        out = bytearray(self.w * self.h * 4)
        for i, (r, g, b, a) in enumerate(self.px):
            if a <= 1e-6:
                out[i * 4:i * 4 + 4] = bytes((*OUTLINE, 0))
                continue
            out[i * 4:i * 4 + 4] = bytes((round(clamp01(r / a) * 255), round(clamp01(g / a) * 255),
                                           round(clamp01(b / a) * 255), round(clamp01(a) * 255)))
        return out


# ---------------------------------------------------------------- Long Lưu
LL = {'robe': hexc('#3b82d6'), 'robe_dark': hexc('#24548f'), 'skin': hexc('#f2c9a0'),
      'skin_dark': hexc('#c99a74'), 'hair': hexc('#2b2230'), 'sash': hexc('#f5f0e1'),
      'pants': hexc('#2f3a56'), 'pants_dark': hexc('#1f273b'), 'staff': hexc('#8a5a35'),
      'water': hexc('#6fe3ff'), 'eye': hexc('#1b1e2b')}
LL_CELL = (304, 256)
LL_PIVOT = (144, 244)


def limb(px, py, angle, length):
    """Góc tính từ hướng thẳng xuống, dương là về phía trước (+x); trục y của ảnh hướng xuống."""
    return px + math.sin(angle) * length, py + math.cos(angle) * length


def draw_long_luu(pose):
    c = Canvas(*LL_CELL)
    fx, fy = LL_PIVOT
    r = math.radians
    # Đế giày (tâm bàn chân + bán kính) chạm đúng dòng pivot khi chân thẳng.
    hip = (fx + pose.get('dx', 0), fy - 69 + pose.get('bob', 0) - pose.get('lift', 0))
    lean = r(pose.get('lean', 0))
    neck = (hip[0] + math.sin(lean) * 52, hip[1] - math.cos(lean) * 52)

    def leg(thigh, shin, color, dark):
        knee = limb(*hip, r(thigh), 32)
        foot = limb(*knee, r(thigh + shin), 30)
        c.part(capsule(*hip, *knee, 10, 9), color)
        c.part(capsule(*knee, *foot, 9, 8), color)
        toe = (foot[0] + 11, foot[1] + 1)
        c.part(capsule(foot[0] - 3, foot[1], *toe, 6), dark)

    def arm(upper, fore, color):
        shoulder = (neck[0] - math.sin(lean) * 6, neck[1] + 8)
        elbow = limb(*shoulder, r(upper), 25)
        hand = limb(*elbow, r(upper + fore), 23)
        c.part(capsule(*shoulder, *elbow, 8, 7), color)
        c.part(capsule(*elbow, *hand, 7, 6), color)
        return hand

    # Lớp sau: chân, tay sau.
    leg(pose['back_thigh'], pose['back_shin'], LL['pants_dark'], LL['pants_dark'])
    arm(pose['back_arm'], pose['back_fore'], LL['robe_dark'])
    # Thân áo: hình thang từ vai tới vạt dưới hông.
    sx, sy = math.sin(lean), math.cos(lean)
    hem = (hip[0] - sx * 12, hip[1] + sy * 12)
    shoulders = (neck[0] - sx * 4, neck[1] + sy * 4)
    nx, ny = sy, sx  # pháp tuyến ngang của thân
    torso = poly([(shoulders[0] - nx * 17, shoulders[1] + ny * 17), (shoulders[0] + nx * 17, shoulders[1] - ny * 17),
                  (hem[0] + nx * 24, hem[1] - ny * 24), (hem[0] - nx * 24, hem[1] + ny * 24)])
    leg(pose['front_thigh'], pose['front_shin'], LL['pants'], LL['pants_dark'])
    c.part(torso, LL['robe'])
    belt = (hip[0] + sx * 4, hip[1] - sy * 4)
    c.part(rbox(*belt, 21, 5, 2, lean), LL['sash'], outline=3)
    tail = math.radians(pose.get('sash_tail', 35))
    c.part(capsule(belt[0] - 18, belt[1], belt[0] - 18 - math.cos(tail) * 22, belt[1] + math.sin(tail) * 22, 4, 2),
           LL['sash'], outline=3)
    # Đầu, tóc búi, mắt hướng phải.
    head = (neck[0] + sx * 20, neck[1] - 22)
    c.part(circle(head[0] - 14, head[1] - 16, 10), LL['hair'])
    c.part(circle(*head, 22), LL['skin'])
    c.part(poly([(head[0] - 23, head[1] + 2), (head[0] - 22, head[1] - 14), (head[0] - 8, head[1] - 24),
                 (head[0] + 14, head[1] - 22), (head[0] + 22, head[1] - 8), (head[0] + 4, head[1] - 12),
                 (head[0] - 10, head[1] - 4)]), LL['hair'], outline=0)
    c.part(rbox(head[0] - 1, head[1] - 9, 22, 3, 2), LL['water'], outline=2)
    c.part(ellipse(head[0] + 11, head[1] + 1, 3.2, 5.0), LL['eye'], outline=0)
    # Gậy và tay trước.
    hand = arm(pose['front_arm'], pose['front_fore'], LL['robe'])
    sa = r(pose['staff'])
    dx, dy = math.sin(sa), -math.cos(sa)
    tip = (hand[0] + dx * 70, hand[1] + dy * 70)
    butt = (hand[0] - dx * 46, hand[1] - dy * 46)
    c.part(capsule(*butt, *tip, 4.5), LL['staff'], outline=3)
    c.part(ellipse(tip[0] - dx * 6, tip[1] - dy * 6, 9, 6, sa + math.pi / 2), LL['water'], outline=3)
    c.part(circle(*hand, 7), LL['skin'], outline=3)
    return c


LL_POSES = {
    'idle': [dict(bob=b, lean=2, back_thigh=-8, back_shin=4, front_thigh=10, front_shin=-6, back_arm=-12,
                  back_fore=-20, front_arm=18, front_fore=-55 + b * 2, staff=20 + b, sash_tail=30 + b * 4)
             for b in (0, 1, 2, 1)],
    'run': [dict(bob=b, lean=12, back_thigh=bt, back_shin=bs, front_thigh=ft, front_shin=fs, back_arm=-ba,
                 back_fore=-40, front_arm=ba, front_fore=-70, staff=70, sash_tail=10 + i * 4)
            for i, (b, bt, bs, ft, fs, ba) in enumerate([
                (0, -30, -20, 34, -10, 30), (-3, -14, -60, 22, -4, 18), (-1, 6, -40, 2, -30, -4),
                (0, 30, -10, -30, -20, -30), (-3, 18, -6, -14, -60, -18), (-1, 0, -30, 6, -40, 4)])],
    'jump': [
        dict(lift=6, lean=6, back_thigh=10, back_shin=-30, front_thigh=40, front_shin=-70, back_arm=-40,
             back_fore=-30, front_arm=60, front_fore=-60, staff=45, sash_tail=-20),
        dict(lift=10, lean=0, back_thigh=30, back_shin=-90, front_thigh=60, front_shin=-100, back_arm=-70,
             back_fore=-20, front_arm=80, front_fore=-40, staff=80, sash_tail=0),
        dict(lift=4, lean=-4, back_thigh=-12, back_shin=-10, front_thigh=24, front_shin=-20, back_arm=-90,
             back_fore=10, front_arm=100, front_fore=-10, staff=110, sash_tail=60),
    ],
    'attack': [
        dict(bob=2, dx=-6, lean=-6, back_thigh=-24, back_shin=10, front_thigh=20, front_shin=-12, back_arm=-30,
             back_fore=-30, front_arm=-110, front_fore=-20, staff=-60, sash_tail=20),
        dict(bob=3, dx=-8, lean=-10, back_thigh=-28, back_shin=12, front_thigh=24, front_shin=-16, back_arm=-20,
             back_fore=-40, front_arm=-150, front_fore=-10, staff=-30, sash_tail=10),
        dict(bob=4, dx=8, lean=18, back_thigh=-40, back_shin=10, front_thigh=40, front_shin=-30, back_arm=-50,
             back_fore=-10, front_arm=88, front_fore=-4, staff=90, sash_tail=60),
        dict(bob=4, dx=8, lean=16, back_thigh=-38, back_shin=8, front_thigh=38, front_shin=-28, back_arm=-40,
             back_fore=-20, front_arm=60, front_fore=10, staff=130, sash_tail=50),
        dict(bob=2, dx=2, lean=6, back_thigh=-16, back_shin=4, front_thigh=18, front_shin=-10, back_arm=-20,
             back_fore=-20, front_arm=30, front_fore=-40, staff=60, sash_tail=35),
    ],
}
LL_CLIPS = {'idle': (180, True, []), 'run': (90, True, []), 'jump': (120, False, [(1, 'apex')]),
            'attack': (80, False, [(2, 'hit')])}


# ---------------------------------------------------------------- Kẻ Giữ Đập
KG = {'stone': hexc('#8b6b4f'), 'stone_dark': hexc('#5e4836'), 'moss': hexc('#5f8f3e'),
      'plank': hexc('#a0784a'), 'band': hexc('#3a3f4a'), 'eye': hexc('#7ff3ff'), 'water': hexc('#4fb7e8')}
KG_CELL = (400, 384)
KG_PIVOT = (200, 368)


def draw_boss(pose):
    c = Canvas(*KG_CELL)
    fx, fy = KG_PIVOT
    bob = pose.get('bob', 0)
    hip = (fx + pose.get('dx', 0), fy - 90 + bob)
    r = math.radians

    def leg(offset, angle, color):
        top = (hip[0] + offset, hip[1])
        foot = limb(*top, r(angle), 64)
        c.part(capsule(*top, *foot, 26, 20), color)
        # Đáy bàn chân (tâm + 12) nằm trên dòng pivot.
        c.part(rbox(foot[0] + 6, foot[1] + 14, 32, 12, 8), color)

    def arm(shoulder, upper, fore, color, fist_color):
        elbow = limb(*shoulder, r(upper), 62)
        fist = limb(*elbow, r(upper + fore), 58)
        c.part(capsule(*shoulder, *elbow, 26, 24), color)
        c.part(capsule(*elbow, *fist, 24, 22), color)
        c.part(rbox(*fist, 30, 26, 10, r(upper + fore)), fist_color)
        return fist

    lean = r(pose.get('lean', 0))
    chest = (hip[0] + math.sin(lean) * 90, hip[1] - math.cos(lean) * 90)
    back_shoulder = (chest[0] - 64, chest[1] - 30)
    front_shoulder = (chest[0] + 58, chest[1] - 28)
    arm(back_shoulder, pose['back_arm'], pose['back_fore'], KG['stone_dark'], KG['stone_dark'])
    leg(-34, pose.get('back_leg', 0), KG['stone_dark'])
    leg(36, pose.get('front_leg', 0), KG['stone'])
    c.part(rbox(chest[0], chest[1], 96, 92, 34, lean), KG['stone'])
    # Tấm ván đập trước ngực, hai đai sắt.
    c.part(rbox(chest[0] + 6, chest[1] + 18, 60, 54, 10, lean), KG['plank'])
    for k in (-22, 30):
        c.part(rbox(chest[0] + 6 - math.sin(lean) * k, chest[1] + 18 + k, 62, 6, 2, lean), KG['band'], outline=2)
    # Đầu nhỏ lún giữa hai vai, mắt khe phát sáng.
    head = (chest[0] + 18 + math.sin(lean) * 70, chest[1] - 86)
    c.part(rbox(*head, 40, 30, 14, lean), KG['stone'])
    c.part(rbox(head[0] + 10, head[1] - 2, 24, 5, 3, lean), KG['eye'], outline=3, line=KG['band'])
    c.part(ellipse(chest[0] - 50, chest[1] - 70, 34, 14, -0.3), KG['moss'], outline=3)
    fist = arm(front_shoulder, pose['front_arm'], pose['front_fore'], KG['stone'], KG['stone'])
    c.part(ellipse(front_shoulder[0], front_shoulder[1] - 4, 30, 24), KG['stone'])
    c.part(ellipse(front_shoulder[0] - 6, front_shoulder[1] - 22, 22, 8, 0.2), KG['moss'], outline=3)
    if pose.get('splash'):
        for i, (ox, oy, rr) in enumerate([(-34, -6, 12), (0, -16, 9), (30, -4, 11)]):
            c.part(circle(fist[0] + ox, fist[1] + 24 + oy - i, rr), KG['water'], outline=3)
    return c


KG_POSES = {
    'idle': [dict(bob=b, back_arm=-14, back_fore=-10, front_arm=18 - b, front_fore=-12) for b in (0, 2, 4, 2)],
    'run': [dict(bob=b, lean=6, back_leg=bl, front_leg=fl, back_arm=-ba, back_fore=-20, front_arm=ba, front_fore=-20)
            for b, bl, fl, ba in [(0, -16, 16, 20), (4, 0, 0, 4), (0, 16, -16, -16), (4, 0, 0, 4)]],
    'attack': [
        dict(bob=4, lean=-8, back_arm=-28, back_fore=-10, front_arm=150, front_fore=10),
        dict(bob=6, lean=-12, back_arm=-32, back_fore=-14, front_arm=168, front_fore=4),
        dict(bob=10, lean=16, dx=6, back_arm=-24, back_fore=-14, front_arm=40, front_fore=-40, splash=True),
        dict(bob=10, lean=12, dx=6, back_arm=-20, back_fore=-14, front_arm=34, front_fore=-34, splash=True),
        dict(bob=4, lean=6, back_arm=-16, back_fore=-12, front_arm=26, front_fore=-14),
    ],
}
KG_CLIPS = {'idle': (240, True, []), 'run': (200, True, []), 'attack': (120, False, [(2, 'hit')])}


# ---------------------------------------------------------------- VFX
VFX_CELL = (96, 96)


def draw_vfx(kind, i):
    c = Canvas(*VFX_CELL)
    m = 48
    if kind == 'hit':
        s = [0.45, 0.8, 1.0, 0.9][i]
        fade = [1.0, 1.0, 0.8, 0.45][i]
        c.part(star(m, m, 38 * s, 14 * s, 6, 0.3 * i), hexc('#ffd84a'), alpha=fade)
        c.part(star(m, m, 20 * s, 8 * s, 6, 0.3 * i + 0.5), hexc('#fffaf0'), outline=0, alpha=fade)
    elif kind == 'guard':
        s = [0.5, 0.85, 1.05][i]
        fade = [1.0, 0.85, 0.5][i]
        c.part(ring(m, m, 32 * s, 5), hexc('#87cfeb'), outline=3, alpha=fade)
        c.part(ring(m, m, 18 * s, 3), hexc('#e8f8ff'), outline=0, alpha=fade)
    elif kind == 'slash':
        rot = [-0.5, 0.0, 0.45][i]
        fade = [0.9, 1.0, 0.55][i]
        ox, oy = math.cos(rot) * -14, math.sin(rot) * -14
        c.part(crescent(m, m, 40, ox, oy, 36), hexc('#6fe3ff'), outline=3, alpha=fade)
    elif kind == 'dust':
        s = [0.5, 0.8, 1.0, 1.1][i]
        fade = [1.0, 0.9, 0.6, 0.35][i]
        for ox, oy, rr in ((-22, 20, 14), (0, 14, 18), (22, 20, 14)):
            c.part(circle(m + ox * s, m + oy, rr * s), hexc('#d8c8a8'), outline=3, alpha=fade)
    elif kind == 'bolt':
        wob = [0, 3][i]
        c.part(ellipse(m + 6, m, 30, 13 - wob / 2), hexc('#4fb7e8'))
        c.part(poly([(m - 40, m - 10 + wob), (m - 8, m - 4), (m - 8, m + 4), (m - 40, m + 10 - wob)]), hexc('#6fe3ff'))
        c.part(ellipse(m + 12, m - 3, 12, 5), hexc('#e8fbff'), outline=0)
    return c


VFX_CLIPS = {'hit': (4, 60, False), 'guard': (3, 70, False), 'slash': (3, 60, False),
             'dust': (4, 80, False), 'bolt': (2, 90, True)}


# ---------------------------------------------------------------- Bản đồ Đập Cổ
TILE = 128


def draw_tile(kind):
    c = Canvas(TILE, TILE)
    stone, dark, moss = hexc('#3b3d47'), hexc('#33353e'), hexc('#4f7a3a')
    # Gạch kín mép để lát liền: không viền ngoài ở biên ô.
    for row in range(4):
        y = row * 32 + 16
        offset = 0 if row % 2 == 0 else 32
        for col in range(-1, 3):
            x = col * 64 + offset + 32
            c.part(rbox(x, y, 30, 14, 4), stone if (row + col) % 3 else dark, outline=2, line=hexc('#2a2c34'))
    if kind == 'top':
        c.part(rbox(64, 6, 70, 10, 0), moss, outline=0)
        for x in range(4, TILE, 16):
            c.part(poly([(x, 14), (x + 8, 14), (x + 4, 24 + (x * 7) % 9)]), moss, outline=0)
    return c


def draw_backdrop(kind):
    if kind == 'sky':
        c = Canvas(64, 256)
        c.gradient(hexc('#1f2a38'), hexc('#3a4d5c'))
        return c
    if kind == 'hills':
        c = Canvas(512, 192)
        # Mỗi đồi vẽ thêm bản dịch ±512 để hai mép ô lặp khớp nhau.
        for x, w, h, col in [(60, 180, 120, '#33495a'), (260, 220, 150, '#2d4252'), (452, 200, 100, '#2d4252')]:
            for shift in (-512, 0, 512):
                c.part(ellipse(x + shift, 192, w, h), hexc(col), outline=0)
        return c
    if kind == 'dam':
        c = Canvas(512, 256)
        wall, dark, water = hexc('#3f4149'), hexc('#2f3138'), hexc('#3c7391')
        c.part(rbox(256, 160, 270, 96, 0), wall, outline=0)
        for x in range(0, 512, 128):
            c.part(rbox(x + 64, 72, 34, 22, 6), dark, outline=3, line=hexc('#24262c'))
            c.part(rbox(x + 64, 170, 20, 88, 4), water, outline=0, alpha=0.85)
            c.part(rbox(x + 58, 160, 4, 70, 2), hexc('#7fb3cc'), outline=0, alpha=0.6)
        c.part(rbox(256, 66, 270, 6, 0), dark, outline=0)
        return c
    c = Canvas(256, 96)  # sậy gần, phía sau người chơi
    for i, x in enumerate(range(6, 256, 22)):
        h = 50 + (i * 37) % 40
        c.part(capsule(x, 96, x + 4, 96 - h, 3, 2), hexc('#34503a'), outline=0)
        c.part(ellipse(x + 4, 96 - h, 4, 10), hexc('#4f4333'), outline=0)
    return c


# ---------------------------------------------------------------- Đóng gói
def pack(frames, padding, extrude, max_width=2048):
    """Shelf packer theo thứ tự cho trước: mỗi clip một hàng nếu vừa; trả rect và kích thước atlas."""
    x, y, row_h, width = padding, padding, 0, 0
    rects = []
    current_row = None
    for name, canvas, row in frames:
        w, h = canvas.w, canvas.h
        if (row != current_row and x > padding) or x + w + padding > max_width:
            x, y, row_h = padding, y + row_h + padding, 0
        current_row = row
        rects.append((name, canvas, x, y))
        x += w + padding
        row_h = max(row_h, h)
        width = max(width, x)
    return rects, width, y + row_h + padding


def blit(atlas, aw, rects, extrude):
    for _, canvas, ox, oy in rects:
        data = canvas.rgba()
        w, h = canvas.w, canvas.h
        for yy in range(-extrude, h + extrude):
            sy = min(max(yy, 0), h - 1)
            for xx in range(-extrude, w + extrude):
                sx = min(max(xx, 0), w - 1)
                s = (sy * w + sx) * 4
                d = ((oy + yy) * aw + ox + xx) * 4
                atlas[d:d + 4] = data[s:s + 4]


def png_bytes(w, h, rgba):
    raw = bytearray()
    stride = w * 4
    for y in range(h):
        row = rgba[y * stride:(y + 1) * stride]
        raw.append(1)  # bộ lọc Sub
        out = bytearray(stride)
        for i in range(stride):
            left = row[i - 4] if i >= 4 else 0
            out[i] = (row[i] - left) & 0xFF
        raw += out

    def chunk(tag, data):
        return struct.pack('>I', len(data)) + tag + data + struct.pack('>I', zlib.crc32(tag + data) & 0xFFFFFFFF)
    return (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', w, h, 8, 6, 0, 0, 0))
            + chunk(b'IDAT', zlib.compress(bytes(raw), 9)) + chunk(b'IEND', b''))


def png_pixels(data):
    """Giải mã PNG RGBA8 không interlace (đủ cho file do script này ghi) để so pixel khi --check."""
    pos, idat, w, h = 8, b'', 0, 0
    while pos < len(data):
        n = struct.unpack('>I', data[pos:pos + 4])[0]
        tag, body = data[pos + 4:pos + 8], data[pos + 8:pos + 8 + n]
        if tag == b'IHDR':
            w, h = struct.unpack('>II', body[:8])
        elif tag == b'IDAT':
            idat += body
        pos += 12 + n
    raw = zlib.decompress(idat)
    stride = w * 4
    out = bytearray(w * h * 4)
    prev = bytearray(stride)
    for y in range(h):
        f = raw[y * (stride + 1)]
        line = bytearray(raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)])
        for i in range(stride):
            a = line[i - 4] if i >= 4 else 0
            b = prev[i]
            c = prev[i - 4] if i >= 4 else 0
            if f == 1:
                line[i] = (line[i] + a) & 0xFF
            elif f == 2:
                line[i] = (line[i] + b) & 0xFF
            elif f == 3:
                line[i] = (line[i] + (a + b) // 2) & 0xFF
            elif f == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                line[i] = (line[i] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 0xFF
        out[y * stride:(y + 1) * stride] = line
        prev = line
    return w, h, out


def sheet(sheet_id, texture_id, subject, role, plane, tpu, padding, extrude, frames, clips, pivots, path):
    rects, aw, ah = pack(frames, padding, extrude)
    atlas = bytearray(aw * ah * 4)
    for i in range(0, len(atlas), 4):
        atlas[i:i + 4] = bytes((*OUTLINE, 0))
    blit(atlas, aw, rects, extrude)
    doc = {
        'format': 'myva.sprite-sheet', 'format_version': 1, 'id': sheet_id, 'texture': texture_id,
        'role': role, 'plane': plane, 'texels_per_unit': tpu, 'padding': padding, 'extrude': extrude,
        'frames': [{'name': name, 'rect': [x, y, canvas.w, canvas.h], 'pivot': list(pivots(name, canvas))}
                   for name, canvas, x, y in rects],
        'clips': clips,
    }
    if subject:
        doc['subject'] = subject
    return {'texture': (path + '/atlas.png', aw, ah, atlas), 'doc': doc}


def clip(name, count, frame_ms, loop, events=()):
    return {'name': name, 'frames': [f'{name}.{i}' for i in range(count)], 'frame_ms': frame_ms, 'loop': loop,
            'events': [{'frame': f, 'name': n} for f, n in events]}


def build():
    sheets = []
    frames = [(f'{n}.{i}', draw_long_luu(p), n) for n, poses in LL_POSES.items() for i, p in enumerate(poses)]
    clips = [clip(n, len(LL_POSES[n]), ms, loop, ev) for n, (ms, loop, ev) in LL_CLIPS.items()]
    sheets.append(('character.long_luu', 'long_luu', sheet('long_luu.sheet', 'long_luu.atlas', 'long-luu', 'character',
                   'gameplay', GAMEPLAY_TPU, 4, 0, frames, clips, lambda *_: LL_PIVOT, 'long_luu')))
    frames = [(f'{n}.{i}', draw_boss(p), n) for n, poses in KG_POSES.items() for i, p in enumerate(poses)]
    clips = [clip(n, len(KG_POSES[n]), ms, loop, ev) for n, (ms, loop, ev) in KG_CLIPS.items()]
    sheets.append(('boss.ke_giu_dap', 'ke_giu_dap', sheet('ke_giu_dap.sheet', 'ke_giu_dap.atlas', 'ke-giu-dap', 'boss',
                   'gameplay', GAMEPLAY_TPU, 4, 0, frames, clips, lambda *_: KG_PIVOT, 'ke_giu_dap')))
    frames = [(f'{n}.{i}', draw_vfx(n, i), n) for n, (count, _, _) in VFX_CLIPS.items() for i in range(count)]
    clips = [clip(n, count, ms, loop) for n, (count, ms, loop) in VFX_CLIPS.items()]
    sheets.append(('vfx.combat', 'vfx', sheet('vfx.combat', 'vfx.combat_atlas', None, 'vfx', 'gameplay', GAMEPLAY_TPU,
                   4, 0, frames, clips, lambda _, c: (c.w // 2, c.h // 2), 'vfx')))
    frames = [('ground.top', draw_tile('top'), 0), ('ground.fill', draw_tile('fill'), 0)]
    sheets.append(('map.dap_co', 'dap_co/ground', sheet('dap_co.ground', 'dap_co.ground_atlas', None, 'tileset',
                   'gameplay', GAMEPLAY_TPU, 4, 2, frames, [], lambda *_: (0, 0), 'dap_co/ground')))
    frames = [(n, draw_backdrop(n), i) for i, n in enumerate(['sky', 'hills', 'dam', 'reeds'])]
    sheets.append(('map.dap_co', 'dap_co/backdrop', sheet('dap_co.backdrop', 'dap_co.backdrop_atlas', None, 'backdrop',
                   'backdrop', BACKDROP_TPU, 4, 2, frames, [], lambda _, c: (0, c.h), 'dap_co/backdrop')))
    layout = {
        'format': 'myva.map-layout', 'format_version': 1, 'id': 'dap_co.layout', 'name': 'Đập Cổ — đấu trường',
        'ground': {'sheet': 'dap_co.ground', 'top': 'ground.top', 'fill': 'ground.fill', 'surface_y': 0, 'depth': 384},
        'layers': [
            {'sheet': 'dap_co.backdrop', 'frame': 'sky', 'z': -40, 'y': 0, 'stretch': [1800, 560]},
            {'sheet': 'dap_co.backdrop', 'frame': 'hills', 'z': -30, 'y': 150, 'repeat': True},
            {'sheet': 'dap_co.backdrop', 'frame': 'dam', 'z': -20, 'y': 0, 'repeat': True},
            {'sheet': 'dap_co.backdrop', 'frame': 'reeds', 'z': -10, 'y': 0, 'repeat': True},
        ],
    }
    return sheets, layout


def write_json(path, doc):
    data = (json.dumps(doc, ensure_ascii=False, indent=2) + '\n').encode()
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return data


def entry(asset_id, kind, rel, data, deps, extra=None):
    item = {'id': asset_id, 'kind': kind, 'path': rel, 'version': 1, 'bytes': len(data),
            'sha256': hashlib.sha256(data).hexdigest(), 'dependencies': deps, 'provenance': PROVENANCE}
    if extra:
        item.update(extra)
    return item


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument('--check', action='store_true', help='vẽ lại và so pixel với file đã commit')
    args = parser.parse_args()
    sheets, layout = build()
    if args.check:
        bad = 0
        for _, _, s in sheets:
            rel, w, h, pixels = s['texture']
            cw, ch, committed = png_pixels((OUT / rel).read_bytes())
            diff = max((abs(a - b) for a, b in zip(pixels, committed)), default=0) if (cw, ch) == (w, h) else 256
            ok = (cw, ch) == (w, h) and diff <= 2
            print(f"{'PASS' if ok else 'FAIL'} {rel}: {w}x{h}, chênh kênh lớn nhất {diff}")
            bad += not ok
            sheet_doc = json.loads((OUT / rel.replace('atlas.png', 'sheet.json')).read_text())
            if sheet_doc != s['doc']:
                print(f'FAIL {rel}: metadata sheet khác bản sinh lại')
                bad += 1
        if json.loads((OUT / 'dap_co/layout.json').read_text()) != layout:
            print('FAIL dap_co/layout.json: khác bản sinh lại')
            bad += 1
        sys.exit(1 if bad else 0)

    assets, bundles = [], {}
    for bundle, path, s in sheets:
        rel, w, h, pixels = s['texture']
        data = png_bytes(w, h, bytes(pixels))
        (OUT / rel).parent.mkdir(parents=True, exist_ok=True)
        (OUT / rel).write_bytes(data)
        doc = s['doc']
        assets.append(entry(doc['texture'], 'texture', rel, data, [],
                            {'texture': {'width': w, 'height': h, 'format': 'rgba8_srgb'}}))
        sheet_rel = path + '/sheet.json'
        assets.append(entry(doc['id'], 'sprite_sheet', sheet_rel, write_json(OUT / sheet_rel, doc), [doc['texture']]))
        bundles.setdefault(bundle, []).extend([doc['id'], doc['texture']])
    data = write_json(OUT / 'dap_co/layout.json', layout)
    assets.append(entry(layout['id'], 'map_layout', 'dap_co/layout.json', data, ['dap_co.ground', 'dap_co.backdrop']))
    bundles['map.dap_co'].append(layout['id'])
    kinds = {'character': 'character', 'boss': 'boss', 'map': 'map', 'vfx': 'vfx'}
    manifest = {
        'format': 'myva.asset-manifest', 'format_version': 1, 'content_version': CONTENT_VERSION,
        'texels_per_unit': {'gameplay': GAMEPLAY_TPU, 'backdrop': BACKDROP_TPU},
        'limits': LIMITS,
        'assets': assets,
        'bundles': [{'id': b, 'kind': kinds[b.split('.')[0]], 'assets': ids, 'dependencies': [],
                     'budget': BUDGETS[b]} for b, ids in bundles.items()],
    }
    write_json(OUT / 'manifest.json', manifest)
    for item in assets:
        print(f"{item['id']:<24} {item['bytes']:>8} B  {item['path']}")
    print(f'Manifest: {OUT / "manifest.json"}')


if __name__ == '__main__':
    main()
