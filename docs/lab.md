# Lab — 실행 중에 수치를 바꾸고, 느끼고, `tuning.rs`로 가져가기

KB `guides/poc-and-lab-before-content.md`의 구현. 첫 게임은 수치 한 곳(`tuning.rs`) + 시나리오 진입점까지였고 값 하나에 빌드 한 번이었다. 여기서는 **빌드 없이** 바꾼다.

## 흐름
1. `F1`(Select)로 랩을 연다. 게임은 계속 돈다(이동·공격·회피·락온은 그대로 들어간다).
2. ↑↓로 값을 고르고 ←→로 바꾼다(Ctrl/LB = 10배). 바로 반영된다. `R`(Y)은 그 값만 기본값으로.
3. `C`(LT)로 지금 값들을 **슬롯 B**에 담고, `V`(RT)로 A/B를 번갈아 느낀다. "둘 중 어느 쪽"이 "이 값이 맞나"보다 답이 쉽다.
4. `E`(B)로 내보내기 → 콘솔에 `pub const NAME: f32 = 값;` 줄들(기본값과 같은 줄은 `(default)` 표시). 사람이 `src/tuning.rs`의 `SPECS` `def`에 옮겨 적고 커밋한다. **게임이 저장 파일을 읽지 않는다**(진실은 하나).
5. 시나리오 `1~4`(Start = 다음)로 상황을 바꾼다. 랩마다 어울리는 시나리오가 있다.

## 값(`tuning.rs` `SPECS` 순서) — 랩의 "랩"들
| 랩 | 값 | 어울리는 시나리오 | 묻는 것 |
|---|---|---|---|
| Camera | quarter pitch·dist·fov·follow·**yaw** / lock dist·height·shoulder·fov·look ahead·**frame x**·yaw / blend / collide r | 1 보스, 3 기둥 | 전환이 튀지 않나, 멀미 없나, 플레이어가 1/3에 서나, 벽에서 당겨지나 |
| Movement | speed·accel·turn / dodge dist·time | 4 빈 방, 2 떼 | 떼 사이를 빠져나가는 손맛, 회피가 공짜가 아닌가 |
| Combat | attack startup·active·recovery·range·arc / hitstop·knockback | 2 떼, 1 보스 | 한 방이 묵직한가, 범위가 읽히나 |
| Lock-on | range·lose dist | 1 보스 | 언제 잡히고 언제 풀리나 |
| Enemy | mob speed·telegraph / boss telegraph·swing arc | 1·2 | 예고를 읽고 피할 수 있나 |
| Render | face shade | 2 떼 | 상자가 입체로 읽히나(0 = 민짜) |
| Camera(0007·0008) | lock yaw max·near slow·pull out·**look ahead**·**quarter lead** | 1 보스, 3 기둥 | 보스에 바짝 붙어 돌 때 멀미 vs 보스가 화면 밖으로 빠짐, 기둥에서 풀려 나오는 속도 |

## 기록
값을 정하면 결정 기록 한 줄(어느 날, 누가, A/B 무엇과 비교해 무엇을 골랐나). 판정은 사람. `cargo test`의 시험은 "규칙"(한 스윙 한 번, 락온 기준 즉시, 당기기 최소 거리)만 본다 — 손맛은 못 잰다.

## 터치(0006)
폰에서는 오른쪽 위 LAB → HUD 아래 한 줄 ▲ ▼ − + R B A/B(−+는 누르고 있으면 반복). 고른 값 하나만 위에 보인다. 내보내기는 없다(폰에 콘솔이 없다) — 화면의 값을 알려 주면 된다.

## 아직 없는 것
- 성능 랩(적 수 vs 프레임 시간): P5에서 `mob count` 값과 함께.
- 프리셋을 파일로 저장(웹 localStorage) — 내보내기 글을 복사하는 것으로 충분한지 먼저 본다.
