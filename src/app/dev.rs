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
    /// PoC·개발 아레나(`scenario::LIST`).
    Scenario(Scenario),
    /// 연구소(메뉴의 설정 아래 줄과 같은 창, 0079).
    Lab,
}

/// 줄 표(묶음, 시작할 것) — 묶음 안 순서 = 표 순서. 전투·적 첫 줄 = 보스 아레나(시험 스크립트: 오른쪽 둘 · 확인).
pub(super) const DEV_LIST: [(DevCat, DevStart); 12] = [
    (DevCat::Tools, DevStart::Lab),
    (DevCat::Tools, DevStart::Scenario(Scenario::Empty)),
    (DevCat::Stage, DevStart::Entrance),
    (DevCat::Stage, DevStart::Gate),
    (DevCat::Combat, DevStart::Scenario(Scenario::Boss)),
    (DevCat::Combat, DevStart::Scenario(Scenario::Pillars)),
    (DevCat::Combat, DevStart::Scenario(Scenario::Duel)),
    (DevCat::Combat, DevStart::Scenario(Scenario::Elites)),
    (DevCat::Combat, DevStart::Scenario(Scenario::Horde)),
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
            DevStart::Scenario(sc) => {
                let e = scenario::entry(sc);
                scenario_text(e.name, e.checks).0
            }
            DevStart::Lab => t(dv::LAB_START),
        }
    }

    fn note(self) -> String {
        match self {
            DevStart::Entrance | DevStart::Gate => t(dv::STAGE_NOTE).to_string(),
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
        }
    }
}

impl App {
    /// 개발 창 한 틱: 뒤로·✕·창 밖 = 나가기, 좌우·땅 돌리기(Q/E)·탭 누르기 = 묶음, 위아래·굴리기 = 줄, 확인·줄 누르기 = 시작.
    pub(super) fn tick_dev(&mut self, a: &Actions, tab: usize) {
        let tap = a.tap.and_then(|p| self.hit_at(p));
        if a.back || a.pause || tap == Some(Hit::Close) {
            self.go(Go::Back);
            return;
        }
        let n = DEV_CATS.len();
        let step = (a.menu_right || a.turn_right) as i32 - (a.menu_left || a.turn_left) as i32;
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
                self.cam = Camera::new(self.world.player.pos, &self.t);
                self.go(Go::Root(Scene::Play));
            }
            DevStart::Scenario(sc) => {
                self.sc = sc;
                self.world = scenario::build(sc);
                self.world.set_weapon(self.weapon);
                self.cam = Camera::new(self.world.player.pos, &self.t);
                self.go(Go::Root(Scene::Sandbox));
            }
            DevStart::Lab => self.go(Go::Open(Scene::Lab)),
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
            for (k, l) in ui::wrap(&self.ui, &e.note(), w, 0.9)
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
    }
}
