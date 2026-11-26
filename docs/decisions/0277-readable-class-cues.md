# 0277 — 클래스의 결이 화면 · 귀에 읽히게(급소 · 분노 · 걸어 당김 · 작은 것 키우기 · 쌍단검 그림 · 터치 바꾸기 버튼)

- 날짜: 2026-11-26
- 상태: 채택 · 만듦(2026-11-26, `release/v0.9`)
- 까닭: 판정 체크리스트 120-1 · 121-3 · 121-4 · 122-6 · 123-4 · 123-5 · 124-2가 같은 것을 물었다 — 규칙은 있는데 **보이지 · 들리지 않는다**
  (급소는 "피해 수가 커질 뿐", 분노는 "동작 빠르기뿐", 끌기는 "넉백 대신 미끄러짐", 날아가는 것 · 왼손 날은 "쿼터뷰에서 몇 화소"). 사장이 Claude에게
  맡긴 작은 것들 — 수치(받는 피해 · 패링 판정 시간 · 회피 · 균형의 수)는 하나도 바꾸지 않는다.

## 원칙
1. 월드는 그림 · 소리를 모른다(0141) — 새로 읽혀야 하는 순간은 `Event` 하나로 내고, 이펙트(`fx.rs`)와 소리(`sfx.rs`)는 앱이 그 사건을 듣고 낸다.
   상태가 이어지는 것(분노 · 끌리는 중)은 렌더가 월드를 읽어 그린다.
2. 한 이펙트는 2~3색, 빨강 없음(예고 전용 — 시험 `no_effect_is_red_and_a_parry_reads_cool`), 한 사건은 한 묶음에만(`every_event_belongs_to_at_most_one_group`).
3. **그림이 판정보다 크지 않다**(0145 "바깥 변은 닿는 거리"): 키우는 것은 맞는 반지름(탄은 `BOLT_RADIUS` × 크기 배 — 적의 몸 겉에서 잰 거리) 안에서만.
   폭이 이미 차면 **높이 · 밝기**로 키운다.

## 만든 것

### 1. 급소(쌍단검 — `Art::sneak` · 그림자 걸음)
- 손의 덤(`Knack`)을 싣는 방랑자의 타는 모두 `world::wound` 한 곳을 지난다(= `hurt_by` + 급소가 들었으면 `Event::Sneak { at }`). 급소의 셈은
  `sneak_lands`(등 뒤 · 드러난 몸 `sneak_open`, 또는 그림자 걸음 `sure`) — `hurt_by`의 급소 갈래와 같은 함수라 피해와 표지가 어긋나지 않는다.
  타입이 스스로 낸 피해(`Knack::NONE` — 새는 피 · 화상)는 `hurt_by` 그대로.
- 그림(`Fx::on_strike`): 서늘한 은빛(연보라 기운 — `SNEAK`) 큰 번쩍임(평타의 따뜻한 흰 별보다 크다, 0.13초) + 맞은 쪽에 직각으로 **엇갈리는 빛줄 넷**(X — 날이 지나간
  자국) + 천천히 떠오르는 은빛 반짝임 여섯(0.45초 — 별이 사라진 뒤에도 "급소였다"가 몇 프레임 남는다). 치명타(금 · 사방의 알갱이) · 패링(하늘빛)과 갈린다.
- 소리 `sfx::Id::Sneak`(합성): 2,640 · 3,990 · 5,310Hz의 짧은 '칭'(0.18초) + 처음 2ms 날 긁힘 — 패링(1,320Hz · 0.5초)보다 높고 짧다. 화면 흔들림은 맞힌 타만큼 한 번 더.

### 2. 분노(대검 — `Art::fury`)
- 가득 찬 순간: `World::watch_fury`(틱 들머리)가 분노가 아래에서 1로 넘어선 틱에 `Event::FuryFull` 하나 — `Player::beat.fury_full`이 걸쇠,
  **`weapons::FURY_REARM` 0.7 밑으로 내려가야 다시** 울린다(넷이 셋이 되는 0.75로는 다시 울리지 않는다 — 끈 가장자리를 드나드는 적에 되풀이되지 않게).
  그림: 발밑에서 밖으로 번지는 주황 고리 + 머리 위로 솟는 불씨 빛줄. 소리 `Id::Fury`: 두 번 거른 숨이 부풀고 밑에서 낮은 웅(65 → 95Hz)이 오른다(0.55초).
- 분노가 있는 동안(반 = 적 둘부터): 렌더 `overlays::fury_glow` — 가슴께 숨 쉬는 주황 빛무리 + 발밑의 옅은 주황 고리(분노만큼 짙고, 가득이면 숨이 빨라진다),
  이펙트 `Fx::watch` — 어깨 높이에서 솟는 불씨(분노만큼 잦다: 가득이면 1초에 스물 남짓). 주황은 빨강이 아니다(초록 0.5 넘게).

### 3. 걸어 당김(낫 — 셋째 타 · 강공격 `Hit::pull`·`Heavy::pull` · 스킬 걸어 당기기)
- 끌기가 거리를 만들면(`MIN_TUG` 0.05 m 넘게 — 이미 몸 앞이면 조용하다) `world::tug`가 그 적에 `Enemy::tug = weapons::PULL_SHOW`(0.3초 — 넉백과 같은
  길이라 미끄러짐의 95%가 끝나는 때)를 걸고 `Event::Pull`. 문지기는 끌리지 않으니 덩굴도 없다. `tug`는 그림만 읽는다(로직 없음).
- 그림: 렌더 `overlays::pull_vines` — 방랑자의 손과 끌리는 적의 몸통을 잇는 **덩굴**(흙빛 뿌리 줄 + 연한 새순의 속, 가운데가 처지며 꿈틀대고 마디에 새순 둘,
  남은 몫만큼 옅어진다) + 이펙트(적의 발밑에서 방랑자 쪽으로 쓸리는 흙 · 새순 부스러기). 소리 `Id::Pull`: 1,100 → 260Hz로 떨어지는 휘익 + 둔한 덜컥.

### 4. 작은 것 키우기(그림만 — 판정은 그대로)
| 것 | 옛 | 새 | 판정과 |
|---|---|---|---|
| 던지는 손도끼(한손검 RB — `overlays::AXE_PARTS`) | 자루 0.58 m × 0.06, 날 0.18 × 0.23 — 도는 몸 끝 0.37 m(판정 0.35 밖) | 굵은 자루 0.46 × 0.07, 넓은 날 0.20 × 0.28, 쇳빛 빛무리 0.34 m — 도는 몸 끝 0.344 m | `BOLT_RADIUS` × 크기 배 안(시험) |
| 던진 방패(0276 — `actors::SHIELD_SCALE`) | 모델 × 1.1(지름 0.4 m) | × 1.75(반지름 0.33 m) | 모델의 방패 부위 경계 구 × 배 ≤ 0.35(시험이 GLB를 읽는다) |
| 쌍단검의 짝(`w_daggers` · `w_dagger_l` — `wanderer.py` `DAGGER_K`) | 날 0.33 m | × 1.3(날 0.43 m, 손잡이 · 코등이까지 같은 꼴) | 닿는 거리 1.44 m보다 한참 짧다 |
| 땅 내려찍기(대검 RB — `overlays::quake_wave`) | 앞머리 반 폭 0.75 m × 크기 배 — 판정의 **두 배**(맞지 않은 적 위로도 지나갔다) | 반 폭 0.33(판정 안) · 돌 조각 0.13(옆 0.2) — 대신 앞머리에서 0.85 m 솟는 흙 기둥 · 높이 튀는 돌 · 굵은 흰 속 | 옆으로 ≤ 0.35 × 크기 배(시험) |

쌍단검의 짝은 Blender로 다시 구웠다(`blender -b -P tools/art/blender/wanderer_skin.py` — 바뀐 것은 두 날의 정점 자리 · 법선뿐, 정점 5,676 · 삼각형 4,995 그대로,
`check_glb` 문제 0, **0 크레딧**). 왼손 날만 키우면 짝이 어긋나 둘 다 같은 배로 했다.

### 5. 쌍단검의 아이템 그림
`item_view::View::twin` — 쌍단검만 같은 부위를 눕힌 각만 거울로(`-roll`) 한 번 더 그려 **두 날이 가운데에서 엇갈린다**(`render/items.rs`, 경계 구는 같다 · 실루엣 그림도).
날 하나로는 한손검 · 완드와 갈리지 않았다. 터치 공격 버튼의 13×13 그림(`touch::Kind::Daggers`)은 이미 엇갈린 두 날이라 그대로(12칸 시험 그대로).

### 6. 터치 바꾸기 버튼
교체 공격의 창(`World::swap_strikes` — HUD 보조무기 칸의 금빛 테와 같은 때)이 열린 동안 `touch::Kind::Swap` 버튼이 락온 중의 락온 버튼처럼 **금빛**(채움 · 테 · 그림 —
`touch::lit`). 앱이 `touch::View::swap_strike`로 넘긴다(보조무기가 있을 때만).

## 시험
- 월드: `a_sneak_hit_says_so_and_a_plain_hit_does_not`(daggers) · `fury_full_rings_once_per_crossing`(greatsword) · `the_third_cut_drags_a_mob_in_but_not_a_warden`에 덩굴 · 사건(scythe).
- 이펙트: `a_sneak_hit_glints_cold_and_crossed_apart_from_a_plain_hit` · `fury_embers_rise_only_while_the_greatsword_is_furious` · 빨강 없음 · 한 묶음 시험에 새 사건 셋.
- 소리: `the_sneak_fury_and_pull_cues_sound_apart`(+ 모든 합성음의 봉우리 · 조용한 끝).
- 그림: `enlarged_throws_still_draw_no_wider_than_they_hit`(render) · `only_the_daggers_draw_as_a_crossed_pair`(item_view) · `the_swap_button_lights_gold_only_while_a_swap_strike_is_open`(touch).
- 헤드리스: `tools/test/fx.js … pull`(엘리트 아레나의 굳은 잔재 — 새 갈래, 한 줄의 잔재는 셋째 타 전에 쓰러져 끌 것이 남지 않았다) · `skills reap`(기다림 1.0 → 2.4초 —
  떼가 걸어 당기기 3.2 m 안에 들게) · `bare daggers,greatsword,shield` · `rb axe,quake` · `swap.js icon,touch`(쌍단검을 멘 기록의 장비 탭 · 터치 장치의 교체 공격).

## 확인
`cargo test` 998 통과(무시 20), fmt, clippy 다섯 대상(wasm 기본 · `--no-default-features` · Android · Windows · iOS — macOS 대상은 이 서버에 없어 건너뜀),
wasm 릴리스 빌드, 헤드리스 CI 넷(`shot` · `touch` · `pwa` · `save`) 통과, 위 헤드리스로 그림을 눈으로 봤다(급소의 은빛 별 · 분노의 주황 고리 · 빛무리 · 불씨 ·
걸어 당기기의 덩굴 · 손도끼 · 흙 기둥 · 던진 방패 · 엇갈린 두 날 · 금빛 바꾸기 버튼 → 창이 닫히면 흰 테).
