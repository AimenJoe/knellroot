"""제3자 라이선스 고지 파일 생성기(0135, nail tools/licenses.py에서 가져옴). 의존성·에셋 출처가 바뀌면 다시 돌린다(CLAUDE.md "커밋 전").

실행: python3 tools/licenses.py  → assets/THIRD_PARTY_LICENSES.txt (PC 묶음에 실린다 — build-desktop.yml)
- cargo metadata로 모든 크레이트(모든 플랫폼)의 이름·버전·SPDX를 모으고, 크레이트 폴더의 LICENSE*/COPYING*/NOTICE* 원문을 붙인다.
- 같은 원문은 한 번만 싣고 해당 크레이트를 나열한다.
- 웹 JS 복사본(mq_js_bundle.js = macroquad, macroquad-gamepads = gamepads), 글꼴(고운바탕 OFL),
  모델(Meshy 결과물 CC BY 4.0 — 표기 의무가 있다)·소리(CC0) 출처는 assets/LICENSES.md 표에서 읽어 싣는다.
MIT·BSD·Apache·ISC·Unicode 라이선스는 바이너리 배포에도 저작권 고지와 라이선스 원문을 함께 줄 것을 요구한다.
"""
import hashlib
import json
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, 'assets', 'THIRD_PARTY_LICENSES.txt')
PREFIXES = ('license', 'licence', 'copying', 'notice', 'unlicense')


def license_texts(pkg_dir):
    out = []
    try:
        names = sorted(os.listdir(pkg_dir))
    except FileNotFoundError:
        return out
    for n in names:
        if n.lower().startswith(PREFIXES):
            p = os.path.join(pkg_dir, n)
            if os.path.isfile(p):
                with open(p, encoding='utf-8', errors='replace') as f:
                    out.append((n, f.read().strip()))
    return out


def meshy_models():
    """assets/LICENSES.md 표에서 CC BY 4.0 줄(Meshy 결과물)의 파일과 작품 이름."""
    rows = []
    with open(os.path.join(ROOT, 'assets', 'LICENSES.md'), encoding='utf-8') as f:
        for line in f:
            if 'CC BY 4.0' not in line or not line.startswith('|'):
                continue
            cells = [c.strip() for c in line.strip().strip('|').split('|')]
            name = re.search(r'"([^"]+)"', cells[2])
            rows.append((cells[0].replace('`', ''), name.group(1) if name else '?'))
    return rows


def main():
    meta = json.loads(subprocess.check_output(
        ['cargo', 'metadata', '--format-version', '1'], cwd=ROOT))
    groups = {}   # hash -> [filename, text, [crates]]
    no_file = []
    for p in sorted(meta['packages'], key=lambda p: (p['name'], p['version'])):
        if p['name'] == 'knellroot':
            continue
        label = f"{p['name']} {p['version']} ({p.get('license') or 'license file'})"
        texts = license_texts(os.path.dirname(p['manifest_path']))
        if not texts:
            no_file.append(label)
            continue
        for fname, text in texts:
            h = hashlib.sha256(text.encode()).hexdigest()
            groups.setdefault(h, [fname, text, []])[2].append(label)

    lines = [
        'Knellroot — 제3자 라이선스 고지 / Third-party license notices',
        '이 파일은 tools/licenses.py가 만든다. 손으로 고치지 않는다. 출처 전체: assets/LICENSES.md',
        '',
        '== 모델 (AI 생성, CC BY 4.0 — https://creativecommons.org/licenses/by/4.0/) ==',
        '아래 몸 모델은 Gemini 그림을 Meshy(https://www.meshy.ai) 무료 플랜 Image to 3D로 만든 것이다(무료 플랜 결과물은 CC BY 4.0).',
        '이 레포의 스크립트가 줄이고 색을 다시 굽고 부위로 잘라 뼈에 붙였다(바꾼 것). 동작·무기·장신구·소품 모델은 이 레포의 스크립트.',
    ]
    lines += [f'  {f}: "{n}", made with Meshy, CC BY 4.0 (modified)' for f, n in meshy_models()]
    lines += [
        '',
        '== 글꼴 ==',
        'assets/fonts/GowunBatang-UI-{Regular,Bold}.ttf — Gowun Batang(고운바탕), Copyright 2021 The Gowun Batang Project Authors,',
        '  SIL Open Font License 1.1. 원문: assets/fonts/GowunBatang-OFL.txt (예약 폰트 이름 없음. 게임에 쓰는 글자만 남긴 서브셋)',
        '',
        '== 웹 JS ==',
        'web/mq_js_bundle.js: macroquad(not-fl3) — MIT OR Apache-2.0 (아래 macroquad 항목의 원문)',
        'web/macroquad-gamepads-0.1.js: gamepads(fornwall) — MIT OR Apache-2.0 (아래 gamepads 항목의 원문)',
        '',
        '== 소리 (CC0 1.0 — 표기 의무 없음, 고마움으로 적는다) ==',
        '효과음: Kenney (www.kenney.nl) — Impact Sounds, Interface Sounds, RPG Audio',
        '효과음: OpenGameArt — Kenney Interface Sounds, 50 RPG sound effects · SubspaceAudio 512 Sound Effects ·',
        '  artisticdude RPG Sound Pack, Swishes Sound Pack · Bobjt Gem collect SFX',
        '',
        '== Rust 크레이트 ==',
    ]
    for fname, text, crates in sorted(groups.values(), key=lambda g: g[2][0]):
        lines.append('-' * 72)
        lines.append(f'[{fname}] 적용 대상:')
        lines.extend('  ' + c for c in crates)
        lines.append('')
        lines.append(text)
        lines.append('')
    if no_file:
        lines.append('-' * 72)
        lines.append('라이선스 파일이 크레이트에 없는 항목(SPDX 표기만):')
        lines.extend('  ' + c for c in no_file)
    with open(OUT, 'w', encoding='utf-8') as f:
        f.write('\n'.join(lines) + '\n')
    print(f'{OUT}: {len(groups)} texts, {sum(len(g[2]) for g in groups.values())} crate entries, {len(no_file)} without file')


if __name__ == '__main__':
    sys.exit(main())
