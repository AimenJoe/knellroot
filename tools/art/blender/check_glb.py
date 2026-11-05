#!/usr/bin/env python3
"""assets/models/*.glb 검사(블렌더 없이 python3만). 게임 로더(`src/model.rs`)가 기대하는 꼴인지 본다.

    python3 tools/art/blender/check_glb.py [glb ...]

파일마다: 노드 이름, 애니메이션 이름 + 가장 늦은 입력 시각(= 게임이 보는 길이), 이미지 수(정확히 1),
스킨 수(아마추어에서 생길 수 있음 — 괜찮다), 메시에 JOINTS_0/WEIGHTS_0이 없는지, 정점 수(노드가 쓰는 메시마다 합),
glow_ 노드, 텍스처 표본이 NEAREST인지, 삼각형 감김이 법선과 맞는지(뒤집힌 면).
문제가 하나라도 있으면 종료 코드 1.
"""
import glob
import json
import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))

# 정점 예산(캐릭터 700, 보스 1600, 소품 400)
# wanderer: 0051 무기 넷(검·망치·지팡이·쇠뇌)이 한 모델에 있고 게임은 든 것만 보인다(나머지 크기 0) — 보이는 정점은 약 2,300.
BUDGET = {"wanderer": 2600, "root_husk": 760, "thorn_beast": 780, "gatekeeper": 2600}
PROP_BUDGET = 400
# 동작 이름과 길이(초) — 명세 그대로인지 본다.
EXPECT = {
    "wanderer": {"idle": 1.2, "walk": 0.8, "attack": 0.43, "dodge": 0.25, "hurt": 0.35, "turn_r": 0.25, "turn_l": 0.25,
                 "strafe_l": 0.6, "strafe_r": 0.6, "walk_back": 0.7, "charge": 0.3, "heavy": 0.58, "drink": 0.8},
    "root_husk": {"idle": 1.6, "walk": 1.0, "attack": 1.3, "hurt": 0.3},
    "thorn_beast": {"idle": 1.4, "run": 0.5, "charge_windup": 0.6, "charge": 0.4, "recover": 0.7, "hurt": 0.3},
    "gatekeeper": {"idle": 2.0, "walk": 1.4, "slam": 1.5, "charge_windup": 0.8, "charge": 0.5,
                   "wave": 1.6, "hurt": 0.4, "wake": 2.0},
    "gate": {"open": 1.5},
    "core_crystal": {"pulse": 1.0},
    "bell": {"toll": 2.0},
}
MUST_NODES = {"gate": ["door_l", "door_r"], "gatekeeper": ["glow_core", "glow_eyes"], "wanderer": ["head_geo", "sword", "glow_lamp"],
              "root_husk": ["glow_eye"], "thorn_beast": ["glow_eyes"], "brazier": ["glow_flame"],
              "core_crystal": ["glow_core"]}

CT = {5120: ("b", 1), 5121: ("B", 1), 5122: ("h", 2), 5123: ("H", 2), 5125: ("I", 4), 5126: ("f", 4)}
NC = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}


def read_glb(path):
    data = open(path, "rb").read()
    magic, ver, length = struct.unpack_from("<4sII", data, 0)
    assert magic == b"glTF" and ver == 2, "glb 머리가 아니다"
    off = 12
    js, bin_ = None, b""
    while off < length:
        clen, ctype = struct.unpack_from("<II", data, off)
        chunk = data[off + 8 : off + 8 + clen]
        if ctype == 0x4E4F534A:
            js = json.loads(chunk)
        elif ctype == 0x004E4942:
            bin_ = chunk
        off += 8 + clen
    return js, bin_


def accessor(js, bin_, i):
    a = js["accessors"][i]
    bv = js["bufferViews"][a["bufferView"]]
    fmt, size = CT[a["componentType"]]
    n = NC[a["type"]]
    stride = bv.get("byteStride", size * n)
    base = bv.get("byteOffset", 0) + a.get("byteOffset", 0)
    out = []
    for k in range(a["count"]):
        vals = struct.unpack_from("<" + fmt * n, bin_, base + k * stride)
        out.append(vals if n > 1 else vals[0])
    return out


def check(path):
    name = os.path.splitext(os.path.basename(path))[0]
    js, bin_ = read_glb(path)
    problems = []
    nodes = js.get("nodes", [])
    node_names = [n.get("name", "") for n in nodes]
    meshes = js.get("meshes", [])
    verts = 0
    flipped = 0
    tris = 0
    for n in nodes:
        if "mesh" not in n:
            continue
        for p in meshes[n["mesh"]]["primitives"]:
            at = p["attributes"]
            for bad in ("JOINTS_0", "WEIGHTS_0"):
                if bad in at:
                    problems.append(f"메시 {n.get('name')}에 {bad}")
            for need in ("POSITION", "NORMAL", "TEXCOORD_0"):
                if need not in at:
                    problems.append(f"메시 {n.get('name')}에 {need} 없음")
            if "targets" in p:
                problems.append(f"메시 {n.get('name')}에 모프 타깃")
            pos = accessor(js, bin_, at["POSITION"])
            nrm = accessor(js, bin_, at["NORMAL"])
            verts += len(pos)
            idx = accessor(js, bin_, p["indices"]) if "indices" in p else list(range(len(pos)))
            for t in range(0, len(idx), 3):
                a, b, c = (pos[idx[t + k]] for k in range(3))
                e1 = [b[i] - a[i] for i in range(3)]
                e2 = [c[i] - a[i] for i in range(3)]
                cr = (e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0])
                nn = nrm[idx[t]]
                tris += 1
                if sum(cr[i] * nn[i] for i in range(3)) < 0:
                    flipped += 1
    if flipped:
        problems.append(f"뒤집힌 삼각형 {flipped}/{tris}")
    images = js.get("images", [])
    if len(images) != 1:
        problems.append(f"이미지 {len(images)}개(1이어야)")
    for im in images:
        if im.get("mimeType") != "image/png":
            problems.append(f"이미지 형식 {im.get('mimeType')}")
    for s in js.get("samplers", []):
        if s.get("magFilter") != 9728:
            problems.append(f"표본 magFilter {s.get('magFilter')}(NEAREST 9728이어야)")
    if len(js.get("materials", [])) != 1:
        problems.append(f"재질 {len(js.get('materials', []))}개(1이어야)")
    anims = {}
    for a in js.get("animations", []):
        tmax = 0.0
        for smp in a["samplers"]:
            tmax = max(tmax, js["accessors"][smp["input"]]["max"][0])
        anims[a.get("name", "")] = round(tmax, 4)
    for an, dur in EXPECT.get(name, {}).items():
        if an not in anims:
            problems.append(f"동작 {an} 없음")
        elif abs(anims[an] - dur) > 0.006:
            problems.append(f"동작 {an} 길이 {anims[an]} (명세 {dur})")
    for nn in MUST_NODES.get(name, []):
        if nn not in node_names:
            problems.append(f"노드 {nn} 없음")
    budget = BUDGET.get(name, PROP_BUDGET)
    if verts > budget:
        problems.append(f"정점 {verts} > 예산 {budget}")
    glow = [n for n in node_names if n.startswith("glow_")]
    print(f"== {name}  ({os.path.getsize(path)} B)")
    print(f"   정점 {verts} / 예산 {budget}, 삼각형 {tris}, 이미지 {len(images)}, 스킨 {len(js.get('skins', []))}, 재질 {len(js.get('materials', []))}")
    print(f"   노드: {', '.join(node_names)}")
    print(f"   glow: {', '.join(glow) or '-'}")
    if anims:
        print("   동작: " + ", ".join(f"{k} {v:g}s" for k, v in anims.items()))
    for p in problems:
        print(f"   !! {p}")
    return name, verts, anims, glow, problems


def main(paths):
    if not paths:
        paths = sorted(glob.glob(os.path.join(REPO, "assets", "models", "*.glb")))
    bad = 0
    for p in paths:
        _, _, _, _, probs = check(p)
        bad += len(probs)
    print(f"\n{len(paths)}개 파일, 문제 {bad}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
