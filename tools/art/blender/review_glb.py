#!/usr/bin/env python3
"""assets/models/*.glb 리뷰 표(블렌더 없이 python3 + Pillow). 모델을 고치기 전·후에 돌려 숫자로 본다(0139).

    python3 tools/art/blender/review_glb.py [glb ...]

모델마다: 정점·삼각형, 쉬는 자세의 키(m — 동작에서만 숨는 부위도 센다: 방랑자는 든 지팡이 끝까지)와 그 키가 게임 화면에서 몇 px인지(쿼터뷰·락온 — 0061 체크리스트 4 "게임 거리의 px 먼저"),
매끈한 면의 비율(산 것은 둥글게·돌과 뿌리는 각지게, 0037·0061), 텍스처 크기, 면이 쓰는 텍셀의 밝기(넓이로 가중한 평균과 10~90% 범위)·채도.
밝기는 sRGB 값 그대로의 휘도(0..1)라 서로 견주는 용도다 — 게임 화면의 밝기는 빛·`ACTOR_LIFT`가 더해진 뒤다.
카메라 값은 `src/tuning.rs` 기본값을 적어 둔 것(바꾸면 여기도): 쿼터뷰 거리 20·시야 42°, 락온 거리 7·시야 58°, 내부 세로 360.
"""
import colorsys
import glob
import io
import math
import os
import struct
import sys

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import check_glb as K  # noqa: E402

INTERNAL_H = 360
QUARTER = (20.0, 42.0)
LOCK = (7.0, 58.0)


def px_per_m(dist, fov_deg):
    return INTERNAL_H / (2 * dist * math.tan(math.radians(fov_deg) / 2))


def accessor(js, bin_, i):
    a = js["accessors"][i]
    bv = js["bufferViews"][a["bufferView"]]
    fmt, size = K.CT[a["componentType"]]
    n = K.NC[a["type"]]
    off = bv.get("byteOffset", 0) + a.get("byteOffset", 0)
    stride = bv.get("byteStride", size * n)
    return [struct.unpack_from("<" + fmt * n, bin_, off + k * stride) for k in range(a["count"])]


def quat_rotate(q, v):
    x, y, z, w = q
    vx, vy, vz = v
    tx, ty, tz = 2 * (y * vz - z * vy), 2 * (z * vx - x * vz), 2 * (x * vy - y * vx)
    return (vx + w * tx + (y * tz - z * ty), vy + w * ty + (z * tx - x * tz), vz + w * tz + (x * ty - y * tx))


def world_points(js, bin_):
    """쉬는 자세의 모든 정점(월드) — 노드의 이동·회전·크기를 부모부터 곱한다."""
    nodes = js["nodes"]
    parent = {c: i for i, n in enumerate(nodes) for c in n.get("children", [])}

    def to_world(i, p):
        while i is not None:
            n = nodes[i]
            s = n.get("scale", (1, 1, 1))
            p = quat_rotate(n.get("rotation", (0, 0, 0, 1)), tuple(a * b for a, b in zip(p, s)))
            p = tuple(a + b for a, b in zip(p, n.get("translation", (0, 0, 0))))
            i = parent.get(i)
        return p

    out = []
    for i, n in enumerate(nodes):
        if "mesh" not in n:
            continue
        for pr in js["meshes"][n["mesh"]]["primitives"]:
            out += [to_world(i, p) for p in accessor(js, bin_, pr["attributes"]["POSITION"])]
    return out


def review(path):
    js, bin_ = K.read_glb(path)
    img = js["images"][0]
    bv = js["bufferViews"][img["bufferView"]]
    tex = Image.open(io.BytesIO(bin_[bv.get("byteOffset", 0):bv.get("byteOffset", 0) + bv["byteLength"]])).convert("RGB")
    W, H = tex.size
    px = tex.load()
    verts = tris = 0
    smooth = flat = 0.0
    samples = []  # (넓이, 휘도, 채도)
    for m in js["meshes"]:
        for pr in m["primitives"]:
            P = accessor(js, bin_, pr["attributes"]["POSITION"])
            N = accessor(js, bin_, pr["attributes"]["NORMAL"])
            UV = accessor(js, bin_, pr["attributes"]["TEXCOORD_0"])
            idx = [i[0] for i in accessor(js, bin_, pr["indices"])]
            verts += len(P)
            tris += len(idx) // 3
            for t in range(0, len(idx), 3):
                a, b, c = idx[t:t + 3]
                e1 = [P[b][k] - P[a][k] for k in range(3)]
                e2 = [P[c][k] - P[a][k] for k in range(3)]
                fn = (e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0])
                area = math.sqrt(sum(x * x for x in fn))
                if area < 1e-12:
                    continue
                fn = [x / area for x in fn]
                if min(sum(fn[k] * N[v][k] for k in range(3)) for v in (a, b, c)) > 0.999:
                    flat += area
                else:
                    smooth += area
                u = sum(UV[v][0] for v in (a, b, c)) / 3
                w = sum(UV[v][1] for v in (a, b, c)) / 3
                r, g, bl = px[min(W - 1, max(0, int(u * W))), min(H - 1, max(0, int(w * H)))]
                lum = (0.2126 * r + 0.7152 * g + 0.0722 * bl) / 255
                samples.append((area, lum, colorsys.rgb_to_hsv(r / 255, g / 255, bl / 255)[1]))
    total = sum(s[0] for s in samples) or 1.0
    lum = sum(a * l for a, l, _ in samples) / total
    sat = sum(a * s for a, _, s in samples) / total
    samples.sort(key=lambda s: s[1])
    acc, lo, hi = 0.0, None, None
    for a, l, _ in samples:
        acc += a
        if lo is None and acc >= 0.1 * total:
            lo = l
        if hi is None and acc >= 0.9 * total:
            hi = l
    ys = [p[1] for p in world_points(js, bin_)]
    height = max(ys) - min(0.0, min(ys))
    return {"name": os.path.basename(path)[:-4], "verts": verts, "tris": tris, "tex": f"{W}x{H}", "h": height,
            "smooth": 100 * smooth / max(1e-9, smooth + flat), "lum": lum, "lo": lo or 0.0, "hi": hi or 0.0, "sat": sat}


def main(argv):
    paths = argv or sorted(glob.glob(os.path.join(K.REPO, "assets", "models", "*.glb")))
    q, lk = px_per_m(*QUARTER), px_per_m(*LOCK)
    print(f"쿼터뷰 {q:.1f} px/m(1px ≈ {100 / q:.1f}cm) · 락온 {lk:.1f} px/m(1px ≈ {100 / lk:.1f}cm)")
    print(f"{'모델':14} {'정점':>5} {'삼각형':>5} {'텍스처':>8} {'키 m':>5} {'쿼터 px':>6} {'락온 px':>6} {'매끈 %':>6} {'밝기':>5} {'10~90%':>10} {'채도':>5}")
    for p in paths:
        r = review(p)
        print(f"{r['name']:14} {r['verts']:6} {r['tris']:7} {r['tex']:>9} {r['h']:6.2f} {r['h'] * q:8.0f} {r['h'] * lk:8.0f} "
              f"{r['smooth']:8.0f} {r['lum']:6.2f}  {r['lo']:.2f}~{r['hi']:.2f} {r['sat']:6.2f}")


if __name__ == "__main__":
    main(sys.argv[1:])
