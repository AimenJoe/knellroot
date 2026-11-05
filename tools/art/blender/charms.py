"""장신구 여섯(0068): 장비 창 아이콘·도는 그림용 작은 물건 — `charms.glb` 한 파일에 장신구마다 메시 부위 하나(이름 = 기록 키).

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
    """단단한 껍질(최대 체력 +1): 둥근 등딱지 — 갈색 돔 + 밝은 테 + 등 위 금빛 무늬 셋."""
    g.blob((x, 0, 0.02), (0.40, 0.34, 0.06), "p_boot", r=1.0, n=3)  # 테
    g.blob((x, 0, 0.09), (0.34, 0.28, 0.16), "brown", r=1.0, n=3)  # 돔
    for dx, dy in ((0.0, 0.0), (-0.08, 0.05), (0.08, -0.05)):
        g.blob((x + dx, dy, 0.17), (0.07, 0.07, 0.03), "p_belt", r=1.0, n=2)


def feet(g, x):
    """가벼운 발(회피 기력 −30% · 받는 피해 +1): 깃털 달린 작은 짚신."""
    g.blob((x, 0, 0.03), (0.16, 0.32, 0.05), "p_belt", r=1.0, n=3)  # 바닥(짚)
    g.box((x, -0.06, 0.07), (0.15, 0.03, 0.04), "p_scarf")  # 끈 앞
    g.box((x, 0.06, 0.07), (0.15, 0.03, 0.04), "p_scarf")  # 끈 뒤
    # 깃털: 뒤꿈치에 꽂혀 비스듬히 솟는다(밑동을 피벗으로 돌려 바닥에 붙어 있게).
    g.blob((x, 0.12, 0.17), (0.05, 0.08, 0.24), "p_white", r=1.0, n=2, rot=(-20, 25, 0), pivot=(x, 0.12, 0.06),
           taper=(0.5, 0.5))
    g.blob((x, 0.12, 0.08), (0.05, 0.05, 0.05), "p_sky", r=1.0, n=2)  # 꽂은 자리


def ear(g, x):
    """종지기의 귀(패링 창 +0.06초): 작은 놋쇠 종 + 걸이 고리 + 추."""
    g.blob((x, 0, 0.11), (0.24, 0.24, 0.22), "p_gold", r=0.8, n=3, taper=(0.55, 0.55))  # 종 몸
    g.blob((x, 0, 0.01), (0.27, 0.27, 0.04), "p_gold", r=1.0, n=2)  # 입술
    g.blob((x, 0, -0.05), (0.07, 0.07, 0.07), "bronze", r=1.0, n=2)  # 추
    g.box((x, 0, 0.26), (0.12, 0.03, 0.07), "bronze")  # 고리
    g.box((x, 0, 0.30), (0.04, 0.03, 0.04), "bronze")


def hunger(g, x):
    """굶주린 뿌리(처치 시 기력 · 등잔 −1): 보라 뿌리가 감은 이빨 — 뿌리만 각지게(0037)."""
    g.box((x, 0, 0.14), (0.10, 0.10, 0.30), "bone", taper=(0.25, 0.25))  # 이빨
    g.blob((x, 0, 0.02), (0.14, 0.14, 0.06), "bone", r=1.0, n=2)  # 뿌리 쪽
    g.box((x, 0, 0.08), (0.16, 0.16, 0.04), "root", rot=(0, 0, 20))  # 감은 뿌리
    g.box((x, 0, 0.15), (0.13, 0.13, 0.035), "droot", rot=(0, 0, -25))
    g.box((x + 0.07, 0.02, 0.06), (0.04, 0.04, 0.16), "root", rot=(0, 35, 0), taper=(0.5, 0.5))  # 뻗은 끝
    g.box((x - 0.06, -0.03, 0.04), (0.035, 0.035, 0.13), "droot", rot=(0, -40, 10), taper=(0.5, 0.5))


def hand(g, x):
    """무거운 손(주는 피해 +1 · 공격 속도 −15%): 돌 장갑 — 손바닥 덩어리 + 손가락 넷 + 엄지 + 이끼 테."""
    g.blob((x, 0, 0.10), (0.20, 0.13, 0.20), "lstone", r=0.7, n=2)  # 손바닥
    for i, dx in enumerate((-0.066, -0.022, 0.022, 0.066)):
        g.blob((x + dx, 0, 0.24 + (0.01 if i in (1, 2) else 0)), (0.045, 0.10, 0.09), "stone", r=0.8, n=2)
    g.blob((x - 0.12, 0, 0.13), (0.06, 0.09, 0.10), "stone", r=0.8, n=2, rot=(0, -30, 0))  # 엄지
    g.blob((x, 0, 0.00), (0.22, 0.15, 0.05), "moss", r=0.6, n=2)  # 손목 테


def pouch(g, x):
    """잔불 주머니(잔불 +50%): 불씨가 비치는 천 주머니 + 끈 + 입구의 불씨."""
    g.blob((x, 0, 0.10), (0.24, 0.22, 0.22), "p_mask", r=1.0, n=3)  # 주머니
    g.blob((x, 0, 0.22), (0.14, 0.13, 0.06), "p_mask", r=1.0, n=2)  # 오므린 목
    g.box((x, 0, 0.20), (0.16, 0.15, 0.025), "p_scarf")  # 끈
    g.blob((x, 0, 0.27), (0.08, 0.08, 0.06), "ember", r=1.0, n=2)  # 비치는 불씨
    g.blob((x + 0.05, 0.03, 0.30), (0.03, 0.03, 0.03), "amber", r=1.0, n=2)


# 이름 = 장신구 기록 키(`src/charms.rs` `key`, 바꾸지 않는다) = 게임이 찾는 부위 이름(`item_view.rs`).
CHARMS = [("shell", shell), ("feet", feet), ("ear", ear), ("hunger", hunger), ("hand", hand), ("pouch", pouch)]


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
