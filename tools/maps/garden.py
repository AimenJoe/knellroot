#!/usr/bin/env python3
"""셋째 지역(잔뿌리의 정원, 임시명 — 0185, 설계 docs/lore/story-arc-and-quests.md Stage 3) 지도 생성기 — MAP·HEIGHTS를 같은 칸으로.

침투형 지역(뿌리의 생장·침범·잔존이 한 곳에). 북쪽 = 깊은 곳(구역은 위로 갈수록 다음 구역, 0113 규칙):
  A 입구와 물 고인 낮은터(남쪽 — 오르는 길 U·도착 P는 +2m 턱, 경사로로 0m 낮은터 — 물웅덩이 `~`, 비적대 생태, 첫 싸움을 강요하지 않는다)
  → B 갈라진 뿌리길(+1m — 가운데 본길(정면 싸움)과 서쪽 좁은 샛길, 굵은 뿌리 덩어리가 가른다)
  → C 빈 껍질 웅덩이(0m — 넓은 혼합 생태, 마른 물길 `=`, 청소 행동과 적대 행동이 한 화면에)
  → D 문턱 아래(+1m — 문지기 W, 가운데는 비우고 가장자리에 엄폐).
E(아래로 열린 굴)는 캠페인 확장이 승인될 때의 연결 후보라 지금 지도에는 없다(마지막 지역 — 바닥 길 D가 없다).
지름길은 없다(화로 셋).

    python3 tools/maps/garden.py            # 미리보기만(지도 | 높이)
    python3 tools/maps/garden.py --write    # src/stage/garden.rs 의 MAP·HEIGHTS를 바꿔 쓴다

높이 글자: 36진수 v -> (v - 4) * 0.5m, '/' = 경사로(곧게 이어진 경사로 칸이 양 끝 높이를 잇는다 — 칸마다 0.5m 이하).
빈 곳(허공)은 낮은 땅(−2m). 쓴 뒤에는 cargo test stage:: (도달·높이·화로 안전). 다시 돌려도 같은 지도(멱등).
표지: `~` 고인 물 · `=` 마른 물길(걷는 바닥 위의 그림 표지 — 렌더가 읽는다).
"""
import sys
SRC = 'src/stage/garden.rs'
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

LOW, MID, LEDGE = '4', '6', '8'   # 0m · +1m · +2m
# D 문턱 아래(+1m) — 북쪽 끝, 문지기 싸움터. 가운데는 비우고 가장자리에 엄폐(뿌리·돌무더기 소품).
room(11, 0, 33, 12, MID, 'c')
# C 빈 껍질 웅덩이(0m) — 넓은 혼합 생태. D로 오르는 경사로(행 12..13, 가운데 다섯 칸).
room(2, 12, 42, 29, LOW, 'a')
for x in range(20, 25):
    put(x, 12, '.', '/'); put(x, 13, '.', '/')
# 웅덩이 가장자리의 굳은 뿌리 덩어리(한가운데를 막지 않는다 — 싸움터는 넓게).
for (x0, y0, x1, y1) in [(6, 15, 9, 16), (34, 16, 37, 17), (14, 24, 16, 26), (29, 24, 31, 26)]:
    block(x0, y0, x1, y1, 'a')
# B 갈라진 뿌리길(+1m) — 본길(가운데)과 서쪽 샛길을 굵은 뿌리 덩어리가 가른다. 동쪽은 곁방.
room(2, 29, 42, 45, MID, 'c')
block(8, 31, 15, 44, 'c')     # 본길과 샛길 사이의 뿌리 벽
block(29, 31, 31, 44, 'c')    # 본길과 동쪽 곁방 사이
for x in range(29, 32):
    put(x, 37, '.', MID)      # 곁방 문(한 칸 폭, 뿌리 벽 두께만큼)
# B → C: 본길 경사로(행 29..30) + 샛길 경사로(서쪽, 행 29..30).
for x in range(19, 26):
    put(x, 29, '.', '/'); put(x, 30, '.', '/')
for x in range(3, 7):
    put(x, 29, '.', '/'); put(x, 30, '.', '/')
# A 입구와 물 고인 낮은터 — 낮은터(0m, 행 46..50)와 남쪽 턱(+2m, 행 55..58).
room(4, 45, 40, 59, LOW, 'c')
floor(5, 55, 39, 58, LEDGE)
block(5, 51, 17, 54, 'c'); block(27, 51, 39, 54, 'c')
for x in range(18, 27):
    for y in range(51, 55):
        put(x, y, '.', '/')   # 턱 → 낮은터: 네 칸 경사로(2m)
# A → B: 경사로(행 45..46, 가운데 일곱 칸).
for x in range(19, 26):
    put(x, 45, '.', '/'); put(x, 46, '.', '/')
water = [(x, y) for y in (47, 48, 49) for x in range(7, 13)] + [(x, y) for y in (48, 49, 50) for x in range(30, 37)]
for x, y in water:
    M[y][x] = '~'
channel = [(x, 20) for x in range(5, 40) if x not in range(18, 27)] + [(x, 21) for x in range(10, 18)]
for x, y in channel:
    if M[y][x] == '.':
        M[y][x] = '='
marks = {
    # 도착·오르는 길·화로·숨은 물건·빛줄기·문지기
    'U': [(22, 58)], 'P': [(22, 56)],
    'R': [(11, 56), (22, 43), (36, 15)],
    'K': [(4, 31), (41, 14)],
    'H': [(5, 47)],
    'O': [(38, 40)],
    'C': [(3, 23)],
    'S': [(22, 49), (5, 38), (22, 22), (22, 7)],
    'W': [(22, 5)],
    # 서 있는 적(구역 등급 3): a 잔재 · T 가시짐승 · h 등딱지짐승 · o 꼬투리 · e 청소자(같은 잔재, 다른 역할 — 먹고 있다).
    # A에는 적대 개체를 두지 않는다(설계 — 첫 싸움을 강요하지 않음). B 초입은 하나씩, C는 섞어서.
    'a': [(22, 37), (18, 33), (26, 33), (12, 22), (32, 22), (25, 15), (40, 26)],
    'T': [(23, 41), (34, 41), (8, 18)],
    'h': [(20, 34), (28, 18), (17, 14)],
    'o': [(4, 35), (36, 33), (38, 23)],
    'e': [(10, 26), (13, 18), (27, 27), (8, 24)],
    # m 이끼먹이(비적대 — 물가에서 뜯고 다가오면 달아난다): 낮은터 물웅덩이 곁 넷 · 웅덩이 가장자리 둘.
    'm': [(14, 47), (6, 50), (28, 47), (38, 49), (24, 26), (39, 24)],
    # 엘리트: X 굳은 등딱지짐승(웅덩이 가운데) · Y 되풀이 꼬투리(샛길 끝) · Z 남는 잔재(웅덩이 동쪽)
    'X': [(20, 18)], 'Y': [(5, 42)], 'Z': [(39, 19)],
    # 소품: r 뿌리 · u 돌무더기 · i 오래된 돌기둥 조각(성소 건축을 되풀이하지 않게 둘만)
    'r': [(13, 3), (31, 3), (13, 10), (31, 10), (3, 14), (41, 28), (17, 36), (27, 36), (6, 57), (38, 57), (24, 47)],
    'u': [(15, 7), (29, 7), (9, 27), (35, 27), (20, 40), (14, 49)],
    'i': [(18, 2), (26, 2)],
}
for c, spots in marks.items():
    for x, y in spots:
        if M[y][x] not in '.':
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
    pre, rest = s.split('const HEIGHTS: &str = "\n', 1); _, rest = rest.split('";', 1)
    s = pre + 'const HEIGHTS: &str = "\n' + out_h + '\n";' + rest
    open(SRC, 'w').write(s)
    print('wrote', SRC)
