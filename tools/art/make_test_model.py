#!/usr/bin/env python3
"""P4 스파이크용 시험 모델(0013): Blockbench가 내보내는 glTF와 같은 꼴을 코드로 찍는다 — 사람이 Blockbench로 진짜 파일을
내보내기 전에 로더부터 돌려 보려고. 실제 Blockbench 파일이 오면 그것으로 바꾸고 차이(축·단위·스킨 여부)를 0013에 적는다.

꼴: 부위마다 노드(그룹 = 피벗), 노드에 상자 메시 하나(정점 POSITION·NORMAL·TEXCOORD_0, 면마다 4정점),
텍스처 PNG 하나를 data URI로 내장(Blockbench "Embed textures"), 애니메이션 "walk"(팔·다리 회전 채널, LINEAR, 0.8초 고리)와
"idle"(몸통 위아래 translation). 단위 m, +Y 위, 모델 앞 = +Z.

    python3 tools/art/make_test_model.py   # → assets/models/test_critter.gltf
"""
import base64, io, json, math, struct, sys
from pathlib import Path
from PIL import Image

OUT = Path(__file__).resolve().parents[2] / "assets" / "models" / "test_critter.gltf"

# 텍스처: 16×16, 4×4 칸(칸 4px). 부위마다 칸 하나 — 가장자리 1px은 조금 어둡게(픽셀 상자 느낌).
CELLS = {
    "body": (0, 0, (178, 120, 74)),
    "head": (1, 0, (214, 168, 112)),
    "arm": (2, 0, (150, 96, 60)),
    "leg": (3, 0, (110, 72, 48)),
    "eye": (0, 1, (30, 24, 20)),
}


def texture_png():
    img = Image.new("RGBA", (16, 16), (255, 0, 255, 255))
    for cx, cy, col in CELLS.values():
        for y in range(4):
            for x in range(4):
                edge = x in (0, 3) or y in (0, 3)
                c = tuple(int(v * (0.8 if edge else 1.0)) for v in col) + (255,)
                img.putpixel((cx * 4 + x, cy * 4 + y), c)
    buf = io.BytesIO()
    img.save(buf, "PNG")
    return buf.getvalue()


def uv_cell(name):
    cx, cy, _ = CELLS[name]
    # 칸 안쪽 반 픽셀을 빼 경계 번짐을 막는다.
    u0, v0 = (cx * 4 + 0.5) / 16, (cy * 4 + 0.5) / 16
    u1, v1 = (cx * 4 + 3.5) / 16, (cy * 4 + 3.5) / 16
    return u0, v0, u1, v1


def box(lo, hi, cell):
    """상자(피벗 기준 좌표) → (pos, nrm, uv, idx). 면 6 × 정점 4."""
    (x0, y0, z0), (x1, y1, z1) = lo, hi
    u0, v0, u1, v1 = uv_cell(cell)
    faces = [
        ((0, 0, 1), [(x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1)]),
        ((0, 0, -1), [(x1, y0, z0), (x0, y0, z0), (x0, y1, z0), (x1, y1, z0)]),
        ((1, 0, 0), [(x1, y0, z1), (x1, y0, z0), (x1, y1, z0), (x1, y1, z1)]),
        ((-1, 0, 0), [(x0, y0, z0), (x0, y0, z1), (x0, y1, z1), (x0, y1, z0)]),
        ((0, 1, 0), [(x0, y1, z1), (x1, y1, z1), (x1, y1, z0), (x0, y1, z0)]),
        ((0, -1, 0), [(x0, y0, z0), (x1, y0, z0), (x1, y0, z1), (x0, y0, z1)]),
    ]
    pos, nrm, uv, idx = [], [], [], []
    for n, quad in faces:
        b = len(pos)
        for p, (u, v) in zip(quad, [(u0, v1), (u1, v1), (u1, v0), (u0, v0)]):
            pos.append(p)
            nrm.append(n)
            uv.append((u, v))
        idx += [b, b + 1, b + 2, b, b + 2, b + 3]
    return pos, nrm, uv, idx


def merge(*parts):
    pos, nrm, uv, idx = [], [], [], []
    for p, n, u, i in parts:
        b = len(pos)
        pos += p
        nrm += n
        uv += u
        idx += [b + k for k in i]
    return pos, nrm, uv, idx


# 부위: (이름, 부모, 피벗(부모 기준), 메시). 키 ≈ 1.4m.
PARTS = [
    ("root", None, (0, 0, 0), None),
    ("body", "root", (0, 0.7, 0), box((-0.25, 0.0, -0.17), (0.25, 0.45, 0.17), "body")),
    ("head", "body", (0, 0.45, 0), merge(box((-0.2, 0.0, -0.18), (0.2, 0.3, 0.2), "head"),
                                          box((-0.12, 0.14, 0.2), (-0.04, 0.2, 0.22), "eye"),
                                          box((0.04, 0.14, 0.2), (0.12, 0.2, 0.22), "eye"))),
    ("arm_l", "body", (-0.32, 0.4, 0), box((-0.07, -0.42, -0.07), (0.07, 0.02, 0.07), "arm")),
    ("arm_r", "body", (0.32, 0.4, 0), box((-0.07, -0.42, -0.07), (0.07, 0.02, 0.07), "arm")),
    ("leg_l", "root", (-0.12, 0.7, 0), box((-0.09, -0.7, -0.09), (0.09, 0.0, 0.09), "leg")),
    ("leg_r", "root", (0.12, 0.7, 0), box((-0.09, -0.7, -0.09), (0.09, 0.0, 0.09), "leg")),
]


def quat_x(deg):
    h = math.radians(deg) / 2
    return (math.sin(h), 0.0, 0.0, math.cos(h))


def main():
    names = [p[0] for p in PARTS]
    blob = bytearray()
    buffer_views, accessors, meshes = [], [], []

    def add_view(data, target=None):
        while len(blob) % 4:
            blob.append(0)
        off = len(blob)
        blob.extend(data)
        bv = {"buffer": 0, "byteOffset": off, "byteLength": len(data)}
        if target:
            bv["target"] = target
        buffer_views.append(bv)
        return len(buffer_views) - 1

    def add_acc(values, ctype, typ, target=None, minmax=False):
        flat = [c for v in values for c in (v if isinstance(v, tuple) else (v,))]
        fmt = {5126: "f", 5123: "H"}[ctype]
        data = struct.pack("<%d%s" % (len(flat), fmt), *flat)
        acc = {"bufferView": add_view(data, target), "componentType": ctype, "count": len(values), "type": typ}
        if minmax:
            n = len(values[0])
            acc["min"] = [min(v[i] for v in values) for i in range(n)]
            acc["max"] = [max(v[i] for v in values) for i in range(n)]
        accessors.append(acc)
        return len(accessors) - 1

    nodes = []
    for name, parent, pivot, mesh in PARTS:
        node = {"name": name, "translation": list(pivot)}
        if mesh:
            pos, nrm, uv, idx = mesh
            prim = {
                "attributes": {
                    "POSITION": add_acc(pos, 5126, "VEC3", 34962, minmax=True),
                    "NORMAL": add_acc(nrm, 5126, "VEC3", 34962),
                    "TEXCOORD_0": add_acc(uv, 5126, "VEC2", 34962),
                },
                "indices": add_acc(idx, 5123, "SCALAR", 34963),
                "material": 0,
            }
            meshes.append({"name": name, "primitives": [prim]})
            node["mesh"] = len(meshes) - 1
        nodes.append(node)
    for i, (name, parent, _, _) in enumerate(PARTS):
        if parent is not None:
            nodes[names.index(parent)].setdefault("children", []).append(i)

    # 애니메이션: walk(0.8초 고리) — 팔·다리가 서로 반대로 ±30°, idle — 몸통이 숨쉬듯 2cm.
    animations = []
    times = [0.0, 0.2, 0.4, 0.6, 0.8]
    t_acc = add_acc(times, 5126, "SCALAR", minmax=False)
    accessors[t_acc]["min"], accessors[t_acc]["max"] = [0.0], [0.8]
    swing = [0, 30, 0, -30, 0]
    chans, samps = [], []
    for name, sign in [("leg_l", 1), ("leg_r", -1), ("arm_l", -1), ("arm_r", 1)]:
        vals = [quat_x(sign * a) for a in swing]
        samps.append({"input": t_acc, "output": add_acc(vals, 5126, "VEC4"), "interpolation": "LINEAR"})
        chans.append({"sampler": len(samps) - 1, "target": {"node": names.index(name), "path": "rotation"}})
    animations.append({"name": "walk", "channels": chans, "samplers": samps})
    it = add_acc([0.0, 0.6, 1.2], 5126, "SCALAR")
    accessors[it]["min"], accessors[it]["max"] = [0.0], [1.2]
    bob = add_acc([(0, 0.7, 0), (0, 0.68, 0), (0, 0.7, 0)], 5126, "VEC3")
    animations.append({"name": "idle", "channels": [{"sampler": 0, "target": {"node": names.index("body"), "path": "translation"}}],
                       "samplers": [{"input": it, "output": bob, "interpolation": "LINEAR"}]})

    png = texture_png()
    gltf = {
        "asset": {"version": "2.0", "generator": "knellroot tools/art/make_test_model.py (Blockbench 꼴 흉내)"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": nodes,
        "meshes": meshes,
        "materials": [{"pbrMetallicRoughness": {"baseColorTexture": {"index": 0}, "metallicFactor": 0}, "alphaMode": "MASK"}],
        "textures": [{"sampler": 0, "source": 0}],
        "samplers": [{"magFilter": 9728, "minFilter": 9728}],
        "images": [{"mimeType": "image/png", "uri": "data:image/png;base64," + base64.b64encode(png).decode()}],
        "buffers": [{"byteLength": len(blob), "uri": "data:application/octet-stream;base64," + base64.b64encode(bytes(blob)).decode()}],
        "bufferViews": buffer_views,
        "accessors": accessors,
        "animations": animations,
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(gltf, indent=1))
    print(OUT, OUT.stat().st_size, "bytes", len(nodes), "nodes", len(meshes), "meshes")


if __name__ == "__main__":
    sys.exit(main())
