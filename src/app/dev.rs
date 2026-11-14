//! 개발 창(0083, nail 0269 `Scene::Dev` 구조 그대로): 위에 묶음 탭이 한 줄로 다 보이고(좌우·Q/E·탭), 왼쪽은 그 묶음의 줄(위아래·굴리기),
//! 오른쪽은 고른 줄의 이름·설명, 아래 안내 한 줄. 줄은 시작할 것만(돌아가기 줄·무기 줄 없음 — 뒤로 = 나가기, 무기는 장비 창).
//! 줄마다 묶음·시작할 것을 표 한 곳(`DEV_LIST`)에 두고 묶음별 목록은 거기서 거른다(nail `scenario::Entry { cat }`).
//! 개발판에만. 여기서 시작한 스테이지 판은 기록 슬롯에 쓰지 않는다(`App::dev_run`, 0075).
use super::menu::Hit;
use super::*;
use crate::i18n::dev::{self as dv, scenario_text, t};

/// 묶음(탭 순서 = 이 순서, 도구가 앞 — nail 0269).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum DevCat {
    Tools,
    Stage,
    Combat,
    Perf,
}

pub(super) const DEV_CATS: [DevCat; 4] =
    [DevCat::Tools, DevCat::Stage, DevCat::Combat, DevCat::Perf];

impl DevCat {
    fn name(self) -> &'static str {
        t(match self {
            DevCat::Tools => dv::TAB_TOOLS,
            DevCat::Stage => dv::TAB_STAGE,
            DevCat::Combat => dv::TAB_COMBAT,
            DevCat::Perf => dv::TAB_PERF,
        })
    }
}

/// 개발 창에서 시작할 것 하나.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum DevStart {
    /// 첫 스테이지 처음부터(입구) — 기록은 쓰지 않는다.
    Entrance,
    /// 문 앞(문지기 결투) — 기록은 쓰지 않는다.
    Gate,
    /// 둘째 지역 어귀(0118 — 이 판에 첫 문지기를 이긴 셈) — 기록은 쓰지 않는다.
    Village,
    /// 셋째 지역 어귀(0185 — 이 판에 두 문지기를 이긴 셈) — 기록은 쓰지 않는다.
    Garden,
    /// 둘째 지역의 첫 주민 곁(0122 — 말풍선·돌아봄 판정) — 기록은 쓰지 않는다.
    Residents,
    /// 둘째 지역의 새긴 돌 곁(0129 — 글 판 판정) — 기록은 쓰지 않는다.
    Stones,
    /// 둘째 지역의 첫 상자 곁(0131 — 떨어진 아이템·빛기둥 판정) — 기록은 쓰지 않는다.
    Chest,
    /// PoC·개발 아레나(`scenario::LIST`).
    Scenario(Scenario),
    /// 연구소(메뉴의 설정 아래 줄과 같은 창, 0079).
    Lab,
    /// 기술 화면(0151 → 0161 — 멈춤의 기술 줄과 같은 화면, 읽기·끼우기): 아레나에는 멈춤이 없어 여기서 끼운다.
    Skills,
    /// 기술 판을 모두 끝 단으로·패시브 칸을 모두 열림으로(0158·0161 — 켬/끔): 개발 판·아레나에서 단과 고르기를 시험한다. 기록에 쓰지 않는다.
    SkillsMax,
    /// 끝 단으로 올린 판의 3단 고르기를 모두 뒤집는다(ㄱ ↔ ㄴ). 기록에 쓰지 않는다.
    SkillsFlip,
}

/// 줄 표(묶음, 시작할 것) — 묶음 안 순서 = 표 순서. 전투·적 첫 줄 = 보스 아레나(시험 스크립트: 오른쪽 둘 · 확인).
pub(super) const DEV_LIST: [(DevCat, DevStart); 27] = [
    (DevCat::Tools, DevStart::Lab),
    (DevCat::Tools, DevStart::Scenario(Scenario::Empty)),
    (DevCat::Tools, DevStart::Skills),
    (DevCat::Tools, DevStart::SkillsMax),
    (DevCat::Tools, DevStart::SkillsFlip),
    (DevCat::Stage, DevStart::Entrance),
    (DevCat::Stage, DevStart::Gate),
    (DevCat::Stage, DevStart::Village),
    (DevCat::Stage, DevStart::Residents),
    (DevCat::Stage, DevStart::Stones),
    (DevCat::Stage, DevStart::Chest),
    (DevCat::Stage, DevStart::Garden),
    (DevCat::Combat, DevStart::Scenario(Scenario::Boss)),
    (DevCat::Combat, DevStart::Scenario(Scenario::Pillars)),
    (DevCat::Combat, DevStart::Scenario(Scenario::Duel)),
    (DevCat::Combat, DevStart::Scenario(Scenario::Elites)),
    (DevCat::Combat, DevStart::Scenario(Scenario::Newcomers)),
    (DevCat::Combat, DevStart::Scenario(Scenario::Bellkeeper)),
    (DevCat::Combat, DevStart::Scenario(Scenario::WardenTwo)),
    (DevCat::Combat, DevStart::Scenario(Scenario::BellkeeperTwo)),
    (DevCat::Combat, DevStart::Scenario(Scenario::Mossward)),
    (DevCat::Combat, DevStart::Scenario(Scenario::MosswardTwo)),
    (DevCat::Combat, DevStart::Scenario(Scenario::Horde)),
    (DevCat::Combat, DevStart::Scenario(Scenario::Skills)),
    (DevCat::Combat, DevStart::Scenario(Scenario::Tray)),
    (DevCat::Perf, DevStart::Scenario(Scenario::Stress40)),
    (DevCat::Perf, DevStart::Scenario(Scenario::Stress80)),
];

/// 묶음 `tab`의 줄.
pub(super) fn dev_items(tab: usize) -> Vec<DevStart> {
    let cat = DEV_CATS[tab.min(DEV_CATS.len() - 1)];
    DEV_LIST
        .iter()
        .filter(|(c, _)| *c == cat)
        .map(|(_, e)| *e)
        .collect()
}

impl DevStart {
    fn name(self) -> &'static str {
        match self {
            DevStart::Entrance => t(dv::ENTRANCE),
            DevStart::Gate => t(dv::GATE),
            DevStart::Village => t(dv::VILLAGE),
            DevStart::Garden => t(dv::GARDEN),
            DevStart::Residents => t(dv::RESIDENTS),
            DevStart::Stones => t(dv::STONES),
            DevStart::Chest => t(dv::CHEST),
            DevStart::Scenario(sc) => {
                let e = scenario::entry(sc);
                scenario_text(e.name, e.checks).0
            }
            DevStart::Lab => t(dv::LAB_START),
            DevStart::Skills => t(dv::SKILLS_START),
            DevStart::SkillsMax => t(dv::SKILLS_MAX),
            DevStart::SkillsFlip => t(dv::SKILLS_FLIP),
        }
    }

    fn note(self) -> String {
        match self {
            DevStart::Entrance
            | DevStart::Gate
            | DevStart::Village
            | DevStart::Garden
            | DevStart::Residents
            | DevStart::Stones
            | DevStart::Chest => t(dv::STAGE_NOTE).to_string(),
            DevStart::Scenario(sc) => {
                let n = scenario::LIST
                    .iter()
                    .position(|e| e.scenario == sc)
                    .unwrap_or(0)
                    + 1;
                let e = scenario::entry(sc);
                let checks = scenario_text(e.name, e.checks).1;
                fill(t(dv::SCENARIO_NOTE), &[&n.to_string(), checks])
            }
            DevStart::Lab => t(dv::LAB_NOTE).to_string(),
            DevStart::Skills => t(dv::SKILLS_NOTE).to_string(),
            // 지금 상태({0})는 앱이 채운다(`App::dev_note`).
            DevStart::SkillsMax => t(dv::SKILLS_MAX_NOTE).to_string(),
            DevStart::SkillsFlip => t(dv::SKILLS_FLIP_NOTE).to_string(),
        }
    }
}

/// 개발 창의 기술 판 줄을 고르면(0158): "다 올리기"는 켬/끔(켜면 끝 단 · 고르기 ㄱ), "고르기 바꾸기"는 끝 단 판의 고르기를
/// 모두 뒤집는다(꺼져 있으면 ㄴ으로 켠다). 순수 — 시험.
pub(super) fn dev_board_step(
    now: Option<crate::skills::Board>,
    e: DevStart,
) -> Option<crate::skills::Board> {
    use crate::skills::{Board, Pick};
    match (e, now) {
        (DevStart::SkillsMax, None) => Some(Board::maxed(Pick::A)),
        (DevStart::SkillsMax, Some(_)) => None,
        (DevStart::SkillsFlip, None) => Some(Board::maxed(Pick::B)),
        (DevStart::SkillsFlip, Some(b)) => Some(Board::maxed(b.pick[0].other())),
        (_, now) => now,
    }
}

impl App {
    /// 개발 창 한 틱: 뒤로·✕·창 밖 = 나가기, 좌우·스킬 버튼(Q/E, 0151)·탭 누르기 = 묶음, 위아래·굴리기 = 줄, 확인·줄 누르기 = 시작.
    pub(super) fn tick_dev(&mut self, a: &Actions, tab: usize) {
        let tap = a.tap.and_then(|p| self.hit_at(p));
        if a.back || a.pause || tap == Some(Hit::Close) {
            self.go(Go::Back);
            return;
        }
        let n = DEV_CATS.len();
        let step = (a.menu_right || a.tab_next) as i32 - (a.menu_left || a.tab_prev) as i32;
        let to = match tap {
            Some(Hit::Cell(1, i)) => Some(i),
            _ if step != 0 => Some((tab as i32 + step).rem_euclid(n as i32) as usize),
            _ => None,
        };
        if let Some(to) = to.filter(|&i| i != tab) {
            // 묶음은 장면 칸의 상태(0066): 지금 칸을 새 묶음으로 바꿔 연다(커서는 첫 줄).
            self.go(Go::Back);
            self.go(Go::Open(Scene::Dev { tab: to as u8 }));
            self.snd(crate::sfx::Id::UiMove);
            return;
        }
        let items = dev_items(tab);
        if let Some(i) = self.menu_pick(a, items.len()) {
            if let Some(&e) = items.get(i) {
                self.dev_start(e);
            }
        }
    }

    /// 개발 창에서 시작(0075): 스테이지 판은 기록 슬롯에 쓰지 않는다(`dev_run` — 슬롯을 고르면 풀린다).
    fn dev_start(&mut self, e: DevStart) {
        match e {
            DevStart::Entrance | DevStart::Gate => {
                self.dev_run = true;
                self.restart();
                if e == DevStart::Gate {
                    self.run.dev_skip_to_gate(&mut self.world);
                }
                self.cam = self.cam.renew(self.world.player.pos, &self.t);
                self.go(Go::Root(Scene::Play));
            }
            DevStart::Village => {
                self.dev_run = true;
                self.restart();
                // 이 판에 첫 문지기를 이긴 셈 — 내려가는 뿌리 굴(0174)부터 보인다.
                self.down = 1;
                self.passage = Some(crate::descent::Passage::new(
                    1,
                    false,
                    self.run.def.depth_m,
                    crate::stage::STAGES[1].depth_m,
                ));
                self.go(Go::Root(Scene::Play));
            }
            DevStart::Garden => {
                self.dev_run = true;
                self.restart();
                // 이 판에 두 문지기를 이긴 셈 — 마을에서 내려가는 뿌리 굴부터.
                self.down = 2;
                self.cross_to(1, false);
                self.passage = Some(crate::descent::Passage::new(
                    2,
                    false,
                    crate::stage::STAGES[1].depth_m,
                    crate::stage::STAGES[2].depth_m,
                ));
                self.go(Go::Root(Scene::Play));
            }
            DevStart::Residents => {
                self.dev_run = true;
                self.restart();
                self.down = 1;
                self.cross_to(1, false);
                if let Some(&(_, at)) = self.run.residents.first() {
                    self.world.player.pos = at + vec3(0.0, 0.0, 1.6);
                    self.world.player.prev = self.world.player.pos;
                    self.cam = self.cam.renew(self.world.player.pos, &self.t);
                }
                self.go(Go::Root(Scene::Play));
            }
            DevStart::Stones => {
                self.dev_run = true;
                self.restart();
                self.down = 1;
                self.cross_to(1, false);
                if let Some(&at) = self.run.stones.last() {
                    self.world.player.pos = at + vec3(0.0, 0.0, 1.4);
                    self.world.player.prev = self.world.player.pos;
                    self.cam = self.cam.renew(self.world.player.pos, &self.t);
                }
                self.go(Go::Root(Scene::Play));
            }
            DevStart::Chest => {
                self.dev_run = true;
                self.restart();
                self.down = 1;
                self.cross_to(1, false);
                if let Some(&at) = self.run.chests.first() {
                    self.world.player.pos = at + vec3(-1.2, 0.0, 0.0);
                    self.world.player.prev = self.world.player.pos;
                    self.cam = self.cam.renew(self.world.player.pos, &self.t);
                }
                self.go(Go::Root(Scene::Play));
            }
            DevStart::Scenario(sc) => {
                self.sc = sc;
                self.world = scenario::build(sc);
                self.fx.clear();
                self.world.set_weapon(self.weapon);
                self.cam = self.cam.renew(self.world.player.pos, &self.t);
                self.go(Go::Root(Scene::Sandbox));
                // 놀이터가 선 뒤에: 판(`App::board`)이 놀이터인지를 보고 처음의 둘을 배운 것으로 친다(0166).
                self.apply_board();
            }
            DevStart::Lab => self.go(Go::Open(Scene::Lab)),
            DevStart::Skills => self.show(Go::Open(Scene::Skills {
                host: flow::Host::Dev,
            })),
            // 기술 판(0158): 메모리의 덮어쓰기만 바꾼다 — 기록(`progress`)과 슬롯은 건드리지 않는다. 창은 그대로(설명 줄이 상태를 보인다).
            DevStart::SkillsMax | DevStart::SkillsFlip => {
                self.dev_board = dev_board_step(self.dev_board, e);
                self.apply_board();
                self.snd(crate::sfx::Id::Buy);
            }
        }
    }

    /// 고른 줄의 설명 — 기술 판 줄은 지금 상태를 채운다.
    fn dev_note(&self, e: DevStart) -> String {
        match e {
            DevStart::SkillsMax | DevStart::SkillsFlip => {
                let state = match self.dev_board {
                    None => dv::BOARD_SLOT,
                    Some(b) if b.pick[0] == crate::skills::Pick::A => dv::BOARD_MAX_A,
                    Some(_) => dv::BOARD_MAX_B,
                };
                fill(&e.note(), &[t(state)])
            }
            _ => e.note(),
        }
    }

    /// 개발 창 그림(nail `draw_dev`): 창 틀 · 묶음 탭 줄 · 왼쪽 줄 목록 · 오른쪽 이름·설명 · 아래 안내.
    pub(super) fn draw_dev_screen(&mut self, f: &Frame, tab: usize) {
        let tab = tab.min(DEV_CATS.len() - 1);
        self.ui.dim(0.45);
        let u = self.ui.u(1.0);
        let px = self.ui.px(1.0);
        let panel = self.panel_rect(f, 440.0, 226.0);
        let top = self.ui.panel(panel, Some(t(dv::DEV_TITLE)));
        let x = self.ui.close(panel);
        self.hits.push((x, Hit::Close));
        let (lx, rx_end) = (panel.x + u * 10.0, panel.x + panel.w - u * 10.0);
        // 묶음 탭: 한 줄에 다 — 고른 탭은 밝은 단추. 폭은 글자에 맞추고 남으면 고르게 나눈다.
        let tab_h = px * 1.6;
        let gap = u * 4.0;
        let widths: Vec<f32> = DEV_CATS
            .iter()
            .map(|c| self.ui.width(c.name(), 1.0) + u * 14.0)
            .collect();
        let room = rx_end - lx - gap * (DEV_CATS.len() - 1) as f32;
        let extra = ((room - widths.iter().sum::<f32>()) / DEV_CATS.len() as f32).max(0.0);
        let mut tx = lx;
        for (i, c) in DEV_CATS.iter().enumerate() {
            let r = Rect::new(tx, top, widths[i] + extra, tab_h);
            self.ui.button(r, c.name(), i == tab);
            self.hits.push((r, Hit::Cell(1, i)));
            tx += r.w + gap;
        }
        // 왼쪽 줄 목록 · 오른쪽 이름·설명(세로 선으로 가름).
        let body = top + tab_h + u * 8.0;
        let hint_y = panel.y + panel.h - u * 8.0 - px;
        let split = panel.x + panel.w * 0.45;
        draw_line(split, body, split, hint_y - u * 4.0, u.max(1.0), ui::BORDER);
        let items = dev_items(tab);
        let rows: Vec<ui::Row> = items.iter().map(|e| ui::Row::plain(e.name())).collect();
        let area = Rect::new(
            lx - u * 4.0,
            body,
            split - lx - u * 4.0,
            hint_y - u * 6.0 - body,
        );
        let cursor = self.cursor.min(items.len().saturating_sub(1));
        let hits = self.ui.list(&rows, cursor, &mut self.view, area);
        self.push_hits(&hits);
        if let Some(e) = items.get(cursor) {
            let dx = split + u * 10.0;
            let w = rx_end - dx;
            let name = self.ui.fit(e.name(), w, 1.0);
            self.ui.text(&name, dx, body, 1.0, AMBER);
            let line_h = px * 1.3;
            let fit = ((hint_y - u * 6.0 - body - px * 1.6) / line_h)
                .floor()
                .max(1.0) as usize;
            for (k, l) in ui::wrap(&self.ui, &self.dev_note(*e), w, 0.9)
                .iter()
                .take(fit)
                .enumerate()
            {
                self.ui
                    .text(l, dx, body + px * 1.6 + line_h * k as f32, 0.9, DIM);
            }
        }
        let hint = self.ui.fit(t(dv::DEV_HINT), panel.w - u * 20.0, 0.85);
        self.ui.text(&hint, lx, hint_y, 0.85, DIM);
        self.push_outside(panel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0075·0083: 시나리오는 모두 개발 창의 줄에 꼭 한 번, 묶음은 비지 않는다.
    #[test]
    fn every_scenario_sits_once_and_no_group_is_empty() {
        for e in scenario::LIST.iter() {
            let n = DEV_LIST
                .iter()
                .filter(|(_, d)| *d == DevStart::Scenario(e.scenario))
                .count();
            assert_eq!(n, 1, "{}", e.name);
        }
        for (i, c) in DEV_CATS.iter().enumerate() {
            assert!(!dev_items(i).is_empty(), "{c:?} is empty");
        }
        // 시험 스크립트(shot·mouse·leak): 전투·적(셋째 묶음) 첫 줄 = 보스 아레나, 성능(넷째) 첫 줄 = 부하 40.
        assert_eq!(dev_items(2)[0], DevStart::Scenario(Scenario::Boss));
        assert_eq!(dev_items(3)[0], DevStart::Scenario(Scenario::Stress40));
        // 도구 묶음: 기술 판 줄 둘은 끝에(앞 줄들의 자리는 그대로 — 헤드리스 길).
        assert_eq!(
            dev_items(0)[2..],
            [DevStart::Skills, DevStart::SkillsMax, DevStart::SkillsFlip]
        );
    }

    /// 0158: 개발 창의 "다 올리기"는 켬/끔, "고르기 바꾸기"는 끝 단 판의 고르기를 모두 뒤집는다(꺼져 있으면 ㄴ으로 켠다).
    /// 다른 줄은 판을 건드리지 않는다.
    #[test]
    fn the_dev_board_rows_toggle_max_ranks_and_flip_every_pick() {
        use crate::skills::{Board, Pick, MAX_RANK, ULT_MAX_RANK};
        let on = dev_board_step(None, DevStart::SkillsMax).expect("on");
        assert!(on.rank.iter().all(|&r| r == MAX_RANK) && on.ult == [ULT_MAX_RANK; 3]);
        assert!(on.pick.iter().all(|&p| p == Pick::A));
        assert_eq!(dev_board_step(Some(on), DevStart::SkillsMax), None, "off");
        let b = dev_board_step(Some(on), DevStart::SkillsFlip).expect("stays on");
        assert!(b.pick.iter().all(|&p| p == Pick::B));
        assert_eq!(dev_board_step(Some(b), DevStart::SkillsFlip), Some(on));
        assert_eq!(
            dev_board_step(None, DevStart::SkillsFlip),
            Some(Board::maxed(Pick::B))
        );
        assert_eq!(dev_board_step(Some(on), DevStart::Lab), Some(on));
    }
}
