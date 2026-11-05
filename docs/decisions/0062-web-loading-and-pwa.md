# 0062. 웹 로딩 화면 + PWA(설치·오프라인)

- 날짜: 2026-11-05
- 상태: 채택·구현. 사장: "웹버전의 로딩 인디케이터 및 pwa"
- 근거: KB `reference/macroquad/wasm-build.md`("로딩 화면 (번들을 고치지 않고)"), `reference/web/pwa-install-fullscreen-orientation.md`, 첫 게임 nail 0269(서비스 워커 틀)·로딩 덮개.

## 무엇이 문제였나
- 웹판은 wasm(약 2.6 MB)을 받고 컴파일하는 동안 검은 화면이었다 — 느린 회선에서는 멈춘 것처럼 보인다.
- 폰에서 홈 화면에 깔 수 없고(매니페스트 없음), 오프라인으로 뜨지 않는다.

## 결정
- **로딩 화면은 글 없이**: 앱 아이콘의 호박색 종(`web/icons/bell-128.png`, 32px 그림을 최근접 4배)이 흔들리고 아래 막대가 찬다. 바탕은 게임의 지우는 색 `#12121f`. 이 시점엔 게임의 언어·글꼴이 없어서(nail과 같은 까닭) 글을 쓰지 않는다. 움직임 줄이기 설정이면 흔들지 않는다.
- **진행 = wasm 바이트**: nail과 달리 모델·글꼴·소리가 모두 바이너리 안이라 따로 받는 파일이 없다. 번들보다 먼저 `fetch`를 감싸 `*.wasm` 바이트로 0~90%, `max(Content-Length, 받은 양 × 1.05)`로 나누고 95%에서 멈춘다(압축 전송). 조각을 모으지 않고 흘려 보내 `compileStreaming`이 받는 동안 컴파일한다.
- **걷는 때 = 첫 프레임 뒤**: `src/main.rs`가 첫 `next_frame().await` 뒤에 `knell_loaded()`(플러그인 `knell_loading`) — 그 전에 걷으면 검은 화면이 한 번 보인다.
- **실패는 붉게**: 번들은 컴파일·시작 실패를 `console.error(Error)`로만 알린다 → 걷기 전에 오면 막대를 붉게, 종은 멈추고 회색. 첫 프레임 전 패닉은 `error` 이벤트로 — 번들을 읽은 뒤에 듣는다(번들의 알려진 잡음 quad_net `register_plugin`이 읽을 때 나서).
- **PWA**: `web/manifest.webmanifest`(fullscreen·landscape·색·아이콘 셋), 아이콘은 `tools/icons/make_icons.py`의 `web_icons`가 Android 아이콘과 같은 원본에서(192·512 둥근, maskable 512 — 그림 60% + 바탕 꽉, apple-touch 180 불투명). dev 판은 이름 "Knellroot (dev)"(조립이 바꾼다).
- **서비스 워커 `web/sw.js`**(nail 0269 틀): 캐시 이름 = `knell-<빌드 아이디>`(조립이 `__BUILD__`를 `build.txt`와 같은 값으로) — wasm 이름에 해시가 없어 배포마다 sw.js가 바뀌어야 새 판이 깔린다. `skipWaiting` 없음 → 새 판은 다음 실행부터. 페이지·`build.txt`·매니페스트는 네트워크 먼저, 나머지는 캐시 먼저. https·localhost에서만, PR 미리 보기(`pr-N--`)에는 깔지 않는다. 기록(localStorage)은 건드리지 않는다.
- **헤더**: `_headers`에 `sw.js`·매니페스트 `no-cache`, 매니페스트 MIME.

## 확인
- `tools/test/pwa.js`(CI 연기 시험에 더함): 64 KB씩 흘린 wasm에 막대 값 24개가 중간을 지나고 거꾸로 가지 않음, 첫 프레임 뒤 덮개가 사라짐, 매니페스트·아이콘 크기, 서비스 워커 activated, 캐시 이름 = `knell-<빌드>`, wasm 미리 담김, 네트워크를 끊고 다시 읽어도 타이틀이 뜸. 콘솔 에러 없음. `shot.js` 다시 통과.

## 사람이 볼 것
- 폰(Android Chrome)에서 "홈 화면에 추가" → 아이콘(둥근·maskable 잘림), 가로 전체 화면으로 뜨는지. iPhone은 홈 화면 앱의 기록이 Safari 탭과 따로다(KB, nail에서 확인).
- 느린 회선에서 종·막대가 보기 좋은지.
