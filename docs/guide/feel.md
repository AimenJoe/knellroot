# 손맛 — 타격 · 맞음 · 이펙트 · 소리

`CLAUDE.md`에서 옮긴 규칙이다(0278) — 문장은 그대로이고 같은 힘을 가진다. 고치는 곳: `src/fx.rs` · `src/fx/` · `src/sfx.rs` · `src/app/cue.rs`.
이 영역의 파일을 고치기 전에 읽고, 구조를 바꿨으면 여기를 고친다. 굵은 머리말에서 문단만 갈랐다.

## 타격 줌 · 휘두름

타격 줌·휘두름(0145): 센 타격(강타·자세 깨기·치명타·급소(0284)·패링·딱 맞춘 떼기)의 순간에만 `Camera::punch_in`(거리의 몫, 리그 자체를 옮긴다 — 그림·조준 광선·가림이 같은 눈) — 평타에는 넣지 않는다(작은 타격은 킥). 히트스톱 동안은 `Camera::freeze`로 머문다. 새 "큰 사건"은 `app/cue.rs` `cue`의 그 사건 갈래에 줌 한 줄(사건 → 소리 · 흔들림 · 킥 · 줌의 와일드카드 없는 표, 0278 R15). 휘두름의 초승달(`render::crescent`)은 두 겹(빛 띠 + 밝은 날) — 키울 때는 안쪽으로만(`slash_inner`), **바깥 변은 닿는 거리**(판정보다 크게 그리지 않는다). 날아가는 것도 같다(0277): 탄의 그림은 맞는 반지름(`BOLT_RADIUS` × 크기 배) 안 — 키울 때는 높이 · 밝기로(`enlarged_throws_still_draw_no_wider_than_they_hit`가 손도끼 · 땅 내려찍기 · 던진 방패를 본다). 쪽은 `weapons::SWEEPS`(동작마다 — 가로는 휘두른 쪽으로 사라지고, 세로·찌르기는 가운데가 밝고 양 끝부터 사라진다 `slash_arcs`, 0195), 동작 시계는 `weapons::clip_rate`로 타의 판정 시작에 맞춘다(들어 올림 `Hit::windup`이 있는 타 — 대검 — 는 `weapons::clip_at`: 판정 시작까지만 늘이고 그 뒤는 그대로의 빠르기, 0284)(남의 동작을 빌린 타 — 맨손·대시 — 도 닿는 순간에 판정, 표의 구간 = Blender `HIT_WINDOWS`, 시험).

## 맞음 피드백

맞음 피드백(0143): 방랑자가 맞은 틱에 다섯 겹 — 세계 멈춤(`HURT_HITSTOP`, 이어 맞으면 반씩)·화면 가장자리 붉은 테(`hud::draw_edge`, 랩 `hurt.edge`)·부서지는 체력 칸·흰 몸(`HURT_WHITE`)·밀려난 쪽 카메라 킥. 그림은 `Player::hurt`의 `since`(로직 시계 — 히트스톱 동안 0에 머문다)와 `blow`를 읽는다. 붉은색은 바닥 예고와 화면 테두리에만(월드 이펙트는 여전히 금지). 적이 방랑자에게 주는 새 피해 경로를 만들면 이 한 자리(`world/enemy_tick.rs`의 `wanderer_takes_hit` — 체력을 깎는 곳은 여기뿐)를 타게 한다 — 따로 체력만 깎으면 피드백이 빠진다.

## 이펙트

이펙트(0141)는 `fx.rs`(앱만 안다 — 소리와 같은 길): 월드 사건은 맞는 묶음(`Fx::on_strike`·`on_ail`·`on_skill_event`·`on_body` — `on_events`가 차례로 묻는다, 한 사건은 한 묶음에만: 시험 `every_event_belongs_to_at_most_one_group`, 0251 R11)의 match 한 줄 + 그 몸은 묶음의 파일 `fx/strike.rs`(적의 공격 꼴은 `blow_<꼴>`)·`fx/ail.rs`·`fx/skill_events.rs`의 `ev_<사건>`, 스킬마다의 판정은 `on_skill`의 한 줄 + `fx/skills.rs`의 `fx_<스킬>`(정체성 — 묶지 않는다, 0278 R25a). 낸 그림은 `fx/traces.rs`가 지킨다(`fx_traces` — 일부러 바꿨으면 `FX_WRITE=1 cargo test fx_traces`), 사건이 없는 몸짓(회피·착지·적의 돌진)은 `Fx::watch`가 상태 변화로. 월드에 그림을 넣지 않는다. 한 이펙트는 2~3색, 타격은 한가운데 번쩍임(`Shape::Star`, 0.15초 안) + 불똥 + 먼지의 겹, 꾸밈(먼지)만 `decor`(많으면 건너뛴다), 빨강은 쓰지 않는다(위험 예고 전용 — 시험). 빛 덩이는 정육면체가 아니라 카메라를 보는 마름모·별(`render::gem`·`spark4`). 적이 맞으면 피 대신 **뿌리 진액**(보라, 0182 — `Fx::sap`: 방울 `Shape::Drop`이 옆·위로 튀어 땅에 닿으면 `Fx::settle`이 얼룩 `Splat`으로 — 곧장 뒤로 튀면 맞은 몸에 가려 안 보인다). 새 이펙트는 개발 키 **F6**(한 프레임 = 한 틱)을 켜고 `tools/test/fx.js`로 틱마다 찍어 본다 — 실제 시간으로는 안 잡힌다.

## 소리

소리(0064 M5)는 `sfx.rs`(앱만 안다): 파일 표 `FILES`(`assets/sfx`, CC0 — 출처는 LICENSES) + 합성(`synth`·`ambience`, 순수 함수라 시험). 사건의 소리는 `app/cue.rs` `cue`의 갈래(0278 R15 — 흔들림 · 킥 · 줌과 한 줄, 고정 표 `cue_snapshot.txt`), 상태 변화(회피 시작·예고 시작·깨어남)는 앱 `world_sounds`가 듣고 낸다 — 월드에 소리를 넣지 않는다. 자주 나는 소리의 간격은 `sfx::same_gap`. 예고 소리는 모양마다 달라야 한다(시험).
