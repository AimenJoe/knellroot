# web/

브라우저용 정적 파일. `tools/web/assemble.sh dist`가 `index.html` + `*.js` + 매니페스트·`icons/` + `knellroot.wasm`을 `dist/`에 모은다(`sw.js`의 `__BUILD__`는 빌드 아이디로).

- 로딩 화면·PWA(0062): `index.html`의 덮개(종 + 막대, wasm 바이트로 진행, 첫 프레임 뒤 `knell_loaded`가 걷음), `manifest.webmanifest`, `sw.js`(캐시 = 빌드 아이디, `skipWaiting` 없음). 아이콘은 손으로 만들지 않는다 — `python3 tools/icons/make_icons.py`. 시험은 `tools/test/pwa.js`.

- `mq_js_bundle.js`: macroquad 레포 `js/mq_js_bundle.js` 복사본(nail `web/`에서 가져옴, macroquad 0.4.16 줄기, 수정 없음). macroquad 버전을 올리면 같은 태그의 파일로 교체. 핫링크 금지(KB `reference/macroquad/wasm-build.md`).
- `macroquad-gamepads-0.1.js`: `gamepads` 0.1.7 플러그인 복사본(nail에서 두 곳 고친 판: 버전 정수 `65543`, 비표준 매핑 패드 허용 — nail 0004).
