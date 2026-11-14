//! 큰 창의 여정 탭(0183): 왼쪽 = 닿은 지역의 목표(표 차례 — 지역마다 갈 길 먼저), 오른쪽 = 고른 목표의 이름 · 갈래와 지역 ·
//! 상태 한 줄 · 풀이 · 받을 것. 이룬 것은 확인 = 받기(멈춤·화로 둘 다 — 받기는 사는 일이 아니다).
//! 줄 끝의 표지: ◆ = 받을 수 있다 · n/m = 하는 중 · ◇ = 받았다(흐리게).
use super::tabs::{Ln, PaneHead, HEAD_H};
use super::*;
use crate::journey::{self, Branch, Reward};
use flow::Host;

impl App {
    /// 여정 한 틱: 뒤로·✕·창 밖 = 닫기, 위아래·굴리기·줄 누르기 = 고르기, 확인(또는 고른 줄 다시 누르기) = 받기.
    pub(super) fn tick_journey(&mut self, a: &Actions, _host: Host) {
        let mut a = *a;
        let before = self.cursor;
        self.pane_scroll_lines(&mut a);
        let a = &a;
        let tap = a.tap.and_then(|p| self.hit_at(p));
        if a.back || a.pause || tap == Some(menu::Hit::Close) {
            self.go(Go::Back);
            return;
        }
        let goals = journey::visible(&self.progress);
        let picked = self.menu_pick(a, goals.len());
        if self.cursor != before {
            self.scroll = 0.0;
        }
        if let Some(i) = picked {
            let g = goals[i.min(goals.len() - 1)];
            if journey::ready(g, &self.progress) {
                self.take_goal(g);
            } else {
                self.snd(crate::sfx::Id::Deny);
            }
        }
    }

    /// 받기: 기록에 적고(다시 받지 않는다) 잔불은 지금 지닌 것에, 기술 점수는 기록의 셈이 받은 목표에서 센다. 바로 저장.
    fn take_goal(&mut self, g: &journey::Goal) {
        self.progress.journey.push(g.id.to_string());
        self.progress.journey.sort();
        if let Reward::Embers(n) = g.reward {
            self.world.embers += n;
            self.progress.embers = self.world.embers;
        }
        self.snd(crate::sfx::Id::Chest);
        self.toast = Some((fill(s().toast_journey_took, &[tr(g.title)]), TOAST_TIME));
        self.persist();
    }

    /// 여정의 알림(0183): 새로 이룬 목표가 있으면 "여정 — 이름" 한 줄. 기록을 처음 본 때(불러오기·새 판)는 이미 이룬 것을
    /// 조용히 적어 둔다 — 이어할 때마다 지난 일을 다시 알리지 않게.
    pub(super) fn journey_watch(&mut self) {
        let now: Vec<&'static str> = journey::visible(&self.progress)
            .into_iter()
            .filter(|g| journey::done(g, &self.progress))
            .map(|g| g.id)
            .collect();
        let Some(told) = self.journey_told.as_mut() else {
            self.journey_told = Some(now);
            return;
        };
        let fresh: Vec<&'static str> = now
            .iter()
            .copied()
            .filter(|id| !told.contains(id))
            .collect();
        told.extend(fresh.iter().copied());
        if let Some(g) = fresh
            .last()
            .and_then(|id| journey::GOALS.iter().find(|g| g.id == *id))
        {
            // 다른 알림(핵·숨은 물건)이 떠 있으면 그것을 지우지 않는다 — 여정 탭 머리의 점이 남는다.
            if self.toast.is_none() {
                self.toast = Some((fill(s().toast_journey, &[tr(g.title)]), TOAST_TIME));
            }
        }
    }

    /// 여정 탭 그림: 왼쪽 목표 줄, 오른쪽 고른 목표.
    pub(super) fn draw_journey(&mut self, f: &Frame, host: Host) {
        let s = s();
        let scene = Scene::Journey { host };
        let w = self.draw_tab_window(f, scene);
        let p = self.progress.clone();
        let goals = journey::visible(&p);
        let rows: Vec<ui::Row> = goals
            .iter()
            .map(|g| {
                let (have, need) = journey::count(g, &p);
                let (mark, dim) = if journey::claimed(g, &p) {
                    ("◇".to_string(), true)
                } else if journey::done(g, &p) {
                    ("◆".to_string(), false)
                } else {
                    (format!("{have}/{need}"), false)
                };
                let mut r = ui::Row::value(tr(g.title), mark).dimmed(dim);
                r.tag = true;
                r
            })
            .collect();
        self.draw_left_list(&w, &rows, 0.0);
        let cursor = self.cursor.min(goals.len().saturating_sub(1));
        let g = goals[cursor];
        let (have, need) = journey::count(g, &p);
        let (state, ink) = if journey::claimed(g, &p) {
            (s.journey_claimed.to_string(), DIM)
        } else if journey::done(g, &p) {
            (s.journey_ready.to_string(), ui::MINT)
        } else {
            (
                fill(s.journey_doing, &[&have.to_string(), &need.to_string()]),
                DIM,
            )
        };
        let branch = match g.kind {
            Branch::Main => s.journey_main,
            Branch::Find => s.journey_find,
        };
        let region = tr(crate::stage::STAGES[g.stage].name);
        self.draw_pane_head(
            &w,
            &PaneHead {
                name: tr(g.title),
                gold: journey::done(g, &p),
                kind: format!("{branch} · {region}"),
                state: (state, ink),
                icon: false,
            },
        );
        let reward = match g.reward {
            Reward::Embers(n) => fill(s.journey_embers, &[&n.to_string()]),
            Reward::Point => s.journey_point.to_string(),
        };
        let mut lines: Vec<Ln> = self.pane_wrap(&w, tr(g.hint), INK);
        lines.push(Ln::Gap);
        let col = if journey::claimed(g, &p) { DIM } else { AMBER };
        lines.extend(self.pane_wrap(&w, &fill(s.journey_reward, &[&reward]), col));
        self.draw_pane(&w, HEAD_H, &lines);
        let right = w.panel.x + w.panel.w;
        // 받을 수 있을 때만 "받기"(다른 줄에서 확인은 아무 일도 하지 않는다 — 안내도 하지 않는다).
        let mut pairs = vec![(Act::Walk, s.verb_choose)];
        if journey::ready(g, &p) {
            pairs.push((Act::Confirm, s.verb_take));
        }
        self.tab_foot(f, &w, scene, right, &pairs);
    }
}
