//! 전투 스킬 표(0150 제안 → 0151 결정, S0). 스킬 = 방랑자가 되찾은 **몸의 기술**(무기와 상관없다 — 무기의 스페셜은 RB, `weapons.rs`).
//! 줄 = (다시 쓰기까지·시동·후딜·모양·피해). 스킬은 기력을 쓰지 않고 저마다의 대기로 돈다 — 스킬이 아닌 것으로 적을 쓰러뜨리면
//! 모든 대기가 당겨진다("기세", `skill.momentum s`). 큰 기술은 대기가 아니라 게이지(평타·패링으로만 찬다).
//!
//! 코드에 스킬별 숫자를 박지 않는다 — 모양·수치는 이 표, 전체 배율·기세·게이지는 랩(`skill.*`).
//! 새 스킬 = `Id` 한 줄 + `SKILLS` 한 줄 + (새 모양이면 `Form` 한 갈래 + `world/skill_tick.rs`의 한 갈래) + 터치 그림 + 글 + 시험.
//!
//! **기술 판(0158 G0·G1)**: 스킬은 기술 점수로 배우고 올린다 — 다섯 단(2단 피해 · 3단 둘 가운데 하나 `Twist` · 4단 제 치수 ·
//! 5단 맺음 `Cap`), 큰 기술은 세 단. 단의 숫자는 아래 `RANK_*`, 스킬마다의 고르기·맺음은 표의 `picks`·`cap`, 줄은 `row`
//! (쓴 점수 합 `ROW_NEEDS`로 열린다). 판의 셈(`Board` — 점수·사기·되돌리기)은 순수하고, 월드는 끼운 둘의 `Loadout`만 받는다
//! (`Skill::at(단, 고르기)` = 그 단의 실제 값 `Eff`). 새 단 효과 = `Twist`/`Cap` 한 갈래 + 표의 값 + `skill_tick.rs`의 한 줄 + 글 + 시험.
//!
//! **G2(0158)**: 스킬 여덟(버팀 `Brace` · 되울림 `Echo` · 던지기 `Throw` · 잔불 흩기 `Field`의 네 모양이 더해졌다), 큰 기술 셋
//! (`Ult` — 쟁반 뒤집기는 처음부터, 몰아치기·한 줄기는 판에서 2점으로 배운다. **끼우는 큰 기술은 하나**). 전용 동작은 `Clip`
//! (이름 + 시동·이어짐·후딜 구간의 길이 — 구간의 경계가 판정의 순간이라 `Clip::at`이 단·고르기가 바꾼 길이를 구간에 맞춘다).
//!
//! **나무 하나(0161)**: 패시브 15칸(`progression::Cell`)도 이 판의 칸이다(`Entry::Cell` — 칸마다 1점, 잠금은 줄뿐). 스킬·큰 기술은
//! 갈래 셋(흐름·맞받기·자리 — `Skill::branch`·`Ult::branch`)에 서고, 판의 칸 순서 `ENTRIES`가 곧 화면의 차례다. 기술 점수의 샘은
//! 총 레벨 + 처음 쓰러뜨린 엘리트 + 핵 + 종소리 조각(`points`) — 화폐 "마디"는 없다.
//!
//! 파일(0278 R25b — 옛 `skills.rs` 5,504줄을 나눔, 겉은 그대로 `skills::…`): `hands`(쥔 꼴의 비트) · `form`(모양과 성질) ·
//! `clip`(전용 동작) · `table`(`Id` · `Skill` · `SKILLS`) · `ranks`(단 · 고르기 · 맺음 · `Skill::at`) · `ult`(궁극기) ·
//! `loadout`(끼운 칸 · 월드에 넘기는 것) · `entries`(트리의 칸과 줄) · `board`(트리의 셈).

mod board;
mod clip;
mod entries;
mod form;
mod hands;
mod loadout;
mod ranks;
mod table;
mod ult;

pub use board::*;
pub use clip::*;
pub use entries::*;
pub use form::*;
pub use hands::*;
pub use loadout::*;
pub use ranks::*;
pub use table::*;
pub use ult::*;
