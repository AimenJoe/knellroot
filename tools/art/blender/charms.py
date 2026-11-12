"""장신구 아홉(0068 · 떠난 마을 셋 0127): 장비 창 아이콘·도는 그림용 작은 물건 — `charms.glb` 한 파일에 장신구마다 메시 부위 하나(이름 = 기록 키).

게임은 부위 하나만 골라(다른 부위는 크기 0) 경계 상자 가운데에 놓고 그린다(`Renderer::draw_item`) — 파일 안의 자리는 미리보기용.
0061 체크리스트: 귀여움 문법이 먼저(둥근 덩어리 `blob`), 큰 색 덩어리 + 실루엣, 각진 것은 뿌리만(0037), 채도 높은 빨강 없음.
AI 사용: 모양·색은 Claude가 이 스크립트로 코드로 찍었다(그림 생성 AI·외부 에셋 없음).

    blender --background --python tools/art/blender/charms.py
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

# 미리보기에서 나란히 보이게 x로 띄운다(게임은 부위마다 가운데로 옮기니 상관없다).
STEP = 0.55


def shell(g, x):
    """단단한 껍질(최대 체력 +1): 둥근 등딱지 — 밝은 테 + 갈색 돔 + 등 위의 금빛 비늘판(가운데 하나·둘레 넷)."""
    g.lathe((x, 0, 0.0), [(0.200, 0.0), (0.215, 0.022), (0.200, 0.048)], "p_boot", sides=10, smooth=True, squash=(1.0, 0.86),
            caps=(True, False))  # 테
    g.lathe((x, 0, 0.0), [(0.190, 0.040), (0.172, 0.105), (0.120, 0.160), (0.0, 0.185)], "brown", sides=10, smooth=True,
            squash=(1.0, 0.86), caps=(False, False))  # 돔
    g.blob((x, 0, 0.178), (0.105, 0.090, 0.030), "p_belt", r=1.0, n=2)
    for dx, dy in ((0.118, 0.0), (-0.118, 0.0), (0.0, 0.102), (0.0, -0.102)):
        g.blob((x + dx, dy, 0.150), (0.074, 0.064, 0.030), "p_belt", r=1.0, n=2, rot=(dy * 330, -dx * 330, 0))


def feet(g, x):
    """가벼운 발(회피 기력 −30% · 받는 피해 +1): 깃털 달린 작은 짚신 — 바닥 + 엇건 끈 둘 + 뒤꿈치에서 솟은 깃."""
    g.blob((x, 0, 0.03), (0.17, 0.33, 0.05), "p_belt", r=1.0, n=3)  # 바닥(짚)
    for y0, y1 in ((-0.10, 0.02), (0.02, -0.10)):
        g.limb([(x - 0.078, y0, 0.045), (x - 0.045, (y0 + y1) / 2 - 0.01, 0.105), (x + 0.045, (y0 + y1) / 2 + 0.01, 0.105),
                (x + 0.078, y1, 0.045)], [0.016, 0.017, 0.017, 0.016], "p_scarf", sides=5, smooth=True)  # 엇건 끈
    g.limb([(x - 0.070, 0.105, 0.045), (x, 0.130, 0.090), (x + 0.070, 0.105, 0.045)], [0.015, 0.016, 0.015], "p_scarf", sides=5,
           smooth=True)  # 뒤꿈치 끈
    # 깃: 납작한 잎 꼴이 뒤꿈치에서 비스듬히 솟아 끝이 뒤로 휜다 — 대는 밝게, 끝은 하늘빛.
    g.limb([(x, 0.125, 0.07), (x + 0.012, 0.150, 0.17), (x + 0.030, 0.185, 0.27), (x + 0.052, 0.235, 0.345), (x + 0.066, 0.285, 0.385)],
           [0.018, 0.050, 0.056, 0.038, 0.0], ["p_white", "p_white", "p_white", "p_sky"], sides=4, roll=0, flat=0.22)
    g.blob((x, 0.125, 0.075), (0.045, 0.045, 0.045), "p_sky", r=1.0, n=2)  # 꽂은 자리


def ear(g, x):
    """종지기의 귀(패링 창 +0.06초): 작은 놋쇠 종 — 벌어진 입술 · 어깨의 띠 · 걸이 고리 · 아래로 보이는 추."""
    import math
    g.lathe((x, 0, 0.0), [(0.150, 0.0), (0.158, 0.014), (0.128, 0.040), (0.108, 0.110), (0.092, 0.170), (0.056, 0.215), (0.0, 0.232)],
            ["p_gold", "p_gold", "p_gold", "bronze", "p_gold", "p_gold"], sides=10, smooth=True, hard=(3, 4), caps=(True, False),
            cap_mat="bronze")
    g.blob((x, 0, -0.030), (0.074, 0.074, 0.074), "bronze", r=1.0, n=2)  # 추
    ring = [(x + 0.052 * math.cos(t), 0.0, 0.272 + 0.052 * math.sin(t)) for t in (2 * math.pi * k / 10 for k in range(11))]
    g.limb(ring, [0.016] * 11, "bronze", sides=5, smooth=True)  # 걸이 고리


def hunger(g, x):
    """굶주린 뿌리(처치 시 기력 · 등잔 −1): 굽은 송곳니를 보라 뿌리가 감아 오른다 — 뿌리만 각지게(0037)."""
    import math
    g.limb([(x - 0.02, 0, 0.0), (x - 0.03, 0, 0.11), (x - 0.01, 0, 0.22), (x + 0.04, 0, 0.31), (x + 0.10, 0, 0.365)],
           [0.085, 0.080, 0.062, 0.036, 0.0], "bone", sides=8, smooth=True, caps=(True, False))  # 송곳니
    coil = []
    for k in range(13):
        t = k / 12
        a = math.radians(-60 + 560 * t)
        r = 0.098 - 0.030 * t
        cx = x - 0.025 + 0.02 * t
        coil.append((cx + r * math.cos(a), r * math.sin(a), 0.015 + 0.20 * t))
    g.limb(coil, [0.034 - 0.018 * k / 12 for k in range(12)] + [0.0], ["root", "droot"] * 6, sides=4, roll=45)  # 감은 뿌리
    g.limb([(x + 0.07, -0.04, 0.02), (x + 0.15, -0.07, 0.035), (x + 0.21, -0.05, 0.085)], [0.030, 0.022, 0.0], "droot", sides=4,
           roll=45)  # 땅으로 뻗은 끝
    g.limb([(x - 0.09, 0.05, 0.02), (x - 0.16, 0.08, 0.03), (x - 0.20, 0.06, 0.07)], [0.026, 0.018, 0.0], "root", sides=4, roll=45)


def hand(g, x):
    """무거운 손(강타 피해 +1 — 0127, 예전 주는 피해 +1 · 공격 속도 −15%): 돌 장갑 — 손바닥 덩어리 + 손가락 넷 + 엄지 + 이끼 테."""
    g.blob((x, 0, 0.10), (0.20, 0.13, 0.20), "lstone", r=0.7, n=2)  # 손바닥
    for i, dx in enumerate((-0.066, -0.022, 0.022, 0.066)):
        g.blob((x + dx, 0, 0.24 + (0.01 if i in (1, 2) else 0)), (0.045, 0.10, 0.09), "stone", r=0.8, n=2)
    g.blob((x - 0.12, 0, 0.13), (0.06, 0.09, 0.10), "stone", r=0.8, n=2, rot=(0, -30, 0))  # 엄지
    g.blob((x, 0, 0.00), (0.22, 0.15, 0.05), "moss", r=0.6, n=2)  # 손목 테


def pouch(g, x):
    """잔불 주머니(잔불 +50%): 배 부른 천 주머니 — 끈으로 조인 목 · 벌어진 주름 입 · 입에서 비치는 불씨 · 늘어진 끈 끝."""
    g.lathe((x, 0, 0.0), [(0.070, 0.0), (0.130, 0.025), (0.150, 0.090), (0.125, 0.160), (0.062, 0.205)], "p_mask", sides=10,
            smooth=True, caps=(True, False))  # 주머니
    g.lathe((x, 0, 0.0), [(0.062, 0.205), (0.070, 0.218), (0.062, 0.232)], "p_belt", sides=10, smooth=True, caps=(False, False))  # 끈
    g.lathe((x, 0, 0.0), [(0.062, 0.232), (0.105, 0.275), (0.070, 0.262)], "p_mask", sides=7, caps=(False, False))  # 주름 입
    g.blob((x, 0, 0.262), (0.105, 0.105, 0.060), "ember", r=1.0, n=2)  # 비치는 불씨
    g.blob((x + 0.035, 0.02, 0.305), (0.036, 0.036, 0.036), "amber", r=1.0, n=1)
    g.limb([(x + 0.066, -0.02, 0.218), (x + 0.120, -0.05, 0.205), (x + 0.150, -0.06, 0.140)], [0.013, 0.013, 0.011], "p_belt",
           sides=5, smooth=True)
    g.blob((x + 0.152, -0.06, 0.125), (0.040, 0.040, 0.040), "p_belt", r=1.0, n=1)  # 끈 끝 매듭


def cloth(g, x):
    """꿰맨 천(체력 +1 · 기력 회복 −10%, 0127): 접힌 천 두 겹 + 굵은 바늘땀 + 실을 문 바늘."""
    g.blob((x, 0, 0.04), (0.38, 0.30, 0.07), "p_scarf", r=0.6, n=2)  # 아래 겹
    g.blob((x + 0.02, 0.02, 0.11), (0.32, 0.24, 0.06), "p_mask", r=0.6, n=2, rot=(0, 0, 12))  # 위 겹
    for k in range(4):
        g.box((x - 0.12 + k * 0.08, -0.08, 0.15), (0.028, 0.085, 0.018), "p_belt", rot=(0, 0, 35 if k % 2 == 0 else -35))  # 엇건 땀
    g.limb([(x + 0.03, 0.05, 0.15), (x + 0.11, 0.08, 0.21), (x + 0.20, 0.11, 0.29)], [0.012, 0.011, 0.0], "p_steel", sides=4)  # 바늘
    g.limb([(x + 0.035, 0.05, 0.155), (x - 0.02, 0.09, 0.19), (x - 0.07, 0.07, 0.16), (x - 0.10, 0.02, 0.145)],
           [0.006, 0.006, 0.006, 0.006], "p_belt", sides=4)  # 꿴 실


def jar(g, x):
    """물 받는 항아리(한 모금 +1 · 등잔 −1, 0127): 배 부른 흙 항아리 + 밝은 입술 + 고인 물 + 떨어지는 물방울."""
    g.lathe((x, 0, 0.0), [(0.080, 0.0), (0.140, 0.030), (0.160, 0.110), (0.135, 0.190), (0.095, 0.235)], "brown", sides=10,
            smooth=True, caps=(True, False))  # 몸
    g.lathe((x, 0, 0.0), [(0.095, 0.235), (0.118, 0.250), (0.112, 0.268), (0.088, 0.262)], "p_boot", sides=10, smooth=True,
            caps=(False, False))  # 입술
    g.lathe((x, 0, 0.0), [(0.090, 0.256), (0.0, 0.258)], "p_sky", sides=10, caps=(False, False))  # 물
    g.lathe((x, 0, 0.385), [(0.0, -0.040), (0.030, -0.018), (0.022, 0.012), (0.0, 0.052)], "p_sky", sides=6, smooth=True,
            caps=(False, False))  # 물방울


def bellshard(g, x):
    """깨진 종 조각(열린 틈 피해 +1, 0127): 옆으로 누운 종의 깨진 반쪽 — 바깥은 놋쇠, 안은 그을린 청동, 깨진 테는 각지고 밝다. 곁에 조각 둘."""
    # 종의 아래쪽 2/3만(위는 깨져 나갔다), 일곱 모로 각지게 — 깨진 테에 밝은 선이 붙는다.
    prof = [(0.165, 0.0), (0.172, 0.016), (0.140, 0.045), (0.118, 0.120), (0.104, 0.175)]
    g.lathe((x, 0, 0.115), prof, ["p_gold", "p_gold", "p_gold", "amber"], sides=7, rot=(0, 78, 20), caps=(False, False))
    g.lathe((x, 0, 0.115), [(r - 0.022, z) for r, z in prof], "bronze", sides=7, rot=(0, 78, 20), caps=(False, False))  # 안쪽
    g.box((x + 0.20, -0.10, 0.03), (0.11, 0.07, 0.03), "p_gold", rot=(8, -12, 30), taper=(0.5, 0.7))  # 떨어진 조각
    g.box((x + 0.13, 0.13, 0.025), (0.07, 0.06, 0.025), "amber", rot=(-6, 10, -25), taper=(0.6, 0.5))


# 이름 = 장신구 기록 키(`src/charms.rs` `key`, 바꾸지 않는다) = 게임이 찾는 부위 이름(`item_view.rs`).
CHARMS = [("shell", shell), ("feet", feet), ("ear", ear), ("hunger", hunger), ("hand", hand), ("pouch", pouch),
          ("cloth", cloth), ("jar", jar), ("bellshard", bellshard)]


def build(models_dir=C.MODELS_DIR, preview_dir=C.PREVIEW_DIR):
    m = C.Model("charms")
    for i, (key, make) in enumerate(CHARMS):
        make(m.geo(key), (i - (len(CHARMS) - 1) / 2) * STEP)
    m.build()
    path = m.export(models_dir)
    m.preview(os.path.join(preview_dir, "charms.png"), None, 0.0, size=640, floor=False)
    return path


if __name__ == "__main__":
    build()
