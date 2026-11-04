# 0024. 키 배치: 소울라이크식(공격만 우리식) + 다시 붙이기

- 날짜: 2026-11-03
- 상태: 채택·구현(사장: 소울라이크 배치 표를 보고 "너의 추천대로 소울라이크식으로 가자, 다만 공격만 우리식. 그 후 패링 시스템이랑 키맵핑 기능 넣어줘")

## 근거
- 소울라이크 기본(엘든 링 wiki.gg·Shacknews, 다크 소울 3 Fextralife — 2차, 서로 맞음): RB 공격·RT 강공격·LB 방어·LT 전투기, B 구르기(누르면 달리기), A 점프, X 아이템, Y 상호작용, R3 락온, 십자키 바꾸기. PC: 클릭 공격·오른쪽 클릭 방어·Space 회피·Q 락온·E 상호작용·R 아이템.
- 액션 로그라이트(Hades, KB controller-layout-and-remapping): 오른쪽 넷에 공격·대시.
- 쿼터뷰 떼싸움은 오른쪽 엄지 연타(공격)가 편하고, 락온 보스전은 소울라이크 손 모양이 익숙하다 → 사장 선택: 소울라이크 + 공격만 X/J.

## 기본 배치
| 동작 | 패드 | 키보드 | 바꿀 수 있나 |
|---|---|---|---|
| 이동 | 왼쪽 스틱·십자 | WASD·화살표 | 고정 |
| 공격 | X | J | ○ |
| 회피 | B | Space | ○ |
| 패링(0025) | LB | K | ○ |
| 락온 | R3 | L | ○ |
| 땅 돌리기 왼쪽·오른쪽 | LT·RT | Q·E | ○ |
| 조준 | 오른쪽 스틱 | — | 고정 |
| 멈춤 | Start | Esc·P | 고정 |
| 메뉴 확인·뒤로 | A·B | Enter(Space·J)·Esc | 고정 |
- 엄지를 스틱에서 떼지 않고 돌릴 수 있게 땅 돌리기는 트리거(걷다 돌리고 곧장 직진 — 0023). RB·RT 강공격 자리와 Y·X(아이템)는 비워 둔다. → 0030: 강공격은 공격 꾹, RB는 등잔.
- 개발판 랩이 열려 있으면 어깨·트리거는 랩 단계(LB 10배·LT 담기·RT 바꾸기)라 패링·돌리기로 읽지 않는다.

## 다시 붙이기(KB controller-layout-and-remapping, nail 0022의 틀)
- 배치표 `input::Bindings` = 동작 여섯 → (키 하나 `KEYS`의 자리, 패드 버튼 하나 `Pad`). 겹치면 맞바꾼다. 화면 안내(`input::prompt`)는 지금 장치·지금 배치에서 읽는다.
- 화면: 타이틀·멈춤의 "조작 설정"(`Scene::Controls`, 흐름 `app/controls.rs`). 줄 = 동작 여섯 + 기본값으로 + 돌아가기, 칸 = 키보드|패드(좌우), 확인 → "누르세요… 5", 그 장치의 다음 눌림을 붙인다. 취소: 키보드 칸 Esc, 패드 칸 Start, 5초. 확인을 누른 그 틱의 눌림은 잡지 않는다(Space·A는 확인이면서 붙일 수 있는 키).
- 저장: `prefs.rs` — 웹 localStorage 정수 칸(`knell_pref_get/set`, web/index.html), 데스크톱·Android 기록 폴더 `prefs.txt`(Android는 `/data/data/<패키지>/files`, 개발판 `.dev`도 프로세스 이름에서). 언어도 같이 저장한다(전엔 매번 한국어로 시작). 칸이 깨지면 기본값.
- 마우스·Android 키보드는 아직 안 받는다(필요하면 다시 붙이기 표에 칸을 더한다).
- 시험: `confirm_waits_for_the_next_key_and_swaps_on_conflict`, `the_pad_column_takes_a_pad_button_and_start_cancels`, `waiting_times_out_and_reset_and_back_rows_work`, `prefs_round_trip_and_unknown_storage_reads_as_defaults`, `the_native_prefs_file_round_trips`.
