#!/usr/bin/env python3
"""넷째 지역(작업층, 임시명 — 0208, 설계 docs/decisions/0208-stage-3-worn-workings-design.md) 지도 생성기 — MAP·HEIGHTS를 같은 칸으로.

멈추지 못한 일터(잔존·재활동). 북쪽 = 깊은 곳(구역은 위로 갈수록 다음 구역, 0113 규칙). 0208 13절 5(사장 "지름길 넣으려면 좀더
입체적인 스테이지여야")에 따라 **높이가 겹치는 층**으로 — 한 칸에 높이 하나뿐이라(층을 포갤 수 없다) 위층은 아래층 곁을 따라
내려다보며 지나가고, 내려갈 때는 뛰어내리고(한 방향) 올라올 때는 화물 승강대(문 `G` + 장치 `V` — 0113의 문을 승강대로 쓴다):
  A 내림 갱(남쪽 — 도착·오르는 길은 +5m 작업대, 서쪽으로 +2.5m 작업대를 거쳐 회랑으로 내려간다. 첫 화로)
  → B 운반 회랑(0m — 길고 곧은 레일 `|`(수레가 오가는 줄, 0208 4절), 서쪽 곁방, 동쪽은 +5m 위층 회랑이 내려다보는 벼랑)
  ↗ 위층 회랑(+5m — A에서 곧장 북쪽으로. B를 내려다보며 수레의 박자를 먼저 본다. 끝에서 B로 뛰어내린다. 북쪽 끝은 승강대 꼭대기)
  → C 배수장(둘레 회랑 +1.5m · 가운데 꺼진 웅덩이 −0.5m — 둘째 화로, 승강대 장치 `V`: 당기면 C ↔ 위층 회랑 ↔ A가 이어진다)
  → D 권양기 갱도(0m — 둥근 갱, 문지기 `W`. 문 밖 곁방에 셋째 화로).
마지막 지역이라 바닥 길(`D`)이 없다 — 핵을 거두면 판이 끝난다(엔딩이 정원에서 이리로 옮겨 온다).

    python3 tools/maps/workings.py            # 미리보기만(지도 | 높이)
    python3 tools/maps/workings.py --write    # src/stage/workings.rs 의 MAP·HEIGHTS를 바꿔 쓴다

높이 글자: 36진수 v -> (v - 4) * 0.5m, '/' = 경사로(곧게 이어진 경사로 칸이 양 끝 높이를 잇는다 — 칸마다 0.5m 이하).
빈 곳(허공)은 낮은 땅(−2m). 쓴 뒤에는 cargo test stage:: (도달·높이·화로 안전). 다시 돌려도 같은 지도(멱등).
표지: `|` 레일(걷는 바닥 위의 표지 — 수레가 그 줄을 오간다, S4-4).
"""
import sys
SRC = 'src/stage/workings.rs'
W, H = 45, 64
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
def ramp_rows(x0, x1, y0, y1):
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            put(x, y, '.', '/')
def ramp_cols(x0, x1, y0, y1):
    ramp_rows(x0, x1, y0, y1)

SUMP, LOW, RING, BENCH, UPPER = '3', '4', '7', '9', 'e'   # −0.5m · 0m · +1.5m · +2.5m · +5m
WALL = 'i'                                                # +7m — 위층(+5m)보다 높게(갱의 벽)

# D 권양기 갱도(0m) — 북쪽 끝, 문지기 싸움터. 둥근 갱(네 귀퉁이를 벽으로 깎는다), 가운데는 비운다.
room(9, 0, 35, 12, LOW, WALL)
for (x0, y0, x1, y1) in [(10, 1, 12, 2), (32, 1, 34, 2), (10, 10, 12, 11), (32, 10, 34, 11)]:
    block(x0, y0, x1, y1, WALL)
for x in range(20, 25):
    put(x, 12, '.', LOW)                     # 갱도의 문(열린 채 — 이 지역의 문 `G`는 승강대다)
for x in range(11, 34):
    for y in (8, 9):
        put(x, y, '|', LOW)                  # 갱도를 가로지르는 레일(문지기 2단계에 수레가 돈다 — 0208 6절)
# 문 밖 곁방(0m) + C로 오르는 경사로(1.5m를 넷 칸에).
room(16, 12, 28, 17, LOW, WALL)
for x in range(20, 25):
    put(x, 12, '.', LOW)
ramp_rows(20, 24, 15, 17)

# C 배수장 — 둘레 회랑(+1.5m, 폭 셋) + 가운데 꺼진 웅덩이(−0.5m). 둘레에서 웅덩이로는 2m 벼랑(뛰어내림), 동·서 경사로로 오간다.
room(4, 17, 40, 34, RING, WALL)
for x in range(20, 25):
    put(x, 17, '.', '/')
floor(8, 21, 36, 30, SUMP)
ramp_cols(8, 11, 25, 26)                     # 서쪽 경사로: 둘레(x 7, +1.5) → 웅덩이(x 12, −0.5)
ramp_cols(33, 36, 25, 26)                    # 동쪽 경사로
# 웅덩이의 펌프 받침(배경 소품 자리 — 둘, 가운데는 비운다)
block(14, 23, 15, 24, RING); block(29, 27, 30, 28, RING)

# 승강대: C 둘레 남동(+1.5m)에서 위층 회랑 북쪽 끝(+5m)까지 — 문 `G`(한 줄, 0113의 문) + 경사 승강로 일곱 칸.
for x in range(36, 40):
    put(x, 34, 'G', RING)
ramp_rows(36, 39, 35, 41)
block(31, 35, 35, 41, WALL)                  # 승강로 서쪽 벽(회랑과 가른다)
block(40, 35, 40, 41, WALL)

# B 운반 회랑(0m) — 길고 곧은 레일, 서쪽 곁방 셋. C 둘레 남쪽(+1.5m)에서 경사로로 내려온다.
room(4, 34, 30, 48, LOW, WALL)
ramp_rows(6, 11, 34, 37)                     # C → B(1.5m를 다섯 칸에)
for y in range(36, 48):
    for x in (17, 18):
        put(x, y, '|', LOW)                  # 레일 두 줄(수레 폭)
block(5, 40, 9, 40, WALL); block(5, 44, 9, 44, WALL)   # 서쪽 곁방 칸막이(창고·쉼터 자리)
# 동쪽: 위층 회랑이 내려다보는 벼랑 — 벽을 걷는다(B 0m · 회랑 +5m, 내려다보고 뛰어내린다).
for y in range(42, 48):
    put(30, y, '.', LOW)

# 위층 회랑(+5m) — A의 작업대에서 북쪽으로, 승강로 꼭대기까지.
room(30, 41, 41, 59, UPPER, WALL)
floor(31, 42, 40, 59, UPPER)
for y in range(42, 48):
    put(30, y, '.', LOW)                     # 벼랑(위의 room이 벽으로 덮은 것을 다시 연다)
floor(36, 41, 39, 41, '/')                   # 승강로 끝(경사로가 회랑 높이로)

# A 내림 갱 — 도착 작업대(+5m, 남쪽 끝 세 줄)와 서쪽 +2.5m 작업대, 그 아래 B로 내려가는 경사로.
room(4, 48, 41, 63, BENCH, WALL)
floor(5, 49, 29, 54, BENCH)
floor(5, 60, 40, 62, UPPER)
floor(31, 49, 40, 59, UPPER)
block(5, 55, 9, 59, WALL); block(17, 55, 29, 59, WALL)
ramp_rows(10, 16, 55, 59)                    # 도착 작업대(+5m) → 서쪽 작업대(+2.5m): 2.5m를 여섯 칸에
for x in range(20, 27):
    for y in range(44, 50):
        put(x, y, '.', '/')                  # 서쪽 작업대(+2.5m) → B(0m): 2.5m를 일곱 칸에
for y in range(49, 60):
    put(30, y, '#', WALL)                    # 서쪽 작업대와 위층 회랑은 벽으로 가른다(오가는 길은 남쪽 도착 작업대)
marks = {
    # 도착·오르는 길·화로·숨은 물건·빛줄기·문지기·승강대 장치
    'U': [(22, 62)], 'P': [(22, 61)],
    'R': [(13, 61), (6, 19), (22, 14)],
    'V': [(38, 32)],
    'K': [(38, 43)],
    'O': [(6, 46), (38, 19)],
    'H': [(26, 50)],
    'C': [(6, 36)],
    'N': [(34, 61), (34, 19), (18, 14)],
    'S': [(16, 52), (12, 42), (22, 25), (22, 6)],
    'W': [(22, 5)],
    # 서 있는 적(구역 등급 4 — S4-3에서 새 종으로 바꾼다. 지금은 있는 종 자리표): a 잔재 · h 등딱지짐승 · o 꼬투리 · e 청소자
    'a': [(13, 38), (24, 41), (10, 46), (20, 52), (26, 22), (18, 29), (35, 31)],
    'h': [(24, 37), (16, 27), (31, 23)],
    'o': [(7, 42), (37, 28), (36, 51)],
    'e': [(26, 46), (11, 31), (28, 29)],
    # 엘리트: X 굳은(웅덩이 가운데) · Y 되풀이(회랑 북쪽) · Z 남는(둘레 동쪽)
    'X': [(22, 27)], 'Y': [(13, 36)], 'Z': [(38, 22)],
    # 소품: i 권양기 기둥(갱도) · u 돌무더기
    'i': [(15, 4), (29, 4), (14, 11), (30, 11)],
    'u': [(25, 36), (8, 38), (33, 45), (26, 19), (12, 51)],
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
