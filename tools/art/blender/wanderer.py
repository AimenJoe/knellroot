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


def classify(c):
    """면 가운데 자리 → 부위(앞 = -Y, 캐릭터 왼쪽 = +X, 키 1.4m). 치수는 높이 띠마다 정점을 찍어 본 것."""
    from mathutils import Vector
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
TAIL_LIFT = 40.0  # 도: 뿌리에서 끝을 위로 세우는 각(원본의 들림에 더한다) — 0156: 50 → 40, 두건 자락 밑으로 나오게
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


# 0155·0156 두건: 뒤에서 보면 무늬 없는 보라색 공 + 뿔 같은 귀였다(사장 "두건 좀 이상하지 않니?"). 0153의 두건 꼬리는 뒤통수에 붙은 띠로,
# 0155의 꼭지는 "요정 모자"로 읽혔다(사장) — 위로 솟은 끝은 모자의 것이다. 두건을 두건으로 만드는 것은 **천이 머리에서 어깨로 흘러내리는 것**:
# 공은 목에서 쏙 들어가고(머리 + 목), 두건은 가장 넓은 둘레에서 그대로 늘어져 어깨에 얹힌다. 그리고 얼굴을 두른 말린 테.
HOOD_TIP_AT = (0.0, 0.30, 0.80)  # 뼈만 남긴다(동작 표가 부른다) — 0156부터 붙은 부위 없음


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


def _cloth(g, rings, mat, inner, axis_y=0.02, thick=0.012, inner_from=0):
    """열린 천(고리마다 같은 수의 점을 이은 띠 면): 바깥 면(mat) + 조금 안쪽의 안감 면(inner) — 한 겹 면은 안에서 보면 사라진다.
    inner_from = 안감을 붙이기 시작하는 띠(위쪽 띠는 머리에 가려 안이 안 보인다 — 정점을 아낀다)."""
    from mathutils import Vector
    uo, ui = C.cell_mid(mat, "side"), C.cell_mid(inner, "side")
    go, gi = g._group(), g._group()
    for i in range(len(rings) - 1):
        a, b = rings[i], rings[i + 1]
        for k in range(len(a) - 1):
            quad = [a[k], a[k + 1], b[k + 1], b[k]]
            c = sum(quad, Vector()) / 4
            out = Vector((c.x, c.y - axis_y, 0.0)).normalized()
            n = (quad[1] - quad[0]).cross(quad[3] - quad[0])
            if n.dot(out) < 0:
                quad.reverse()
            g.quads.append((quad, [uo] * 4, go))
            if i >= inner_from:
                g.quads.append(([q - out * thick for q in reversed(quad)], [ui] * 4, gi))


def dress_hood(m, head):
    import math

    import bmesh
    from mathutils import Vector
    from mathutils.bvhtree import BVHTree
    soften_hood(head, m.ext_img)
    bm = bmesh.new()
    bm.from_mesh(head.data)
    bm.faces.ensure_lookup_table()
    hood = {f.index for f in _hood_faces(bm, m.ext_img)}
    tree = BVHTree.FromBMesh(bm)

    def hit(origin, direction, fallback):
        """머리 메시에 쏜 광선이 닿는 자리(덧붙인 테가 실제 두건 면에 앉게 — 타원체 어림으로 두면 떠서 고리처럼 보였다)."""
        loc = tree.ray_cast(Vector(origin), Vector(direction).normalized())[0]
        return loc if loc is not None else Vector(fallback)

    g = m.geo("hood_trim_geo", "head")
    # ① 늘어진 자락: 두건의 가장 넓은 둘레에서 어깨까지 내려오는 천(옆·뒤 — 앞은 얼굴과 목도리라 연다). 아래로 갈수록 주름이 잡히고
    #    끝단이 물결친다. 이것이 "머리 + 목"의 공을 "어깨에 얹힌 두건"으로 바꾼다.
    n, cy = 22, 0.02
    axis = Vector((0.0, cy, 0.0))
    rings = [[], [], [], []]
    for k in range(n + 1):
        t = math.radians(-34 + 248 * k / n)
        d = Vector((math.cos(t), math.sin(t), 0.0))
        fold = math.cos(7 * (t - math.pi / 2))
        end = max(0.0, 1.0 - min(k, n - k) / 4.0) ** 2  # 앞 끝: 두건 옆면 속으로 파고들어 자른 가장자리가 안 보이게

        def dome(z):
            """그 높이에서 돔 겉면까지의 거리(축에서) — 실제 면을 따라야 자락이 턱 없이 나온다(타원 어림은 띠를 두른 것처럼 보였다)."""
            at = hit(axis + d * 1.5 + Vector((0, 0, z)), -d, axis + d * 0.40 + Vector((0, 0, z)))
            return (Vector((at.x, at.y, 0.0)) - axis).length

        r_top, r_out = dome(0.935), dome(0.835)
        hem = 0.580 - 0.035 * max(0.0, math.sin(t)) ** 2 - 0.014 * fold + 0.070 * end
        for j, (z, r) in enumerate(((0.935, r_top - 0.014), (0.835, r_out + 0.003), (0.705, r_out * (0.985 + 0.012 * fold)),
                                    (hem, r_out * (0.885 + 0.028 * fold)))):
            r *= 1.0 - (0.05, 0.07, 0.16, 0.20)[j] * end
            rings[j].append(axis + d * r + Vector((0, 0, z)))
    _cloth(g, rings, "p_hood", "p_hood_in", axis_y=cy, inner_from=2)
    # ② 얼굴을 두른 말린 테: 앞에서 얼굴 가운데로부터 바깥으로 훑어 처음 두건 천을 만나는 자리(텍스처 색) — 거기에 도톰한 줄을 앉힌다.
    cz, pts = 0.90, []
    for k in range(21):
        t = math.radians(-42 + 264 * k / 20)
        found, streak = None, 0
        for j in range(60):
            r = 0.14 + 0.007 * j
            loc, _, idx, _ = tree.ray_cast(Vector((r * math.cos(t), -1.5, cz + r * math.sin(t))), Vector((0, 1, 0)))
            if loc is None:
                break
            streak = streak + 1 if idx in hood else 0
            if streak == 1:
                found = loc
            if streak >= 2:
                break
            if streak == 0:
                found = None
        if found is not None and streak >= 2:
            pts.append(found)
    for _ in range(2):  # 들쭉날쭉한 경계를 한 번 고른다
        pts = [pts[0]] + [(pts[i - 1] + pts[i] * 2 + pts[i + 1]) / 4 for i in range(1, len(pts) - 1)] + [pts[-1]]
    rim = [tuple(q + Vector((0, -0.010, 0))) for q in pts]
    g.limb(rim, [0.020] + [0.032] * (len(rim) - 2) + [0.020], "p_hood_in", sides=5, smooth=True, caps=(True, True))
    # ③ 뒤통수의 솔기: 정수리 뒤에서 자락 끝까지 내려오는 가는 줄(짙은 안감 색) — 뒤에서 본 민짜 면에 "천을 이은 자리"가 생긴다.
    seam = []
    for k in range(7):
        z = 1.20 - (1.20 - 0.84) * k / 6
        at = hit((0.0, 1.5, z), (0, -1, 0), (0.0, 0.36, z))
        seam.append((0.0, at.y - 0.004, z))
    seam += [(0.0, q.y + 0.004, q.z) for q in (ring[n // 2] for ring in rings[1:])]
    g.limb(seam, [0.012] * (len(seam) - 1) + [0.008], "p_hood_in", sides=5, smooth=True, caps=(False, True))
    print(f"[wanderer] hood: rim {len(rim)} pts")
    bm.free()


# 뼈 자리(새 몸, 키 1.4m). 손은 장갑 가운데, 어깨는 소매 위.
HAND = (0.29, -0.085, 0.31)
SHOULDER = (0.185, -0.05, 0.52)


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
    m.bone("head", (0, -0.02, 0.62), "body")
    m.bone("arm_l", (sx, sy, sz), "body")
    m.bone("arm_r", (-sx, sy, sz), "body")
    m.bone("hand_r", (-hx, hy, hz), "arm_r")
    m.bone("hand_l", (hx, hy, hz), "arm_l")
    m.bone("leg_l", (0.08, -0.07, 0.125), "root")
    m.bone("leg_r", (-0.08, -0.07, 0.125), "root")
    m.bone("cloak", (0, 0.15, 0.50), "body")  # 0037 동작 표가 부르는 뼈 — 새 몸에는 따로 흔들 망토가 없다
    m.bone("tail", TAIL_AT, "body")
    m.bone("lamp", (hx, hy - 0.05, hz - 0.04), "hand_l")
    m.hide("lamp")
    # 0153 뒷모습의 디테일(게임에서 가장 오래 보는 쪽 — 밋밋한 두건 뒤통수와 옷 등판뿐이었다).
    m.bone("hood_tip", HOOD_TIP_AT, "head")

    objs = m.external(MESHY_LOW, classify, MESHY_PARTS)
    recenter_tail(objs["tail_geo"], objs["body_geo"])

    dress_hood(m, objs["head_geo"])

    # 허리 주머니(왼쪽 엉덩이 뒤 — 꼬리를 옮기며 메운 자리를 덮는다): 갈색 몸 + 크림 덮개 + 금빛 단추.
    g = m.geo("pouch_geo", "body")
    g.blob((0.225, 0.075, 0.265), (0.11, 0.14, 0.125), "p_boot", r=0.6, n=2)
    g.blob((0.232, 0.075, 0.315), (0.115, 0.15, 0.055), "p_cream", r=0.7, n=2)
    g.blob((0.288, 0.075, 0.285), (0.022, 0.04, 0.04), "p_gold", r=0.9, n=1)

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


PREVIEWS = [("idle", 0.0, ""), ("idle", 0.0, "_front"), ("attack", 0.15, "_attack"), ("parry", 0.1, "_parry"),
            ("jump", 0.1, "_jump"), ("plunge", 0.06, "_plunge_up"), ("plunge", 0.29, "_plunge_down"),
            ("charge", 0.3, "_charge"), ("heavy", 0.18, "_heavy"),
            ("drink", 0.4, "_drink"), ("cast", 0.12, "_cast"),
            ("attack3", 0.18, "_attack3"), ("hammer1", 0.3, "_hammer1"), ("hammer_slam", 0.18, "_hammer_slam"),
            ("staff4", 0.24, "_staff4"), ("xbow_shoot", 0.08, "_xbow"), ("throw", 0.2, "_throw"), ("turn_r", 0.13, "_turn"), ("strafe_l", 0.15, "_strafe")]
CHECK = [("walk", 0.0), ("walk", 0.2), ("attack", 0.08), ("attack", 0.18), ("dodge", 0.15), ("hurt", 0.06), ("turn_r", 0.06), ("turn_r", 0.13)]


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
