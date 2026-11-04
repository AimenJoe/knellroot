"""0037 그림 방향 시안(움직이지 않는 덩어리 모델 — 디자인 비교용, 게임에 들어가지 않는다).

    blender --background --python tools/art/blender/concepts.py

방랑자 셋(A 후드 아이 · B 작은 동물 · C 종 모자 아이) + 침식된 잔재 하나를 같은 원칙으로 세운다:
2등신, 눈은 크고 얼굴 아래쪽·가깝게, 입 없음, 볼, 디테일은 정체성 넷만, 색은 큰 셋 + 크림 테 + 금빛 하나.
→ docs/art/concept-0037/<이름>_{q,f}.png(쿼터뷰·정면).
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

OUT = os.path.join(C.REPO, "docs", "art", "concept-0037")


def face(g, cz, fy, eye=0.11, gap=0.095, low=-0.03, blush=True):
    """얼굴 앞면 y = fy(앞이 -y), 얼굴 가운데 높이 cz. 눈은 가운데보다 `low`만큼 아래."""
    for x in (gap, -gap):
        g.blob((x, fy - 0.01, cz + low), (eye * 0.75, 0.03, eye), "p_eye", r=0.95, n=3)
        g.box((x - 0.018 * (1 if x > 0 else -1), fy - 0.027, cz + low + eye * 0.22), (0.03, 0.01, 0.035), "p_white", drop=("+y",))
        if blush:
            g.blob((x * 1.55, fy + 0.005, cz + low - eye * 0.62), (0.08, 0.02, 0.04), "p_blush", r=0.9, n=2)


def body_common(m, cloak, trim, hood, legs=True):
    g = m.geo("body", None)
    g.blob((0, 0, 0.36), (0.42, 0.36, 0.42), cloak, r=0.7, taper=(0.72, 0.76))  # 콩 모양 몸(망토)
    g.blob((0, 0, 0.17), (0.43, 0.37, 0.05), trim, r=0.7, n=3)  # 밑단 테
    g.blob((0, 0.16, 0.36), (0.36, 0.06, 0.38), hood, r=0.5, n=3, taper=(0.8, 1))  # 등 망토
    for sx in (1, -1):
        g.blob((0.215 * sx, -0.01, 0.40), (0.12, 0.13, 0.18), cloak, r=0.8, n=3)  # 짧고 통통한 팔
        g.blob((0.225 * sx, -0.02, 0.29), (0.11, 0.11, 0.09), "p_skin", r=0.95, n=3)  # 손
        if legs:
            g.blob((0.09 * sx, -0.03, 0.06), (0.15, 0.20, 0.13), "p_boot", r=0.7, n=3)  # 통통한 발(망토 밑으로)
    return g


def variant_a():
    """A 후드 아이: 지금 방랑자를 원칙대로 — 머리 더 크게, 얼굴 구멍 넓게, 곰 귀 후드."""
    m = C.Model("concept_a")
    body_common(m, "p_cloak", "p_cream", "p_hood")
    g = m.geo("head", None)
    g.blob((0, 0.05, 0.86), (0.66, 0.58, 0.62), "p_hood", r=0.85, n=5)  # 후드
    g.blob((0, -0.06, 0.82), (0.50, 0.46, 0.46), "p_skin", r=0.95, n=5)  # 얼굴
    g.blob((0, -0.03, 0.83), (0.56, 0.36, 0.53), "p_hood_in", r=1.0, n=4)  # 얼굴 구멍 둘레(얇게)
    for x in (0.2, -0.2):
        g.blob((x, 0.04, 1.18), (0.15, 0.12, 0.16), "p_hood", r=0.85, n=3)
        g.blob((x, -0.02, 1.18), (0.08, 0.03, 0.09), "p_blush", r=0.9, n=2)
    face(g, 0.80, -0.29)
    g.blob((0, -0.20, 0.55), (0.40, 0.30, 0.10), "p_scarf", r=0.7, n=3)  # 목도리
    g.blob((-0.2, -0.05, 0.20), (0.07, 0.07, 0.08), "p_gold", r=0.8, n=2, taper=(0.55, 0.55))  # 허리 종
    return m


def variant_b():
    """B 작은 동물 방랑자: 첫 게임 세계(동물 마을)와 잇는 쪽 — 둥근 너구리 얼굴, 후드 밖 귀, 줄무늬 꼬리."""
    m = C.Model("concept_b")
    body_common(m, "p_cloak", "p_cream", "p_hood")
    g = m.geo("head", None)
    g.blob((0, 0.0, 0.82), (0.66, 0.56, 0.54), "p_hair", r=0.95, n=5)  # 털 얼굴(캐러멜)
    g.blob((0, -0.20, 0.76), (0.40, 0.18, 0.28), "p_cream", r=0.95, n=4)  # 주둥이·볼 크림
    for x in (0.17, -0.17):
        g.blob((x, -0.21, 0.80), (0.16, 0.06, 0.13), "p_pants", r=0.95, n=3)  # 눈 둘레 무늬(너구리)
    g.blob((0, 0.10, 0.98), (0.62, 0.42, 0.32), "p_hood", r=0.85, n=4)  # 작은 두건(뒤·위)
    for x in (0.22, -0.22):
        g.blob((x, 0.0, 1.12), (0.15, 0.10, 0.18), "p_hair", r=0.85, n=3)  # 귀
        g.blob((x, -0.05, 1.11), (0.08, 0.03, 0.10), "p_blush", r=0.9, n=2)
    face(g, 0.82, -0.29, eye=0.10, gap=0.13, low=-0.02, blush=False)
    g.blob((0, -0.295, 0.74), (0.07, 0.04, 0.05), "p_eye", r=0.95, n=2)  # 코
    g.blob((0, -0.20, 0.55), (0.40, 0.30, 0.10), "p_scarf", r=0.7, n=3)
    for i, z in enumerate((0.20, 0.30, 0.40)):
        g.blob((0.0, 0.28 + i * 0.06, z), (0.16, 0.16, 0.12), "p_hair" if i % 2 == 0 else "p_pants", r=0.95, n=3)  # 줄무늬 꼬리
    g.blob((-0.2, -0.05, 0.20), (0.07, 0.07, 0.08), "p_gold", r=0.8, n=2, taper=(0.55, 0.55))
    return m


def variant_c():
    """C 종 모자 아이: 이름의 종을 머리에 — 종 모양 모자(챙이 넓게 퍼짐) 아래 동그란 얼굴, 꼭대기 고리."""
    m = C.Model("concept_c")
    body_common(m, "p_cloak", "p_cream", "p_scarf")
    g = m.geo("head", None)
    g.blob((0, -0.03, 0.78), (0.50, 0.46, 0.46), "p_skin", r=0.95, n=5)  # 얼굴
    g.blob((0, 0.02, 0.98), (0.56, 0.52, 0.34), "p_hood", r=0.9, n=5, taper=(0.62, 0.62))  # 종 몸(위가 좁다)
    g.blob((0, 0.02, 0.86), (0.62, 0.58, 0.12), "p_hood_in", r=0.95, n=4)  # 종 입술(둥글게 말린 테)
    g.blob((0, 0.02, 1.18), (0.10, 0.06, 0.10), "p_gold", r=0.9, n=2)  # 꼭대기 고리
    g.blob((0, -0.27, 0.95), (0.10, 0.02, 0.10), "p_gold", r=0.9, n=2)  # 종 앞의 금빛 무늬
    face(g, 0.76, -0.26)
    g.blob((0, -0.18, 0.55), (0.36, 0.28, 0.09), "p_belt", r=0.7, n=3)  # 목 띠
    return m


def husk():
    """침식된 잔재: 살아 있던 작은 것이 색을 잃은 모양 — 같은 둥근 몸·귀인데 바랜 잿빛, 금 간 자리로 보라 뿌리가 돋고,
    눈 하나는 빈 구멍, 하나는 호박색 점. 귀여움의 문법을 지키되 색과 뿌리가 '잃었다'를 말한다."""
    m = C.Model("concept_husk")
    g = m.geo("body", None)
    g.blob((0, 0, 0.32), (0.50, 0.44, 0.46), "f_ash", r=0.8, n=4, taper=(0.8, 0.8))  # 바랜 몸
    g.blob((0, -0.01, 0.72), (0.56, 0.50, 0.48), "f_ash", r=0.95, n=5)  # 바랜 머리
    for x in (0.2, -0.2):
        g.blob((x, 0.02, 0.98), (0.14, 0.10, 0.16), "f_ash_d", r=0.85, n=3)  # 귀(축 처짐)
    g.blob((0.11, -0.26, 0.70), (0.10, 0.03, 0.12), "shadow", r=0.95, n=3)  # 빈 눈구멍
    g.blob((-0.11, -0.26, 0.70), (0.09, 0.03, 0.11), "f_ash_d", r=0.95, n=3)  # 감긴 눈 자리
    g.blob((0.11, -0.28, 0.70), (0.035, 0.02, 0.035), "ember", r=0.9, n=2)  # 호박색 점 하나
    for x, z, rz in ((0.16, 1.02, -25), (-0.06, 1.06, 15)):
        g.box((x, 0.0, z), (0.05, 0.05, 0.24), "root", rot=(0, rz, 0), taper=(0.2, 0.2))  # 머리에 돋은 뿌리
    g.box((-0.15, -0.23, 0.36), (0.03, 0.02, 0.26), "root", rot=(0, 28, 0))  # 몸의 금(보라 뿌리 줄)
    g.blob((0, 0, 0.06), (0.60, 0.54, 0.10), "droot", r=0.7, n=3)  # 땅에 박힌 뿌리 밑동
    for sx in (1, -1):
        g.blob((0.24 * sx, -0.02, 0.30), (0.11, 0.12, 0.16), "f_ash_d", r=0.8, n=3)  # 늘어진 팔
    return m


def main():
    os.makedirs(OUT, exist_ok=True)
    for name, fn in (("a_hood_child", variant_a), ("b_little_beast", variant_b), ("c_bell_hat", variant_c), ("husk", husk)):
        m = fn()
        m.build()
        m.preview(os.path.join(OUT, f"{name}_q.png"), None, 0.0)
        m.preview(os.path.join(OUT, f"{name}_f.png"), None, 0.0, yaw=-12.0, elev=12.0)
        print("[concept]", name)


if __name__ == "__main__":
    main()
