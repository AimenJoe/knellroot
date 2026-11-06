# Lab — 실행 중에 수치를 바꾸고, 느끼고, `tuning.rs`로 가져가기

KB `guides/poc-and-lab-before-content.md`의 구현. 첫 게임은 수치 한 곳(`tuning.rs`) + 시나리오 진입점까지였고 값 하나에 빌드 한 번이었다. 여기서는 **빌드 없이** 바꾼다.

## 흐름(0070 — 0066 창 틀 안의 개발 창)
1. 랩을 연다: 멈춤 메뉴 끝 줄 **LAB**(개발판), 키보드 `F1`. 터치는 왼쪽 위 멈춤 → LAB. 창이 열린 동안 게임은 멈춘다.
2. 랩 창 = 위 넷(**using** A/B 바꾸기 · **store current as B** · **export to console** · **reset all to defaults**) + 값 묶음 목록(카메라 쿼터뷰·락온·전환, 이동·회피, 공격·콤보, 강공격, 패링, 점프, 땅 돌리기, 락온·자동 조준, 피격·스태미나, 등잔, 마법·던지기, 자세, 동작, 그림 — `lab::GROUPS`, 이름 앞머리로 저절로 묶인다). 바뀐 값이 있는 묶음은 `*n`.
3. 묶음 창: 줄 = `이름 ◀ 값 ▶`. ←→(◀▶ 탭)로 한 걸음, 땅 돌리기 버튼(LT·RT / Q·E)으로 10걸음, 확인 = 그 값만 기본값으로. 바뀐 값은 `*`, 아래 줄에 기본값.
4. **바꾼 값은 바로 들어가고 창을 닫은 뒤에도 계속 쓰인다**(처음부터 해도 — 게임을 끄면 사라진다). **기본값은 얼려 있다(0013)** — 정한 값은 내보내서 판정 뒤에만 `tuning.rs`로.
5. "둘 중 어느 쪽"이 "이 값이 맞나"보다 답이 쉽다: store → 값을 바꾸고 → using으로 A/B를 번갈아 닫고 싸워 본다.
6. export → 콘솔에 `pub const NAME: f32 = 값;` 줄들(기본값과 같은 줄은 `(default)`). 사람이 `src/tuning.rs`의 `SPECS` `def`에 옮겨 적고 커밋한다. **게임이 저장 파일을 읽지 않는다**(진실은 하나).
7. 시나리오 `1~7`(아레나에서)로 상황을 바꾼다.

옛 오버레이(게임이 돌며 바꾸기, 랩 전용 입력 아홉·터치 버튼 여덟)는 0070에서 걷었다. 쓰지 않던 값 셋(`heavy.damage`·`cast.damage` — 무기 표 0051이 대신, `render.model` — PoC 스파이크)도 지웠다.

## 값(`tuning.rs` `SPECS` 순서) — 랩의 "랩"들
| 랩 | 값 | 어울리는 시나리오 | 묻는 것 |
|---|---|---|---|
| Camera | quarter pitch·dist·fov·follow·**yaw** / lock dist·height·shoulder·fov·look ahead·**frame x**·yaw / blend / collide r | 1 보스, 3 기둥 | 전환이 튀지 않나, 멀미 없나, 플레이어가 1/3에 서나, 벽에서 당겨지나 |
| Movement | speed·accel·turn / dodge dist·time | 4 빈 방, 2 떼 | 떼 사이를 빠져나가는 손맛, 회피가 공짜가 아닌가 |
| Combat | attack startup·active·recovery·range·arc / hitstop·knockback | 2 떼, 1 보스 | 한 방이 묵직한가, 범위가 읽히나 |
| Lock-on | range·lose dist | 1 보스 | 언제 잡히고 언제 풀리나 |
| Enemy | mob speed·telegraph / boss telegraph·swing arc | 1·2 | 예고를 읽고 피할 수 있나 |
| Render | face shade / horizon haze | 2 떼, 5 쟁반 | 상자가 입체로 읽히나(0 = 민짜), 지평선 아지랑이가 돌 때 덜 어지럽게 하나(0 = 끔) |
| Feel (0018) | attack lunge / hurt stun·iframes·knock / stamina max·regen·delay·dodge·attack | 1 보스, 2 떼 | 내딛기가 손에 붙나, 맞으면 멈칫하나(답답하지 않게), 스태미나가 판단을 만드나 |
| Heavy (0030) | heavy hold start·charge·startup·active·recovery·damage·full bonus·range·arc·lunge·knock·stagger·boss stagger·stamina | 2 떼, 1 보스 | 꾹이 실수로 모이지 않나, 다 모은 강타가 시원한가, 꾹만 남지 않나 |
| Look (0033) | render toon·rim·outline px | 7 잡몹 80(비용), 1 보스(모양) | 외곽선·툰이 화풍에 맞나, 폰에서 외곽선 비용 |
| Flask (0031) | flask heal·raise·time·lock·walk | 2 떼, 1 보스 | 0.8초가 떼 속에서 쓸 만한가, 연타해도 한 모금씩인가 |
| Turn (0012·0029) | turn camera·overshoot·settle·snap kick·plant·cooldown / spill range·slide per m·max·outward·stagger / boss mass·stagger / slam dmg·stagger | 5 쟁반, 2 떼 | 돌릴 때 멀미 없나, 쏟아지는 게 시원한가, 남발되지 않나 |
| Camera(0007~0011) | lock yaw max·near slow·pull out·**look ahead**·**quarter lead**·**blend turn x**·**blend exit x**·move.hold after unlock·**quarter yaw mode**(0 고정·1 따라감·2 90° 단위) | 1 보스, 3 기둥 | 보스에 바짝 붙어 돌 때 멀미 vs 보스가 화면 밖으로 빠짐, 기둥에서 풀려 나오는 속도 |

## 기록
값을 정하면 결정 기록 한 줄(어느 날, 누가, A/B 무엇과 비교해 무엇을 골랐나). 판정은 사람. `cargo test`의 시험은 "규칙"(한 스윙 한 번, 락온 기준 즉시, 당기기 최소 거리)만 본다 — 손맛은 못 잰다.

## 터치
왼쪽 위 멈춤 → LAB → 같은 창(◀▶ 탭·끌기 굴리기, 창 밖 탭 = 닫기). 내보내기는 콘솔이 없는 폰에서는 못 본다 — 화면의 값을 알려 주면 된다.

## 아직 없는 것
- 성능 랩(적 수 vs 프레임 시간): P5에서 `mob count` 값과 함께.
- 프리셋을 파일로 저장(웹 localStorage) — 내보내기 글을 복사하는 것으로 충분한지 먼저 본다.
