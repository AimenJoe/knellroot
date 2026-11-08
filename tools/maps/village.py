#!/usr/bin/env python3
"""둘째 지역(떠난 마을) 지도 생성기(0121, v0.5.0 C1) — MAP·HEIGHTS 두 격자를 같은 칸으로 만든다(성소 생성기와 같은 꼴).

비탈을 따라 아래로(북쪽 = 깊은 곳 — 구역은 위(작은 행)로 갈수록 다음 구역, 0113 규칙):
  위쪽 거주지(+4m, 남쪽 — 오르는 길 U·도착 P·화로) → 뿌리 사이 골목(+2.5m, 구불구불) → 아래쪽 광장(0m, 싸움터)
  → 종탑 터(−1m, 깨진 종 L·문지기 W).
지름길은 없다(0124 — 사장 2026-11-08: 사다리를 열고 지나온 길을 되돌아가는 일만 생겼다, 화로 셋으로 충분).
주민 자리(C7): 1 불 지키는 사람(거주지 동쪽) · 2 물 모으는 사람(골목 동쪽 곁방) · 3 꿰매는 사람(광장 남동 턱 위) —
모든 적의 제자리에서 끈(18m) 밖(시험 residents_live_beyond_every_enemy_leash).

    python3 tools/maps/village.py            # 미리보기만(지도 | 높이)
    python3 tools/maps/village.py --write    # src/stage/village.rs 의 MAP·HEIGHTS를 바꿔 쓴다

높이 글자: '.' = 0, 36진수 v -> (v - 4) * 0.5m, '/' = 경사로(곧게 이어진 경사로 칸이 양 끝 높이를 잇는다 — 칸마다 0.5m 이하).
빈 곳(허공)은 낮은 땅(−2m). 쓴 뒤에는 cargo test stage:: (도달·높이·화로 안전·주민 안전). 다시 돌려도 같은 지도(멱등).
"""
import sys
SRC = 'src/stage/village.rs'
W, H = 45, 60
M = [[' '] * W for _ in range(H)]
HT = [['0'] * W for _ in range(H)]

def put(x, y, c, h=None):
    M[y][x] = c
    if h is not None:
        HT[y][x] = h
def room(x0, y0, x1, y1, h, wall_h=None):
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            if x in (x0, x1) or y in (y0, y1):
                if M[y][x] == ' ':
                    put(x, y, '#', wall_h if wall_h else h)
            else:
                put(x, y, '.', h)
def floor(x0, y0, x1, y1, h):
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            put(x, y, '.', h)
def block(x0, y0, x1, y1, h):
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            put(x, y, '#', h)

GROUND, ALLEY, HOME, BELL = '4', '9', 'c', '2'   # 0m · +2.5m · +4m · −1m
# 1. 종탑 터(−1m) — 북쪽 끝.
room(12, 0, 32, 13, BELL, '6')
# 2. 아래쪽 광장(0m) — 넓은 싸움터. 종탑 터로 내려가는 경사로(행 12..14, 가운데 다섯 칸).
room(2, 13, 42, 30, GROUND)
for x in range(20, 25):
    put(x, 13, '.', '/'); put(x, 14, '.', '/')
# 광장 남동 턱(+1m) — 꿰매는 사람의 천막 자리(경사로 하나로 오른다).
floor(37, 25, 41, 29, '6')
put(36, 27, '.', '/')
# 3. 뿌리 사이 골목(+2.5m) — 뿌리 벽으로 구불구불.
room(8, 30, 36, 47, ALLEY, 'c')
for (x0, y0, x1, y1) in [(12, 33, 20, 34), (24, 37, 32, 38), (12, 41, 22, 42), (26, 43, 28, 46)]:
    block(x0, y0, x1, y1, 'c')
# 골목 → 광장: 동쪽 경사로(행 31..36, 세 칸 폭) — 광장 벽 행 30에 틈.
for x in range(30, 33):
    put(x, 30, '.', '/')
    for y in range(31, 36):
        put(x, y, '.', '/')
# 물 모으는 사람의 곁방(골목 동쪽, 한 칸 틈).
room(36, 38, 43, 46, ALLEY, 'c')
put(36, 42, '.', ALLEY)
# 4. 위쪽 거주지(+4m) — 남쪽, 도착.
room(6, 47, 38, 59, HOME, 'c')
# 거주지 → 골목: 서쪽 경사로(행 45..47, 세 칸 폭).
for x in range(12, 15):
    for y in range(45, 48):
        put(x, y, '.', '/')
marks = {
    # 도착·오르는 길·화로·장치·숨은 물건·빛줄기·종·주민 자리
    'U': [(22, 57)], 'P': [(22, 55)],
    'R': [(10, 55), (30, 39), (22, 17)],
    'K': [(9, 45)],
    'S': [(30, 52), (20, 39), (14, 24), (22, 8), (40, 42)],
    'L': [(22, 2)], 'W': [(22, 6)],
    '1': [(34, 51)], '2': [(41, 43)], '3': [(39, 27)],
    # 서 있는 적: a 잔재 · T 가시짐승 · h 등딱지짐승 · o 꼬투리 · e 청소자(같은 잔재, 다른 역할)
    # 0124: 성소만큼 촘촘하게(15 → 33 + 엘리트 셋) — 주민 끈 밖·화로 곁·도착 자리는 비운다(시험).
    'a': [(16, 49), (14, 58), (12, 39), (20, 35), (24, 40), (14, 44), (9, 31),
          (18, 26), (26, 25), (6, 25), (12, 28), (31, 15), (36, 15)],
    'T': [(16, 32), (23, 46), (8, 22), (16, 15), (30, 19)],
    'h': [(22, 44), (18, 39), (14, 21), (28, 20), (24, 27)],
    'o': [(26, 32), (28, 34), (16, 36), (10, 18), (33, 17), (4, 20)],
    'e': [(22, 50), (9, 49), (20, 28), (10, 27)],
    # 엘리트(0121 — 셋, 지금 성질): X 굳은 등딱지짐승(광장 가운데) · Y 되풀이 꼬투리(골목) · Z 남는 잔재(광장 동쪽)
    'X': [(20, 23)], 'Y': [(10, 35)], 'Z': [(27, 23)],
    # 소품
    'i': [(14, 17), (30, 17), (16, 3), (28, 3)],
    'r': [(10, 32), (34, 33), (18, 45), (30, 45), (8, 28), (40, 16)],
    'u': [(26, 52), (12, 51), (36, 22), (6, 15)],
    'b': [(14, 2), (30, 2), (8, 49), (36, 49)],
}
for c, spots in marks.items():
    for x, y in spots:
        if M[y][x] != '.':
            raise SystemExit(f"{c} at ({x},{y}) lands on {M[y][x]!r}")
        M[y][x] = c
out_m = '\n'.join(''.join(r) for r in M)
out_h = '\n'.join(''.join(r) for r in HT)
for y in range(H):
    print(f"{y:2} {''.join(M[y])}  {''.join(HT[y])}")
if '--write' in sys.argv:
    s = open(SRC).read()
    pre, rest = s.split('const MAP: &str = "\n', 1); _, rest = rest.split('";', 1)
    s = pre + 'const MAP: &str = "\n' + out_m + '\n";' + rest
    if 'const HEIGHTS: &str = "\n' in s:
        pre, rest = s.split('const HEIGHTS: &str = "\n', 1); _, rest = rest.split('";', 1)
        s = pre + 'const HEIGHTS: &str = "\n' + out_h + '\n";' + rest
    else:
        s = s.rstrip('\n') + '\n\n/// 높낮이(0121): `MAP`과 같은 칸 — 생성기가 함께 쓴다.\nconst HEIGHTS: &str = "\n' + out_h + '\n";\n'
    open(SRC, 'w').write(s)
    print('wrote', SRC)
