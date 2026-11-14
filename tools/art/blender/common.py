"""Knellroot 초안 3D 모델 공용 도구(bpy). 모델 스크립트(wanderer.py 등)가 이것만 불러 쓴다.

꼴(게임 로더 `src/model.rs`가 읽는 것에 맞춤):
- 상자(직육면체·위가 좁은 상자·기운 상자)와 둥근 덩어리(`blob`, 0036), 다듬은 꼴(`stone`·`lathe`·`limb`, 0139)로 부위를 만든다.
  부위 = 메시 오브젝트 하나 = glTF 노드 하나.
- 뼈(아마추어)에 부위를 parent_type='BONE'으로 붙인다 — 강체. 정점 가중치·스키닝 없음.
- 텍스처는 파일마다 하나: 128×128 아틀라스(코드로 찍음, 가장 가까운 픽셀). 모든 부위가 재질 하나를 같이 쓴다.
- 예외(0038): 바깥 메시(Meshy로 만든 몸)를 `Model.external`로 부위마다 잘라 같은 식으로 뼈에 붙이고,
  그 텍스처와 아틀라스를 `merge_textures`가 한 장으로 합친다(로더는 이미지 하나만 읽는다).
- 동작(Action)마다 glTF 애니메이션 하나(이름 그대로). 100fps로 찍어 0.43초 같은 길이가 정확히 프레임에 맞는다.

좌표: 단위 m, 블렌더 Z 위, 발·바닥 = z 0. 캐릭터 앞 = 블렌더 -Y(내보내면 glTF +Z).
캐릭터의 오른쪽 = 블렌더 -X(앞이 -Y이고 위가 +Z이므로). arm_r·leg_r은 x < 0 쪽에 있다.

자세 각도는 "월드 축" 오일러(도, XYZ 순 = X 먼저)로 적는다. 부모 뼈가 돌면 자식은 그 돌아간 틀에서 돈다.
- X+ : 위로 선 것(몸통)은 앞으로 숙고, 아래로 늘어진 것(팔·다리)은 뒤로 간다. 팔을 앞으로 들기 = X−.
       앞으로 뻗은 것(머리·칼날)은 아래로 숙는다.
- Y+ : 위로 선 것은 +X(캐릭터 왼쪽)로 기운다. 늘어진 팔을 옆으로 벌리기 = 왼팔 Y−, 오른팔 Y+.
- Z+ : 위에서 보아 반시계 = 캐릭터가 왼쪽으로 돈다.
"""
import math
import os
import random
import sys

import bmesh
import bpy
from mathutils import Euler, Matrix, Quaternion, Vector

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
MODELS_DIR = os.path.join(REPO, "assets", "models")
PREVIEW_DIR = os.path.join(REPO, "docs", "art", "previews")

FPS = 100
# 동작 곡선을 몇 프레임마다 직선 키로 다시 박나(3 = 약 33Hz, + 키 시각·끝 프레임).
FRAME_STEP = 3

# ---------------------------------------------------------------- 팔레트
# 붉은색은 위험 예고에만 쓴다 — 여기에는 채도 높은 빨강이 없다.
PAL = {
    # 0037 "색 = 생명": 세계는 해 질 녘(숯빛 검정 대신 짙은 남보라), 침식은 바랜 잿빛 + 보라 뿌리, 불빛만 따뜻하게.
    "charcoal": "#262739",
    "slate": "#34364f",
    "stone": "#5f5c7a",
    "lstone": "#8f8ba3",
    "brown": "#7a6260",
    "bone": "#b9b3c4",
    "moss": "#3f6d6a",
    "root": "#6b4a7e",
    "droot": "#4a3340",
    "amber": "#e09a3a",
    "ember": "#f2c46d",
    "teal": "#3f6b6a",
    "steel": "#a7adb0",
}


def hexrgb(h):
    h = h.lstrip("#")
    return tuple(int(h[i : i + 2], 16) / 255.0 for i in (0, 2, 4))


def mix(a, b, t):
    return tuple(x + (y - x) * t for x, y in zip(a, b))


# ---------------------------------------------------------------- 아틀라스
# 128×128(0036에서 64 → 128). 위(y 0..31, 위에서부터) = 8px 칸 16×4 — 재질마다 옆면 칸 + 윗면 칸(조금 밝게), 한 줄에 열여섯.
# 그 아래(y 32..63) = 16px 큰 칸 4×2 — 넓은 면(벽·문)용 재질 넷, 옆면 + 윗면. 그 오른쪽과 아래 절반(y 64..)은 8px 칸이 이어진다(0139 `k_*`).
ATLAS = 128
# (이름, 색, 무늬). 무늬: noise·cloth·leather·speck·streak·glow·metal·flat·grain
SMALL = [
    ("charcoal", PAL["charcoal"], "noise"),
    ("slate", PAL["slate"], "cloth"),
    ("stone", PAL["stone"], "noise"),
    ("lstone", PAL["lstone"], "noise"),
    ("brown", PAL["brown"], "leather"),
    ("bone", PAL["bone"], "noise"),
    ("moss", PAL["moss"], "speck"),
    ("root", PAL["root"], "streak"),
    ("droot", PAL["droot"], "streak"),
    ("amber", PAL["amber"], "glow"),
    ("ember", PAL["ember"], "glow"),
    ("teal", PAL["teal"], "cloth"),
    ("steel", PAL["steel"], "metal"),
    # 어두운 청동: 바랜 갈색을 숯빛 쪽으로(종).
    ("bronze", "#6a5a6e", "metal"),
    # 후드 속 그늘: 숯빛보다 한 단계 더 어둡게, 무늬 없음.
    ("shadow", "#14131d", "flat"),
    ("wood", "#5e4a50", "grain"),
    # 0036 파스텔(귀여운 쪽): 무늬·가장자리 그늘 없이 거의 단색(`soft`) — 블록 테두리가 마인크래프트처럼 보였다.
    ("p_hood", "#b9a7e0", "soft"),
    ("p_hood_in", "#8f7fbf", "soft"),
    ("p_cloak", "#a6dcc6", "soft"),
    ("p_scarf", "#f5b39b", "soft"),
    ("p_skin", "#fde4d2", "soft"),
    ("p_blush", "#f4a3b4", "soft"),
    ("p_eye", "#3a2b45", "soft"),
    ("p_white", "#fffaf3", "soft"),
    ("p_boot", "#c79a80", "soft"),
    ("p_pants", "#8f88bd", "soft"),
    ("p_belt", "#ebc47c", "soft"),
    ("p_steel", "#dde7f1", "soft"),
    ("p_gold", "#f2cf7e", "soft"),
    ("p_lamp", "#f0d9a6", "soft"),
    ("p_cream", "#f6ead2", "soft"),
    ("p_hair", "#e6b48c", "soft"),
    ("p_sky", "#a9d4f0", "soft"),
    # 0037 침식 = 색을 잃음: 살아 있던 파스텔이 바랜 잿빛 라벤더로.
    ("f_ash", "#a29eb0", "soft"),
    ("f_ash_d", "#6f6a80", "soft"),
    ("p_mask", "#86675c", "soft"),
    # 0139 다듬은 소품: 무늬·칸 테두리 없는 단색 칸(`soft`) — 돌 사이 금·모서리 선은 기하와 렌더(0061 선 둘)가 긋는다.
    # 세계 팔레트(direction.md 해 질 녘 #2c2e47 · #454463 · #5f5c7a)와 구운 지형(`scenery.rs` WALL·WALL_TOP)에 맞춘다.
    ("k_deep", "#2c2e47", "soft"),
    ("k_dark", "#454463", "soft"),
    ("k_stone", "#5f5c7a", "soft"),
    ("k_light", "#7d7997", "soft"),
    ("k_moss", "#3f6d6a", "soft"),
    ("k_root", "#6b4a7e", "soft"),
    ("k_rootl", "#8a66a3", "soft"),
    ("k_droot", "#4a3340", "soft"),
    ("k_wood", "#5a4650", "soft"),
    ("k_woodl", "#76606a", "soft"),
    ("k_bronze", "#7c6658", "soft"),
    ("k_bronzel", "#a38a6c", "soft"),
    ("k_amber", "#e09a3a", "soft"),
    ("k_ember", "#f2c46d", "soft"),
    ("k_core", "#ffe9b4", "soft"),
    ("k_bone", "#b9b3c4", "soft"),
    # 불꽃 끝(짙은 호박빛 — 빨강은 위험 예고 전용이라 쓰지 않는다).
    ("k_flame", "#cf7a2c", "soft"),
    # 상자의 나무(따뜻한 갈색 — 손댈 수 있는 물건은 푸른 달빛 아래에서도 돌과 갈려야 한다)·놋쇠 띠.
    ("k_chest", "#7c5340", "soft"),
    ("k_chestl", "#96694c", "soft"),
    ("k_brass", "#c09a5c", "soft"),
    # 주민의 생활 물건(0142): 천막 천(따뜻한 베이지 — 산 것 곁의 물건)·그 그늘.
    ("k_cloth", "#b89478", "soft"),
    ("k_clothd", "#8f6f5c", "soft"),
    # 살아 있는 작은 생명(0185 — 이끼먹이): 등의 연두 이끼·몸의 따뜻한 크림(0037 파스텔 = 생명).
    ("p_moss", "#9cc9a0", "soft"),
    ("p_mossd", "#6f9f7e", "soft"),
]
# 큰 칸: blocks(돌 쌓기)·bark(뿌리 결)·door(널빤지 + 청동 띠)
BIG = [
    ("wall", PAL["stone"], "blocks"),
    ("wall_light", PAL["lstone"], "blocks"),
    ("bark", PAL["root"], "bark"),
    ("door", "#4f4052", "door"),
]


def _cell_rects():
    """재질 이름 → {'side': (x0, y0, size), 'top': (...)}(픽셀, 위에서부터)."""
    rects = {}
    for i, (name, _, _) in enumerate(SMALL):
        if i < 32:
            cx, row = i % 16, (i // 16) * 2
        elif i < 48:
            # 서른셋째부터는 큰 칸 오른쪽 빈 자리(x 64.., y 32..)에 여덟씩.
            j = i - 32
            cx, row = 8 + j % 8, 4 + (j // 8) * 2
        else:
            # 마흔아홉째부터는 아래 절반(y 64..)에 한 줄 열여섯씩(0139).
            j = i - 48
            cx, row = j % 16, 8 + (j // 16) * 2
        rects[name] = {"side": (cx * 8, row * 8, 8), "top": (cx * 8, (row + 1) * 8, 8)}
    for i, (name, _, _) in enumerate(BIG):
        cx, row = i % 2, i // 2
        # 큰 칸 한 줄에 재질 둘: [옆 | 위 | 옆 | 위]
        x0 = cx * 32
        y0 = 32 + row * 16
        rects[name] = {"side": (x0, y0, 16), "top": (x0 + 16, y0, 16)}
    return rects


RECTS = _cell_rects()


def _paint_cell(px, x0, y0, size, base, pattern, rng, top):
    """px: [y][x] → (r,g,b). 무늬는 은은하게(밝기 ±5% 안팎), 가장자리 1px은 어둡게."""
    lift = 1.13 if top else 1.0

    def put(x, y, c, k=1.0):
        k *= lift
        c = tuple(min(1.0, v * k + (0.012 if top else 0.0)) for v in c)
        px[y0 + y][x0 + x] = c

    for y in range(size):
        for x in range(size):
            edge = x in (0, size - 1) or y in (0, size - 1)
            # 순서 있는 디더(2×2 바이어) + 결정적 잡음
            bayer = ((0, 2), (3, 1))[y % 2][x % 2] / 3.0 - 0.5
            n = rng.uniform(-1, 1)
            c = base
            k = 1.0 + 0.035 * bayer + 0.03 * n
            if pattern == "cloth":
                k += -0.05 if y % 2 else 0.02
            elif pattern == "leather":
                k += 0.05 if (x * 3 + y * 5) % 7 == 0 else 0.0
            elif pattern == "speck":
                if rng.random() < 0.12:
                    c = mix(base, hexrgb("#5f8f86"), 0.7)
            elif pattern == "streak":
                k += 0.07 * math.sin(x * 1.9 + math.sin(y * 0.7) * 1.3)
            elif pattern == "grain":
                k += 0.06 * math.sin(y * 2.3 + x * 0.3)
            elif pattern == "metal":
                if y == 1 or y == 2:
                    k += 0.18
                if (x + y) % 5 == 0:
                    k += 0.06
            elif pattern == "glow":
                # 가운데로 갈수록 엷은 불씨색, 가장자리는 그대로
                d = max(abs(x - (size - 1) / 2), abs(y - (size - 1) / 2)) / (size / 2)
                c = mix(hexrgb(PAL["ember"]), base, d)
                if base == hexrgb(PAL["ember"]):
                    c = mix((1.0, 0.93, 0.74), base, d)
                k = 1.0 + 0.02 * n
            elif pattern == "flat":
                k = 1.0 + 0.015 * n
            elif pattern == "soft":
                k = 1.0 + 0.008 * n
            elif pattern == "blocks":
                h = size // 2
                course = y // h
                yy = y % h
                joints = (size // 2,) if course == 0 else (size // 4, 3 * size // 4)
                mortar = yy == h - 1 or x in joints
                block = (course, sum(1 for j in joints if x > j))
                rb = random.Random(hash(block) & 0xFFFF)
                k += rb.uniform(-0.07, 0.07)
                if mortar:
                    k *= 0.68
                elif pattern == "blocks" and base == hexrgb(PAL["stone"]) and yy == 0 and rng.random() < 0.25:
                    c = hexrgb(PAL["moss"])
            elif pattern == "bark":
                k += 0.10 * math.sin(x * 1.2 + y * 0.45) + 0.04 * math.sin(y * 1.7)
                if rng.random() < 0.03:
                    c = hexrgb(PAL["droot"])
            elif pattern == "door":
                if x % 5 == 0:
                    k *= 0.7
                if y in (3, 4, 11, 12):
                    c = hexrgb("#6a5a6e")
                    k = 1.0 + (0.15 if y in (3, 11) else 0.0)
                    if x % 5 == 2:
                        c = hexrgb(PAL["steel"])
                        k = 0.75
            if pattern == "soft":
                pass
            elif edge and pattern != "glow":
                k *= 0.78
            elif edge:
                k *= 0.88
            put(x, y, c, k)


def atlas_pixels():
    """128×128 RGB 행렬(위에서부터). 같은 입력이면 늘 같은 그림."""
    px = [[(0.5, 0.5, 0.5)] * ATLAS for _ in range(ATLAS)]
    for name, col, pat in SMALL + BIG:
        rng = random.Random(sum(map(ord, name)) * 7919)
        x0, y0, s = RECTS[name]["side"]
        _paint_cell(px, x0, y0, s, hexrgb(col), pat, rng, top=False)
        rng = random.Random(sum(map(ord, name)) * 7919)
        x0, y0, s = RECTS[name]["top"]
        _paint_cell(px, x0, y0, s, hexrgb(col), pat, rng, top=True)
    return px


def make_material(model_name):
    px = atlas_pixels()
    img = bpy.data.images.new(model_name + "_atlas", ATLAS, ATLAS, alpha=False)
    flat = []
    for y in range(ATLAS - 1, -1, -1):  # 블렌더 이미지는 아래 줄부터
        for x in range(ATLAS):
            r, g, b = px[y][x]
            flat += (r, g, b, 1.0)
    img.pixels.foreach_set(flat)
    img.file_format = "PNG"
    img.pack()
    mat = bpy.data.materials.new(model_name + "_mat")
    mat.use_nodes = True
    nt = mat.node_tree
    bsdf = nt.nodes["Principled BSDF"]
    bsdf.inputs["Roughness"].default_value = 1.0
    tex = nt.nodes.new("ShaderNodeTexImage")
    tex.image = img
    tex.interpolation = "Closest"
    nt.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
    return mat


def cell_uv(mat_name, face):
    """칸의 UV 사각형(블렌더 UV, v는 아래에서부터). 가장자리 0.1px 안쪽."""
    x0, y0, s = RECTS[mat_name][face]
    e = 0.1
    u0, u1 = (x0 + e) / ATLAS, (x0 + s - e) / ATLAS
    v1, v0 = 1.0 - (y0 + e) / ATLAS, 1.0 - (y0 + s - e) / ATLAS
    return u0, v0, u1, v1


def cell_mid(mat_name, face="side"):
    """칸 가운데 한 점의 UV — 면 전체를 한 색으로(무늬·칸 테두리를 안 쓴다, 0139)."""
    x0, y0, s = RECTS[mat_name][face]
    return ((x0 + s / 2) / ATLAS, 1.0 - (y0 + s / 2) / ATLAS)


# ---------------------------------------------------------------- 상자
# 모서리 번호 (ix, iy, iz) ∈ {0,1}³. 면마다 바깥에서 보아 반시계.
_FACES = {
    "+z": ((0, 0, 1), (1, 0, 1), (1, 1, 1), (0, 1, 1)),
    "-z": ((0, 0, 0), (0, 1, 0), (1, 1, 0), (1, 0, 0)),
    "+x": ((1, 0, 0), (1, 1, 0), (1, 1, 1), (1, 0, 1)),
    "-x": ((0, 0, 0), (0, 0, 1), (0, 1, 1), (0, 1, 0)),
    "+y": ((1, 1, 0), (0, 1, 0), (0, 1, 1), (1, 1, 1)),
    "-y": ((0, 0, 0), (1, 0, 0), (1, 0, 1), (0, 0, 1)),
}


class Geo:
    """부위 하나의 상자 모음 → 메시 오브젝트 하나."""

    def __init__(self, name, bone=None):
        self.name = name
        self.bone = bone
        self.quads = []  # (네 점, 네 UV, 부드럽게)

    def box(self, c, s, mat, rot=(0, 0, 0), taper=(1, 1), shift=(0, 0), drop=(), pivot=None, top=True):
        """c 가운데, s 크기(x,y,z). taper = 윗면(+z) 크기 배율, shift = 윗면을 x·y로 민다(기운 상자).
        rot = 도(XYZ, 월드 축), pivot(기본 가운데)을 돌리는 점으로. drop = 뺄 면('-z' 등: 바닥에 닿아 안 보이는 면).
        top = 윗면에 밝은 칸을 쓸지."""
        sx, sy, sz = s
        tx, ty = taper
        R = Euler([math.radians(a) for a in rot], "XYZ").to_matrix()
        cv = Vector(c)
        pv = Vector(pivot) if pivot is not None else cv

        def corner(ix, iy, iz):
            x = (ix - 0.5) * sx
            y = (iy - 0.5) * sy
            z = (iz - 0.5) * sz
            if iz:
                x = x * tx + shift[0]
                y = y * ty + shift[1]
            p = cv + Vector((x, y, z))
            return pv + R @ (p - pv)

        for key, corners in _FACES.items():
            if key in drop:
                continue
            face = "top" if (key == "+z" and top) else "side"
            u0, v0, u1, v1 = cell_uv(mat, face)
            pts = [corner(*k) for k in corners]
            uvs = [(u0, v0), (u1, v0), (u1, v1), (u0, v1)]
            self.quads.append((pts, uvs, False))
        return self

    def blob(self, c, s, mat, r=0.5, n=4, rot=(0, 0, 0), pivot=None, taper=(1, 1), drop_bottom=False):
        """둥근 덩어리(0036): 크기 s 상자의 모서리를 반지름 `r × (가장 짧은 반폭)`으로 굴린 것(r 1 ≈ 공·알약). 면마다 n×n 격자,
        정점을 이웃 면과 함께 써서 음영이 매끈하다(부드러운 법선). 색은 칸 가운데 하나(파스텔 `soft` 칸과 쓴다).
        taper = 윗면(+z) 쪽 x·y 배율(아래에서 위로 고르게), drop_bottom = 아래 면 빼기."""
        hx, hy, hz = (v * 0.5 for v in s)
        rr = min(hx, hy, hz) * max(0.0, min(1.0, r))
        inner = (hx - rr, hy - rr, hz - rr)
        R = Euler([math.radians(a) for a in rot], "XYZ").to_matrix()
        cv = Vector(c)
        pv = Vector(pivot) if pivot is not None else cv
        x0, y0, sz = RECTS[mat]["side"]
        uv = ((x0 + sz / 2) / ATLAS, 1.0 - (y0 + sz / 2) / ATLAS)

        def at(p):
            q = [max(-inner[i], min(inner[i], p[i])) for i in range(3)]
            d = Vector((p[0] - q[0], p[1] - q[1], p[2] - q[2]))
            if d.length > 1e-9:
                d = d.normalized() * rr
            v = Vector(q) + d
            k = (v.z + hz) / (2 * hz) if hz > 0 else 0.0
            v.x *= 1 + (taper[0] - 1) * k
            v.y *= 1 + (taper[1] - 1) * k
            return pv + R @ (cv + v - pv)

        # 모서리 가까이 촘촘히(사인 간격) — 둥근 곳에 점이 모이게.
        ts = [math.sin(math.pi / 2 * (2 * i / n - 1)) for i in range(n + 1)]
        half = (hx, hy, hz)
        for axis in range(3):
            for sign in (-1, 1):
                if drop_bottom and axis == 2 and sign < 0:
                    continue
                a1, a2 = [i for i in range(3) if i != axis]
                for i in range(n):
                    for j in range(n):
                        quad = []
                        for di, dj in ((0, 0), (1, 0), (1, 1), (0, 1)):
                            p = [0.0, 0.0, 0.0]
                            p[axis] = sign * half[axis]
                            p[a1] = ts[i + di] * half[a1]
                            p[a2] = ts[j + dj] * half[a2]
                            quad.append(at(p))
                        # 바깥에서 보아 반시계가 되게 — `_FACES`와 같은 감김(+x·+z·−y는 그대로, 나머지는 뒤집기).
                        if (sign > 0) == (axis == 1):
                            quad.reverse()
                        self.quads.append((quad, [uv] * 4, True))
        return self

    # --- 다듬은 꼴(0139): 깎은 돌·돌린 꼴·꺾인 관. 면은 칸 가운데 한 색(`cell_mid`), 다각형 그대로(`quads`에 3각·n각도 들어간다).
    def _group(self):
        """매끈한 면 묶음 번호(같은 묶음 + 같은 자리 = 같은 정점). True(blob)와 겹치지 않게 2부터."""
        self._grp = getattr(self, "_grp", 1) + 1
        return self._grp

    def stone(self, c, s, mat, bevel=0.06, rot=(0, 0, 0), taper=(1, 1), shift=(0, 0), drop=(), pivot=None,
              edges="tv", top_mat=None, bevel_mat=None):
        """모서리를 깎은 돌(`box`와 같은 치수·기울임). 건물 모서리는 둥글게(direction.md 기둥 2) — 깎인 면이 제 빛 단을 받아
        모서리가 한 칸 밝거나 어둡게 읽힌다. edges: 't' 윗면 둘레 · 'v' 세로 모서리 · 'b' 아랫면 둘레(바닥에 닿으면 뺀다).
        top_mat = 윗면 칸(기본 mat의 밝은 칸), bevel_mat = 깎인 면 칸(기본 mat). drop = 뺄 본디 면('-z' 등)."""
        sx, sy, sz = s
        tx, ty = taper
        R = Euler([math.radians(a) for a in rot], "XYZ").to_matrix()
        cv = Vector(c)
        pv = Vector(pivot) if pivot is not None else cv
        bm = bmesh.new()
        cs, iz_of = {}, {}
        for ix in (0, 1):
            for iy in (0, 1):
                for iz in (0, 1):
                    x, y, z = (ix - 0.5) * sx, (iy - 0.5) * sy, (iz - 0.5) * sz
                    if iz:
                        x, y = x * tx + shift[0], y * ty + shift[1]
                    v = bm.verts.new((x, y, z))
                    cs[(ix, iy, iz)] = v
                    iz_of[v] = iz
        planes = {}
        for key, corners in _FACES.items():
            f = bm.faces.new([cs[k] for k in corners])
            f.normal_update()
            planes[key] = (f.normal.copy(), f.normal.dot(f.verts[0].co))
        pick = []
        for e in bm.edges:
            k = iz_of[e.verts[0]] + iz_of[e.verts[1]]
            if {2: "t", 1: "v", 0: "b"}[k] in edges:
                pick.append(e)
        if bevel > 0 and pick:
            bmesh.ops.bevel(bm, geom=pick, offset=bevel, offset_type="OFFSET", segments=1, profile=0.5,
                            affect="EDGES", clamp_overlap=True)
        bm.normal_update()
        for f in bm.faces:
            n = f.normal
            orig = next((k for k, (pn, pd) in planes.items()
                         if n.dot(pn) > 0.9999 and abs(pn.dot(f.verts[0].co) - pd) < 1e-5), None)
            if orig in drop:
                continue
            if orig == "+z":
                uv = cell_mid(top_mat or mat, "side" if top_mat else "top")
            elif orig is not None:
                uv = cell_mid(mat, "side")
            else:
                uv = cell_mid(bevel_mat or mat, "top" if n.z > 0.25 else "side")
            pts = [pv + R @ (cv + v.co - pv) for v in f.verts]
            self.quads.append((pts, [uv] * len(pts), False))
        bm.free()
        return self

    def _sweep(self, rings, mats, smooth=False, hard=(), cap0=None, cap1=None):
        """고리(점 목록 — 점 하나면 뾰족한 끝)를 차례로 이은 띠. 고리는 진행 쪽에서 마주 보아 반시계.
        mats = 띠마다 (칸 이름, 'side'|'top'). smooth면 띠끼리 정점을 같이 쓴다 — hard에 적은 고리에서는 끊는다(또렷한 테).
        cap0·cap1 = 처음·끝 고리를 막는 면의 (칸, 면)."""
        grp = self._group() if smooth else False
        for i in range(len(rings) - 1):
            a, b = rings[i], rings[i + 1]
            uv = cell_mid(*mats[i])
            if smooth and i in hard:
                grp = self._group()
            n = max(len(a), len(b))
            for k in range(n):
                if len(a) == 1:
                    pts = [a[0], b[(k + 1) % n], b[k]]
                elif len(b) == 1:
                    pts = [a[k], a[(k + 1) % n], b[0]]
                else:
                    pts = [a[k], a[(k + 1) % n], b[(k + 1) % n], b[k]]
                self.quads.append((pts, [uv] * len(pts), grp))
        if cap0 and len(rings[0]) > 2:
            self.quads.append((list(reversed(rings[0])), [cell_mid(*cap0)] * len(rings[0]), False))
        if cap1 and len(rings[-1]) > 2:
            self.quads.append((list(rings[-1]), [cell_mid(*cap1)] * len(rings[-1]), False))
        return self

    def lathe(self, c, profile, mat, sides=8, rot=(0, 0, 0), pivot=None, smooth=False, hard=(), phase=0.5,
              squash=(1, 1), caps=(False, True), cap_mat=None):
        """돌려 만든 꼴(기둥 북·그릇·종·결정·불꽃). profile = [(반지름, 높이)…] 아래에서 위로, c 기준. 반지름 0 = 뾰족한 끝.
        mat = 칸 이름 하나 또는 띠마다 하나(len(profile) − 1개). 위를 보는 띠는 밝은 칸.
        smooth = 둥글게(정점 공유, 툰 띠가 감긴다 — 산 것·둥근 건물), 아니면 면마다 각지게(뿌리·결정·테 — 밝은 선이 붙는다).
        phase = 첫 모서리 각(1 = 한 변만큼), squash = x·y 배율(타원), caps = (아래, 위) 막기."""
        R = Euler([math.radians(a) for a in rot], "XYZ").to_matrix()
        cv = Vector(c)
        pv = Vector(pivot) if pivot is not None else cv
        mats = [mat] * (len(profile) - 1) if isinstance(mat, str) else list(mat)
        rings = []
        for r, z in profile:
            if r <= 1e-6:
                rings.append([pv + R @ (cv + Vector((0, 0, z)) - pv)])
                continue
            ring = []
            for k in range(sides):
                a = 2 * math.pi * (k + phase) / sides
                ring.append(pv + R @ (cv + Vector((r * math.cos(a) * squash[0], r * math.sin(a) * squash[1], z)) - pv))
            rings.append(ring)
        cells = []
        for i in range(len(profile) - 1):
            (r0, z0), (r1, z1) = profile[i], profile[i + 1]
            up = (r0 - r1) / max(1e-6, math.hypot(r0 - r1, z1 - z0))
            cells.append((mats[i], "top" if up > 0.5 else "side"))
        cm = cap_mat or mats[-1]
        return self._sweep(rings, cells, smooth, hard,
                           (cap_mat or mats[0], "side") if caps[0] else None, (cm, "top") if caps[1] else None)

    def limb(self, pts, radii, mat, sides=4, roll=0.0, flat=1.0, smooth=False, caps=(False, False)):
        """꺾인 관(뿌리·가지·들보): 점들을 따라 n각 단면, 점마다 반지름(0 = 뾰족한 끝). 기본은 각진 면 — 뿌리는 각지게(0037),
        네모 단면(roll 0)은 한 모서리가 위를 봐서 등줄기에 밝은 선이 붙는다. flat = 단면의 위아래 배율(납작한 뿌리)."""
        P = [Vector(p) for p in pts]
        mats = [mat] * (len(P) - 1) if isinstance(mat, str) else list(mat)
        rings, u = [], None
        for i, p in enumerate(P):
            t = (P[min(i + 1, len(P) - 1)] - P[max(i - 1, 0)]).normalized()
            if u is None:
                u = t.cross(Vector((0, 0, 1)))
                if u.length < 1e-4:
                    u = Vector((1, 0, 0))
            else:
                u = u - t * u.dot(t)
            u.normalize()
            v = t.cross(u)
            r = radii[i]
            if r <= 1e-6:
                rings.append([p])
                continue
            rings.append([p + (u * math.cos(a) + v * (math.sin(a) * flat)) * r
                          for a in (2 * math.pi * k / sides + math.radians(roll) for k in range(sides))])
        cells = [(m, "side") for m in mats]
        return self._sweep(rings, cells, smooth, (),
                           (mats[0], "side") if caps[0] else None, (mats[-1], "side") if caps[1] else None)

    def verts(self):
        return len(self.quads) * 4


def geo_object(geo, mat):
    me = bpy.data.meshes.new(geo.name)
    bm = bmesh.new()
    uvl = bm.loops.layers.uv.new("UVMap")
    shared = {}
    for pts, uvs, smooth in geo.quads:
        if smooth:
            # 둥근 덩어리는 정점을 같이 쓴다(매끈한 법선). 같은 묶음(`smooth` 값) + 같은 자리 = 같은 정점.
            vs = []
            for p in pts:
                key = (smooth, round(p[0], 5), round(p[1], 5), round(p[2], 5))
                if key not in shared:
                    shared[key] = bm.verts.new(p)
                vs.append(shared[key])
            if len(set(vs)) < len(vs):
                continue
        else:
            vs = [bm.verts.new(p) for p in pts]
        try:
            f = bm.faces.new(vs)
        except ValueError:
            continue
        f.smooth = bool(smooth)
        for loop, uv in zip(f.loops, uvs):
            loop[uvl].uv = uv
    bm.to_mesh(me)
    bm.free()
    me.materials.append(mat)
    obj = bpy.data.objects.new(geo.name, me)
    bpy.context.scene.collection.objects.link(obj)
    return obj


# ---------------------------------------------------------------- 바깥 메시(0038)
def import_split(path, classify, names):
    """한 덩어리 메시 GLB(Meshy → `meshy_convert.py`로 줄이고 색을 구운 것, 텍스처 하나)를 가져와
    면 가운데 자리 `classify(Vector) -> 이름`으로 부위 오브젝트(names)마다 나눈다. 강체 부위라 잘린 면은 이어 붙이지 않는다.
    돌려주는 것: ({이름: 오브젝트}, 텍스처 이미지)."""
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=path)
    new = [o for o in bpy.data.objects if o not in before]
    src = next(o for o in new if o.type == "MESH")
    me = src.data.copy()
    me.transform(src.matrix_world)
    # glTF에서 온 법선은 "사용자 법선"으로 붙어 와 면마다 매끈/각짐을 무시한다 → 지우고 블렌더가 다시 계산하게.
    tmp = bpy.data.objects.new("_split_tmp", me)
    bpy.context.scene.collection.objects.link(tmp)
    bpy.ops.object.select_all(action="DESELECT")
    tmp.select_set(True)
    bpy.context.view_layer.objects.active = tmp
    if me.has_custom_normals:
        bpy.ops.mesh.customdata_custom_splitnormals_clear()
    if hasattr(me, "use_auto_smooth"):
        me.use_auto_smooth = False
    bpy.data.objects.remove(tmp)
    # 줄이면서 접힌 곳(귀·주름)은 면 법선과 매끈한 정점 법선이 반대일 수 있다 → 그 면만 각지게(외곽선 컬링·검사 `check_glb`의 뒤집힌 면).
    for poly in me.polygons:
        if any(poly.normal.dot(me.vertices[i].normal) < 0.0 for i in poly.vertices):
            poly.use_smooth = False
    img = next(n.image for m in me.materials for n in m.node_tree.nodes if n.type == "TEX_IMAGE" and n.image)
    parts = {}
    for name in names:
        pm = me.copy()
        bm = bmesh.new()
        bm.from_mesh(pm)
        dead = [f for f in bm.faces if classify(f.calc_center_median()) != name]
        bmesh.ops.delete(bm, geom=dead, context="FACES")
        bmesh.ops.delete(bm, geom=[v for v in bm.verts if not v.link_faces], context="VERTS")
        bm.to_mesh(pm)
        bm.free()
        if not pm.polygons:
            continue
        # 자르면 경계의 정점 법선이 다시 계산된다 — 그때 어긋난 면도 각지게(종지기 0132: 자른 뒤 1면).
        pm.update()
        for poly in pm.polygons:
            if poly.use_smooth and any(poly.normal.dot(pm.vertices[i].normal) < 0.0 for i in poly.vertices):
                poly.use_smooth = False
        o = bpy.data.objects.new(name, pm)
        bpy.context.scene.collection.objects.link(o)
        parts[name] = o
    for o in new:
        bpy.data.objects.remove(o)
    return parts, img


def face_colors(path):
    """줄인 Meshy GLB(`meshy_convert.py` 결과, 텍스처는 옆의 `<이름>_tex.png`)의 삼각형마다 (가운데 자리(블렌더 축), (h, s, v)).
    색 = 세 꼭짓점 + 가운데 UV의 텍스처 색 평균. 자리로는 못 가르는 부위(꼬투리·종·핵 — 0132)를 색으로 고를 때:
    `[c for c, hsv in face_colors(p) if 조건(*hsv)]` → `classify`에서 `near_any(그 목록, 면 가운데)`."""
    import colorsys
    import json
    import struct
    data = open(path, "rb").read()
    off, js, bn = 12, None, b""
    while off < len(data):
        n, kind = struct.unpack_from("<II", data, off)
        chunk = data[off + 8:off + 8 + n]
        if kind == 0x4E4F534A:
            js = json.loads(chunk)
        else:
            bn = chunk
        off += 8 + n

    def acc(i, fmt, k):
        a = js["accessors"][i]
        bv = js["bufferViews"][a["bufferView"]]
        base = bv.get("byteOffset", 0) + a.get("byteOffset", 0)
        size = struct.calcsize("<" + fmt)
        return [struct.unpack_from("<" + fmt * k, bn, base + j * size * k) for j in range(a["count"])]

    prim = js["meshes"][0]["primitives"][0]
    pos = acc(prim["attributes"]["POSITION"], "f", 3)
    uv = acc(prim["attributes"]["TEXCOORD_0"], "f", 2)
    ifmt = {5123: "H", 5125: "I"}[js["accessors"][prim["indices"]]["componentType"]]
    idx = [i[0] for i in acc(prim["indices"], ifmt, 1)]
    img = bpy.data.images.load(path.replace(".glb", "_tex.png"), check_existing=True)
    W, H = img.size
    px = img.pixels[:]

    def texel(u, v):
        x = int(u * W) % W
        y = H - 1 - int(v * H) % H  # glTF v는 위에서, 블렌더 화소 줄은 아래에서
        k = (y * W + x) * 4
        return px[k:k + 3]

    out = []
    for t in range(0, len(idx), 3):
        tri = idx[t:t + 3]
        uvs = [uv[i] for i in tri]
        uvs.append(tuple(sum(c) / 3 for c in zip(*uvs)))
        rgb = [sum(c) / 4 for c in zip(*[texel(*q) for q in uvs])]
        c = [sum(pos[i][a] for i in tri) / 3 for a in range(3)]
        out.append(((c[0], -c[2], c[1]), colorsys.rgb_to_hsv(*rgb)))  # glTF(Y 위) → 블렌더(Z 위)
    bpy.data.images.remove(img)
    return out


def near_any(points, c, tol=2e-3):
    """가져온 면 가운데 `c`가 목록의 한 자리와 같은가(가져오며 생기는 부동소수 차이만 봐준다)."""
    x, y, z = c
    return any(abs(x - a) < tol and abs(y - b) < tol and abs(z - d) < tol for a, b, d in points)


def darken_grey(img, k, sat_max):
    """바깥 텍스처(Meshy)의 잿빛 칸(채도 < sat_max)만 k배 어둡게 — 원본 파일은 그대로, 게임용 사본만.
    Meshy 색은 밝은 회색이라 해 질 녘 성소에서 혼자 하얗게 뜬다(0037: 바랜 잿빛 라벤더). 호박빛·보라 뿌리·이끼는 그대로."""
    import numpy as np
    px = np.array(img.pixels[:], dtype=np.float32).reshape(-1, 4)
    rgb = px[:, :3]
    mx, mn = rgb.max(axis=1), rgb.min(axis=1)
    sat = np.where(mx > 1e-4, (mx - mn) / np.maximum(mx, 1e-4), 0.0)
    rgb[sat < sat_max] *= k
    px[:, :3] = rgb
    img.pixels.foreach_set(px.ravel())
    img.update()


def merge_textures(ext_objs, ext_img, atlas_objs, atlas_img, name):
    """로더는 첫 이미지 하나만 읽는다(`src/model.rs`) → 바깥 텍스처(W×H)와 코드 아틀라스(ATLAS²)를 (2W)×H 한 장으로:
    바깥은 왼쪽 반, 아틀라스는 오른쪽 위 모서리. UV를 옮기고 모든 부위가 새 재질 하나를 쓴다."""
    import numpy as np

    W, H = ext_img.size
    A = atlas_img.size[0]
    assert H >= A
    CW, CH = 2 * W, H
    out = np.zeros((CH, CW, 4), dtype=np.float32)
    out[:, :, 3] = 1.0
    e = np.array(ext_img.pixels[:], dtype=np.float32).reshape(H, W, 4)  # 블렌더 이미지는 아래 줄부터
    a = np.array(atlas_img.pixels[:], dtype=np.float32).reshape(A, A, 4)
    out[:, :W] = e
    out[CH - A:, W:W + A] = a
    img = bpy.data.images.new(name + "_tex", CW, CH, alpha=False)
    img.pixels.foreach_set(out.ravel())
    img.file_format = "PNG"
    img.pack()
    mat = bpy.data.materials.new(name + "_mat")
    mat.use_nodes = True
    nt = mat.node_tree
    bsdf = nt.nodes["Principled BSDF"]
    bsdf.inputs["Roughness"].default_value = 1.0
    tex = nt.nodes.new("ShaderNodeTexImage")
    tex.image = img
    tex.interpolation = "Closest"
    nt.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])

    def remap(o, f):
        for d in o.data.uv_layers.active.data:
            d.uv = f(d.uv.x, d.uv.y)
        o.data.materials.clear()
        o.data.materials.append(mat)

    for o in ext_objs:
        remap(o, lambda u, v: (u * W / CW, v))
    for o in atlas_objs:
        remap(o, lambda u, v: ((W + u * A) / CW, (CH - A + v * A) / CH))
    return mat


# ---------------------------------------------------------------- 모델
def reset_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    sc = bpy.context.scene
    sc.render.fps = FPS
    sc.render.fps_base = 1.0
    return sc


class Model:
    """사용: m = Model('wanderer'); m.bone(...); g = m.geo('body_geo', 'body'); g.box(...);
    m.build(); m.action(...); m.export(); m.preview(...)."""

    def __init__(self, name):
        self.name = name
        self.scene = reset_scene()
        self.mat = make_material(name)
        self.geos = []
        self.ext = []  # (이름, 뼈, 오브젝트) — 바깥 메시 부위(0038)
        self.ext_img = None
        self.bones = []  # (이름, 머리(피벗), 부모)
        self.rig = None
        self.objects = {}
        self.actions = []
        # 평소엔 크기 0으로 숨는 뼈(손에 드는 소품 — 그 동작만 's': 1로 꺼낸다).
        self.hidden = set()
        # 동작의 'l'(위치) 배율 — 몸이 작아져도 같은 동작 표를 쓴다(0036 방랑자 2.3등신: 웅크림이 땅을 뚫지 않게).
        self.loc_scale = 1.0

    # --- 만들기
    def bone(self, name, head, parent=None):
        self.bones.append((name, tuple(head), parent))

    def geo(self, name, bone=None):
        g = Geo(name, bone)
        self.geos.append(g)
        return g

    def external(self, path, classify, parts, img_name=None):
        """바깥 메시를 부위로 나눠 뼈에 붙인다. parts: {부위 이름: 뼈}. 텍스처는 build에서 아틀라스와 한 장으로 합친다."""
        objs, img = import_split(path, classify, list(parts))
        self.ext_img = img
        for name, o in objs.items():
            self.ext.append((name, parts[name], o))
        return objs

    def build(self):
        for g in self.geos:
            if g.quads:
                self.objects[g.name] = geo_object(g, self.mat)
        for name, _, o in self.ext:
            self.objects[name] = o
        if self.ext:
            atlas = self.mat.node_tree.nodes["Image Texture"].image
            merge_textures([o for _, _, o in self.ext], self.ext_img,
                           [self.objects[g.name] for g in self.geos if g.name in self.objects], atlas, self.name)
        if not self.bones:
            return
        arm = bpy.data.armatures.new(self.name + "_rig")
        rig = bpy.data.objects.new(self.name + "_rig", arm)
        self.scene.collection.objects.link(rig)
        bpy.context.view_layer.objects.active = rig
        rig.select_set(True)
        bpy.ops.object.mode_set(mode="EDIT")
        eb = arm.edit_bones
        for name, head, parent in self.bones:
            b = eb.new(name)
            b.head = head
            # 뼈는 모두 위(+Z)를 향한다 — 자세 각도 변환이 뼈마다 같다.
            b.tail = (head[0], head[1], head[2] + 0.1)
            b.roll = 0.0
            if parent:
                b.parent = eb[parent]
        bpy.ops.object.mode_set(mode="OBJECT")
        for bone_name, o in [(g.bone, self.objects.get(g.name)) for g in self.geos] + [(b, o) for _, b, o in self.ext]:
            if bone_name and o is not None:
                mw = o.matrix_world.copy()
                o.parent = rig
                o.parent_type = "BONE"
                o.parent_bone = bone_name
                o.matrix_world = mw
        self.rig = rig
        for pb in rig.pose.bones:
            pb.rotation_mode = "QUATERNION"

    def hide(self, bone):
        """이 뼈(와 그 아래 부위)는 동작이 's'를 주지 않으면 크기 0 — 소품을 그 동작에서만 보인다."""
        self.hidden.add(bone)

    # --- 동작
    def action(self, name, dur, keys, loop=False, linear=()):
        """keys: [(초, {뼈: {'r': (x,y,z)도, 'l': (x,y,z)m 월드 축, 's': 배율}})].
        뼈마다: 0초에 키가 없으면 쉬는 자세, 끝에 키가 없으면 loop면 0초 자세·아니면 마지막 키를 끝까지.
        모든 뼈를 0초와 끝에 박아 동작을 바꿀 때 남는 자세가 없다. linear = 그 초에서 시작하는 구간을 직선으로(빠른 타)."""
        rig = self.rig
        end = int(round(dur * FPS))
        act = bpy.data.actions.new(name)
        act.use_fake_user = True
        rig.animation_data_create()
        rig.animation_data.action = act
        rest = {"r": (0, 0, 0), "l": (0, 0, 0), "s": 1.0}
        for pb in rig.pose.bones:
            bn = pb.name
            ks = sorted(((t, p[bn]) for t, p in keys if bn in p), key=lambda k: k[0])
            if not ks or ks[0][0] > 1e-6:
                ks.insert(0, (0.0, {}))
            if ks[-1][0] < dur - 1e-6:
                ks.append((dur, ks[0][1] if loop else ks[-1][1]))
            prev_q = None
            Rb = pb.bone.matrix_local.to_3x3()
            Rbi = Rb.inverted()
            for t, pose in ks:
                f = int(round(t * FPS))
                r = pose.get("r", rest["r"])
                l = tuple(v * self.loc_scale for v in pose.get("l", rest["l"]))
                s = pose.get("s", 0.0 if bn in self.hidden else rest["s"])
                Rw = Euler([math.radians(a) for a in r], "XYZ").to_matrix()
                q = (Rbi @ Rw @ Rb).to_quaternion()
                if prev_q is not None and prev_q.dot(q) < 0:
                    q = -q
                prev_q = q
                pb.rotation_quaternion = q
                pb.location = Rbi @ Vector(l)
                pb.scale = (s, s, s) if not isinstance(s, tuple) else s
                pb.keyframe_insert("rotation_quaternion", frame=f, group=bn)
                pb.keyframe_insert("location", frame=f, group=bn)
                pb.keyframe_insert("scale", frame=f, group=bn)
        lin = {int(round(t * FPS)) for t in linear}
        for fc in act.fcurves:
            for kp in fc.keyframe_points:
                if int(round(kp.co[0])) in lin:
                    kp.interpolation = "LINEAR"
                else:
                    kp.interpolation = "BEZIER"
                    kp.handle_left_type = kp.handle_right_type = "AUTO_CLAMPED"
            fc.update()
        # 곡선(베지어)을 FRAME_STEP마다 + 키 시각 + 끝 프레임에서 읽어 직선 키로 다시 박는다.
        # 내보내기는 그 키를 그대로(LINEAR) 쓴다 — 블렌더의 표본 찍기는 끝 프레임을 빠뜨려 길이가 틀어진다.
        frames = sorted(set(range(0, end + 1, FRAME_STEP)) | {end} | {int(round(t * FPS)) for t, _ in keys})
        # 뼈의 위치·회전·크기 묶음(fcurve 3~4개)마다: 내내 쉬는 자세면 통째로 뺀다(게임 로더는 채널이 없는
        # 노드를 쉬는 자세로 둔다), 내내 같은 값이면 처음·끝 두 키만.
        groups = {}
        for fc in act.fcurves:
            groups.setdefault(fc.data_path, []).append(fc)
        for path, fcs in groups.items():
            rest_v = 1.0 if path.endswith("scale") else 0.0
            table = []
            for fc in fcs:
                vals = [fc.evaluate(f) for f in frames]
                rv = (1.0 if fc.array_index == 0 else 0.0) if path.endswith("rotation_quaternion") else rest_v
                table.append((fc, vals, rv))
            const = all(max(v) - min(v) < 1e-5 for _, v, _ in table)
            if const and all(abs(v[0] - rv) < 1e-5 for _, v, rv in table):
                for fc, _, _ in table:
                    act.fcurves.remove(fc)
                continue
            for fc, vals, _ in table:
                pts = [(f, v) for f, v in zip(frames, vals)]
                if const:
                    pts = [pts[0], pts[-1]]
                fc.keyframe_points.clear()
                fc.keyframe_points.add(len(pts))
                for kp, (f, v) in zip(fc.keyframe_points, pts):
                    kp.co = (f, v)
                    kp.interpolation = "LINEAR"
                fc.update()
        act.use_frame_range = True
        act.frame_start, act.frame_end = 0, end
        act.use_cyclic = loop
        # ACTIONS 내보내기는 NLA에 있는 동작을 하나씩 내보낸다 → 동작마다 트랙에 넣는다.
        tr = rig.animation_data.nla_tracks.new()
        tr.name = name
        tr.strips.new(name, 0, act)
        rig.animation_data.action = None
        self.actions.append((name, dur, loop))
        return act

    # --- 내보내기
    def export(self, out_dir=MODELS_DIR):
        os.makedirs(out_dir, exist_ok=True)
        path = os.path.join(out_dir, self.name + ".glb")
        bpy.ops.object.select_all(action="DESELECT")
        bpy.ops.export_scene.gltf(
            filepath=path,
            export_format="GLB",
            export_yup=True,
            export_texcoords=True,
            export_normals=True,
            export_materials="EXPORT",
            export_image_format="AUTO",
            export_cameras=False,
            export_lights=False,
            export_extras=False,
            export_skins=True,
            export_animations=bool(self.actions),
            export_animation_mode="ACTIONS",
            export_force_sampling=False,
            export_optimize_animation_size=True,
            export_anim_slide_to_zero=False,
            export_bake_animation=False,
        )
        print(f"[knellroot] wrote {path}")
        return path

    # --- 미리보기
    def pose_at(self, action=None, t=0.0):
        if not self.rig:
            return
        ad = self.rig.animation_data
        if ad:
            for tr in ad.nla_tracks:
                tr.mute = True
            ad.action = None
        # 먼저 쉬는 자세로 — 동작에 채널이 없는 뼈는 게임에서도 쉬는 자세다(빠진 채널 = 쉬는 자세).
        for pb in self.rig.pose.bones:
            pb.rotation_quaternion = (1, 0, 0, 0)
            pb.location = (0, 0, 0)
            pb.scale = (1, 1, 1)
        if ad and action:
            ad.action = bpy.data.actions[action]
        self.scene.frame_set(int(round(t * FPS)))

    def preview(self, path, action=None, t=0.0, size=320, yaw=-35.0, elev=32.0, floor=True, frame=None):
        """Workbench(텍스처 색) 직교 렌더. 카메라는 앞(-Y)에서 오른쪽(-X)으로 yaw도, 위로 elev도.
        모델의 경계 상자에 맞춰 화면을 채운다. frame=(가운데 Vector, ortho_scale)을 주면 그 화면에 고정(동작 띠 — 크기 변화가 보이게).
        돌려주는 것 = 쓴 (가운데, ortho_scale)."""
        sc = self.scene
        self.pose_at(action, t)
        dg = bpy.context.evaluated_depsgraph_get()
        pts = []
        for o in self.objects.values():
            oe = o.evaluated_get(dg)
            mw = oe.matrix_world
            pts += [mw @ v.co for v in oe.data.vertices]
        lo = Vector([min(p[i] for p in pts) for i in range(3)])
        hi = Vector([max(p[i] for p in pts) for i in range(3)])
        c = (lo + hi) / 2 if frame is None else frame[0]
        ya, ea = math.radians(yaw), math.radians(elev)
        d = Vector((math.sin(ya) * math.cos(ea), -math.cos(ya) * math.cos(ea), math.sin(ea)))
        cam = sc.camera
        if cam is None:
            cd = bpy.data.cameras.new("preview_cam")
            cd.type = "ORTHO"
            cd.clip_start, cd.clip_end = 0.1, 200
            cam = bpy.data.objects.new("preview_cam", cd)
            sc.collection.objects.link(cam)
            sc.camera = cam
        cam.location = c + d * 40
        cam.rotation_mode = "QUATERNION"
        cam.rotation_quaternion = (-d).to_track_quat("-Z", "Y")
        bpy.context.view_layer.update()
        ci = cam.matrix_world.inverted()
        ext = 0.0
        for p in pts:
            q = ci @ p
            ext = max(ext, abs(q.x), abs(q.y))
        cam.data.ortho_scale = max(ext * 2 * 1.12, 0.5) if frame is None else frame[1]
        if floor and "preview_floor" not in bpy.data.objects:
            bpy.ops.mesh.primitive_plane_add(size=1, location=(0, 0, -0.002))
            fl = bpy.context.object
            fl.name = "preview_floor"
            fm = bpy.data.materials.new("preview_floor_mat")
            fm.diffuse_color = (0.16, 0.17, 0.19, 1)
            fl.data.materials.append(fm)
        if "preview_floor" in bpy.data.objects:
            fl = bpy.data.objects["preview_floor"]
            fl.scale = (max(hi.x - lo.x, hi.y - lo.y) * 1.6 + 1,) * 3
            fl.location = (c.x, c.y, -0.002)
        if sc.world is None:
            sc.world = bpy.data.worlds.new("preview_world")
        sc.world.color = (0.36, 0.38, 0.42)
        sc.render.engine = "BLENDER_WORKBENCH"
        sh = sc.display.shading
        sh.light = "STUDIO"
        sh.color_type = "TEXTURE"
        sh.show_object_outline = True
        sh.object_outline_color = (0.05, 0.05, 0.06)
        sh.show_cavity = False
        sc.view_settings.view_transform = "Standard"
        sc.render.resolution_x = sc.render.resolution_y = size
        sc.render.resolution_percentage = 100
        sc.render.film_transparent = False
        sc.render.image_settings.file_format = "PNG"
        sc.render.filepath = path
        os.makedirs(os.path.dirname(path), exist_ok=True)
        bpy.ops.render.render(write_still=True)
        print(f"[knellroot] preview {path}")
        return c.copy(), cam.data.ortho_scale


def out_dirs():
    """`blender ... -- [모델 폴더] [미리보기 폴더]`로 바꿀 수 있다(검사용)."""
    argv = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    models = argv[0] if len(argv) > 0 else MODELS_DIR
    prev = argv[1] if len(argv) > 1 else PREVIEW_DIR
    return models, prev
