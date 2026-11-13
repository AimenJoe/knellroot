#!/usr/bin/env python3
"""0번째 지역(침묵의 성소) 지도 생성기(0113) — MAP·HEIGHTS 두 격자를 같은 칸으로 만든다.

손으로 두 격자를 맞추면 칸이 어긋나기 쉬워(경사로·벽 높이) 방·바닥·경사로·표지를 코드로 놓는다.
성소(위 12줄, 문지기 방)는 지금 src/stage/sanctuary.rs 의 것을 그대로 읽어 온다 — 그 방을 고치려면 파일에서 고치고 이것을 다시 돌린다.

    python3 tools/maps/sanctuary.py            # 미리보기만(지도 | 높이)
    python3 tools/maps/sanctuary.py --write    # src/stage/sanctuary.rs 의 MAP·HEIGHTS를 바꿔 쓴다

높이 글자: '.' = 0, 36진수 v -> (v - 4) * 0.5m, '/' = 경사로(곧게 이어진 경사로 칸 묶음이 양 끝 높이를 잇는다 — 칸마다 0.5m 이하로,
시험 every_floor_is_reachable_and_every_spawn_can_walk_to_the_fight). 빈 곳(허공)은 낮은 땅(-2m).
쓴 뒤에는 cargo test stage:: (도달·높이·화로 안전·엘리트)를 본다. 다시 돌려도 같은 지도가 나온다(멱등).
"""
import sys
SRC = 'src/stage/sanctuary.rs'
src = open(SRC).read()
cur_map = src.split('const MAP: &str = "\n')[1].split('";')[0].split('\n')
cur_h = src.split('const HEIGHTS: &str = "\n')[1].split('";')[0].split('\n')
W, H = 43, 66
M = [[' '] * W for _ in range(H)]
HT = [['0'] * W for _ in range(H)]   # 빈 곳(허공) 기본은 낮은 땅(−2m)

def put(x, y, c, h=None):
    M[y][x] = c
    if h is not None:
        HT[y][x] = h
def room(x0, y0, x1, y1, h, wall_h=None):
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            edge = x in (x0, x1) or y in (y0, y1)
            if edge:
                if M[y][x] == ' ':
                    put(x, y, '#', wall_h if wall_h else h)
            else:
                put(x, y, '.', h)
def floor(x0, y0, x1, y1, h, c='.'):
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            put(x, y, c, h)
def heights(x0, y0, x1, y1, h):
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            HT[y][x] = h

# 1. 성소(지금 파일의 위 12줄 그대로) + 둘레 언덕.
for y in range(12):
    for X, (c, h) in enumerate(zip(cur_map[y].ljust(W), cur_h[y].ljust(W))):
        M[y][X] = c
        HT[y][X] = h
# 2. 뜰(넓은 싸움터) — 북쪽 벽 12줄에 문(19..23).
room(5, 12, 37, 31, '.')
for x in range(19, 24):
    put(x, 12, 'G', '6')
    put(x, 13, '.', '/')           # 문 앞 경사로(뜰 0 → 문 +1m)
# 뜰의 높낮이: 북서 단(+1m)·가운데 꺼진 터(−0.5m)·남동 단(+0.5m)
heights(7, 15, 11, 18, '6');  heights(12, 16, 12, 17, '/')
heights(18, 23, 24, 26, '3'); heights(20, 22, 22, 22, '/'); heights(20, 27, 22, 27, '/')
heights(29, 25, 35, 29, '5'); heights(31, 24, 33, 24, '/')
# 3. 서쪽 성벽길(+3m) — 북쪽 끝에 문 장치, 뜰로 떨어지는 틈(한 방향 지름길), 남쪽 끝 경사로로 뿌리 굴.
room(0, 12, 5, 37, 'a', 'a')
floor(1, 13, 4, 32, 'a')
put(5, 13, '.', 'a'); put(5, 14, '.', 'a')
for y in range(12, 38):
    if M[y][5] == '#':
        HT[y][5] = 'a'
floor(1, 33, 4, 37, '/')
# 4. 동쪽 무너진 탑(+1m) — 뜰 동쪽 벽의 틈으로 경사로.
room(37, 15, 42, 28, '6', '6')
put(37, 21, '.', '/'); put(37, 22, '.', '/')
# 5. 무너진 길(뜰 남쪽 → 다리).
for x in range(18, 25):
    put(x, 31, '.', '.')
room(15, 31, 27, 47, '.')
for x in range(18, 25):
    put(x, 31, '.', '.')
# 6. 뿌리 굴(서남, 0m) — 길 서쪽 틈으로 이어지고 성벽길 경사로가 내려온다.
room(0, 37, 15, 50, '.')
floor(1, 37, 4, 38, '/')
for y in range(41, 44):
    put(15, y, '.', '.')
# 7. 다리(길 → 입구, 0 → −1m) + 양옆 낭떠러지.
for x in range(19, 24):
    put(x, 47, '.', '.')
room(18, 47, 24, 52, '/')
for x in range(19, 24):
    put(x, 47, '.', '.')
floor(19, 48, 23, 51, '/')
for y in range(47, 53):
    for x in (18, 24):
        if M[y][x] == '#':
            HT[y][x] = '0'
# 8. 입구 폐허(−1m).
room(10, 52, 32, 65, '2', '2')
for x in range(19, 24):
    put(x, 52, '.', '2')

marks = {
    # 화로·상자·숨은 물건·장치·시작
    'R': [(30, 14), (8, 47), (25, 59)],
    # 상자(0117 — 위에서부터 차례 = 기록 번호): 탑(장신구) · 성벽길(쇠뇌) · 뜰 남동 단(지팡이) · 뿌리 굴 안쪽(망치).
    # 입구 폐허에는 상자·숨은 물건을 두지 않는다(초반에 강해지는 것이 몰리지 않게, 사장 2026-11-08).
    'C': [(40, 17), (2, 18), (34, 29), (2, 49)],
    'F': [(40, 26)],
    # 0123(아이템 덜 후하게): 뜰의 한 모금·뿌리 굴의 종소리 조각을 뺐다 — 뒤 차례라 앞 것의 기록 이름은 그대로.
    'K': [(1, 32)],
    'V': [(2, 13)],
    # 시작(0171): 꺼진 화로(R 25,59) 곁 — 방랑자는 거기 엎드려 깨어난다. 쉬는 거리(2.4m) 바로 밖이라 일어나자마자 '쉬기'가 뜨지는 않는다.
    'P': [(24, 60)],
    # 서 있는 적(0113): a 잔재 · T 가시짐승, 엘리트 X 굳은 잔재 · Y 되풀이 가시짐승 · Z 남는 잔재
    # 0117: 넓어진 만큼 밀도를 다시 — 24 → 39(엘리트 셋 따로).
    'a': [(10, 23), (12, 25), (17, 19), (23, 17), (27, 22), (30, 27), (32, 28),   # 뜰
          (15, 29), (25, 30), (34, 18), (7, 25),
          (2, 26), (3, 20), (2, 15), (3, 23),                                      # 성벽길
          (39, 19), (41, 21),                                                      # 탑
          (18, 35), (24, 36), (20, 41), (23, 43), (16, 38),                        # 길
          (5, 42), (10, 40), (12, 43), (13, 39),                                    # 뿌리 굴
          (17, 54)],                                                               # 입구(다리 앞)
    'T': [(20, 15), (26, 26), (9, 16), (3, 29), (40, 23), (39, 25), (21, 38),
          (14, 21), (28, 24), (25, 42), (19, 45), (25, 54)],
    'X': [(21, 20)], 'Y': [(32, 26)], 'Z': [(3, 41)],
    # 소품
    'i': [(9, 21), (14, 27), (28, 19), (33, 21), (16, 15), (26, 15), (17, 33), (25, 33), (17, 44), (25, 44), (10, 44)],
    'r': [(7, 29), (35, 14), (24, 28), (11, 24), (12, 39), (2, 45), (12, 54), (30, 54), (41, 16)],
    'u': [(8, 27), (30, 16), (20, 18), (21, 34), (7, 44), (14, 62), (28, 55), (38, 27)],
    'w': [(13, 20), (31, 23), (19, 39), (17, 58)],
    'b': [(6, 13), (36, 13), (6, 30), (36, 30), (13, 48), (12, 64), (31, 57), (16, 46), (26, 46), (19, 50), (23, 50)],
}
for c, spots in marks.items():
    for x, y in spots:
        if M[y][x] not in '.':
            raise SystemExit(f"{c} at ({x},{y}) lands on {M[y][x]!r}")
        M[y][x] = c
# 성소 안의 표지(W·L·K·H·b·r)는 지금 파일에서 그대로 들어왔다.
out_m = '\n'.join(''.join(r) for r in M)
out_h = '\n'.join(''.join(r) for r in HT)
for y in range(H):
    print(f"{y:2} {''.join(M[y])}  {''.join(HT[y])}")
if '--write' in sys.argv:
    s = src
    pre, rest = s.split('const MAP: &str = "\n', 1); _, rest = rest.split('";', 1)
    s = pre + 'const MAP: &str = "\n' + out_m + '\n";' + rest
    pre, rest = s.split('const HEIGHTS: &str = "\n', 1); _, rest = rest.split('";', 1)
    s = pre + 'const HEIGHTS: &str = "\n' + out_h + '\n";' + rest
    open(SRC, 'w').write(s)
    print('wrote', SRC)
