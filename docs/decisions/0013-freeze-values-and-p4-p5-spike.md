# 0013. 값 얼리기 + P4·P5 스파이크 준비

- 날짜: 2026-11-03
- 상태: 채택·구현(사장: "일단 값 프리징하고, 2번(P4·P5) 테스트해 볼 수 있게 준비해 줘").

## 왜
같은 날 세 갈래 교차검증(절차·설계·기술)이 같은 말을 했다:
- 오늘 커밋 대부분이 카메라 다듬기(0007~0011)와 땅 돌리기(0012)였고, **엔진 출구 조건(0001)인 P4(모델 파이프라인)·P5(성능)는 0** — KB의 "가장 비싼 실패부터"와 반대 순서.
- 판정은 0건인데 기본값은 대화로 줄줄이 바뀌었다(지상각 55→50→44→40, 회전 0.4→0.6→0.8, 대기 2.5→4). 랩 A/B → export 흐름을 건너뛰었다.

## 결정
1. **값 얼리기**: 지금 기본값 57개를 `docs/tuning-frozen.txt`에 적고, 시험 `frozen_defaults_match_the_snapshot`이 `tuning.rs`와 같은지 본다. 기본값을 바꾸려면: 랩 A/B → export → **판정**(phase-0-poc의 기준, 사람) → 결정 기록 → `FREEZE_WRITE=1 cargo test frozen`으로 파일을 다시 쓰고 같은 커밋에. 측정용 값을 새로 더하는 것은 기존 줄을 건드리지 않으면 된다(이번에 `render.model` 하나).
2. **P5 측정 준비**
   - 시나리오 6 "stress 40"·7 "stress 80"(아레나 22m, 잡몹이 고리 몇 겹으로 쫓아와 뭉친다, 기둥 넷).
   - HUD 노란 줄(폰에서도 보임): `cpu`(우리 코드가 한 프레임에 쓴 시간 — 틱 + 그리기 명령, 1초 평균·최대), `frame`(프레임 간격), `draws`(그리기 호출 수, `telemetry::capture_frame`으로 30프레임마다 한 프레임을 센다), `verts`(모델 정점 수).
   - `tools/test/stress.js`: 6·7 × 상자/모델 스크린숏(헤드리스).
3. **P4 로더 준비**
   - `src/model.rs`: `gltf` 1.4.1(`default-features = false`, `utils`·`names`) + `base64` — 노드 트리(TRS)·노드마다 메시(POSITION·NORMAL·TEXCOORD_0)·노드 TRS 애니메이션(LINEAR·STEP, CUBICSPLINE은 값만)·내장 PNG(data URI 또는 glb). 스키닝 없음(부위 강체).
   - `render::ModelBatch`: CPU에서 부위 행렬 × 정점, 면마다 고정 빛 음영을 정점 색에, 텍스처 하나로 모아 보낸다.
   - 시험 모델 `assets/models/test_critter.gltf` — `tools/art/make_test_model.py`가 Blockbench 꼴(그룹 = 피벗 노드, 상자 메시, 내장 텍스처, `walk`·`idle`)을 흉내 내 찍는다. **진짜 Blockbench 파일이 아니다** — 사람이 내보낸 파일로 축·단위·스킨 여부를 확인해야 P4가 닫힌다.
   - 랩 `render.model 0box 1gltf`(맨 끝 값): 잡몹을 시험 모델로.

## 잰 것(헤드리스 Chromium = 소프트웨어 렌더라 frame ms는 판정 근거가 아님, 두 번 돌림)
| 장면 | cpu 평균 | draws | 모델 정점 |
|---|---|---|---|
| 잡몹 40 · 상자 | 2.7~3.1ms | 97~99 | — |
| 잡몹 40 · 모델 | 2.5~2.9ms | **23** | 7,680 |
| 잡몹 80 · 상자 | 3.4~3.8ms | 179 | — |
| 잡몹 80 · 모델 | 4.5~4.6ms | **25~27** | 15,360 |
- 상자는 배우마다 면 → 윤곽선(선) → 코(면)라 그리기 모드가 바뀌어 배치가 끊긴다: 그리기 호출 ≈ 적 수 × 2 + 19. 모델 배치는 17마리쯤마다 한 번.
- **macroquad의 그리기 호출 하나 상한**: 정점 10,000·색인 5,000 미만(0.4.16 `quad_gl.rs` `geometry`, `Conf::draw_call_*_capacity` 기본값). 넘치면 경고만 내고 잘라 버린다 — 처음엔 40마리를 한 덩어리로 보내 모델이 하나도 안 그려졌다. 상한을 올리면 호출 슬롯마다 버퍼가 커진다(기술 검토: 슬롯당 약 400KB 추정) — 지금은 끊어 보낸다.
- **wasm 크기**: `gltf` + `base64`(+ serde_json)로 567,758 → 1,121,532 B(gzip 198 → 355KB). 다시 볼 조건: 무거우면 glTF를 빌드 때 우리 형식으로 바꿔 넣고(도구에서), 게임은 그 형식만 읽는다.

## 사람이 할 것(P4·P5 판정)
- **실기 측정**: https://knellroot.netlify.app → 6(잡몹 40)·7(잡몹 80), 랩 맨 끝 `render.model`을 0/1 — PC Chrome과 Android Chrome(핸드헬드급)에서 노란 줄을 사진으로. P5 기준: 데스크톱 60fps(frame ≈ 16.7ms), Android 30fps(≈ 33ms) 이상.
- **Blockbench 시험 모델**: 웹판(web.blockbench.net, 설치 없음) 또는 데스크톱판. Generic Model → 상자 둘(몸·팔) → 팔 그룹에 회전 키프레임 둘 → File › Export › glTF(Export Animations 켬, Embed textures 켬) → 파일을 `tools/art/sources/`에. 로더가 그대로 읽는지, 축·단위·그룹이 노드인지 스킨인지 본다.

## 다시 볼 조건
- 실기에서 Android가 30fps를 못 넘으면: 윤곽선을 한 패스로 모으기(상자), 그리기 호출 상한 올리기 실험, 그래도 안 되면 0001의 출구.
- Blockbench 파일이 스킨(joints)으로 나오면: 조인트 하나 = 가중치 1.0이면 지금 CPU 변형에 행렬 하나만 더하면 된다(KB 모델 노트).
