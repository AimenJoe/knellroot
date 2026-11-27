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
# wanderer: 0051 무기 넷(검·망치·지팡이·쇠뇌)이 한 모델에 있고 게임은 든 것만 보인다(나머지 크기 0). 0153: 몸을 정점 2,800·텍스처 512로 다시
# 줄이고 두건 꼬리·주머니를 더했다 — 보이는 정점은 약 3,200(한 그리기 호출 10,000 미만, 0135).
# 0155: 무기 넷·등잔을 상자 조합에서 제 꼴(잎 꼴 날·통 머리·굽은 활)로, 두건 꼭지 — 4,524(무기 넷이 1,233). 색인은 13,668 — 한 호출 15,000 아래를 지킨다.
# 0193: 스킨 몸(Meshy 리깅, 줄여 정점 2,963) + 무기 넷·등잔 그대로 — 4,196.
# 0272 2단계: 창(자루·촉·끈·천 조각 — 정점 백 남짓)을 더해 4,800 → 5,000(한 그리기 호출의 상한 10,000 · 색인 15,000은 그대로 지킨다).
# 0274 ④: 대검(긴 날 · 코등이 · 감긴 잔뿌리 · 두 손 손잡이 — 정점 232)을 더해 5,300 → 5,500(삼각형 4,885 = 색인 14,655 —
# 모든 부위를 합쳐도 한 그리기 호출의 색인 15,000 밑, 게임은 든 무기 하나만 그린다).
# 0274 ⑤: 낫(휜 날 여섯 마디 · 자루 · 고리 · 잔뿌리 — 정점 144, 삼각형 80)을 더해 5,500 → 5,650(삼각형 4,965 = 색인 14,895 —
# 모든 부위를 합쳐도 한 그리기 호출의 색인 15,000 밑을 지키려고 날의 밝은 선 · 자루 끝 구슬은 넣지 않았다).
# 0274 ⑥: 완드(막대 · 다섯 모의 잿빛 돌 — 삼각형 서른)를 더해 5,650 → 5,700(삼각형 4,995 = 색인 14,985 — 모든 부위를 합쳐도 한 그리기
# 호출의 색인 15,000 밑).
# charms: 0155에서 돌린 꼴·꺾인 관으로 다시(1,939) — 한 번에 하나만 그린다(아이콘은 캐시).
BUDGET = {"hushed_stag": 8800, "mossgrazer": 900, "packbearer": 1000, "repeater": 1650, "mossward": 2600, "winchkeeper": 2600, "wanderer": 5700, "root_husk": 760, "thorn_beast": 780, "gatekeeper": 2600, "shellback": 780, "pod": 800, "bellkeeper": 1700, "res_hearth": 780, "res_water": 800, "res_cloth": 800,
          # 0068 장신구 여섯이 한 파일(게임은 하나씩만 그린다) — 하나 280 안팎. 2026-11-23 디테일(사장 "싸구려"): 장신구 아홉 + 덧옷 셋,
          # 메뉴는 파일 한 벌을 그리기 한 번에 넘긴다(다른 부위 크기 0) — 그래서 덧옷은 `garbs`로 나눴다(뿌리껍질 덧옷 1,500 안팎).
          "charms": 4600, "garbs": 3400,
          # 0139 다듬은 소품: 깎은 돌 하나가 정점 64(면마다 따로 — 각진 음영), 스테이지에 60개 안쪽이라 넉넉히.
          # 문은 하나뿐이고 널 여덟 + 띠 + 설주 돌이라 크다.
          "wall_segment": 960, "pillar_broken": 560, "root_clump": 480, "brazier": 440, "gate": 1900, "core_crystal": 400,
          "bell": 800, "rubble": 480, "chest": 560, "stone_tablet": 1100,  # 0242: 종 무늬·글 줄·금·이끼·조약돌(맵에 몇 개뿐)
          # 0142 주민의 생활 물건: 주민 하나에 하나(마을에 셋) — 돌 다섯 + 장작 + 불꽃, 장대 둘 + 천 두 벌.
          "work_fire": 520, "work_tent": 480,
          # 0265 무너진 아치(지도에 둘 — 숲 끝·성소 남쪽 벽) · 0267 쉬는 곳의 화톳불(지역에 몇 개 — 돌 고리·장작·불꽃).
          "arch_ruin": 1900, "hearth": 1150}
PROP_BUDGET = 400
# 동작 이름과 길이(초) — 명세 그대로인지 본다.
EXPECT = {
    # 0193 스킨 몸 + Meshy 동작(`wanderer_skin.py`): 게임 상태에 맞춘 동작은 옛 길이 그대로. 서기·몸짓은 원본 길이(되풀이·몸짓
    # 시계가 모델의 길이를 읽는다), 뒷걸음은 한 주기(1.2), 달리기 `run`이 새로 생겼다. 옆걸음·종 몸짓은 원본이 없다(걷기·건너뜀).
    "wanderer": {"block": 0.2, "idle": 1.87, "idle_look": 1.83, "idle_shake": 1.12, "run": 0.63, "wake": 4.0, "fall": 1.3, "walk": 0.8, "attack": 0.43, "dodge": 0.32, "dodge_rise": 0.2, "hurt": 0.35, "turn_r": 0.25, "turn_l": 0.25,
                 "walk_back": 1.67, "charge": 0.3, "heavy": 0.58, "drink": 0.8,
                 # 0196: 점프는 체공의 앞뒤 반(게임이 위 속도로 자리를 고른다), 줍기 0.9, 쇠뇌(활쏘기 원본) 겨누기 0.3 · 쏘기 0.24.
                 "jump": 0.5, "land": 0.15, "pickup": 0.9, "xbow_charge": 0.3, "xbow_shoot": 0.24, "bow_draw": 0.3, "bow_shoot": 0.24,
                 # 0197: 뒤로 뛰기 회피(회피와 같다 — 0256에 0.25 → 0.32), 장치 당기기 0.65.
                 "dodge_back": 0.32, "pull": 0.65,
                 # 0198 넷째 묶음: 맨손 잽 둘(0.258)·어퍼컷(0.344)·차기 모으기·차기(0.464), 패링 막기·마법은 옛 길이 그대로.
                 "fist1": 0.26, "fist2": 0.26, "fist3": 0.34, "fist_charge": 0.3, "fist_heavy": 0.46, "parry": 0.5, "cast": 0.4,
                 "cast_charge": 0.3,
                 # 0199: 맨손 달리기 앞차기 0.3, 패링 성공의 밀쳐내기 0.4.
                 "fist_dash": 0.3, "parry_push": 0.4, "plunge_land": 0.35,
                 # 0200: 망치 달리기 앞차기(망치 대시 구간 0.688).
                 "hammer_dash": 0.69,
                 # 0274: 한손 망치 3연타(0.4945 · 0.4945 · 0.602).
                 "hammer1": 0.49, "hammer2": 0.49, "hammer3": 0.6,
                 # 0158 G2 스킬 동작 — 길이 = 시동 + 이어짐 + 후딜(`skills/table.rs` 표의 바탕값).
                 "sk_whirl": 0.67, "sk_cut": 0.57, "sk_gather": 0.5, "sk_quake": 0.7, "sk_brace": 1.65, "sk_echo": 0.45,
                 "sk_throw": 0.53, "sk_embers": 0.55, "ult_upend": 1.0, "ult_flurry": 0.5, "ult_line": 0.9,
                 # 0270 C3.
                 "sk_stitch": 0.55, "sk_ram": 0.7, "sk_swat": 0.57, "sk_toll": 0.5, "sk_ripple": 0.9, "sk_snow": 0.55,
                 "sk_flare": 0.5, "sk_snare": 0.43, "ult_bell": 0.9, "ult_blizzard": 0.9,
                 # 0272.
                 "sk_fuse": 0.5, "sk_mine": 0.43, "ult_rain": 0.95,
                 # 0272 2단계 창: 평타 셋 · 달리며 찌르기 · 모으기 · 강타(판정 구간 + 후딜) · 기술 셋 · 질풍 찌르기.
                 "spear1": 0.5, "spear2": 0.5, "spear3": 0.58, "spear_dash": 0.43, "spear_charge": 0.3, "spear_lunge": 0.67,
                 "sk_skewer": 0.66, "sk_pin": 0.46, "sk_pole": 0.55, "ult_gale": 1.45,
                 # 0274 한손검: 자루 치기 · 밀치기 · 재기.
                 "sk_pommel": 0.4, "sk_shove": 0.44, "sk_rally": 0.55,
                 # 0274 ③ 쌍단검: 평타 넷 · 달려들며 찌르기 · 모으기 · 강타 · 빠지며 베기 · 그림자 습격.
                 "dagger1": 0.34, "dagger2": 0.34, "dagger3": 0.37, "dagger4": 0.45, "dagger_dash": 0.34, "dagger_charge": 0.3,
                 "dagger_lunge": 0.47, "sk_slip": 0.48, "ult_ambush": 1.21,
                 # 0274 ④ 대검: 휩쓸기 다섯 · 달려들며 올려 베기 · 모으기 · 강타 · 땅 가르기 · 불씨 휩쓸기 · 칼바람(휘돌이와 같은 한 바퀴).
                 "gs1": 0.56, "gs2": 0.56, "gs3": 0.56, "gs4": 0.6, "gs5": 0.69, "gs_dash": 0.47, "gs_charge": 0.3, "gs_heavy": 0.81,
                 "sk_sunder": 0.7, "sk_sear": 0.53, "ult_tempest": 0.67,
                 # 0274 ⑤ 낫: 걸어 당기며 베기 · 걸어 당기기 · 생기 거두기(휘돌이와 같은 한 바퀴) · 깨어나는 숲(땅을 짚는다).
                 "staff_cast": 0.4, "scythe1": 0.43, "scythe2": 0.43, "scythe_dash": 0.43, "wand_dash": 0.34, "scythe3": 0.56, "scythe_charge": 0.3, "scythe_heavy": 0.58, "sk_reap": 0.48, "sk_withdraw": 0.66, "sk_plunge": 0.9, "sk_boomerang": 0.53, "sk_word": 0.5, "sk_harvest": 0.67, "ult_grove": 1.49,
                 # 0274 ⑥ 완드: 저주 새기기 · 시드는 그늘(손을 내미는 시전) · 치른 값(들어 올렸다 내리꽂고 머문다).
                 "sk_hex": 0.52, "sk_blight": 0.55, "ult_price": 1.5},
    "root_husk": {"idle": 1.6, "walk": 1.0, "attack": 1.3, "hurt": 0.3},
    "thorn_beast": {"idle": 1.4, "run": 0.5, "charge_windup": 0.6, "charge": 0.4, "recover": 0.7, "hurt": 0.3},
    "gatekeeper": {"idle": 2.0, "walk": 1.4, "slam": 1.5, "charge_windup": 0.8, "charge": 0.5,
                   "wave": 1.6, "hurt": 0.4, "wake": 2.0},
    "shellback": {"idle": 1.6, "walk": 1.0, "attack": 1.3, "hurt": 0.3},
    "pod": {"idle": 1.4, "walk": 0.5, "hop": 0.4, "plant": 1.0, "hurt": 0.3},
    "mossgrazer": {"idle": 1.8, "walk": 0.4, "hurt": 0.3},
    "packbearer": {"idle": 1.4, "run": 0.5, "charge_windup": 0.6, "charge": 0.4, "recover": 0.7, "hurt": 0.3},
    "repeater": {"idle": 1.6, "walk": 1.0, "attack": 1.3, "hurt": 0.3},
    "mossward": {"idle": 2.0, "walk": 1.4, "slam": 1.5, "charge_windup": 0.8, "charge": 0.5, "shell": 1.6,
                 "recover": 0.8, "hurt": 0.4, "wake": 2.0},
    "winchkeeper": {"idle": 2.0, "walk": 1.4, "slam": 1.5, "charge_windup": 0.8, "charge": 0.5,
                    "wave": 1.6, "hurt": 0.4, "wake": 2.0, "hook": 1.3, "hurl": 1.5},
    "bellkeeper": {"idle": 2.0, "walk": 1.4, "toll": 1.6, "slam": 1.5, "throw": 1.2, "charge_windup": 0.8,
                   "charge": 0.5, "recover": 0.8, "hurt": 0.4, "wake": 2.0},
    "res_hearth": {"work": 3.2, "look": 2.4},
    "res_water": {"work": 3.6, "look": 2.2},
    "res_cloth": {"work": 2.8, "look": 2.6},
    "gate": {"open": 1.5},
    "core_crystal": {"pulse": 1.0},
    "bell": {"toll": 2.0},
    "chest": {"open": 0.4},
    "hushed_stag": {"idle": 2.4, "walk": 1.2, "slam": 1.5, "charge_windup": 0.9, "charge": 0.5, "rear": 1.6,
                    "recover": 0.9, "hurt": 0.4, "wake": 2.4},
    "lever": {"pull": 0.4},
}
MUST_NODES = {"hushed_stag": ["glow_core", "glow_eyes", "head_geo", "antler_geo", "bell_geo", "antlers", "bell"],
              "mossward": ["glow_core", "glow_eyes", "head_geo", "shell_geo"],
              "winchkeeper": ["glow_core", "glow_eyes", "head_geo", "club"], "packbearer": ["glow_eyes", "head_geo", "load_geo", "load"],
              "repeater": ["glow_eyes", "head_geo", "pick"], "mossgrazer": ["head_geo", "ear_l", "ear_r"], "gate": ["door_l", "door_r"], "gatekeeper": ["glow_core", "glow_eyes"], "wanderer": ["body", "root", "sword", "w_hammer", "w_staff", "w_crossbow", "w_bow", "bow", "w_spear", "spear", "w_daggers", "w_dagger_l", "dagger_l", "w_greatsword", "w_scythe", "w_wand", "glow_lamp", "lamp", "shield"],
              "root_husk": ["glow_eye"], "thorn_beast": ["glow_eyes"], "shellback": ["glow_eyes", "head_geo"], "pod": ["glow_eyes", "head_geo", "seed_geo"],
              "bellkeeper": ["glow_eyes", "glow_core", "bell_geo", "head_geo"], "res_hearth": ["head_geo"], "res_water": ["head_geo"], "res_cloth": ["head_geo"], "brazier": ["glow_flame"],
              "core_crystal": ["glow_core"], "bell": ["bell", "bell_frame"],
              "chest": ["lid", "glow_lock"], "stone_tablet": ["glow_carve"], "lever": ["handle", "glow_grip"],
              "work_fire": ["fire", "glow_fire"], "work_jar": ["jar"], "work_tent": ["torn", "whole"],
              "charms": ["shell", "feet", "ear", "hunger", "hand", "pouch"], "garbs": ["garb_cloak", "garb_bark", "garb_apron"]}

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
            # 0193: 스킨 몸(노드에 skin이 있음)만 뼈 가중치를 갖는다 — 게임이 CPU에서 휜다(`model::skin_into`, 정점마다 넷까지).
            for bad in ("JOINTS_0", "WEIGHTS_0"):
                if bad in at and "skin" not in n:
                    problems.append(f"메시 {n.get('name')}에 {bad}")
            if "skin" in n and not ("JOINTS_0" in at and "WEIGHTS_0" in at):
                problems.append(f"스킨 메시 {n.get('name')}에 가중치 없음")
            if "JOINTS_1" in at:
                problems.append(f"메시 {n.get('name')}에 JOINTS_1(영향 다섯 이상 — 로더는 넷까지)")
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
