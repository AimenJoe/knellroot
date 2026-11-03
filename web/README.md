# web/

브라우저용 정적 파일. `tools/web/assemble.sh dist`가 `index.html` + `*.js` + `knellroot.wasm`을 `dist/`에 모은다.

- `mq_js_bundle.js`: macroquad 레포 `js/mq_js_bundle.js` 복사본(nail `web/`에서 가져옴, macroquad 0.4.16 줄기, 수정 없음). macroquad 버전을 올리면 같은 태그의 파일로 교체. 핫링크 금지(KB `reference/macroquad/wasm-build.md`).
- `macroquad-gamepads-0.1.js`: `gamepads` 0.1.7 플러그인 복사본(nail에서 두 곳 고친 판: 버전 정수 `65543`, 비표준 매핑 패드 허용 — nail 0004).
