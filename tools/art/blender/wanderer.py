"""방랑자(플레이어, 키 1.8m): 후드 망토·청록 목도리·바랜 갈색 띠와 장화·후드 그늘 속 불씨 눈 두 점·오른손의 검.

    blender --background --python tools/art/blender/wanderer.py
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

NAME = "wanderer"


S = 1.15  # 시안(concepts.py variant_b)의 치수 → 게임 키(≈ 1.4m)


def v(*xs):
    return tuple(x * S for x in xs)


# 0038: 몸은 Meshy(AI) 모델 — Gemini 그림 → Meshy Image to 3D(CC BY 4.0) → `meshy_convert.py`로 줄이고 색을 구운 것.
MESHY_LOW = os.path.join(C.REPO, "tools", "art", "sources", "meshy", "raccoon_v1_low.glb")
# 0189: 몸 둘 — v1 "Cozy Bell Raccoon"(연보라 두건·민트 옷, 0038) · v2 "Hooded Raccoon Rogue"(보라 두건 망토·주머니, 사장 2026-11-14
# "난 저 새 모델이 맘에 드는데"). 기본 v2, `KR_WANDERER_BODY=v1`이면 옛 몸으로 다시 굽는다(되돌리기·견주기).
BODY = os.environ.get("KR_WANDERER_BODY", "v2")
if BODY == "v2":
    MESHY_LOW = os.path.join(C.REPO, "tools", "art", "sources", "meshy", "raccoon_v2_low.glb")
CX = 0.0  # 몸 가운데 x(머리로 맞춘 뒤 — 두 손끝이 ±0.35로 맞는다)
# 부위 이름 → 뼈
MESHY_PARTS = {"head_geo": "head", "body_geo": "body", "arm_l_geo": "arm_l", "arm_r_geo": "arm_r",
               "leg_l_geo": "leg_l", "leg_r_geo": "leg_r", "tail_geo": "tail"}


# 팔 = 어깨 → 손끝 선분 둘레의 원기둥(목도리 끝·옷자락이 섞이지 않게). (어깨, 손끝, 반지름), x는 CX 기준.
ARM = ((0.235, -0.07, 0.42), (0.30, -0.085, 0.30), 0.072)  # 장갑 손 + 소매 끝


def _seg_dist(p, a, b):
    from mathutils import Vector
    a, b = Vector(a), Vector(b)
    ab = b - a
    t = max(0.0, min(1.0, (p - a).dot(ab) / ab.length_squared))
    return (p - (a + ab * t)).length


# v2 몸의 치수(키 1.4m, 앞 = -Y, 캐릭터 왼쪽 = +X) — 10cm 격자를 그은 앞·옆 렌더와 높이 띠 정점 분포로 잰 것(0189).
V2_NECK_Z = 0.82  # 두건이 어깨로 늘어진 자락 아래 — 목도리·깃은 몸에 남는다
V2_LEG_Z = 0.16  # 망토 밑단 아래로 나온 검은 발
V2_SHOULDER = (0.24, -0.02, 0.64)
V2_HAND = (0.385, -0.03, 0.45)
V2_ARM_R = 0.085
V2_TAIL_Y = 0.22  # 이보다 뒤(+Y)이고 엉덩이 아래면 꼬리


# v2 망토 밝히기(0189): 원본의 짙은 보라는 어두운 땅속 지역(정원·뿌리 굴)에서 바닥과 같은 밝기의 덩어리가 됐다(게임 화면).
# 원본은 두고 게임용 텍스처 사본에서 보라 계열만 밝기를 올리고 채도를 조금 낮춘다(KB ai-mesh-to-rigid-part-model "AI 텍스처의 밝기").
# 0 = 원본 그대로. `KR_WANDERER_LIFT`로 바꿔 굽는다.
V2_LIFT = float(os.environ.get("KR_WANDERER_LIFT", "0.45"))
V2_DESAT = 0.8


def lift_purples(img, lift=V2_LIFT, desat=V2_DESAT):
    """텍스처의 보라·자주 칸(색상 0.70~0.97)만 V를 (1 − V) × lift만큼 올리고 채도를 desat배로 — 얼굴·털·가죽·꼬리는 그대로."""
    import colorsys
    if lift <= 0.0:
        return
    px = list(img.pixels[:])
    n = 0
    for i in range(0, len(px), 4):
        h, s, v = colorsys.rgb_to_hsv(px[i], px[i + 1], px[i + 2])
        if 0.70 < h < 0.97 and s > 0.15:
            v = v + (1.0 - v) * lift
            s = s * desat
            px[i], px[i + 1], px[i + 2] = colorsys.hsv_to_rgb(h, s, v)
            n += 1
    img.pixels[:] = px
    print(f"[wanderer] v2 cloak lifted: {n} texels (lift {lift})")


def classify_v2(c):
    from mathutils import Vector
    x, y, z = c.x, c.y, c.z
    if z > V2_NECK_Z:
        return "head_geo"
    if y > V2_TAIL_Y and z < 0.62:
        return "tail_geo"
    if z < V2_LEG_Z:
        return "leg_l_geo" if x > 0 else "leg_r_geo"
    p = Vector((x, y, z))
    (sx, sy, sz), (hx, hy, hz) = V2_SHOULDER, V2_HAND
    if _seg_dist(p, (-sx, sy, sz), (-hx, hy, hz)) < V2_ARM_R:
        return "arm_r_geo"
    if _seg_dist(p, (sx, sy, sz), (hx, hy, hz)) < V2_ARM_R:
        return "arm_l_geo"
    return "body_geo"


def classify(c):
    """면 가운데 자리 → 부위(앞 = -Y, 캐릭터 왼쪽 = +X, 키 1.4m). 치수는 높이 띠마다 정점을 찍어 본 것."""
    from mathutils import Vector
    if BODY == "v2":
        return classify_v2(c)
    x, y, z = c.x - CX, c.y, c.z
    if z > 0.628:  # 두건 아래 가장자리 — 목도리는 몸에 남겨 고개를 젖혀도 자른 자리가 덜 드러난다
        return "head_geo"
    if z < 0.125:
        return "leg_l_geo" if x > 0 else "leg_r_geo"
    (sx, sy, sz), (hx, hy, hz), r = ARM
    p = Vector((x, y, z))
    if _seg_dist(p, (-sx, sy, sz), (-hx, hy, hz)) < r:
        return "arm_r_geo"
    if _seg_dist(p, (sx, sy, sz), (hx, hy, hz)) < r:
        return "arm_l_geo"
    if z < 0.62 and (y > 0.17 or (y > 0.06 and x > 0.14)):
        return "tail_geo"
    return "body_geo"


# 0153 꼬리: Meshy 원본의 꼬리는 왼쪽 엉덩이에서 옆으로 뻗는다(위에서 보면 몸 밖 왼쪽으로 0.3m — 사장 "너무 한쪽으로 치우쳐져").
# 부위로 자른 뒤 꼬리만 등 가운데로 돌려 붙이고 조금 세운다 — 뒤에서(락온·쿼터뷰) 줄무늬가 길게 보인다.
TAIL_AT = (0.0, 0.085, 0.235)  # 새 뿌리: 등 가운데, 옷자락 안으로 조금 묻는다
TAIL_LIFT = 50.0  # 도: 뿌리에서 끝을 위로 세우는 각(원본의 들림에 더한다)
TAIL_SIZE = 1.0


def _boundary_loops(bm):
    """열린 가장자리(면 하나에만 붙은 모서리)를 이어진 덩이마다: [(모서리들, 정점들)]."""
    seen, out = set(), []
    for e in bm.edges:
        if not e.is_boundary or e in seen:
            continue
        comp, stack = [], [e]
        while stack:
            x = stack.pop()
            if x in seen:
                continue
            seen.add(x)
            comp.append(x)
            for vert in x.verts:
                stack.extend(y for y in vert.link_edges if y.is_boundary and y not in seen)
        out.append((comp, {vert for x in comp for vert in x.verts}))
    return out


def _pinch(bm, verts):
    """열린 가장자리 하나를 한 점(가장자리의 가운데)으로 오므려 닫는다. 새 면을 만들지 않는다 — 들쭉날쭉한 가장자리를 면으로 메우면
    접힌 조각이 생겨 법선이 어긋났다(0153: `check_glb` 뒤집힌 면). 둘레의 면이 그 점으로 모이고 색(UV)은 그대로다."""
    import bmesh
    from mathutils import Vector
    verts = list(verts)
    c = sum((x.co for x in verts), Vector()) / len(verts)
    bmesh.ops.pointmerge(bm, verts=verts, merge_co=c)


def _flat_where_folded(me):
    """정점을 합치고 면을 메운 뒤: 면 법선과 매끈한 정점 법선이 반대인 면은 각지게(`common.import_split`과 같은 규칙 —
    안 그러면 내보낸 법선이 감김과 어긋나 외곽선 컬링·`check_glb`의 뒤집힌 면에 걸린다)."""
    me.update()
    for poly in me.polygons:
        if poly.use_smooth and any(poly.normal.dot(me.vertices[i].normal) < 0.0 for i in poly.vertices):
            poly.use_smooth = False


def recenter_tail(tail, body):
    """꼬리 부위를 등 가운데로(뿌리 = `TAIL_AT`, 곧게 뒤로 + `TAIL_LIFT`만큼 세움), 잘린 자리(몸의 구멍·꼬리 밑동)는 오므려 닫는다."""
    import math

    import bmesh
    from mathutils import Matrix, Vector

    # 몸: 꼬리를 떼어 낸 구멍(왼쪽 엉덩이 뒤)만 메운다 — 목·팔·다리의 잘린 자리는 그 부위가 가린다.
    bm = bmesh.new()
    bm.from_mesh(body.data)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-5)
    n_body = 0
    for _, verts in _boundary_loops(bm):
        c = sum((x.co for x in verts), Vector()) / len(verts)
        if c.x > 0.03 and c.y > 0.0 and 0.13 < c.z < 0.5:
            _pinch(bm, verts)
            n_body += 1
    bm.to_mesh(body.data)
    bm.free()
    _flat_where_folded(body.data)

    bm = bmesh.new()
    bm.from_mesh(tail.data)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-5)
    loops = _boundary_loops(bm)
    rim = {x for _, verts in loops for x in verts}
    pivot = sum((x.co for x in rim), Vector()) / len(rim)
    mid = sum((x.co for x in bm.verts), Vector()) / len(bm.verts)
    d = mid - pivot
    yaw = math.atan2(d.x, d.y)  # 뒤(+Y)에서 왼쪽(+X)으로 벗어난 각
    n_tail = len(loops)
    # 곧게 뒤로 돌리고(Z), 끝을 세우고(X — +Y를 +Z 쪽으로), 뿌리를 등 가운데로.
    turn = Matrix.Rotation(math.radians(TAIL_LIFT), 4, "X") @ Matrix.Rotation(yaw, 4, "Z")
    to = Vector(TAIL_AT)
    for x in bm.verts:
        x.co = turn @ ((x.co - pivot) * TAIL_SIZE) + to
    for _, verts in _boundary_loops(bm):
        _pinch(bm, verts)
    bm.to_mesh(tail.data)
    bm.free()
    _flat_where_folded(tail.data)
    print(f"[wanderer] tail: pivot {tuple(round(c, 3) for c in pivot)} yaw {math.degrees(yaw):.1f}° "
          f"→ {TAIL_AT}, closed body {n_body} tail {n_tail}")


# 두건(0153 → 0155 → 0156 → 0157): 덧붙인 꼴은 모두 걷고 원본 + 면 고르기만 남겼다 — `dress_hood`.
HOOD_TIP_AT = (0.0, 0.30, 0.80)  # 뼈만 남긴다(동작 표가 부른다) — 붙은 부위 없음


def _lavender_of(img):
    """텍스처 UV → 두건 천인가(밝은 라벤더). 눈가의 짙은 보라는 밝기로 거른다."""
    import colorsys
    wd, hd = img.size
    px = img.pixels[:]

    def lavender(uv):
        x = min(wd - 1, max(0, int(uv[0] * wd)))
        y = min(hd - 1, max(0, int(uv[1] * hd)))
        i = (y * wd + x) * 4
        h, sat, val = colorsys.rgb_to_hsv(px[i], px[i + 1], px[i + 2])
        return 0.62 < h < 0.82 and sat > 0.12 and val > 0.5

    return lavender


def _hood_faces(bm, img):
    lavender = _lavender_of(img)
    uvl = bm.loops.layers.uv.active
    return {f for f in bm.faces if sum(lavender(lp[uvl].uv) for lp in f.loops) * 2 > len(f.loops)}


def soften_hood(head, img):
    """두건 면만 고르게(줄이며 생긴 울퉁불퉁 — 툰 음영이 얼룩으로 감겼다). 두건 = 텍스처가 라벤더인 면, 그 면에만 닿은 정점을 이웃 평균으로.
    얼굴·귀·잘린 목 가장자리는 건드리지 않는다. UV는 그대로라 색은 안 바뀐다."""
    import bmesh
    bm = bmesh.new()
    bm.from_mesh(head.data)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-5)
    hood = _hood_faces(bm, img)
    verts = [x for x in bm.verts if x.co.z > 0.74 and not x.is_boundary and all(f in hood for f in x.link_faces)]
    for _ in range(6):
        bmesh.ops.smooth_vert(bm, verts=verts, factor=0.5, use_axis_x=True, use_axis_y=True, use_axis_z=True)
    bm.to_mesh(head.data)
    bm.free()
    _flat_where_folded(head.data)
    print(f"[wanderer] hood: smoothed {len(verts)} verts of {len(hood)} hood faces")


def dress_hood(m, head):
    """두건은 Meshy 원본 그대로 두고 면만 고른다. 코드로 덧붙인 것은 셋 다 걷었다(0157): 뒤통수의 띠(0153 — 넥타이), 꼭지와 종(0155 —
    요정 모자), 어깨로 늘어진 자락과 얼굴 테(0156 — 투구). 천의 주름·뒷모습은 도형을 붙여서가 아니라 그림·조형에서 와야 한다."""
    soften_hood(head, m.ext_img)


# 뼈 자리(새 몸, 키 1.4m). 손은 장갑 가운데, 어깨는 소매 위.
HAND = (0.29, -0.085, 0.31)
SHOULDER = (0.185, -0.05, 0.52)
if BODY == "v2":
    HAND, SHOULDER = V2_HAND, V2_SHOULDER


def weapons(m, hand):
    """무기 넷(오른손 — 게임이 든 것만 보인다, 다른 부위는 크기 0). 0155: 상자 조합이던 것을 **무기의 꼴**로.
    약속(게임 쪽이 기댄다): 손 = `hand`, 길이는 블렌더 −y(앞)로, 넓은 면이 +z를 본다(`item_view.rs`의 각도), 길이·머리 크기는 0051과 같은 남짓.
    실루엣으로 갈린다: 검 = 잎 꼴 날 + 굽은 코등이, 망치 = 가로 통 머리, 지팡이 = 긴 대 + 결정을 품은 갈래, 쇠뇌 = 굽은 활의 T."""
    x0, y0, z0 = hand

    def at(dx, dy, dz=0.0):
        return (x0 + dx, y0 + dy, z0 + dz)

    # --- 검: 마름모 단면의 잎 꼴 날(등줄기에 밝은 선이 붙는다) · 날 쪽으로 굽은 코등이 · 감은 손잡이 · 자루 끝 구슬.
    g = m.geo("sword", "hand_r")
    g.limb([at(0, -0.066), at(0, -0.15), at(0, -0.40), at(0, -0.53), at(0, -0.605)], [0.050, 0.060, 0.048, 0.034, 0.0],
           "p_steel", sides=4, flat=0.30, caps=(True, False))
    g.limb([at(0, -0.075), at(0, -0.38), at(0, -0.51)], [0.020, 0.017, 0.0], "p_white", sides=4, flat=1.2)  # 등줄기
    g.limb([at(-0.125, -0.098), at(-0.078, -0.070), at(0, -0.058), at(0.078, -0.070), at(0.125, -0.098)],
           [0.017, 0.024, 0.031, 0.024, 0.017], "p_gold", sides=6, smooth=True, caps=(True, True))  # 코등이
    for sx in (-1, 1):
        g.blob(at(sx * 0.130, -0.102), (0.044, 0.044, 0.044), "p_gold", r=1.0, n=2)
    g.limb([at(0, -0.050), at(0, -0.022), at(0, 0.006), at(0, 0.034), at(0, 0.060)], [0.021, 0.024, 0.024, 0.023, 0.020],
           ["brown", "p_boot", "brown", "p_boot"], sides=6, smooth=True)  # 감은 손잡이
    g.blob(at(0, 0.078), (0.062, 0.050, 0.062), "p_gold", r=1.0, n=2)  # 자루 끝

    # --- 망치(종치기): 가로로 누운 통 머리(양 끝 금빛 테 · 가운데 띠) + 감은 자루. 머리가 실루엣의 전부다.
    g = m.geo("w_hammer", "hand_r")
    g.limb([at(0, 0.075), at(0, 0.040), at(0, -0.050), at(0, -0.085), at(0, -0.37)], [0.022, 0.025, 0.025, 0.021, 0.024],
           ["brown", "brown", "p_boot", "p_boot"], sides=6, smooth=True)
    g.blob(at(0, 0.090), (0.066, 0.050, 0.066), "p_gold", r=1.0, n=2)  # 자루 끝
    head = at(0, -0.475)
    g.lathe(head, [(0.082, -0.200), (0.112, -0.182), (0.112, -0.128), (0.096, -0.118), (0.096, -0.034), (0.108, -0.026),
                   (0.108, 0.026), (0.096, 0.034), (0.096, 0.118), (0.112, 0.128), (0.112, 0.182), (0.082, 0.200)],
            ["p_gold", "p_gold", "p_steel", "p_steel", "p_gold", "p_gold", "p_gold", "p_steel", "p_steel", "p_gold", "p_gold"],
            sides=10, rot=(0, 90, 0), smooth=True, hard=(2, 4, 7, 9), caps=(True, True), cap_mat="p_steel")
    g.lathe(at(0, -0.37), [(0.034, 0.0), (0.040, 0.012), (0.034, 0.030)], "p_gold", sides=8, rot=(90, 0, 0), smooth=True)  # 목 테
    g.blob(at(0, -0.598), (0.060, 0.044, 0.060), "p_gold", r=1.0, n=2)  # 머리 위 마개

    # --- 지팡이: 긴 대 · 앞 끝은 두 갈래가 하늘빛 결정을 품는다(결정만 각지게) · 감은 천과 늘어진 끝 · 뒤 끝 물미.
    g = m.geo("w_staff", "hand_r")
    g.limb([at(0, 0.34), at(0, 0.06), at(0, -0.06), at(0, -0.60)], [0.019, 0.022, 0.022, 0.024], ["p_boot", "brown", "p_boot"],
           sides=6, smooth=True)
    g.lathe(at(0, 0.34), [(0.026, 0.0), (0.030, -0.030), (0.0, -0.085)], "p_gold", sides=6, rot=(-90, 0, 0), smooth=True)  # 물미
    g.lathe(at(0, -0.585), [(0.030, 0.0), (0.040, 0.014), (0.030, 0.034)], "p_gold", sides=8, rot=(90, 0, 0), smooth=True)
    for sx in (-1, 1):
        g.limb([at(0, -0.61), at(sx * 0.078, -0.655), at(sx * 0.118, -0.745), at(sx * 0.098, -0.835), at(sx * 0.048, -0.880)],
               [0.024, 0.025, 0.022, 0.017, 0.0], "p_gold", sides=6, smooth=True)
    g.lathe(at(0, -0.752), [(0.0, -0.085), (0.056, 0.0), (0.0, 0.085)], "p_sky", sides=4, rot=(90, 0, 0), caps=(False, False))
    g.lathe(at(0, -0.752), [(0.0, -0.040), (0.026, 0.0), (0.0, 0.040)], "p_white", sides=4, rot=(90, 0, 45),
            squash=(1.0, 2.3), caps=(False, False))  # 결정 속의 밝은 심(위에서 보인다)
    g.limb([at(0, -0.500), at(0, -0.545)], [0.031, 0.031], "p_cloak", sides=6, smooth=True)  # 감은 천
    g.limb([at(0.026, -0.52, -0.01), at(0.050, -0.50, -0.07), at(0.042, -0.47, -0.14)], [0.016, 0.014, 0.0], "p_cloak", sides=4,
           flat=0.4)

    # --- 쇠뇌: 굽은 활(끝이 쏘는 이 쪽으로 휜다) · 걸린 시위(V) · 촉과 깃이 달린 볼트 · 개머리가 굵은 몸 · 앞의 발걸이.
    g = m.geo("w_crossbow", "hand_r")
    g.limb([at(0, 0.085), at(0, 0.030), at(0, -0.10), at(0, -0.30), at(0, -0.385)], [0.058, 0.046, 0.040, 0.036, 0.030], "p_boot",
           sides=4, roll=45, flat=0.85, caps=(True, True))  # 몸
    g.box(at(0, 0.092), (0.086, 0.020, 0.074), "brown")  # 개머리 판
    bow = [(-0.315, -0.235), (-0.245, -0.292), (-0.125, -0.338), (0.0, -0.352), (0.125, -0.338), (0.245, -0.292), (0.315, -0.235)]
    g.limb([at(x, y, 0.026) for x, y in bow], [0.013, 0.020, 0.026, 0.030, 0.026, 0.020, 0.013], "p_steel", sides=4, flat=0.6,
           caps=(True, True))
    g.limb([at(-0.312, -0.238, 0.030), at(0, -0.115, 0.046), at(0.312, -0.238, 0.030)], [0.007, 0.007, 0.007], "p_white", sides=4)
    g.box(at(0, -0.115, 0.040), (0.050, 0.044, 0.034), "p_gold")  # 걸쇠
    g.limb([at(0, -0.125, 0.064), at(0, -0.405, 0.064)], [0.011, 0.011], "p_gold", sides=4, roll=45)  # 볼트 대
    g.lathe(at(0, -0.405, 0.064), [(0.026, 0.0), (0.0, 0.085)], "p_steel", sides=4, rot=(90, 0, 0), caps=(True, False))  # 촉
    for sx in (-1, 1):
        g.box(at(sx * 0.018, -0.165, 0.066), (0.030, 0.060, 0.008), "p_cream", rot=(0, sx * 35, 0))  # 깃
    g.limb([at(-0.040, -0.378, 0.02), at(-0.046, -0.430, 0.02), at(0, -0.458, 0.02), at(0.046, -0.430, 0.02), at(0.040, -0.378, 0.02)],
           [0.010, 0.011, 0.011, 0.011, 0.010], "p_steel", sides=4)  # 발걸이
    g.box(at(0, -0.045, -0.050), (0.016, 0.040, 0.050), "p_steel", rot=(25, 0, 0))  # 방아쇠


def make():
    """0038: Meshy 몸 + 코드로 만든 칼·등잔. 뼈 이름·동작 표는 0037과 같다."""
    m = C.Model(NAME)
    m.loc_scale = 0.55
    hx, hy, hz = HAND
    sx, sy, sz = SHOULDER
    m.bone("root", (0, 0, 0))
    m.bone("body", (0, 0, 0.13), "root")
    m.bone("head", (0, -0.02, V2_NECK_Z if BODY == "v2" else 0.62), "body")
    m.bone("arm_l", (sx, sy, sz), "body")
    m.bone("arm_r", (-sx, sy, sz), "body")
    m.bone("hand_r", (-hx, hy, hz), "arm_r")
    m.bone("hand_l", (hx, hy, hz), "arm_l")
    lx_, ly_, lz_ = (0.14, -0.02, V2_LEG_Z) if BODY == "v2" else (0.08, -0.07, 0.125)
    m.bone("leg_l", (lx_, ly_, lz_), "root")
    m.bone("leg_r", (-lx_, ly_, lz_), "root")
    m.bone("cloak", (0, 0.15, 0.50), "body")  # 0037 동작 표가 부르는 뼈 — 새 몸에는 따로 흔들 망토가 없다
    m.bone("tail", (0.08, V2_TAIL_Y, 0.32) if BODY == "v2" else TAIL_AT, "body")
    m.bone("lamp", (hx, hy - 0.05, hz - 0.04), "hand_l")
    m.hide("lamp")
    # 0153 뒷모습의 디테일(게임에서 가장 오래 보는 쪽 — 밋밋한 두건 뒤통수와 옷 등판뿐이었다).
    m.bone("hood_tip", (0.0, 0.25, 1.15) if BODY == "v2" else HOOD_TIP_AT, "head")

    objs = m.external(MESHY_LOW, classify, MESHY_PARTS)
    if BODY == "v1":
        recenter_tail(objs["tail_geo"], objs["body_geo"])
        dress_hood(m, objs["head_geo"])
        # 허리 주머니(왼쪽 엉덩이 뒤 — 꼬리를 옮기며 메운 자리를 덮는다): 갈색 몸 + 크림 덮개 + 금빛 단추.
        g = m.geo("pouch_geo", "body")
        g.blob((0.225, 0.075, 0.265), (0.11, 0.14, 0.125), "p_boot", r=0.6, n=2)
        g.blob((0.232, 0.075, 0.315), (0.115, 0.15, 0.055), "p_cream", r=0.7, n=2)
        g.blob((0.288, 0.075, 0.285), (0.022, 0.04, 0.04), "p_gold", r=0.9, n=1)
    # v2는 원본에 주머니·띠가 있다 — 코드 꼴을 덧붙이지 않는다(0157).
    if BODY == "v2":
        lift_purples(m.ext_img)

    weapons(m, (-hx, hy, hz))

    # 등잔(왼손, 마실 때만 — 0031). 0155: 덩어리 하나 → 배 부른 병 + 금빛 테 + 손잡이 고리, 불꽃은 물방울 꼴.
    lx, ly, lz = hx, hy - 0.06, hz - 0.10
    g = m.geo("lamp_geo", "lamp")
    g.lathe((lx, ly, lz), [(0.036, -0.062), (0.066, -0.048), (0.078, -0.012), (0.062, 0.024), (0.036, 0.040)], "p_lamp",
            sides=8, smooth=True, caps=(True, False))
    g.lathe((lx, ly, lz), [(0.036, 0.040), (0.046, 0.048), (0.046, 0.060), (0.030, 0.064)], "p_gold", sides=8, smooth=True)
    g.limb([(lx + 0.060, ly, lz + 0.018), (lx + 0.105, ly, lz + 0.012), (lx + 0.112, ly, lz - 0.022), (lx + 0.070, ly, lz - 0.034)],
           [0.011, 0.012, 0.012, 0.010], "p_gold", sides=5, smooth=True)
    g = m.geo("glow_lamp", "lamp")
    g.lathe((lx, ly, lz + 0.064), [(0.0, 0.0), (0.026, 0.018), (0.020, 0.044), (0.0, 0.082)], "ember", sides=6, smooth=True,
            caps=(False, False))

    m.build()
    acts(m)
    return m


def make_blocks():
    """0037 코드 모델(둥근 상자). 0038부터 쓰지 않는다 — 비교·되돌리기용으로 남긴다."""
    m = C.Model(NAME)
    # 0037 B "작은 동물 방랑자": 둥근 너구리 얼굴(눈 둘레 무늬·크림 주둥이)·두건 밖 귀·줄무늬 꼬리, 콩 모양 민트 망토, 복숭아 목도리,
    # 짧은 팔·작은 장화, 허리의 작은 금빛 종. 2등신, 눈은 크고 낮게, 입 없음(docs/art/direction.md).
    # 뼈 이름은 그대로(동작 표를 같이 쓴다) + 꼬리. 동작의 위치 이동은 몸에 맞게 줄인다.
    m.loc_scale = 0.55
    m.bone("root", (0, 0, 0))
    m.bone("body", v(0, 0, 0.16), "root")
    m.bone("head", v(0, 0, 0.56), "body")
    m.bone("arm_l", v(0.20, 0, 0.47), "body")
    m.bone("arm_r", v(-0.20, 0, 0.47), "body")
    m.bone("hand_r", v(-0.225, -0.02, 0.29), "arm_r")
    m.bone("leg_l", v(0.09, 0, 0.14), "root")
    m.bone("leg_r", v(-0.09, 0, 0.14), "root")
    m.bone("cloak", v(0, 0.15, 0.55), "body")
    m.bone("tail", v(0, 0.20, 0.22), "body")
    m.bone("hand_l", v(0.225, -0.02, 0.29), "arm_l")
    m.bone("lamp", v(0.225, -0.07, 0.25), "hand_l")
    m.hide("lamp")

    g = m.geo("body_geo", "body")
    g.blob(v(0, 0, 0.36), v(0.42, 0.36, 0.42), "p_cloak", r=0.7, taper=(0.72, 0.76))  # 콩 모양 망토 몸
    g.blob(v(0, 0, 0.17), v(0.43, 0.37, 0.05), "p_cream", r=0.7, n=3)  # 밑단 테
    g.blob(v(0, -0.02, 0.56), v(0.38, 0.32, 0.09), "p_scarf", r=0.7, n=3)  # 목도리
    g.blob(v(0.10, -0.165, 0.47), v(0.08, 0.04, 0.13), "p_scarf", r=0.7, n=2, rot=(0, 10, 0))  # 목도리 끝
    g.blob(v(-0.205, -0.05, 0.21), v(0.07, 0.07, 0.08), "p_gold", r=0.8, n=2, taper=(0.55, 0.55))  # 허리의 작은 종
    g.blob(v(-0.205, -0.05, 0.175), v(0.025, 0.025, 0.025), "p_boot", r=0.9, n=2)  # 종 추

    g = m.geo("head_geo", "head")
    g.blob(v(0, 0.0, 0.82), v(0.66, 0.56, 0.54), "p_hair", r=0.95, n=5)  # 털 얼굴(캐러멜)
    g.blob(v(0, -0.215, 0.72), v(0.30, 0.16, 0.20), "p_cream", r=1.0, n=4)  # 주둥이(작고 둥글게, 아래로)
    for x in (0.165, -0.165):
        g.blob(v(x * 0.85, -0.225, 0.805), v(0.15, 0.05, 0.16), "p_mask", r=1.0, n=3)  # 눈 둘레 무늬(눈보다 조금 크게)
        g.blob(v(x * 0.82, -0.255, 0.80), v(0.105, 0.03, 0.13), "p_eye", r=1.0, n=3)  # 눈(크고 낮게)
        sx = 1 if x > 0 else -1
        g.box(v(x * 0.82 - 0.02 * sx, -0.273, 0.83), v(0.034, 0.01, 0.038), "p_white", drop=("+y",))  # 눈빛
        g.blob(v(x * 1.32, -0.235, 0.715), v(0.07, 0.02, 0.035), "p_blush", r=0.9, n=2)  # 볼
    g.blob(v(0, -0.30, 0.745), v(0.065, 0.04, 0.045), "p_eye", r=1.0, n=2)  # 코
    g.blob(v(0, 0.10, 1.00), v(0.62, 0.44, 0.34), "p_hood", r=1.0, n=4)  # 작은 두건(뒤·위, 둥글게)
    g.blob(v(0, 0.24, 0.92), v(0.40, 0.10, 0.26), "p_hood", r=0.8, n=3)  # 두건 뒷자락
    for x in (0.22, -0.22):
        g.blob(v(x, 0.0, 1.12), v(0.15, 0.10, 0.18), "p_hair", r=0.85, n=3)  # 귀(두건 밖으로)
        g.blob(v(x, -0.05, 1.11), v(0.08, 0.03, 0.10), "p_blush", r=0.9, n=2)  # 귀 안쪽

    for side, sx in (("l", 1), ("r", -1)):
        g = m.geo(f"arm_{side}_geo", f"arm_{side}")
        g.blob(v(0.215 * sx, -0.01, 0.40), v(0.12, 0.13, 0.18), "p_cloak", r=0.8, n=3)  # 짧고 통통한 소매
        g.blob(v(0.225 * sx, -0.02, 0.29), v(0.11, 0.11, 0.09), "p_hair", r=0.95, n=3)  # 앞발(손)

    g = m.geo("sword", "hand_r")
    y0 = -0.02 * S
    hx, hz = -0.225 * S, 0.29 * S
    g.blob((hx, y0 + 0.06, hz), (0.05, 0.04, 0.05), "p_gold", r=0.9, n=2)  # 칼자루 끝 구슬
    g.box((hx, y0, hz), (0.035, 0.09, 0.035), "p_boot")  # 손잡이
    g.blob((hx, y0 - 0.06, hz), (0.15, 0.035, 0.05), "p_gold", r=0.8, n=2)  # 코등이
    g.box((hx, y0 - 0.31, hz), (0.06, 0.46, 0.022), "p_steel")  # 칼날
    g.box((hx, y0 - 0.29, hz), (0.016, 0.38, 0.026), "p_white")  # 날 가운데 밝은 줄
    g.box((hx, y0 - 0.575, hz), (0.06, 0.022, 0.08), "p_steel", rot=(90, 0, 0), taper=(0.15, 1))  # 칼끝

    for side, sx in (("l", 1), ("r", -1)):
        g = m.geo(f"leg_{side}_geo", f"leg_{side}")
        g.blob(v(0.09 * sx, -0.03, 0.06), v(0.15, 0.20, 0.13), "p_boot", r=0.7, n=3)  # 작은 장화(망토 밑으로)

    g = m.geo("tail_geo", "tail")
    for k, (y, z, w, mat) in enumerate(((0.27, 0.21, 0.17, "p_hair"), (0.33, 0.29, 0.16, "p_mask"),
                                        (0.37, 0.38, 0.145, "p_hair"), (0.395, 0.465, 0.12, "p_cream"))):
        g.blob(v(0, y, z), v(w, w, w * 0.8), mat, r=0.95, n=3)  # 줄무늬 꼬리(위로 말림)

    g = m.geo("lamp_geo", "lamp")
    g.blob(v(0.225, -0.08, 0.20), v(0.10, 0.10, 0.10), "p_lamp", r=0.7, n=3, taper=(0.75, 0.75))  # 등잔 몸
    g = m.geo("glow_lamp", "lamp")
    g.box(v(0.225, -0.08, 0.27), v(0.04, 0.04, 0.06), "ember", taper=(0.4, 0.4))  # 불꽃

    g = m.geo("cloak_geo", "cloak")
    g.blob(v(0, 0.16, 0.36), v(0.36, 0.06, 0.38), "p_hood", r=0.5, n=3, taper=(0.8, 1))  # 등 망토

    m.build()
    acts(m)
    return m


def sq(k):
    """찌그러짐·늘어남(0039): 세로 배율 k, 가로·깊이는 1/√k — 부피 그대로(Lasseter 1987). 뼈는 모두 위(+Z)를 향해 뼈의 Y = 세로."""
    w = 1.0 / k ** 0.5
    return (w, k, w)


def skill_pose(drop=0.0, k=1.0, at=(0, 0), up=0.0, yaw=None, body=(3, 0, 0), head=(-2, 0, 0), arm_r=(-12, 6, 0), hand_r=(50, 0, 0),
               arm_l=(-4, -4, 0), feet=((0, 0), (0, 0)), legs=((0, 0, 0), (0, 0, 0)), lift=(0, 0), tail=(0, 0, -13)):
    """스킬 동작의 한 자세(0158 G2). 기본값 = 쉬는 자세(`idle` 0초).
    drop = 몸을 낮춘다 — 뿌리를 내린 만큼 두 다리를 올려 **발은 땅에 남는다**(뿌리만 내리면 발이 땅 밑으로 간다).
    k = 세로 배율(`sq` — 부피 그대로), at = 뿌리의 (x, y) 옮김, up = 뜀, yaw = 온몸 돌기(뿌리의 z 회전).
    feet = (왼발, 오른발)의 (x, y) 옮김, legs = 두 다리의 회전, lift = 발 들기(다리를 돌리면 앞코·뒤꿈치가 내려가는 만큼).
    옮김은 모두 `loc_scale` 앞의 값이다(동작 표의 'l'과 같다)."""
    rise = drop / k if drop > 0 else 0.0
    root = {"l": (at[0], at[1], up - drop), "s": sq(k)}
    if yaw is not None:
        root["r"] = (0, 0, yaw)
    return {"root": root, "body": {"r": body}, "head": {"r": head},
            "arm_r": {"r": arm_r}, "hand_r": {"r": hand_r}, "arm_l": {"r": arm_l},
            "leg_l": {"r": legs[0], "l": (feet[0][0], feet[0][1], rise + lift[0])},
            "leg_r": {"r": legs[1], "l": (feet[1][0], feet[1][1], rise + lift[1])},
            "tail": {"r": tail}}


def acts(m):
    # 기본 자세(쉬기): 칼끝이 앞아래로.
    rest_hand = {"r": (50, 0, 0)}
    # 0039 원칙(KB concepts/believable-cute-character-motion.md): 게임 판정 시각(선딜·활성)은 그대로 두고,
    # 실감은 끝맺음·찌그러짐(부피 그대로)·겹치기로 더한다. 꼬리·머리의 늦음·착지 찌그러짐은 게임의 2차 동작 층(`secondary.rs`)이 얹는다.

    # idle 2.4초 고리(0153 — 예전 1.2초: 숨 한 번에 꼬리 한 번이라 기계처럼 보였다): 숨 두 번, 그 사이 고개가 한쪽씩 갸웃하고
    # 꼬리는 숨보다 느리게 한 번 오간다(박자가 어긋나야 살아 보인다 — KB believable-cute-character-motion "2차 동작").
    idle0 = {"root": {"l": (0, 0, 0), "s": sq(1.0)}, "body": {"r": (3, 0, 0)}, "head": {"r": (-2, 0, 0)},
             "arm_l": {"r": (-4, -4, 0)}, "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand,
             "leg_l": {"r": (0, 0, 0)}, "leg_r": {"r": (0, 0, 0)},
             "cloak": {"r": (6, 0, 0)}, "tail": {"r": (0, 0, -13)}, "hood_tip": {"r": (0, 0, 0)}}
    breath = {"root": {"l": (0, 0, -0.018), "s": sq(1.03)}, "body": {"r": (5, 0, 0)},
              "arm_l": {"r": (-2, -6, 0)}, "arm_r": {"r": (-10, 8, 0)}, "hand_r": {"r": (54, 0, 0)}, "cloak": {"r": (9, 0, 0)}}
    m.action("idle", 2.4, [
        (0.0, idle0),
        (0.6, dict(breath, head={"r": (0, 4, 0)}, tail={"r": (3, 0, 2)}, hood_tip={"r": (5, 0, 3)})),
        (1.2, dict(idle0, head={"r": (-2, 0, 3)}, tail={"r": (0, 0, 14)}, hood_tip={"r": (0, 0, 0)})),
        (1.8, dict(breath, head={"r": (1, -4, 0)}, tail={"r": (-3, 0, 1)}, hood_tip={"r": (5, 0, -3)})),
    ], loop=True)

    # 가만히 서 있으면 가끔 나오는 몸짓 셋(0153 — 게임이 `idle`의 고리 끝에서 하나씩 돌려 가며 튼다, `src/idle.rs`).
    # 셋 다 `idle`의 0초 자세에서 시작해 거기로 돌아온다(이음매 없이). 작은 화면이라 크게(고개 30°·꼬리 30°).
    # ① 둘러보기 2.6초: 왼쪽을 보고 멈췄다 오른쪽 — 몸이 따라 돌고 꼬리·두건 꼬리는 늦게 반대로.
    m.action("idle_look", 2.6, [
        (0.0, idle0),
        (0.38, dict(idle0, head={"r": (-3, 2, 32)}, body={"r": (3, 0, 9)}, tail={"r": (0, 0, -22)}, hood_tip={"r": (0, 0, -14)})),
        (1.0, dict(idle0, head={"r": (-4, 3, 34)}, body={"r": (4, 0, 10)}, tail={"r": (2, 0, -10)}, hood_tip={"r": (2, 0, -3)},
                   root={"l": (0, 0, -0.012), "s": sq(1.02)})),
        (1.4, dict(idle0, head={"r": (-3, -2, -30)}, body={"r": (3, 0, -9)}, tail={"r": (0, 0, 22)}, hood_tip={"r": (0, 0, 14)})),
        (2.05, dict(idle0, head={"r": (-4, -3, -33)}, body={"r": (4, 0, -10)}, tail={"r": (2, 0, 10)}, hood_tip={"r": (2, 0, 3)},
                    root={"l": (0, 0, -0.012), "s": sq(1.02)})),
        (2.45, idle0),
    ])
    # ② 털기 1.3초: 움츠렸다(예비) 젖은 짐승처럼 몸을 좌우로 빠르게 털고 쭉 펴며 끝난다 — 머리는 몸과 반대로, 꼬리가 가장 크게.
    def shake(k):
        return dict(idle0, body={"r": (4, 0, 16 * k)}, head={"r": (0, 0, -14 * k)}, tail={"r": (0, 0, 32 * k)},
                    hood_tip={"r": (0, 0, -26 * k)}, arm_l={"r": (-4, -4 - 10 * abs(k), 0)}, arm_r={"r": (-12, 6 + 10 * abs(k), 0)})
    m.action("idle_shake", 1.3, [
        (0.0, idle0),
        (0.16, dict(idle0, root={"l": (0, 0, -0.03), "s": sq(0.93)}, body={"r": (9, 0, 0)}, head={"r": (7, 0, 0)})),
        (0.30, shake(1.0)), (0.41, shake(-1.0)), (0.52, shake(0.85)), (0.63, shake(-0.7)), (0.74, shake(0.45)),
        (0.92, dict(idle0, root={"l": (0, 0, 0.02), "s": sq(1.06)}, body={"r": (1, 0, 0)}, head={"r": (-8, 0, 0)},
                    tail={"r": (-8, 0, 0)}, hood_tip={"r": (8, 0, 0)})),
        (1.18, idle0),
    ])
    # ③ 종 만지기 2.2초: 왼손으로 가슴의 종을 건드려 보고 고개를 숙여 들여다본다 — 꼬리 끝이 한 번 까딱.
    touch = dict(idle0, arm_l={"r": (-58, -6, -46)}, head={"r": (13, 4, -9)}, body={"r": (6, 0, -4)},
                 tail={"r": (4, 0, 6)}, hood_tip={"r": (9, 0, 2)})
    m.action("idle_bell", 2.2, [
        (0.0, idle0),
        (0.4, touch),
        (0.7, dict(touch, arm_l={"r": (-50, -6, -40)}, head={"r": (15, 5, -10)})),
        (0.95, dict(touch, arm_l={"r": (-62, -6, -48)}, tail={"r": (-6, 0, 16)})),
        (1.45, dict(touch, head={"r": (10, -3, -4)}, tail={"r": (4, 0, 4)})),
        (2.0, idle0),
    ])

    # walk 0.8초 고리: 다리 ±28°, 팔은 반대로, 지날 때 몸이 오른다
    st = 28
    def walk_key(s):
        # 0038: 뒤뚱 — 내딛는 발 쪽으로 몸이 기울고(Y) 머리는 반대로 조금 버틴다, 꼬리는 크게 반대로.
        return {"root": {"l": (0, 0, 0), "s": sq(0.95)}, "body": {"r": (6, -7 * s, 4 * s)},
                "leg_l": {"r": (-st * s, 0, 0)}, "leg_r": {"r": (st * s, 0, 0)},
                "arm_l": {"r": (18 * s, -4, 0)}, "arm_r": {"r": (-16 - 12 * s, 6, 0)}, "hand_r": rest_hand,
                "head": {"r": (-3, 4 * s, -3 * s)}, "cloak": {"r": (14, 0, -3 * s)}, "tail": {"r": (-8, 6 * s, 24 * s)},
                "hood_tip": {"r": (8, 0, -12 * s)}}
    def pass_key():
        return {"root": {"l": (0, 0, 0.05), "s": sq(1.04)}, "body": {"r": (6, 0, 0)}, "leg_l": {"r": (0, 0, 0)},
                "leg_r": {"r": (0, 0, 0)}, "cloak": {"r": (20, 0, 0)}, "head": {"r": (-6, 0, 0)}, "hood_tip": {"r": (-5, 0, 0)}}
    m.action("walk", 0.8, [(0.0, walk_key(1)), (0.2, pass_key()), (0.4, walk_key(-1)), (0.6, pass_key())], loop=True)

    # attack 0.43초: 0~0.08 칼을 오른쪽 뒤로 당김, 0.08~0.18 앞을 가로질러 베기(오른쪽 → 왼쪽, 조금 위에서 아래로), 0.18~0.43 돌아옴
    m.action("attack", 0.43, [
        (0.0, {"body": {"r": (3, 0, 0)}, "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand,
               "arm_l": {"r": (-4, -4, 0)}}),
        (0.08, {"root": {"l": (0, 0, -0.04), "s": sq(0.92)}, "body": {"r": (4, -6, -38)}, "head": {"r": (0, 0, 20)},
                "arm_r": {"r": (-95, 0, -75)}, "hand_r": {"r": (82, 0, -20)},
                "arm_l": {"r": (-30, -25, 0)}, "leg_l": {"r": (-22, 0, 0)}, "leg_r": {"r": (14, 0, 0)},
                "cloak": {"r": (8, 0, 10)}}),
        (0.13, {"root": {"l": (0, -0.05, -0.06), "s": sq(1.07)}}),
        (0.18, {"root": {"l": (0, -0.05, -0.07), "s": sq(0.95)}, "body": {"r": (14, 6, 40)}, "head": {"r": (0, 0, -24)},
                "arm_r": {"r": (-78, 0, 70)}, "hand_r": {"r": (88, 0, 25)},
                "arm_l": {"r": (15, -30, 0)}, "leg_l": {"r": (-30, 0, 0)}, "leg_r": {"r": (22, 0, 0)},
                "cloak": {"r": (30, 0, -18)}}),
        # 끝맺음: 머리가 몸보다 한 박자 늦게 돌아온다(겹치기) — 0.26에도 아직 벤 쪽을 본다.
        (0.26, {"root": {"l": (0, -0.05, -0.06)}, "body": {"r": (12, 4, 34)}, "head": {"r": (0, 0, -26)},
                "arm_r": {"r": (-62, 0, 62)}, "hand_r": {"r": (80, 0, 20)},
                "arm_l": {"r": (10, -24, 0)}, "leg_l": {"r": (-26, 0, 0)}, "leg_r": {"r": (20, 0, 0)},
                "cloak": {"r": (22, 0, -10)}}),
        (0.43, {"body": {"r": (3, 0, 0)}, "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand,
                "arm_l": {"r": (-4, -4, 0)}, "cloak": {"r": (6, 0, 0)}}),
    ], linear=(0.08,))

    # dodge 0.25초: 낮게 웅크려 앞으로 몸을 던지는 기울기(구르기 비슷)
    m.action("dodge", 0.25, [
        (0.0, {"body": {"r": (3, 0, 0)}, "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand}),
        (0.06, {"root": {"l": (0, -0.05, -0.28), "s": sq(1.08)}, "body": {"r": (48, 0, 0)}, "head": {"r": (10, 0, 0)},
                "leg_l": {"r": (-62, 0, 0)}, "leg_r": {"r": (38, 0, 0)},
                "arm_l": {"r": (38, -15, 0)}, "arm_r": {"r": (40, 15, 0)}, "hand_r": {"r": (100, 0, 0)},
                "cloak": {"r": (18, 0, 0)}}),
        (0.15, {"root": {"l": (0, -0.08, -0.30), "s": sq(0.88)}, "body": {"r": (50, 0, 0)}, "head": {"r": (4, 0, 0)},
                "leg_l": {"r": (-58, 0, 0)}, "leg_r": {"r": (42, 0, 0)},
                "arm_l": {"r": (-10, -20, 0)}, "arm_r": {"r": (-20, 20, 0)}, "hand_r": {"r": (90, 0, 0)},
                "cloak": {"r": (22, 0, 0)}}),
        (0.25, {"root": {"l": (0, 0, -0.06)}, "body": {"r": (14, 0, 0)}, "head": {"r": (0, 0, 0)},
                "leg_l": {"r": (-14, 0, 0)}, "leg_r": {"r": (10, 0, 0)},
                "arm_l": {"r": (-6, -6, 0)}, "arm_r": {"r": (-14, 8, 0)}, "hand_r": rest_hand,
                "cloak": {"r": (24, 0, 0)}}),
    ])

    # hurt 0.35초: 뒤로 젖혀 밀림
    m.action("hurt", 0.35, [
        (0.0, {"body": {"r": (3, 0, 0)}, "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand}),
        (0.06, {"root": {"l": (0, 0.06, -0.03), "s": sq(0.86)}, "body": {"r": (-22, 0, 6)}, "head": {"r": (-28, 0, 0)},
                "arm_l": {"r": (-30, -38, 0)}, "arm_r": {"r": (-40, 35, 0)}, "hand_r": {"r": (30, 0, 0)},
                "leg_l": {"r": (-10, 0, 0)}, "leg_r": {"r": (12, 0, 0)}, "cloak": {"r": (-10, 0, 0)}}),
        (0.16, {"root": {"l": (0, 0.05, -0.05), "s": sq(1.05)}, "body": {"r": (-14, 0, 4)}, "head": {"r": (-12, 0, 0)},
                "arm_l": {"r": (-20, -28, 0)}, "arm_r": {"r": (-30, 25, 0)}, "hand_r": {"r": (38, 0, 0)},
                "leg_l": {"r": (-8, 0, 0)}, "leg_r": {"r": (10, 0, 0)}, "cloak": {"r": (-4, 0, 0)}}),
        (0.35, {"body": {"r": (3, 0, 0)}, "arm_l": {"r": (-4, -4, 0)}, "arm_r": {"r": (-12, 6, 0)},
                "hand_r": rest_hand, "cloak": {"r": (6, 0, 0)}}),
    ])

    # 0034 turn_r / turn_l 0.25초: 몸을 낮췄다가(0.06) 윗몸이 도는 쪽으로 먼저 휙 감기고(0.13) 몸이 따라와 선다.
    # 게임은 같은 동안 몸 방향을 천천히-빠르게-천천히 돌린다(world `PState::Turn`). r의 z: + = 위에서 보아 반시계(왼쪽으로).
    def turn_keys(side):  # side: -1 오른쪽, +1 왼쪽
        return [
            (0.0, {"body": {"r": (3, 0, 0)}, "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand}),
            (0.06, {"root": {"l": (0, 0, -0.28), "s": sq(0.88)}, "body": {"r": (30, 0, 8 * side)}, "head": {"r": (10, 0, 0)},
                    "leg_l": {"r": (-40, -16, 0)}, "leg_r": {"r": (30, 16, 0)},
                    "arm_l": {"r": (-40, -20, 20)}, "arm_r": {"r": (-44, 20, -20)}, "hand_r": {"r": (95, 0, 0)},
                    "cloak": {"r": (18, 0, 0)}}),
            (0.13, {"root": {"l": (0, 0, -0.26), "s": sq(1.05)}, "body": {"r": (24, 0, 48 * side)},
                    "head": {"r": (6, 0, 20 * side)},
                    "leg_l": {"r": (-36, -16, 0)}, "leg_r": {"r": (28, 16, 0)},
                    "arm_l": {"r": (-30, -40, 60 * side)}, "arm_r": {"r": (-34, 40, 60 * side)}, "hand_r": {"r": (95, 0, 0)},
                    "cloak": {"r": (40, 0, -40 * side)}}),
            (0.25, {"root": {"l": (0, 0, -0.04)}, "body": {"r": (6, 0, 6 * side)}, "head": {"r": (0, 0, 0)},
                    "leg_l": {"r": (-4, -6, 0)}, "leg_r": {"r": (4, 6, 0)},
                    "arm_l": {"r": (-6, -6, 0)}, "arm_r": {"r": (-14, 8, 0)}, "hand_r": rest_hand,
                    "cloak": {"r": (14, 0, -10 * side)}}),
        ]
    m.action("turn_r", 0.25, turn_keys(-1), linear=(0.06,))
    m.action("turn_l", 0.25, turn_keys(1), linear=(0.06,))

    # 0034 락온 걸음: 칼을 앞에 겨눈 채 옆으로 게걸음(다리를 벌렸다 모은다), 뒷걸음. 0.6초 고리.
    guard_arm = {"arm_r": {"r": (-62, 4, -22)}, "hand_r": {"r": (22, 0, -16)}, "arm_l": {"r": (-30, -18, 10)}}
    def strafe_key(open_, side):
        k = {"root": {"l": (0, 0, -0.03 if open_ else 0.0)}, "body": {"r": (8, 0, 6 * side)},
             "head": {"r": (0, 0, -4 * side)},
             "leg_l": {"r": (0, (-16 if open_ else 4), 0)}, "leg_r": {"r": (0, (16 if open_ else -4), 0)},
             "cloak": {"r": (12, (8 * side if open_ else 0), 0)}}
        k.update(guard_arm)
        return k
    for name, side in (("strafe_l", 1), ("strafe_r", -1)):
        m.action(name, 0.6, [(0.0, strafe_key(False, side)), (0.15, strafe_key(True, side)),
                             (0.3, strafe_key(False, side)), (0.45, strafe_key(True, side))], loop=True)
    def back_key(sw):
        k = {"root": {"l": (0, 0, 0)}, "body": {"r": (2, 0, 3 * sw)},
             "leg_l": {"r": (20 * sw, 0, 0)}, "leg_r": {"r": (-20 * sw, 0, 0)},
             "head": {"r": (0, 0, 0)}, "cloak": {"r": (6, 0, 0)}}
        k.update(guard_arm)
        return k
    m.action("walk_back", 0.7, [(0.0, back_key(1)), (0.175, dict(back_key(0), root={"l": (0, 0, 0.025)})),
                                (0.35, back_key(-1)), (0.525, dict(back_key(0), root={"l": (0, 0, 0.025)}))], loop=True)


    # 0025·0027 동작. 손 자세 기준은 위와 같다(월드 축 오일러, X− = 팔을 앞으로 든다).
    # parry 0.3초: 칼을 가슴 앞에 가로로 들어 막는다(왼팔은 칼등을 받친다).
    # 2026-11-06: 몸을 칼 쪽으로 조금 더 숙이고 무게를 앞발로(뒤로 물러나는 느낌이 나지 않게 — 받아 내는 자세).
    guard = {"root": {"l": (0, -0.03, -0.05)}, "body": {"r": (12, 0, -14)}, "head": {"r": (-8, 0, 8)},
             "arm_r": {"r": (-82, 0, -38)}, "hand_r": {"r": (8, 0, -78)},
             "arm_l": {"r": (-62, -14, 30)}, "leg_l": {"r": (-16, 0, 0)}, "leg_r": {"r": (12, 0, 0)},
             "cloak": {"r": (10, 0, 0)}}
    m.action("parry", 0.3, [
        (0.0, {"body": {"r": (3, 0, 0)}, "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand}),
        (0.05, guard),
        (0.3, guard),
    ])

    # jump 0.3초(체공 동안 첫 자세를 쥔다): 다리를 접고 팔을 벌려 균형.
    air = {"root": {"l": (0, 0, 0.02)}, "body": {"r": (12, 0, 0)}, "head": {"r": (-6, 0, 0)},
           "leg_l": {"r": (-58, 0, 0)}, "leg_r": {"r": (-24, 0, 0)},
           "arm_l": {"r": (-34, -34, 0)}, "arm_r": {"r": (-28, 30, 0)}, "hand_r": {"r": (62, 0, 0)},
           "cloak": {"r": (38, 0, 0)}}
    m.action("jump", 0.3, [(0.0, air), (0.3, air)])

    # plunge 0.3초: 두 손으로 칼을 머리 위로 들었다가(0.1) 아래로 내리꽂는다.
    m.action("plunge", 0.3, [
        (0.0, {"body": {"r": (-6, 0, 0)}, "head": {"r": (-10, 0, 0)},
               "arm_r": {"r": (-168, 8, 0)}, "arm_l": {"r": (-164, -8, 0)}, "hand_r": {"r": (-20, 0, 0)},
               "leg_l": {"r": (-40, 0, 0)}, "leg_r": {"r": (-20, 0, 0)}, "cloak": {"r": (30, 0, 0)}}),
        (0.12, {"body": {"r": (-10, 0, 0)}, "head": {"r": (-12, 0, 0)},
                "arm_r": {"r": (-175, 8, 0)}, "arm_l": {"r": (-170, -8, 0)}, "hand_r": {"r": (-30, 0, 0)},
                "leg_l": {"r": (-46, 0, 0)}, "leg_r": {"r": (-24, 0, 0)}, "cloak": {"r": (40, 0, 0)}}),
        (0.3, {"root": {"l": (0, -0.04, -0.12)}, "body": {"r": (34, 0, 0)}, "head": {"r": (14, 0, 0)},
               "arm_r": {"r": (-58, 8, 0)}, "arm_l": {"r": (-54, -8, 0)}, "hand_r": {"r": (118, 0, 0)},
               "leg_l": {"r": (-34, 0, 0)}, "leg_r": {"r": (18, 0, 0)}, "cloak": {"r": (12, 0, 0)}}),
    ])

    # 0030 강공격. charge 0.3초: 평타 끝(칼이 왼쪽으로 쓸려 간 자세)에서 칼을 오른쪽 어깨 뒤로 높이 끌어올리고 몸을 비틀어 낮춘다,
    # 0.2초부터는 그 자세를 쥔다(모으는 동안). 첫 키는 평타 0.18초와 같아 이어진다.
    # 0034: 두 손으로 칼을 오른 어깨 뒤 높이 들고 몸을 낮게 비틀어 감는다(왼손이 칼자루로 온다), 망토가 뒤로 날린다.
    windup = {"root": {"l": (0, 0.04, -0.20)}, "body": {"r": (14, -10, -60)}, "head": {"r": (-4, 0, 36)},
              "arm_r": {"r": (-162, 0, -62)}, "hand_r": {"r": (30, 0, -40)},
              "arm_l": {"r": (-150, 10, -70)}, "leg_l": {"r": (-34, -12, 0)}, "leg_r": {"r": (24, 12, 0)},
              "cloak": {"r": (28, 0, 20)}}
    m.action("charge", 0.3, [
        (0.0, {"root": {"l": (0, -0.05, -0.07)}, "body": {"r": (14, 6, 40)}, "head": {"r": (0, 0, -24)},
               "arm_r": {"r": (-78, 0, 70)}, "hand_r": {"r": (88, 0, 25)},
               "arm_l": {"r": (15, -30, 0)}, "leg_l": {"r": (-30, 0, 0)}, "leg_r": {"r": (22, 0, 0)},
               "cloak": {"r": (30, 0, -18)}}),
        (0.2, windup),
        (0.3, windup),
    ])

    # heavy 0.58초(선딜 0.06 + 활성 0.12 + 후딜 0.4): 끌어올린 칼을 크게 앞을 가로질러 낮게 내려친다(평타보다 넓고 깊게),
    # 끝나면 칼끝이 땅 가까이에 머물렀다(무게) 천천히 돌아온다.
    m.action("heavy", 0.58, [
        (0.0, windup),
        (0.06, {"root": {"l": (0, 0.02, -0.22), "s": sq(0.9)}, "body": {"r": (16, -12, -66)}, "head": {"r": (-2, 0, 40)},
                "arm_r": {"r": (-170, 0, -70)}, "hand_r": {"r": (26, 0, -44)},
                "arm_l": {"r": (-158, 10, -76)}, "leg_l": {"r": (-36, -12, 0)}, "leg_r": {"r": (26, 12, 0)},
                "cloak": {"r": (32, 0, 22)}}),
        # 0.18: 앞으로 크게 내디디며 낮게 내리친다(칼끝이 땅 가까이, 두 손).
        (0.12, {"root": {"l": (0, -0.08, -0.24), "s": sq(1.1)}}),
        (0.18, {"root": {"l": (0, -0.18, -0.24), "s": sq(0.84)}, "body": {"r": (34, 10, 46)}, "head": {"r": (14, 0, -30)},
                "arm_r": {"r": (-42, 0, 88)}, "hand_r": {"r": (108, 0, 42)},
                "arm_l": {"r": (-48, -10, 70)}, "leg_l": {"r": (-52, 0, 0)}, "leg_r": {"r": (36, 0, 0)},
                "cloak": {"r": (44, 0, -28)}}),
        (0.36, {"root": {"l": (0, -0.17, -0.22), "s": sq(0.95)}, "body": {"r": (30, 8, 42)}, "head": {"r": (8, 0, -22)},
                "arm_r": {"r": (-36, 0, 84)}, "hand_r": {"r": (110, 0, 38)},
                "arm_l": {"r": (-40, -10, 64)}, "leg_l": {"r": (-50, 0, 0)}, "leg_r": {"r": (34, 0, 0)},
                "cloak": {"r": (30, 0, -16)}}),
        (0.58, {"body": {"r": (3, 0, 0)}, "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand,
                "arm_l": {"r": (-4, -4, 0)}, "cloak": {"r": (6, 0, 0)}}),
    ], linear=(0.06,))

    # 0031 drink 0.8초: 왼손을 들어(0.35초에 입에 닿음 = 삼키는 순간) 고개를 젖혔다가 내린다. 칼은 오른손에 쉬는 자세 그대로.
    sip = {"body": {"r": (-4, 0, 0)}, "head": {"r": (-24, 0, 0)},
           "arm_l": {"r": (-112, -6, -40)}, "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand,
           "cloak": {"r": (4, 0, 0)}}
    m.action("drink", 0.8, [
        (0.0, {"lamp": {"s": 0.0}, "body": {"r": (3, 0, 0)}, "arm_l": {"r": (-4, -4, 0)}, "arm_r": {"r": (-12, 6, 0)},
               "hand_r": rest_hand, "cloak": {"r": (6, 0, 0)}}),
        (0.05, {"lamp": {"s": 1.0}}),
        (0.25, {"body": {"r": (2, 0, 0)}, "head": {"r": (-8, 0, 0)},
                "arm_l": {"r": (-95, -6, -30)}, "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand}),
        (0.35, dict(sip, root={"s": sq(0.94)})),
        (0.45, dict(sip, root={"s": sq(1.0)})),
        (0.6, dict(sip, root={"s": sq(1.05)}, head={"r": (-30, 0, 0)})),
        (0.75, {"lamp": {"s": 1.0}}),
        (0.8, {"lamp": {"s": 0.0}, "body": {"r": (3, 0, 0)}, "head": {"r": (0, 0, 0)}, "arm_l": {"r": (-4, -4, 0)},
               "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand, "cloak": {"r": (6, 0, 0)}}),
    ])

    # 0048 cast 0.4초(쏘기 0.12 = 게임 `cast.startup s`, 끝 0.4 = `cast.time s`): 빈 왼손으로 쏜다, 칼은 오른손에 쉬는 자세.
    # KB believable-cute-character-motion: 예비는 선딜 안에서만 짧게(0.06에 손을 가슴으로 모으며 몸을 비틀어 조금 웅크림),
    # 0.12에 손바닥을 앞으로 쭉 — 몸이 손 쪽으로 늘어나고(부피 그대로), 0.18에 반동(몸이 살짝 뒤로·손이 들림), 머리는 한 박자 늦게
    # 앞을 보다 돌아온다(겹치기), 망토·꼬리는 2차 동작 층이 얹는다.
    gather = {"root": {"l": (0, 0.02, -0.04), "s": sq(0.93)}, "body": {"r": (6, 4, 22)}, "head": {"r": (-4, 0, -8)},
              "arm_l": {"r": (-70, -10, -55)}, "arm_r": {"r": (-6, 10, 0)}, "hand_r": rest_hand,
              "leg_l": {"r": (-10, 0, 0)}, "leg_r": {"r": (8, 0, 0)}, "cloak": {"r": (10, 0, -6)}}
    # 과장(KB: 게임 카메라 거리에서 읽히게): 쿼터뷰 위에서는 큰 두건이 가슴 앞 손을 덮는다 — 손을 어깨 위 앞으로 높이 뻗는다.
    thrust = {"root": {"l": (0, -0.05, -0.02), "s": sq(1.08)}, "body": {"r": (18, 0, -14)}, "head": {"r": (-2, 0, 6)},
              "arm_l": {"r": (-118, -6, 10)}, "arm_r": {"r": (10, 14, 0)}, "hand_r": {"r": (40, 0, 0)},
              "leg_l": {"r": (-20, 0, 0)}, "leg_r": {"r": (16, 0, 0)}, "cloak": {"r": (4, 0, 6)}}
    recoil = {"root": {"l": (0, 0.02, -0.02), "s": sq(0.96)}, "body": {"r": (-4, 0, -6)}, "head": {"r": (-6, 0, 8)},
              "arm_l": {"r": (-128, -4, 8)}, "arm_r": {"r": (4, 12, 0)}, "hand_r": {"r": (44, 0, 0)},
              "leg_l": {"r": (-16, 0, 0)}, "leg_r": {"r": (12, 0, 0)}, "cloak": {"r": (14, 0, 4)}}
    m.action("cast", 0.4, [
        (0.0, {"body": {"r": (3, 0, 0)}, "arm_l": {"r": (-4, -4, 0)}, "arm_r": {"r": (-12, 6, 0)},
               "hand_r": rest_hand, "cloak": {"r": (6, 0, 0)}}),
        (0.06, gather),
        (0.12, thrust),
        (0.18, recoil),
        (0.28, dict(recoil, root={"l": (0, 0, -0.01), "s": sq(1.01)}, body={"r": (2, 0, -3)},
                    arm_l={"r": (-80, -6, 0)})),
        (0.4, {"root": {"l": (0, 0, 0), "s": sq(1.0)}, "body": {"r": (3, 0, 0)}, "head": {"r": (0, 0, 0)},
               "arm_l": {"r": (-4, -4, 0)}, "arm_r": {"r": (-12, 6, 0)}, "hand_r": rest_hand,
               "leg_l": {"r": (0, 0, 0)}, "leg_r": {"r": (0, 0, 0)}, "cloak": {"r": (6, 0, 0)}}),
    ], linear=(0.06,))

    # ── 0051 무기별 동작. 판정 시각은 게임(랩 평타·강타 × 무기 표 `time`)이 정한다: 평타 선딜 0.08·활성 0.10·후딜 0.25 기준.
    # 원칙(KB believable-cute-character-motion): 예비는 선딜 안에서 짧게, 무게는 끝맺음 길이로(망치 길게·지팡이 짧게),
    # 부피 지키는 찌그러짐(sq), 머리는 한 박자 늦게(겹치기).
    rest = {"body": {"r": (3, 0, 0)}, "head": {"r": (0, 0, 0)}, "arm_l": {"r": (-4, -4, 0)}, "arm_r": {"r": (-12, 6, 0)},
            "hand_r": rest_hand, "leg_l": {"r": (0, 0, 0)}, "leg_r": {"r": (0, 0, 0)}, "cloak": {"r": (6, 0, 0)},
            "root": {"l": (0, 0, 0), "s": sq(1.0)}}

    # 검 2타(0.43초): 1타가 끝난 왼쪽에서 되돌려 오른쪽으로 가로 베기.
    m.action("attack2", 0.43, [
        (0.0, {"root": {"l": (0, -0.05, -0.07)}, "body": {"r": (14, 6, 40)}, "head": {"r": (0, 0, -24)},
               "arm_r": {"r": (-78, 0, 70)}, "hand_r": {"r": (88, 0, 25)}, "arm_l": {"r": (15, -30, 0)}}),
        (0.08, {"root": {"l": (0, -0.04, -0.06), "s": sq(0.93)}, "body": {"r": (10, 4, 46)}, "head": {"r": (0, 0, -20)},
                "arm_r": {"r": (-88, 0, 80)}, "hand_r": {"r": (90, 0, 30)}}),
        (0.18, {"root": {"l": (0, -0.06, -0.07), "s": sq(1.05)}, "body": {"r": (12, -6, -40)}, "head": {"r": (0, 0, 18)},
                "arm_r": {"r": (-92, 0, -72)}, "hand_r": {"r": (84, 0, -22)}, "arm_l": {"r": (-28, -22, 0)},
                "leg_l": {"r": (-26, 0, 0)}, "leg_r": {"r": (18, 0, 0)}, "cloak": {"r": (24, 0, 16)}}),
        (0.27, {"body": {"r": (10, -4, -34)}, "head": {"r": (0, 0, 22)}, "arm_r": {"r": (-70, 0, -60)},
                "hand_r": {"r": (80, 0, -18)}}),
        (0.43, rest),
    ], linear=(0.08,))

    # 검 3타(0.52초, time 1.2): 칼을 옆구리로 당겼다(예비) 내디디며 곧게 찌른다 — 몸이 앞으로 길게 늘어나고 끝에 멈칫.
    m.action("attack3", 0.52, [
        (0.0, {"body": {"r": (10, -4, -30)}, "arm_r": {"r": (-70, 0, -60)}, "hand_r": {"r": (80, 0, -18)}}),
        (0.09, {"root": {"l": (0, 0.04, -0.05), "s": sq(0.9)}, "body": {"r": (6, 0, 18)}, "head": {"r": (-4, 0, -10)},
                "arm_r": {"r": (-40, 20, 30)}, "hand_r": {"r": (100, 0, 0)}, "arm_l": {"r": (-30, -20, 0)},
                "leg_l": {"r": (-10, 0, 0)}, "leg_r": {"r": (14, 0, 0)}}),
        (0.18, {"root": {"l": (0, -0.10, -0.06), "s": sq(1.12)}, "body": {"r": (26, 0, -6)}, "head": {"r": (6, 0, 4)},
                "arm_r": {"r": (-92, 4, 4)}, "hand_r": {"r": (92, 0, 0)}, "arm_l": {"r": (20, -30, 0)},
                "leg_l": {"r": (-40, 0, 0)}, "leg_r": {"r": (30, 0, 0)}, "cloak": {"r": (34, 0, 0)}}),
        (0.3, {"root": {"l": (0, -0.09, -0.06), "s": sq(0.97)}, "body": {"r": (22, 0, -4)}, "head": {"r": (10, 0, 2)},
               "arm_r": {"r": (-88, 4, 4)}, "hand_r": {"r": (94, 0, 0)}}),
        (0.52, rest),
    ], linear=(0.09,))

    # 망치 1타(0.73초, time 1.7): 오른 어깨 위로 크게 들어(예비가 선딜 안에서 가장 길다) 앞을 가로질러 내리친다,
    # 머리 무게에 몸이 끌려 한 박자 늦게 선다(긴 끝맺음).
    m.action("hammer1", 0.73, [
        (0.0, rest),
        (0.13, {"root": {"l": (0, 0.03, -0.06), "s": sq(0.9)}, "body": {"r": (-4, -8, -40)}, "head": {"r": (-6, 0, 24)},
                "arm_r": {"r": (-160, 0, -40)}, "hand_r": {"r": (20, 0, -30)}, "arm_l": {"r": (-140, 10, -50)},
                "leg_l": {"r": (-16, 0, 0)}, "leg_r": {"r": (12, 0, 0)}, "cloak": {"r": (20, 0, 14)}}),
        (0.2, {"root": {"l": (0, -0.08, -0.10), "s": sq(1.1)}}),
        (0.3, {"root": {"l": (0, -0.12, -0.14), "s": sq(0.84)}, "body": {"r": (30, 8, 34)}, "head": {"r": (14, 0, -24)},
               "arm_r": {"r": (-50, 0, 70)}, "hand_r": {"r": (110, 0, 30)}, "arm_l": {"r": (-50, -10, 60)},
               "leg_l": {"r": (-44, 0, 0)}, "leg_r": {"r": (32, 0, 0)}, "cloak": {"r": (40, 0, -20)}}),
        (0.5, {"root": {"l": (0, -0.11, -0.12), "s": sq(0.96)}, "body": {"r": (26, 6, 30)}, "head": {"r": (10, 0, -20)},
               "arm_r": {"r": (-44, 0, 66)}, "hand_r": {"r": (112, 0, 28)}, "arm_l": {"r": (-44, -10, 56)}}),
        (0.73, rest),
    ], linear=(0.13,))

    # 망치 2타(0.86초, time 2.0): 끌린 머리를 반대로 크게 휘둘러 한 바퀴 — 몸 둘레(원) 판정.
    m.action("hammer2", 0.86, [
        (0.0, {"body": {"r": (26, 6, 30)}, "arm_r": {"r": (-44, 0, 66)}, "hand_r": {"r": (112, 0, 28)},
               "arm_l": {"r": (-44, -10, 56)}}),
        (0.16, {"root": {"l": (0, 0.0, -0.10), "s": sq(0.9)}, "body": {"r": (14, 0, 70)}, "head": {"r": (0, 0, -30)},
                "arm_r": {"r": (-90, 0, 90)}, "hand_r": {"r": (90, 0, 40)}, "arm_l": {"r": (-80, 0, 80)},
                "leg_l": {"r": (-20, 0, 0)}, "leg_r": {"r": (20, 0, 0)}}),
        (0.26, {"root": {"l": (0, -0.04, -0.12), "s": sq(1.06)}, "body": {"r": (14, 0, -20)}, "head": {"r": (0, 0, 10)},
                "arm_r": {"r": (-90, 0, -40)}, "hand_r": {"r": (90, 0, -20)}, "arm_l": {"r": (-80, 0, -30)},
                "cloak": {"r": (36, 0, 20)}}),
        (0.36, {"root": {"l": (0, -0.08, -0.12), "s": sq(0.92)}, "body": {"r": (16, 0, -80)}, "head": {"r": (0, 0, 30)},
                "arm_r": {"r": (-84, 0, -90)}, "hand_r": {"r": (90, 0, -40)}, "arm_l": {"r": (-74, 0, -80)},
                "cloak": {"r": (40, 0, 28)}}),
        (0.6, {"root": {"l": (0, -0.06, -0.10), "s": sq(0.98)}, "body": {"r": (12, 0, -60)}, "head": {"r": (0, 0, 36)},
               "arm_r": {"r": (-60, 0, -70)}, "hand_r": {"r": (100, 0, -30)}}),
        (0.86, rest),
    ], linear=(0.16,))

    # 망치 모으기(쥔다): 머리를 등 뒤로 넘겨 두 손으로 쥐고 낮게 웅크린다(곧 뛰어오를 몸).
    hwind = {"root": {"l": (0, 0.05, -0.22), "s": sq(0.86)}, "body": {"r": (-10, 0, 0)}, "head": {"r": (-12, 0, 0)},
             "arm_r": {"r": (-176, 0, -12)}, "hand_r": {"r": (-30, 0, 0)}, "arm_l": {"r": (-172, 0, 12)},
             "leg_l": {"r": (-40, 0, 0)}, "leg_r": {"r": (26, 0, 0)}, "cloak": {"r": (30, 0, 0)}}
    m.action("hammer_charge", 0.3, [(0.0, rest), (0.2, hwind), (0.3, hwind)])
    # 망치 내려찍기(0.75초, time 1.3): 뛰어올라(늘어남) 머리 위에서 그대로 땅에 박는다(크게 찌그러짐), 오래 박힌 채 있다가 뽑는다.
    m.action("hammer_slam", 0.75, [
        (0.0, hwind),
        (0.06, {"root": {"l": (0, -0.04, 0.10), "s": sq(1.14)}, "body": {"r": (-14, 0, 0)}, "head": {"r": (-14, 0, 0)},
                "arm_r": {"r": (-180, 0, -8)}, "hand_r": {"r": (-34, 0, 0)}, "arm_l": {"r": (-178, 0, 8)},
                "leg_l": {"r": (-30, 0, 0)}, "leg_r": {"r": (-10, 0, 0)}, "cloak": {"r": (44, 0, 0)}}),
        (0.18, {"root": {"l": (0, -0.16, -0.24), "s": sq(0.8)}, "body": {"r": (46, 0, 0)}, "head": {"r": (22, 0, 0)},
                "arm_r": {"r": (-40, 0, -8)}, "hand_r": {"r": (130, 0, 0)}, "arm_l": {"r": (-40, 0, 8)},
                "leg_l": {"r": (-50, 0, 0)}, "leg_r": {"r": (36, 0, 0)}, "cloak": {"r": (50, 0, 0)}}),
        (0.5, {"root": {"l": (0, -0.15, -0.22), "s": sq(0.95)}, "body": {"r": (40, 0, 0)}, "head": {"r": (14, 0, 0)},
               "arm_r": {"r": (-44, 0, -8)}, "hand_r": {"r": (126, 0, 0)}, "arm_l": {"r": (-44, 0, 8)}}),
        (0.75, rest),
    ], linear=(0.06,))

    # 지팡이 1·2·3타(0.30·0.30·0.32초, time 0.7~0.75): 짧은 찌르기·되찌르기·올려 치기 — 예비 거의 없이, 끝맺음도 짧게(가벼움).
    m.action("staff1", 0.30, [
        (0.0, rest),
        (0.05, {"root": {"s": sq(0.95)}, "body": {"r": (6, 0, 12)}, "arm_r": {"r": (-60, 0, 20)}, "hand_r": {"r": (90, 0, 10)}}),
        (0.12, {"root": {"l": (0, -0.06, -0.04), "s": sq(1.06)}, "body": {"r": (16, 0, -10)}, "head": {"r": (4, 0, 4)},
                "arm_r": {"r": (-88, 0, -6)}, "hand_r": {"r": (92, 0, 0)}, "arm_l": {"r": (-60, -10, -30)},
                "leg_l": {"r": (-22, 0, 0)}, "leg_r": {"r": (16, 0, 0)}}),
        (0.30, rest),
    ], linear=(0.05,))
    m.action("staff2", 0.30, [
        (0.0, {"body": {"r": (16, 0, -10)}, "arm_r": {"r": (-88, 0, -6)}, "hand_r": {"r": (92, 0, 0)}}),
        (0.05, {"root": {"s": sq(0.95)}, "body": {"r": (8, 0, -24)}, "arm_r": {"r": (-70, 0, -40)}, "hand_r": {"r": (90, 0, -20)}}),
        (0.12, {"root": {"l": (0, -0.05, -0.04), "s": sq(1.06)}, "body": {"r": (14, 0, 22)}, "head": {"r": (2, 0, -8)},
                "arm_r": {"r": (-90, 0, 36)}, "hand_r": {"r": (90, 0, 20)}, "arm_l": {"r": (-50, -10, 40)},
                "leg_l": {"r": (16, 0, 0)}, "leg_r": {"r": (-22, 0, 0)}}),
        (0.30, rest),
    ], linear=(0.05,))
    m.action("staff3", 0.32, [
        (0.0, {"body": {"r": (14, 0, 22)}, "arm_r": {"r": (-90, 0, 36)}, "hand_r": {"r": (90, 0, 20)}}),
        (0.06, {"root": {"l": (0, 0, -0.06), "s": sq(0.92)}, "body": {"r": (20, 0, 0)}, "arm_r": {"r": (-30, 0, 0)},
                "hand_r": {"r": (130, 0, 0)}}),
        (0.13, {"root": {"l": (0, -0.03, 0.02), "s": sq(1.1)}, "body": {"r": (-8, 0, 0)}, "head": {"r": (-10, 0, 0)},
                "arm_r": {"r": (-150, 0, 0)}, "hand_r": {"r": (40, 0, 0)}, "arm_l": {"r": (-120, 0, 10)}}),
        (0.32, rest),
    ], linear=(0.06,))
    # 지팡이 4타(0.56초, time 1.3): 한 바퀴 회전 마무리 — 몸 둘레, 끝에 휘청 멈칫(머리·망토가 늦게 따라온다).
    spin = lambda z: {"body": {"r": (10, 0, z)}, "arm_r": {"r": (-90, 0, 80)}, "hand_r": {"r": (90, 0, 40)},
                      "arm_l": {"r": (-90, 0, -80)}, "cloak": {"r": (40, 0, -z * 0.3)}}
    m.action("staff4", 0.56, [
        (0.0, rest),
        (0.1, dict(spin(60), root={"l": (0, 0, -0.08), "s": sq(0.9)}, head={"r": (0, 0, -20)})),
        (0.17, dict(spin(-60), root={"l": (0, 0, -0.06), "s": sq(1.06)})),
        (0.24, dict(spin(-150), root={"l": (0, 0, -0.06), "s": sq(1.04)}, head={"r": (0, 0, 30)})),
        (0.36, dict(spin(-170), root={"l": (0, 0, -0.08), "s": sq(0.94)}, head={"r": (0, 0, 40)})),
        (0.56, rest),
    ], linear=(0.1, 0.17))
    # 지팡이 모으기·떼기(0.52초, time 0.9): 대를 등 뒤로 가로 쥐고 웅크렸다 떼면 둘레를 쓸어 친다.
    swind = {"root": {"l": (0, 0.02, -0.14), "s": sq(0.9)}, "body": {"r": (10, 0, 50)}, "head": {"r": (0, 0, -26)},
             "arm_r": {"r": (-100, 0, 100)}, "hand_r": {"r": (80, 0, 50)}, "arm_l": {"r": (-90, 0, 80)},
             "leg_l": {"r": (-30, 0, 0)}, "leg_r": {"r": (22, 0, 0)}, "cloak": {"r": (24, 0, -14)}}
    m.action("staff_charge", 0.3, [(0.0, rest), (0.18, swind), (0.3, swind)])
    m.action("staff_release", 0.52, [
        (0.0, swind),
        (0.05, dict(spin(-40), root={"l": (0, -0.04, -0.12), "s": sq(1.1)})),
        (0.16, dict(spin(-160), root={"l": (0, -0.06, -0.12), "s": sq(0.9)}, head={"r": (0, 0, 30)})),
        (0.3, dict(spin(-170), root={"l": (0, -0.05, -0.10), "s": sq(1.0)}, head={"r": (0, 0, 40)})),
        (0.52, rest),
    ], linear=(0.05,))

    # 쇠뇌 쏘기(0.24초, Shot time 0.6): 어깨에 대고(예비 0.04) 쏘는 순간 반동으로 몸이 뒤로 튀고 머리가 늦게 따라 젖혀진다.
    aim = {"body": {"r": (4, 0, -10)}, "head": {"r": (2, 0, 6)}, "arm_r": {"r": (-84, 4, 6)}, "hand_r": {"r": (96, 0, 0)},
           "arm_l": {"r": (-80, -4, -40)}, "leg_l": {"r": (-14, 0, 0)}, "leg_r": {"r": (10, 0, 0)}}
    m.action("xbow_shoot", 0.24, [
        (0.0, rest),
        (0.04, dict(aim, root={"l": (0, 0, -0.02), "s": sq(0.96)})),
        (0.08, dict(aim, root={"l": (0, 0.06, 0.0), "s": sq(1.05)}, body={"r": (-8, 0, -10)}, head={"r": (-6, 0, 6)},
                    arm_r={"r": (-96, 4, 6)})),
        (0.14, dict(aim, root={"l": (0, 0.03, -0.01), "s": sq(0.98)}, head={"r": (-10, 0, 6)})),
        (0.24, rest),
    ])
    # 수류탄 던지기(0.5초, 손을 떠나는 순간 0.2 = `throw.startup s`): 왼손을 뒤로 감았다 아래에서 위로 던져 올린다(포물선).
    m.action("throw", 0.5, [
        (0.0, rest),
        (0.12, {"root": {"l": (0, 0.04, -0.06), "s": sq(0.9)}, "body": {"r": (-6, 0, 30)}, "head": {"r": (-6, 0, -14)},
                "arm_l": {"r": (40, -20, 10)}, "arm_r": {"r": (-20, 10, 0)}, "leg_l": {"r": (-12, 0, 0)},
                "leg_r": {"r": (14, 0, 0)}}),
        (0.2, {"root": {"l": (0, -0.04, 0.0), "s": sq(1.1)}, "body": {"r": (8, 0, -16)}, "head": {"r": (-14, 0, 8)},
               "arm_l": {"r": (-150, -10, 0)}, "arm_r": {"r": (-10, 14, 0)}, "leg_l": {"r": (-24, 0, 0)},
               "leg_r": {"r": (18, 0, 0)}, "cloak": {"r": (24, 0, 0)}}),
        (0.32, {"root": {"l": (0, -0.02, -0.02), "s": sq(0.97)}, "body": {"r": (6, 0, -12)}, "head": {"r": (-18, 0, 8)},
                "arm_l": {"r": (-130, -10, 0)}}),
        (0.5, rest),
    ], linear=(0.12,))

    # 0056 원거리 강공격 모으기(쥔다): 마법 = 두 손을 가슴 앞에 모아 빛을 키우며 낮게 웅크리고 몸을 뒤로 젖혀 버틴다(곧 내지를 몸),
    # 쇠뇌 = 어깨에 대고 한쪽 무릎을 낮춰 시위를 끝까지 당긴 채 숨을 참는다. 0.2초에 자세에 닿고 그다음은 쥔다.
    cgather = {"root": {"l": (0, 0.04, -0.10), "s": sq(0.9)}, "body": {"r": (-6, 0, 16)}, "head": {"r": (-6, 0, -6)},
               "arm_l": {"r": (-82, -10, -60)}, "arm_r": {"r": (-50, 20, 40)}, "hand_r": {"r": (70, 0, 20)},
               "leg_l": {"r": (-22, 0, 0)}, "leg_r": {"r": (16, 0, 0)}, "cloak": {"r": (16, 0, -6)}}
    m.action("cast_charge", 0.3, [(0.0, gather), (0.2, cgather), (0.3, dict(cgather, root={"l": (0, 0.04, -0.11), "s": sq(0.89)}))])
    xpull = {"root": {"l": (0, 0.03, -0.12), "s": sq(0.9)}, "body": {"r": (2, 0, -14)}, "head": {"r": (4, 0, 8)},
             "arm_r": {"r": (-86, 4, 6)}, "hand_r": {"r": (96, 0, 0)}, "arm_l": {"r": (-84, -4, -46)},
             "leg_l": {"r": (-34, 0, 0)}, "leg_r": {"r": (28, 0, 0)}, "cloak": {"r": (12, 0, 0)}}
    m.action("xbow_charge", 0.3, [(0.0, aim), (0.2, xpull), (0.3, xpull)])

    # land 0.15초: 내려앉아 무릎을 굽혔다 편다.
    m.action("land", 0.15, [
        (0.0, {"root": {"l": (0, 0, -0.16)}, "body": {"r": (22, 0, 0)}, "head": {"r": (6, 0, 0)},
               "leg_l": {"r": (-34, 0, 0)}, "leg_r": {"r": (24, 0, 0)},
               "arm_l": {"r": (-20, -30, 0)}, "arm_r": {"r": (-20, 30, 0)}, "hand_r": rest_hand,
               "cloak": {"r": (4, 0, 0)}}),
        (0.15, {"body": {"r": (3, 0, 0)}, "arm_l": {"r": (-4, -4, 0)}, "arm_r": {"r": (-12, 6, 0)},
                "hand_r": rest_hand, "cloak": {"r": (6, 0, 0)}}),
    ])

    # wake 4.0초(0171 — 새 판의 첫 장면, 기억 없이 깨어난다): 엎드려 있다(숨만) → 고개가 들린다 → 두 팔로 밀어 일어나 → 웅크려 앉았다
    # → 일어서며 살짝 휘청 → 쉬는 자세. 생김새는 그대로(0157) — 뼈의 키뿐이다. 뿌리를 앞으로 눕히면(발이 축) 몸 두께만큼 땅에
    # 묻히니 그만큼 올린다. 게임은 0초 자세로 멈춰 두었다가(누운 채) 장면이 흐르면 튼다(`src/wake.rs`).
    prone = {"root": {"r": (86, 0, 0), "l": (0, 0, 0.36), "s": sq(1.0)}, "body": {"r": (2, 0, 0)}, "head": {"r": (-34, 0, 40)},
             "arm_l": {"r": (-128, -34, 0)}, "arm_r": {"r": (-112, 40, 0)}, "hand_r": {"r": (20, 0, 0)},
             "leg_l": {"r": (0, 0, 10)}, "leg_r": {"r": (0, 0, -8)},
             "cloak": {"r": (0, 0, 0)}, "tail": {"r": (0, 0, -26)}, "hood_tip": {"r": (0, 0, 8)}}
    # 0175 fall 1.3초: 쓰러짐 — 맞아 젖혀졌다(0.12) 무릎이 꺾여 주저앉고(0.5) 앞으로 넘어져(0.85) 땅에 닿아 조금 튀었다(1.0)
    # 엎드린 채 가라앉는다(1.3 = `wake`의 0초 자세 그대로 — 어둠 속에서 `wake`로 넘어가도 튀지 않는다, 게임 `fall.rs`).
    m.action("fall", 1.3, [
        (0.0, idle0),
        (0.12, {"root": {"l": (0, 0.05, -0.03), "s": sq(0.9)}, "body": {"r": (-20, 0, 6)}, "head": {"r": (-26, 0, 6)},
                "arm_l": {"r": (-34, -40, 0)}, "arm_r": {"r": (-44, 36, 0)}, "hand_r": {"r": (30, 0, 0)},
                "leg_l": {"r": (-10, 0, 0)}, "leg_r": {"r": (12, 0, 0)}, "cloak": {"r": (-12, 0, 0)},
                "tail": {"r": (0, 0, 14)}}),
        (0.5, {"root": {"r": (14, 0, 4), "l": (0, 0.04, -0.3), "s": sq(0.88)}, "body": {"r": (26, 0, -4)},
               "head": {"r": (14, 0, -8)}, "arm_l": {"r": (-18, -24, 0)}, "arm_r": {"r": (-22, 26, 0)},
               "hand_r": {"r": (50, 0, 0)}, "leg_l": {"r": (-40, 0, 0), "l": (0, 0, 0.22)},
               "leg_r": {"r": (-24, 0, 0), "l": (0, 0, 0.22)}, "cloak": {"r": (14, 0, 0)}, "tail": {"r": (0, 0, -6)},
               "hood_tip": {"r": (8, 0, 0)}}),
        (0.85, dict(prone, root={"r": (78, 0, 0), "l": (0, 0, 0.42), "s": sq(0.93)}, head={"r": (-20, 0, 30)},
                    arm_l={"r": (-140, -40, 0)}, arm_r={"r": (-126, 44, 0)}, tail={"r": (0, 0, -8)})),
        (1.0, dict(prone, root={"r": (90, 0, 0), "l": (0, 0, 0.34), "s": sq(1.05)}, tail={"r": (0, 0, -34)})),
        (1.3, prone),
    ])
    m.action("wake", 4.0, [
        (0.0, prone),
        # 숨: 등이 조금 부푼다.
        (0.45, dict(prone, root={"r": (86, 0, 0), "l": (0, 0, 0.375), "s": sq(1.03)}, tail={"r": (0, 0, -20)})),
        (0.9, prone),
        # 고개가 들리고 꼬리가 먼저 깬다(예비).
        (1.35, dict(prone, head={"r": (-52, 0, 12)}, tail={"r": (6, 0, 18)}, hood_tip={"r": (4, 0, -6)},
                    arm_l={"r": (-112, -26, 0)}, arm_r={"r": (-100, 30, 0)})),
        # 두 팔로 민다: 윗몸이 뜬다.
        (2.0, {"root": {"r": (56, 0, 0), "l": (0, 0.06, 0.16), "s": sq(0.97)}, "body": {"r": (4, 0, 0)}, "head": {"r": (-44, 0, 0)},
               "arm_l": {"r": (-74, -16, 0)}, "arm_r": {"r": (-70, 18, 0)}, "hand_r": {"r": (30, 0, 0)},
               "leg_l": {"r": (-34, 0, 6)}, "leg_r": {"r": (-22, 0, -6)},
               "cloak": {"r": (4, 0, 0)}, "tail": {"r": (-8, 0, 4)}, "hood_tip": {"r": (6, 0, 0)}}),
        # 웅크려 앉는다(무게가 발로 온다).
        (2.6, {"root": {"r": (16, 0, 0), "l": (0, 0.04, -0.22), "s": sq(0.88)}, "body": {"r": (22, 0, 0)}, "head": {"r": (-6, 0, 0)},
               "arm_l": {"r": (-26, -22, 0)}, "arm_r": {"r": (-30, 22, 0)}, "hand_r": {"r": (40, 0, 0)},
               "leg_l": {"r": (-30, 0, 0), "l": (0, 0, 0.25)}, "leg_r": {"r": (-12, 0, 0), "l": (0, 0, 0.25)},
               "cloak": {"r": (10, 0, 0)}, "tail": {"r": (4, 0, -10)}, "hood_tip": {"r": (8, 0, 0)}}),
        # 일어선다 — 늘어났다가(넘침) 살짝 휘청.
        (3.1, dict(idle0, root={"l": (0, 0, 0.03), "s": sq(1.06)}, body={"r": (-5, 0, 0)}, head={"r": (-10, 0, 0)},
                   arm_l={"r": (-2, -14, 0)}, arm_r={"r": (-8, 16, 0)}, tail={"r": (0, 0, 10)})),
        (3.45, dict(idle0, root={"l": (0.02, 0, -0.02), "s": sq(0.98)}, body={"r": (7, 0, 5)}, head={"r": (4, 0, -8)},
                    tail={"r": (0, 0, -18)}, hood_tip={"r": (4, 0, 5)})),
        (3.75, dict(idle0, body={"r": (2, 0, -2)}, head={"r": (-3, 0, 3)})),
        (4.0, idle0),
    ])

    skill_acts(m, idle0)


def skill_acts(m, idle0):
    """0158 G2 스킬 전용 동작 열하나(스킬 여덟 `sk_*` + 큰 기술 셋 `ult_*`). 뼈·부위는 그대로다(0157 — 동작만).
    길이 = 시동 + 이어짐 + 후딜(`skills.rs` 표의 바탕값), **구간의 경계가 판정의 순간** — 그 초에 키가 있고 자세가 거기서 닿는다.
    모두 `idle` 0초 자세에서 시작해 거기로 돌아온다(게임은 동작 사이를 섞지 않는다).
    읽히게 하는 법(KB believable-cute-character-motion · 게임 카메라 = 뒤·위 55°에서 본 띠로 확인):
    - 머리(두건)가 지름 0.8m, 팔은 0.23m다. 손은 어디에 두어도 위에서 두건에 덮인다 — 읽히는 것은 **칼·몸 기울기·꼬리**뿐이다.
    - 칼은 두건 그림자 밖으로: 옆(|x| > 0.4)·뒤·어깨 위 바깥. 몸 앞 낮은 곳은 숙인 머리가 덮는다.
    - 칼끝이 카메라 축(위·뒤 ↔ 아래·앞)을 향하면 점으로 줄어든다 — 든 칼은 위·바깥(조금 앞)으로, 넓은 면은 위로.
    - 몸을 낮출 때는 `skill_pose`의 drop(발이 땅에 남는다). 찌그러짐은 뿌리의 `sq`만.
    - 예비는 시동 안에서 짧게, 판정으로 가는 마지막 구간은 직선(linear), 끝맺음은 넘쳤다 돌아오게, 머리·꼬리는 몸과 반대로 늦게."""
    sp = skill_pose

    # ── 휘돌이 sk_whirl 0.67초(시동 0.15 · 돌기 0.22 · 후딜 0.30): 오른쪽으로 감았다(0.08) 온몸이 왼쪽으로 한 바퀴.
    # 돌기 구간은 0.15초와 0.37초의 자세가 같고(뿌리만 0° → 360°) 각속도가 고르다 — 게임이 그 구간만 되풀이해도 이음매가 없다.
    # 도는 동안: 팔을 넓게 벌려 칼이 수평으로 바깥을 긋고, 몸은 앞·바깥으로 기울고, 꼬리는 뒤로 끌린다. 끝은 미끄러지며 넘쳤다 선다.
    spin = dict(drop=0.10, k=0.95, body=(14, 8, 0), head=(-10, -6, 0), arm_r=(-7, 61, -17), hand_r=(69, 6, -101), arm_l=(4, -78, 23),
                feet=((0.05, 0), (-0.05, 0)), tail=(-34, 0, -38))
    turn = [f / 100 for f in range(0, 22, 3)]  # 0.03초마다 키(그 사이는 직선)
    m.action("sk_whirl", 0.67, [
        (0.0, idle0),
        (0.08, sp(drop=0.16, k=0.88, at=(0, 0.02), body=(10, -8, -52), head=(-4, 0, 30), arm_r=(2, 64, -17), hand_r=(52, 6, -108),
                  arm_l=(-60, -43, 1), feet=((0.03, -0.03), (-0.05, 0.04)), tail=(6, 0, 26))),
        (0.15, sp(yaw=0, **spin)),
    ] + [(0.15 + t, {"root": sp(yaw=360 * t / 0.22, **spin)["root"]}) for t in turn[1:]] + [
        (0.37, sp(yaw=360, **spin)),
        (0.45, sp(drop=0.15, k=0.86, at=(0.03, -0.02), body=(8, -10, 42), head=(-4, 4, -26), arm_r=(-75, 43, 3), hand_r=(75, -20, -27),
                  arm_l=(-1, -47, 11), feet=((0.08, 0.03), (-0.07, -0.04)), tail=(-10, 0, 34))),
        (0.55, sp(drop=0.03, k=1.04, body=(2, 3, -8), head=(-2, 0, 8), arm_r=(-31, 8, 13), hand_r=(54, -3, -15), arm_l=(4, -4, 2),
                  tail=(4, 0, -24))),
        (0.67, idle0),
    ], linear=[0.08] + [0.15 + t for t in turn])

    # ── 가로지르기 sk_cut 0.57초(시동 0.12 · 내달림 0.20 · 후딜 0.25): 칼을 오른 허리 뒤로 당겨 깊이 웅크렸다(0.07),
    # 0.12에 낮게 엎드린 몸이 앞으로 뻗고 칼은 오른쪽 바깥으로 날개처럼 — 달리는 동안(게임이 뿌리를 옮긴다) 그 자세를 쥔 채
    # 칼만 앞-오른쪽에서 뒤-오른쪽으로 끌려 베고, 0.32부터 일어서며 오른쪽 위로 올려 긋는 끝맺음.
    dash = dict(drop=0.14, k=0.9, feet=((0, -0.14), (0, 0.16)), legs=((-20, 0, 0), (26, 0, 0)), lift=(0.02, 0.05))
    m.action("sk_cut", 0.57, [
        (0.0, idle0),
        (0.07, sp(drop=0.18, k=0.84, at=(0, 0.05), body=(22, 0, -38), head=(-12, 0, 30), arm_r=(-2, 32, -6), hand_r=(33, 3, -127),
                  arm_l=(-45, -44, 5), feet=((0, -0.04), (0, 0.06)), tail=(22, 0, 18))),
        (0.12, sp(at=(0, -0.14), body=(40, 0, -10), head=(-28, 0, 8), arm_r=(-94, 66, -55), hand_r=(64, -15, -93), arm_l=(54, -22, 22),
                  tail=(-46, 0, 0), **dash)),
        (0.32, sp(at=(0, -0.16), body=(38, -4, -32), head=(-26, 0, 26), arm_r=(-55, 72, -47), hand_r=(69, 4, -115), arm_l=(67, -5, 32),
                  tail=(-42, 0, 14), **dash)),
        (0.40, sp(drop=0.04, k=1.1, at=(0, -0.10), body=(-4, -6, -52), head=(-4, 0, 34), arm_r=(-113, 63, -64), hand_r=(69, 3, -57),
                  arm_l=(8, -25, 7), feet=((0, -0.08), (0, 0.08)), legs=((-8, 0, 0), (10, 0, 0)), lift=(0, 0.01), tail=(-14, 0, 36))),
        (0.47, sp(drop=0.06, k=0.97, at=(0, -0.06), body=(2, -4, -40), head=(-4, 0, 30), arm_r=(-108, 68, -78), hand_r=(69, 5, -69),
                  arm_l=(1, -19, 4), feet=((0, -0.05), (0, 0.05)), tail=(-4, 0, -6))),
        (0.57, idle0),
    ], linear=(0.07,))

    # ── 끌어모으기 sk_gather 0.50초(시동 0.20 · 후딜 0.30): 두 팔을 앞·바깥으로 던지며 몸이 앞으로 쏠리고(0.10~0.17),
    # 0.20에 홱 — 팔을 가슴으로 당기며 뒤꿈치로 물러앉고 고개를 묻는다(칼은 오른 어깨 뒤로 넘어가고 꼬리가 뒤로 뻗는다),
    # 꼬리가 되튀고(0.28) 앞으로 조금 넘쳤다 선다.
    m.action("sk_gather", 0.50, [
        (0.0, idle0),
        (0.10, sp(drop=0.04, k=1.06, at=(0, -0.08), body=(26, 0, 0), head=(-20, 0, 0), arm_r=(-122, 65, -86), hand_r=(72, -9, -90),
                  arm_l=(-122, -65, 86), feet=((0.03, -0.03), (-0.03, -0.03)), tail=(-18, 0, -8))),
        (0.17, sp(drop=0.05, k=1.08, at=(0, -0.10), body=(32, 0, 0), head=(-24, 0, 0), arm_r=(-128, 67, -109), hand_r=(76, -16, -103),
                  arm_l=(-128, -67, 109), feet=((0.03, -0.03), (-0.03, -0.03)), tail=(-24, 0, -4))),
        (0.20, sp(drop=0.14, k=0.88, at=(0, 0.08), body=(-18, 0, 0), head=(22, 0, 0), arm_r=(-64, 39, 5), hand_r=(-46, 66, -148),
                  arm_l=(-81, -5, -48), feet=((0.02, -0.05), (-0.02, -0.05)), legs=((14, 0, 0), (14, 0, 0)), lift=(0.02, 0.02),
                  tail=(-34, 0, 0))),
        (0.28, sp(drop=0.12, k=0.93, at=(0, 0.07), body=(-12, 0, 0), head=(14, 0, 0), arm_r=(-66, 41, 4), hand_r=(-37, 63, -138),
                  arm_l=(-82, -7, -49), feet=((0.02, -0.04), (-0.02, -0.04)), legs=((10, 0, 0), (10, 0, 0)), lift=(0.02, 0.02),
                  tail=(22, 0, 6))),
        (0.38, sp(drop=0.02, k=1.04, body=(8, 0, 0), head=(-6, 0, 0), arm_r=(-31, 9, 13), hand_r=(44, 2, -22), arm_l=(-20, -3, -9),
                  tail=(-8, 0, -16))),
        (0.50, idle0),
    ], linear=(0.17,))

    # ── 땅울림 sk_quake 0.70초(시동 0.30 · 후딜 0.40): 움츠렸다(0.06) 작게 뛰어 두 손으로 칼을 머리 옆 위로 치켜든다(꼭대기 0.18 —
    # 다리를 접고 몸이 늘어난다), 0.24까지 떠 있다 0.30에 꽂는다: 깊이 찌그러지고 칼이 오른쪽 앞 땅에 눕고 고개가 떨어진다.
    # 0.40까지 박힌 채(꼬리만 되튄다) 있다가 튀어 오르며 선다.
    slam = dict(drop=0.12, at=(0, -0.08), feet=((0.07, 0.02), (-0.07, 0.02)))
    m.action("sk_quake", 0.70, [
        (0.0, idle0),
        (0.06, sp(drop=0.14, k=0.86, body=(12, 0, 0), head=(4, 0, 0), arm_r=(-28, 14, 9), hand_r=(1, -6, -21), arm_l=(-28, -14, -9),
                  tail=(10, 0, 0))),
        (0.18, sp(k=1.14, up=0.34, body=(-16, 0, 0), head=(-6, 0, 0), arm_r=(-123, 73, -119), hand_r=(51, -22, -63),
                  arm_l=(-123, -73, 119), legs=((-46, 0, 0), (-30, 0, 0)), lift=(0.06, 0.04), tail=(-30, 0, 0))),
        (0.24, sp(k=1.08, up=0.30, body=(-12, 0, 0), arm_r=(-118, 72, -110), hand_r=(57, -28, -67), arm_l=(-118, -72, 110),
                  legs=((-40, 0, 0), (-24, 0, 0)), lift=(0.05, 0.03), tail=(-10, 0, 0))),
        (0.30, sp(k=0.78, body=(32, 0, 0), head=(10, 0, 0), arm_r=(-45, 50, -11), hand_r=(59, -14, -78), arm_l=(-45, -50, 11),
                  tail=(-36, 0, 0), **slam)),
        (0.40, sp(k=0.84, body=(30, 0, 0), head=(8, 0, 0), arm_r=(-44, 50, -10), hand_r=(60, -13, -78), arm_l=(-44, -50, 10),
                  tail=(14, 0, 0), **slam)),
        (0.54, sp(drop=0.03, k=1.06, at=(0, -0.03), body=(6, 0, 0), head=(-8, 0, 0), arm_r=(-29, 8, 13), hand_r=(51, 1, -18),
                  arm_l=(-24, -5, -11), tail=(-8, 0, -6))),
        (0.70, idle0),
    ], linear=(0.24,))

    # ── 버팀 sk_brace 1.65초(시동 0.10 · 버팀 1.20 · 후딜 0.35): 0.10에 발을 벌려 낮게 버티고 칼을 가슴 앞에서 오른쪽으로 가로 눕힌다,
    # 고개는 그 뒤로 숙인다. 버티는 1.2초는 숨(0.6초 고리 — 더 낮아졌다 돌아온다)과 잔떨림(0.2초 고리)뿐이라 0.10초와 1.30초의
    # 자세가 같다(게임이 늘려 되풀이해도 된다). 터뜨림은 1.30에 시작해 0.03초 만에 닿는다(되풀이를 지키려고 경계 뒤에 둔다):
    # 몸이 젖혀지며 늘어나고 두 팔이 바깥 위로 터진다, 꼬리가 뒤로 뻗는다 → 내려앉았다 선다.
    def brace(n):
        b = (0, 0.25, 0.75, 1, 0.75, 0.25)[n % 6]  # 숨
        s = 1 if n % 2 else -1  # 떨림
        return sp(drop=0.15 + 0.03 * b, k=0.88 - 0.03 * b, at=(0, -0.02), body=(16 + 3 * b, 1.5 * s, -10), head=(14 + 2 * b, 0, 8 - 2 * s),
                  arm_r=(-104, 11, 68), hand_r=(-114 - 2 * s, 35 + 2 * s, 164 - 2 * s), arm_l=(-89, -55, 22),
                  feet=((0.10, -0.05), (-0.10, 0.06)), tail=(8 + 4 * b, 0, -13 + 7 * s))
    burst = dict(feet=((0.10, -0.03), (-0.10, 0.04)), head=(-16, 0, 0))
    m.action("sk_brace", 1.65, [(0.0, idle0)] + [(round(0.10 + 0.1 * n, 2), brace(n)) for n in range(13)] + [
        (1.33, sp(k=1.16, up=0.03, body=(-20, 0, 0), arm_r=(-120, 72, -113), hand_r=(64, 20, -50), arm_l=(-120, -72, 113),
                  tail=(-36, 0, 0), **burst)),
        (1.44, sp(k=1.05, body=(-14, 0, 0), arm_r=(-75, 79, -79), hand_r=(49, 15, -66), arm_l=(-75, -79, 79), tail=(-10, 0, -8),
                  **burst)),
        (1.54, sp(drop=0.06, k=0.95, body=(8, 0, 0), head=(4, 0, 0), arm_r=(-16, 23, 1), hand_r=(35, -17, -28), arm_l=(-10, -19, 0),
                  feet=((0.05, -0.01), (-0.05, 0.02)), tail=(10, 0, -18))),
        (1.65, idle0),
    ], linear=(0.0, 1.30))

    # ── 되울림 sk_echo 0.45초(시동 0.25 · 후딜 0.20): 칼을 오른 귀 옆에 곧추세우고(0.15 — 얼굴 앞은 뒤에서 두건에 가려 옆으로 뺐다)
    # 왼손은 가슴의 종에(`idle_bell`의 손 자리), 고개를 조금 숙인다. 0.22에 살짝 움츠렸다 0.25에 '땡' — 몸이 톡 솟고 칼이 앞으로
    # 까딱, 고개가 들리고 꼬리가 튄다. 0.29에 되튐, 내리며 선다.
    bell = dict(arm_l=(-58, -6, -46))
    m.action("sk_echo", 0.45, [
        (0.0, idle0),
        (0.15, sp(k=1.05, body=(-6, 0, 4), head=(8, 0, -4), arm_r=(-100, 69, -74), hand_r=(49, -31, -88), tail=(6, 0, 0), **bell)),
        (0.22, sp(drop=0.02, k=0.97, body=(-3, 0, 4), head=(11, 0, -4), arm_r=(-104, 70, -79), hand_r=(42, -29, -93), tail=(10, 0, 4),
                  **bell)),
        (0.25, sp(k=1.12, up=0.04, body=(-10, 0, 4), head=(-6, 0, -4), arm_r=(-95, 69, -66), hand_r=(67, -28, -83), tail=(-14, 0, 18),
                  **bell)),
        (0.29, sp(body=(-4, 0, 4), head=(10, 0, -4), arm_r=(-102, 70, -77), hand_r=(43, -30, -91), tail=(12, 0, -16), **bell)),
        (0.34, sp(k=1.03, body=(-6, 0, 4), head=(4, 0, -4), arm_r=(-100, 69, -74), hand_r=(50, -31, -88), tail=(0, 0, 4), **bell)),
        (0.45, idle0),
    ], linear=(0.22, 0.25))

    # ── 던지기 sk_throw 0.53초(시동 0.18 · 뻗은 채 0.10 · 후딜 0.25): 오른팔을 뒤로 멀리 젖히고 몸을 뒤로 비틀어 뒷발에 싣는다
    # (0.10~0.14 — 왼팔은 앞을 겨눈다), 0.18에 놓는다: 팔이 앞을 가로질러 채찍처럼 나가고 몸이 앞발로 쏟아진다. 0.28까지 뻗은 채
    # 머물고 일어선다. 칼은 손에 그대로 있다(던져진 칼은 게임이 숨기고 따로 그린다).
    lunge = dict(feet=((0.02, -0.14), (-0.03, 0.12)), legs=((-14, 0, 0), (24, 0, 0)), lift=(0.01, 0.04))
    m.action("sk_throw", 0.53, [
        (0.0, idle0),
        (0.10, sp(drop=0.12, k=0.9, at=(0, 0.10), body=(-14, -8, -58), head=(6, 0, 46), arm_r=(-120, 72, -110), hand_r=(78, 0, -59),
                  arm_l=(-30, -57, 17), feet=((0.02, -0.08), (-0.03, 0.12)), tail=(10, 0, 26))),
        (0.14, sp(drop=0.14, k=0.88, at=(0, 0.12), body=(-16, -8, -64), head=(6, 0, 50), arm_r=(-108, 73, -97), hand_r=(77, 0, -57),
                  arm_l=(-29, -62, 21), feet=((0.02, -0.08), (-0.03, 0.12)), tail=(12, 0, 30))),
        (0.18, sp(drop=0.08, k=1.1, at=(0, -0.14), body=(30, 6, 42), head=(-16, 0, -28), arm_r=(-96, 56, -26), hand_r=(79, -1, -73),
                  arm_l=(4, -46, 11), tail=(-32, 0, -28), **lunge)),
        (0.28, sp(drop=0.10, k=0.98, at=(0, -0.15), body=(34, 8, 50), head=(-14, 0, -34), arm_r=(-72, 37, 10), hand_r=(69, 1, -52),
                  arm_l=(-3, -56, 14), tail=(-22, 0, -8), **lunge)),
        (0.40, sp(drop=0.04, k=1.02, at=(0, -0.06), body=(12, 2, 14), head=(-6, 0, -16), arm_r=(-38, 12, 15), hand_r=(51, 2, -27),
                  arm_l=(2, -9, 2), feet=((0.01, -0.06), (-0.01, 0.05)), tail=(4, 0, 14))),
        (0.53, idle0),
    ], linear=(0.14,))

    # ── 잔불 흩기 sk_embers 0.55초(시동 0.25 · 후딜 0.30): 오른쪽으로 돌아 웅크리며 두 손을 오른 허리 아래에 모은다(0.12~0.21),
    # 0.25에 뿌린다: 몸이 솟으며 왼쪽으로 1/4바퀴 돌고 두 팔이 위·바깥으로 넓게 터진다, 고개가 들린다. 0.33에 더 돌아 팔이 옆으로
    # 퍼지고, 되돌아 선다.
    m.action("sk_embers", 0.55, [
        (0.0, idle0),
        (0.12, sp(drop=0.18, k=0.84, body=(24, -6, -36), head=(8, 0, 14), arm_r=(-38, 7, 18), hand_r=(2, -8, 175), arm_l=(-84, 31, -74),
                  feet=((0.03, -0.02), (-0.03, 0.03)), tail=(20, 0, 20))),
        (0.21, sp(drop=0.20, k=0.82, body=(26, -6, -42), head=(10, 0, 16), arm_r=(-40, 8, 18), hand_r=(1, -8, 175), arm_l=(-85, 30, -75),
                  feet=((0.03, -0.02), (-0.03, 0.03)), tail=(24, 0, 24))),
        (0.25, sp(k=1.16, up=0.06, body=(-12, 0, 52), head=(-18, 0, -22), arm_r=(-103, 70, -80), hand_r=(62, 14, -57),
                  arm_l=(-103, -70, 80), feet=((0.06, 0), (-0.06, 0)), legs=((-12, 0, 0), (10, 0, 0)), tail=(-26, 0, -34))),
        (0.33, sp(k=1.04, up=0.02, body=(-10, 0, 62), head=(-14, 0, -12), arm_r=(-31, 76, -40), hand_r=(65, 6, -74), arm_l=(-31, -76, 40),
                  feet=((0.06, 0), (-0.06, 0)), tail=(-10, 0, 10))),
        (0.44, sp(drop=0.06, k=0.95, body=(8, 0, 12), head=(0, 0, 8), arm_r=(-16, 23, 1), hand_r=(35, -17, -28), arm_l=(-10, -19, 0),
                  feet=((0.03, 0), (-0.03, 0)), tail=(8, 0, 20))),
        (0.55, idle0),
    ], linear=(0.21,))

    # ── 큰 기술: 땅 뒤집기 ult_upend 1.00초(시동 정지 0.30 · 들어 올림 0.20 · 후딜 0.50): 세계가 멈춘 0.30초 동안 쥐는 자세 —
    # 한 무릎을 꿇고 왼손으로 땅을 짚는다(0.12에 닿는다), 칼은 오른쪽 뒤로 들고 몸은 왼쪽 앞으로 쏠린다, 0.30까지 더 눌린다.
    # 0.30부터 쟁반을 뒤집듯 — 온몸이 오른쪽으로 비틀리며 솟고(0.42에 다 늘어난다) 두 팔이 높이 터진다, 0.50까지 떠 있다
    # 0.62에 내려앉아 찌그러지고 튀었다 선다.
    kneel = dict(at=(0, -0.04), head=(-26, -6, -14), feet=((0.04, -0.10), (-0.04, 0.14)), legs=((0, 0, 0), (40, 0, 0)), lift=(0, 0.06))
    m.action("ult_upend", 1.00, [
        (0.0, idle0),
        (0.12, sp(drop=0.20, k=0.9, body=(34, 12, 24), arm_r=(65, 41, -18), hand_r=(87, -33, -133), arm_l=(-46, -10, -21),
                  tail=(26, 0, 10), **kneel)),
        (0.30, sp(drop=0.22, k=0.86, body=(37, 12, 28), arm_r=(66, 36, -20), hand_r=(89, -33, -137), arm_l=(-49, -10, -22),
                  tail=(34, 0, 14), **kneel)),
        (0.36, sp(drop=0.08, k=1.1, body=(12, 0, -10), head=(-14, 0, 14), arm_r=(-128, 64, -92), hand_r=(58, 2, -79),
                  arm_l=(-115, -64, 71), feet=((0.04, -0.06), (-0.04, 0.08)), tail=(0, 0, 30))),
        (0.42, sp(k=1.2, up=0.20, body=(-22, -6, -58), head=(-16, 0, 36), arm_r=(-152, 55, -132), hand_r=(102, -48, -81),
                  arm_l=(-158, -49, 130), legs=((12, 0, 0), (-14, 0, 0)), tail=(-24, 0, 38))),
        (0.50, sp(k=1.14, up=0.24, body=(-24, -6, -74), head=(-18, 0, 30), arm_r=(-147, 62, -143), hand_r=(109, -68, -91),
                  arm_l=(-154, -53, 143), legs=((16, 0, 0), (-18, 0, 0)), tail=(-30, 0, 20))),
        (0.62, sp(drop=0.14, k=0.8, body=(26, 0, -24), head=(12, 0, 10), arm_r=(-26, 37, -2), hand_r=(40, -26, -52), arm_l=(-26, -37, 2),
                  feet=((0.08, 0), (-0.08, 0)), tail=(24, 0, -20))),
        (0.76, sp(drop=0.02, k=1.06, body=(2, 0, -6), head=(-8, 0, 2), arm_r=(-24, 8, 10), hand_r=(47, -3, -13), arm_l=(-2, -2, -1),
                  feet=((0.03, 0), (-0.03, 0)), tail=(-8, 0, 4))),
        (0.90, sp(drop=0.01, k=0.99, body=(4, 0, 0), tail=(2, 0, -16))),
        (1.00, idle0),
    ], linear=(0.30, 0.50))

    # ── 큰 기술: 몰아치기 ult_flurry 0.50초(시동 0.30 · 후딜 0.20): 기를 모은다 — 웅크려 두 주먹을 허리 뒤로 당기고(칼은 오른쪽
    # 뒤로 눕는다) 고개를 숙인다, 0.27까지 점점 낮아지며 몸이 떨리고 꼬리가 좌우로 턴다. 0.30에 확 편다: 발을 벌리고 가슴을 열고
    # 칼을 앞으로, 고개를 든다. 끝은 쉬는 자세보다 조금 낮고 칼이 앞으로 나온 자세(이어지는 평타로 넘어가기 좋게).
    def coil(i):
        s = 1 if i % 2 else -1  # 떨림
        return sp(drop=0.16 + 0.008 * i, k=0.84 - 0.01 * i, body=(15 + i, 1.5 * s, 0), head=(14, 0, 2 * s), arm_r=(8, 26, -7),
                  hand_r=(7, -12, -112), arm_l=(8, -26, 7), feet=((0.02, 0), (-0.02, 0)), tail=(-8 - 1.6 * i, 0, 15 * s))
    wide = dict(feet=((0.12, 0.03), (-0.12, -0.04)))
    m.action("ult_flurry", 0.50, [
        (0.0, idle0),
        (0.12, coil(0)), (0.16, coil(1)), (0.20, coil(2)), (0.24, coil(3)), (0.27, coil(4)),
        (0.30, sp(drop=0.08, k=1.12, body=(-12, 0, -16), head=(-16, 0, 12), arm_r=(-69, 34, 12), hand_r=(73, -4, -35), arm_l=(29, -87, 8),
                  tail=(-14, 0, 0), **wide)),
        (0.37, sp(drop=0.10, body=(-6, 0, -14), head=(-10, 0, 10), arm_r=(-69, 36, 10), hand_r=(73, -5, -37), arm_l=(24, -72, 12),
                  tail=(8, 0, -8), **wide)),
        (0.50, sp(drop=0.04, body=(7, 0, -4), head=(-3, 0, 2), arm_r=(-24, 6, 10), hand_r=(48, -1, -14), arm_l=(-1, -4, 0),
                  feet=((0.04, 0), (-0.04, 0)))),
    ], linear=(0.27,))

    # ── 큰 기술: 한 줄기 ult_line 0.90초(시동 정지 0.30 · 베기 0.15 · 후딜 0.45): 세계가 멈춘 0.30초 동안 쥐는 자세 — 두 손으로
    # 칼을 오른 어깨 뒤로 멀리 넘기고 몸을 오른쪽으로 끝까지 감는다(0.14에 닿고 0.30까지 더 죈다, 고개만 앞을 본다).
    # 0.30~0.45 한 번에 크게: 칼이 뒤-오른쪽 → 오른쪽 → 앞 → 왼쪽 아래로 가로지르고(사이 키 셋이 앞을 지나는 길을 잡는다)
    # 몸이 다 풀리며 앞발로 내디딘다. 0.60까지 칼이 왼쪽 뒤 낮게 머문 채 기울어 있다가(무게) 돌아온다.
    back = dict(at=(0, 0.10), feet=((0.03, -0.08), (-0.04, 0.12)))
    step = dict(feet=((0.03, -0.16), (-0.04, 0.14)), legs=((-10, 0, 0), (20, 0, 0)), lift=(0.01, 0.03))
    m.action("ult_line", 0.90, [
        (0.0, idle0),
        (0.14, sp(drop=0.20, k=0.88, body=(4, -12, -76), head=(2, 6, 62), arm_r=(-146, 59, -125), hand_r=(91, -3, -72),
                  arm_l=(-121, -46, -16), tail=(12, 0, 32), **back)),
        (0.30, sp(drop=0.23, k=0.85, body=(4, -13, -84), head=(2, 6, 68), arm_r=(-146, 58, -113), hand_r=(91, 0, -68),
                  arm_l=(-120, -48, -4), tail=(16, 0, 38), **back)),
        (0.34, sp(drop=0.16, body=(12, -6, -40), head=(0, 0, 34), arm_r=(-125, 66, -94), hand_r=(81, 5, -77), arm_l=(-130, -52, 19),
                  feet=((0.03, -0.10), (-0.04, 0.12)), tail=(4, 0, 40))),
        (0.38, sp(drop=0.16, k=1.1, at=(0, -0.10), body=(22, 0, 4), head=(-4, 0, 6), arm_r=(-94, 60, -37), hand_r=(71, -1, -72),
                  arm_l=(-111, -48, -2), feet=((0.03, -0.14), (-0.04, 0.14)), legs=((-8, 0, 0), (16, 0, 0)), lift=(0, 0.02),
                  tail=(-20, 0, 30))),
        (0.42, sp(drop=0.20, at=(0, -0.18), body=(28, 6, 46), head=(4, 0, -26), arm_r=(-87, 41, 10), hand_r=(70, -7, -40),
                  arm_l=(-82, -62, 37), tail=(-26, 0, -6), **step)),
        (0.45, sp(drop=0.22, k=0.86, at=(0, -0.22), body=(22, 8, 72), head=(8, 0, -44), arm_r=(-86, 0, 58), hand_r=(86, -8, 18),
                  arm_l=(-37, -52, 12), tail=(-10, 0, -38), **step)),
        (0.60, sp(drop=0.22, k=0.92, at=(0, -0.22), body=(20, 7, 68), head=(6, 0, -34), arm_r=(-86, -6, 62), hand_r=(89, -8, 24),
                  arm_l=(-32, -51, 11), tail=(6, 0, 12), **step)),
        (0.76, sp(drop=0.06, k=1.02, at=(0, -0.08), body=(12, 2, 24), head=(-2, 0, -18), arm_r=(-49, 4, 25), hand_r=(58, 15, -22),
                  arm_l=(-8, -7, -2), feet=((0.02, -0.06), (-0.02, 0.05)), tail=(-4, 0, -22))),
        (0.90, idle0),
    ], linear=(0.30, 0.34, 0.38, 0.42))


PREVIEWS = [("idle", 0.0, ""), ("idle", 0.0, "_front"), ("attack", 0.15, "_attack"), ("parry", 0.1, "_parry"),
            ("jump", 0.1, "_jump"), ("plunge", 0.06, "_plunge_up"), ("plunge", 0.29, "_plunge_down"),
            ("charge", 0.3, "_charge"), ("heavy", 0.18, "_heavy"),
            ("drink", 0.4, "_drink"), ("cast", 0.12, "_cast"),
            ("attack3", 0.18, "_attack3"), ("hammer1", 0.3, "_hammer1"), ("hammer_slam", 0.18, "_hammer_slam"),
            ("staff4", 0.24, "_staff4"), ("xbow_shoot", 0.08, "_xbow"), ("throw", 0.2, "_throw"), ("turn_r", 0.13, "_turn"), ("strafe_l", 0.15, "_strafe"),
            ("wake", 0.0, "_wake_prone"), ("wake", 2.0, "_wake_push"), ("wake", 2.6, "_wake_crouch")]
CHECK = [("fall", 0.12), ("fall", 0.5), ("fall", 0.85), ("fall", 1.3), ("wake", 0.0), ("wake", 1.35), ("wake", 2.0), ("wake", 2.6), ("wake", 3.1), ("walk", 0.0), ("walk", 0.2), ("attack", 0.08), ("attack", 0.18), ("dodge", 0.15), ("hurt", 0.06), ("turn_r", 0.06), ("turn_r", 0.13)]


def build(models_dir=C.MODELS_DIR, preview_dir=C.PREVIEW_DIR, check_dir=None):
    m = make()
    path = m.export(models_dir)
    for act, t, suffix in PREVIEWS:
        # 얼굴을 보려고 정면에 가깝게 한 장(0036).
        kw = {"yaw": -8.0, "elev": 14.0} if suffix == "_front" else {}
        m.preview(os.path.join(preview_dir, f"{NAME}{suffix}.png"), act, t, **kw)
    if check_dir:
        for act, t in CHECK:
            m.preview(os.path.join(check_dir, f"{NAME}_{act}_{int(t * 100):03d}.png"), act, t)
    return path


if __name__ == "__main__":
    md, pd = C.out_dirs()
    build(md, pd, os.environ.get("KR_CHECK_DIR"))
