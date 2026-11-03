# 고려 대상: t3ssel8r식 "3D 픽셀 아트" 렌더(2026-11-03, 사장: "고려 대상으로만 기억해 두자 — 멋있잖아")

결정 아님. 화풍(톤·그림)을 정할 때 다시 연다. 지금 순서는 P4·P5 먼저(0013).

## 무엇인가(2차 출처 — 원본 영상 "3D pixel art rendering in Unity"는 직접 보지 못함)
- 낮은 해상도로 3D를 그려 정수 배로 키움, 카메라를 픽셀 격자에 붙여 픽셀이 기어가지 않게(texel snapping).
- 1px 외곽선(바깥 실루엣·안쪽 접힌 모서리 따로), 빛을 띠로 끊은 툰 음영, 그림자·풀·손으로 고른 팔레트, 낮은 프레임 애니메이션 — 절반은 미술.
- 출처: Texel Splatting(Ebert, arXiv 2026 — t3ssel8r 인용) https://arxiv.org/html/2603.14587v1 · Unreal 플러그인 설명 https://www.fab.com/listings/816f34f2-1a2e-4910-b3dd-009b5da0873b · 깊이·법선 외곽선(Unity 문서) https://docs.unity3d.com/353/Documentation/Components/script-EdgeDetectEffectNormals.html

## Knellroot와 맞는 것
- 이미 480×270 렌더 타깃 → 정수 배율, 상자 윤곽선이 1px 선(안쪽 모서리 선 구실).
- 빛 띠: 정점 색 음영(`shaded_box`·`ModelBatch`)을 단계로 자르면 된다.
- 깊이 외곽선 후처리: macroquad 0.4.16 `render_target_ex(.., RenderTargetParams { depth: true })`가 `depth_texture`를 준다(소스 확인).

## 부딪히는 것
1. **픽셀 고정은 정사영에서만**("pixel creep can only ever be solved for orthographic projections" — ProPixelizer, 위 논문 인용). 숄더뷰는 원근이어야 하고, 쿼터뷰를 정사영으로 바꾸면 전환이 투영까지 바꾼다 — KB 카메라 노트의 "정사영↔원근은 보간 안 됨, FOV 격차는 멀미"와 충돌. 타협: 쿼터뷰만 멀리서 아주 좁은 FOV(정사영 근사), 숄더뷰는 약간의 반짝임 감수. 땅 돌리기(회전)는 도는 동안 다시 찍혀 반짝인다.
2. 비용: 외곽선 후처리 패스·진짜 그림자(섀도맵 — macroquad에 없음)는 Android 핸드헬드에 부담. 둥근 그림자로 대신.
3. 셰이더만으로는 안 나온다 — 텍스처·팔레트·애니메이션이 같이.

## 음침하게
기법과 분위기는 따로다. 컨셉 팔레트(숯빛 검정·돌 회색·바랜 갈색·호박색 발광) + 빛 띠 2~3단 + 호박색 점광 + 안개로 어둡게. 1px 외곽선은 어두운 화면에서 실루엣을 지켜 준다. 단, 어둡고 낮은 해상도는 예고·적 실루엣을 묻을 수 있다 → P1 판정 때 색약·흑백 검사(첫 게임 `tools/a11y/cvd.py` 방식).

## 다시 열 때 할 반나절 실험(P1 읽힘 + P5 비용)
랩에서 켜고 끄기: 빛 띠 단계 · 팔레트 줄이기 · 깊이 외곽선 후처리 · 쿼터뷰 좁은 FOV. 폰에서 `draws`·`cpu`.
