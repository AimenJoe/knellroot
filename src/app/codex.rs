//! 큰 창의 도감 탭(0130 C5 — 사장 2026-11-09 인터뷰: 깨어난 것을 가까이 본 때 적힘, 본 만큼 관찰 줄이 열림, 모델 + 실루엣 → 0164 탭).
//! 왼쪽 = 묶음 머리(적 · 장신구 — 좌우·누르기; 위 탭과 버튼이 다르다) + 목록(위아래·굴리기), 오른쪽 = 고른 것의 그림과 글.
//! 적: 본 것 = 돌아가는 모델 + 이름 + 관찰(`·`)·추측(`?`) 줄, 못 본 것 = 검은 실루엣 + "???" + 나오는 곳.
//! 장신구: 가진 것 = 돌아가는 그림 + 이름 + 수치 + 세계 안의 말(0129), 못 가진 것 = "???" + 나오는 곳.
use super::menu::Hit;
use super::tabs::{Ln, PaneHead};
use super::*;
use crate::codex::{self, ENTRIES};
use crate::i18n::Tr;
use flow::Host;

/// 몸 안의 묶음 다섯(여정 · 적 · 장신구 · 덧옷 · 무기 — 무기는 0191, 사장 "도감에 무기가 빠졌음", 여정은 0261 탭에서 옮겨 옴,
/// 0265부터 맨 앞: 사장 2026-11-21 "기록 메뉴의 여정이 가장 먼저 보여야").
const GROUPS: usize = 5;
/// 묶음의 자리.
pub(super) const JOURNEY: usize = 0;
const FOES: usize = 1;
const CHARMS: usize = 2;
const GARB: usize = 3;
const WEAPONS: usize = 4;
/// 묶음 머리 줄의 높이(기준 px) — 그 아래부터 목록.
const GROUP_H: f32 = 18.0;
/// 오른쪽 칸의 그림 자리(기준 px): 적은 이름 아래 오른쪽 칸 폭 전체(2026-11-19 사장 "나오는 곳 설명은 다음 줄로, 모델은 가로로 더
/// 채워지게" — 전엔 왼쪽 52 폭에 세우고 나오는 곳을 그 오른쪽에 썼다), 나오는 곳·관찰은 그 아래로 흐른다.
const PIC_TOP: f32 = 17.0;
const PIC_H: f32 = 100.0;
/// 도감 묶음 머리 한 칸의 가장 좁은 넓이(0242, 기준 px).
const GROUP_MIN_W: f32 = 44.0;
/// 물건 그림 칸(0242 — 머리 곁 30 → 이름 아래 64, 기준 px).
const ITEM_PIC: f32 = 96.0;

/// 도감 그림을 손으로 보기(2026-11-19, 사장 "도감의 아이템 및 생물들도 잘 안 보임. 드래그로 살펴볼 수 있어야 함"):
/// 그림 위를 가로로 끌면(패드는 오른쪽 스틱) 돈다. 고른 줄이 바뀌면 처음(저절로 흔들기)으로. 크게 보기는 걷었다(사장 "확대 필요없고").
#[derive(Default)]
pub(super) struct CodexView {
    /// 손으로 돌린 각(없으면 저절로 흔든다).
    yaw: Option<f32>,
    /// 지난 프레임에 그림을 그린 자리(끌기를 받는 곳) — 볼 그림이 없으면(못 가진 물건) 없다.
    rect: Option<Rect>,
}

impl App {
    /// 도감 한 틱: 뒤로·✕·창 밖 = 닫기, 좌우·묶음 머리 누르기 = 묶음(적 ↔ 장신구 — 위 탭은 스킬 버튼이 넘긴다),
    /// 위아래·굴리기·줄 누르기 = 고르기(고름은 보기만 — 할 일 없음).
    pub(super) fn tick_codex(&mut self, a: &Actions, host: Host, tab: usize) {
        let mut a = self.small_strip_input(a);
        let before = self.cursor;
        let u = self.ui.u(1.0);
        tabs::turn_by_hand(&mut a, self.codex_view.rect, &mut self.codex_view.yaw, u);
        self.pane_scroll_lines(&mut a);
        let a = &a;
        let tap = a.tap.and_then(|p| self.hit_at(p));
        if a.back || a.pause || tap == Some(Hit::Close) {
            self.go(Go::Back);
            return;
        }
        let step = a.menu_right as i32 - a.menu_left as i32;
        let to = match tap {
            Some(Hit::Cell(1, i)) => Some(i),
            _ if step != 0 => Some((tab as i32 + step).rem_euclid(GROUPS as i32) as usize),
            _ => None,
        };
        if let Some(to) = to.filter(|&i| i != tab) {
            // 묶음은 장면 칸의 상태(0066) — 지금 칸을 새 묶음으로 바꾼다(커서는 첫 줄).
            self.go(Go::Swap(Scene::Codex {
                host,
                tab: to as u8,
            }));
            self.snd(crate::sfx::Id::UiMove);
            return;
        }
        if tab == JOURNEY {
            self.journey_pick(a, before);
            return;
        }
        let n = self.codex_len(tab);
        let picked = self.menu_pick(a, n);
        if self.cursor != before {
            self.scroll = 0.0;
            self.codex_view = CodexView::default();
        }
        // 0265: 적힌·얻은 칸의 보상은 확인 = 받기(여정과 같다 — 멈춤에서도). 받을 것이 없는 줄의 확인은 아무 일도 없다(보기만).
        if let Some(i) = picked {
            let b = self.codex_bounty(tab, i.min(n.saturating_sub(1)));
            if b.ready(&self.progress) {
                self.take_bounty(&b);
            }
        }
    }

    fn codex_len(&self, tab: usize) -> usize {
        group_len(&self.progress, tab)
    }

    fn codex_bounty(&self, tab: usize, i: usize) -> Bounty {
        bounty_at(&self.progress, tab, i)
    }

    /// 기록 탭에 들어올 때(다른 탭·멈춤에서 — 기록 탭 안의 묶음 넘기기는 아니다) 받을 것이 있는 첫 묶음에서 연다(2026-11-23 사장
    /// "기록에는 새로운 게 있다는 인디케이터가 있지만 들어가 보면 막상 받을 보상이 없는 경우" — 받을 것이 장신구·적 묶음에 있어도 늘
    /// 여정 묶음에서 열려 빈손으로 보였다). 받을 것이 없으면 여정.
    pub(super) fn codex_entry(&self, go: Go) -> Go {
        let into = |s: Scene| match s {
            Scene::Codex { host, tab: 0 } if !matches!(self.scene(), Scene::Codex { .. }) => {
                Scene::Codex {
                    host,
                    tab: self.codex_start() as u8,
                }
            }
            s => s,
        };
        match go {
            Go::Open(s) => Go::Open(into(s)),
            Go::Swap(s) => Go::Swap(into(s)),
            g => g,
        }
    }

    fn codex_start(&self) -> usize {
        start_group(&self.progress)
    }

    /// 받을 도감 칸이 하나라도 있나(기록 탭 머리의 민트 점, 묶음 머리의 민트 글).
    pub(super) fn codex_ready(&self) -> bool {
        any_ready(&self.progress)
    }

    fn codex_group_ready(&self, tab: usize) -> bool {
        group_ready(&self.progress, tab)
    }

    /// 받기: 기록에 적고(다시 받지 않는다) 잔불은 지금 지닌 것에. 바로 저장.
    fn take_bounty(&mut self, b: &Bounty) {
        self.progress.codex_paid.push(b.key.clone());
        self.progress.codex_paid.sort();
        self.world.embers += b.embers;
        self.progress.embers = self.world.embers;
        self.snd(crate::sfx::Id::Chest);
        self.toast = Some((
            fill(s().toast_codex_took, &[tr(b.name), &b.embers.to_string()]),
            TOAST_TIME,
        ));
        self.persist();
    }

    /// 도감 탭 그림(0164): 왼쪽 = 묶음 머리(◀ 적 · 장신구 ▶) + 목록, 오른쪽 = 고른 것의 그림 · 이름 · 글.
    pub(super) fn draw_codex(&mut self, f: &Frame, host: Host, tab: usize) {
        let s = s();
        let scene = Scene::Codex {
            host,
            tab: tab as u8,
        };
        let w = self.draw_tab_window(f, scene);
        let (k, u) = (w.k, w.s);
        // 묶음 머리: 위 탭과 다른 꼴 — 밑줄 하나짜리 작은 글 둘, 양 끝에 ◀ ▶(좌우로 넘긴다).
        let names = [
            s.menu_journey,
            s.codex_tab_foes,
            s.codex_tab_charms,
            s.codex_tab_garb,
            s.codex_tab_weapons,
        ];
        // 받을 것이 있는 묶음(0265 — 여정은 받을 목표, 도감은 받을 칸): 고르지 않은 머리를 민트로.
        let marked: Vec<bool> = (0..GROUPS)
            .map(|t| {
                if t == JOURNEY {
                    crate::journey::any_ready(&self.progress)
                } else {
                    self.codex_group_ready(t)
                }
            })
            .collect();
        let tk = 0.85 * k;
        let ty = w.at(0.0, 2.0).y;
        // 0257(사장 "도감 속에 있는 작은 탭도 마찬가지"): 칸이 왼쪽 열에 다 들지 않으면(긴 언어·좁은 화면) 가로로 굴린다 —
        // ◀ ▶는 늘 그 자리(좌우로 넘긴다는 안내), 넘칠 때는 "더 있다"로 밝아진다.
        let arrow = self.ui.width("◀", tk) + u * 6.0;
        let (x0, x1) = (w.at(4.0, 0.0).x, w.at(tabs::LEFT_W - 6.0, 0.0).x);
        let tws: Vec<f32> = names.iter().map(|n| self.ui.width(n, tk)).collect();
        // 0242(사장 설문 23-9 "짧은 이름의 버튼도 누르기 쉽게 최소 넓이"): 칸은 `GROUP_MIN_W` 이상, 글은 칸 가운데.
        let cws: Vec<f32> = tws.iter().map(|tw| tw.max(GROUP_MIN_W * u)).collect();
        let (at, view) = ui::strip_tabs(
            &mut self.small_strip,
            &cws,
            x0 + arrow,
            x1 - arrow,
            w.origin.y,
            GROUP_H * u,
            u * 12.0,
            0.0,
            tab,
        );
        let (more_l, more_r) = self.small_strip.more();
        let ink = |more: bool| if view.is_some() && more { INK } else { DIM };
        self.ui.text("◀", x0, ty, tk, ink(more_l));
        self.ui
            .text("▶", x1 - self.ui.width("▶", tk), ty, tk, ink(more_r));
        if let Some(v) = view {
            ui::clip(Some(Rect::new(v.x - u * 5.0, v.y, v.w + u * 10.0, v.h)));
        }
        for (i, name) in names.iter().enumerate() {
            let (x, cw) = at[i];
            if let Some(v) = view {
                if x + cw <= v.x || x >= v.x + v.w {
                    continue;
                }
            }
            let tw = tws[i];
            let tx = x + (cw - tw) * 0.5;
            let on = i == tab;
            let whole = view.is_none_or(|v| x >= v.x - 0.5 && x + cw <= v.x + v.w + 0.5);
            if whole {
                let ink = if on {
                    AMBER
                } else if marked[i] {
                    ui::MINT
                } else {
                    DIM
                };
                self.ui.text(name, tx, ty, tk, ink);
            }
            if on {
                draw_rectangle(
                    x,
                    (ty + self.ui.px(tk) + u * 1.5).round(),
                    cw,
                    u.round().max(1.0),
                    AMBER,
                );
            }
            self.hits.push((
                Rect::new(x - u * 5.0, w.origin.y, cw + u * 10.0, GROUP_H * u),
                Hit::Cell(1, i),
            ));
        }
        if view.is_some() {
            ui::clip(None);
        }
        if tab == JOURNEY {
            self.draw_journey(f, &w, scene, GROUP_H);
            return;
        }
        // 왼쪽 목록: 본 것·가진 것은 이름, 아니면 "???".
        let rec = &self.progress.codex.clone();
        // 줄 끝 ◆ = 받을 보상이 있다(0265 — 여정의 표지와 같다).
        let rows: Vec<ui::Row> = (0..self.codex_len(tab))
            .map(|i| {
                let b = self.codex_bounty(tab, i);
                let name = if b.open { tr(b.name) } else { s.codex_unknown };
                if b.ready(&self.progress) {
                    let mut r = ui::Row::value(name, "◆".to_string());
                    r.tag = true;
                    r
                } else {
                    ui::Row::plain(name)
                }
            })
            .collect();
        let cursor = self.cursor.min(rows.len().saturating_sub(1));
        self.draw_left_list(&w, &rows, GROUP_H);
        // 오른쪽: 고른 것.
        let t = get_time() as f32;
        let mut lines: Vec<Ln> = Vec::new();
        let bounty = self.codex_bounty(tab, cursor);
        if tab == FOES {
            let e = &ENTRIES[cursor.min(ENTRIES.len() - 1)];
            let seen = codex::seen(rec, e.kind);
            // 이름 아래 칸 폭 전체에 몸, 그 아래로 나오는 곳 · 관찰.
            let name = if seen {
                tr(crate::species::def(e.kind).name)
            } else {
                s.codex_unknown
            };
            let p = w.at(tabs::RIGHT_X, 0.0);
            let fit = self.ui.fit(name, tabs::RIGHT_W * u, tabs::K_NAME * k);
            let ink = if seen { AMBER } else { INK };
            self.ui.text(&fit, p.x, p.y, tabs::K_NAME * k, ink);
            let pic = w.rect(tabs::RIGHT_X, PIC_TOP, tabs::RIGHT_W, PIC_H);
            let spin = self.codex_view.yaw.unwrap_or(0.6 * (t * 0.6).sin());
            self.renderer.draw_creature(e.kind, pic, spin, t, !seen);
            self.codex_view.rect = Some(pic);
            lines.extend(self.pane_wrap(&w, &fill(s.codex_place, &[tr(e.place)]), PLACE));
            lines.push(Ln::Gap);
            if seen {
                for (guess, l) in codex::open_lines(rec, e) {
                    let head = if guess { "?" } else { "·" };
                    let col = if guess { GUESS } else { DIM };
                    lines.extend(self.pane_wrap(&w, &format!("{head} {}", tr(l)), col));
                    lines.push(Ln::Gap);
                }
                // 타입(0223 V2): 잘 듣는 것·덜 듣는 것 한 줄씩(배율이 1인 타입은 말하지 않는다).
                let (weak, resist) = crate::i18n::ail_leanings(&crate::species::def(e.kind).ail);
                for (t, names) in [(s.ail_weak, weak), (s.ail_resist, resist)] {
                    if !names.is_empty() {
                        lines.extend(self.pane_wrap(&w, &fill(t, &[&names]), DIM));
                    }
                }
            } else {
                lines.extend(self.pane_wrap(&w, s.codex_not_seen, DIM));
            }
            self.push_bounty_line(&w, &bounty, &mut lines);
            self.draw_pane(&w, PIC_TOP + PIC_H + 4.0, &lines);
        } else {
            let things = self.codex_things(tab);
            let d = &things[cursor.min(things.len() - 1)];
            self.draw_pane_head(
                &w,
                &PaneHead {
                    name: if d.owned { tr(d.name) } else { s.codex_unknown },
                    gold: d.owned,
                    // 장신구는 종류까지(0224 — "장신구 · 목걸이": 같은 종류 둘은 함께 못 낀다).
                    kind: match d.item {
                        crate::item_view::Item::Charm(id) => {
                            format!("{} · {}", names[tab], tr(id.kind().name()))
                        }
                        _ => names[tab].to_string(),
                    },
                    state: (String::new(), DIM),
                    icon: false,
                },
            );
            // 0242(사장 설문 8-1 "물건이 너무 작다"): 머리 곁의 30칸 그림 대신 이름 아래 큰 그림 칸(적의 그림 칸과 같은 꼴).
            let icon = w.rect(
                tabs::RIGHT_X + (tabs::RIGHT_W - ITEM_PIC) * 0.5,
                tabs::HEAD_H - 8.0,
                ITEM_PIC,
                ITEM_PIC,
            );
            self.codex_view.rect = None;
            // 무기는 종류부터(0272 — "한손검 · 순례자의 검" + 그 종류의 주특성): 못 얻은 무기도 종류는 보인다(실루엣과 같은 몫).
            if let crate::item_view::Item::Weapon(id) = d.item {
                let name = if d.owned { tr(d.name) } else { s.codex_unknown };
                if let Some((class, main, sub)) = super::equip::class_text(id, name) {
                    lines.extend(self.pane_wrap(&w, &class, ui::INK));
                    if d.owned {
                        lines.extend(self.pane_wrap(&w, &main, ui::AMBER));
                        // 보조특성(0272 1b): 보조무기 슬롯에 멨을 때의 것 — 한 단 흐리게.
                        lines.extend(self.pane_wrap(&w, &sub, DIM));
                    }
                    lines.push(Ln::Gap);
                }
            }
            if d.owned {
                if crate::item_view::has_view(d.item) {
                    self.draw_pane_item_turned(d.item, icon, self.codex_view.yaw);
                    self.codex_view.rect = Some(icon);
                }
                lines.extend(self.pane_wrap(&w, tr(d.desc), DIM));
                if let Some(flavor) = d.flavor {
                    lines.push(Ln::Gap);
                    lines.extend(self.pane_wrap(&w, tr(flavor), GUESS));
                }
            } else {
                // 못 가진 것도 실루엣(2026-11-23 사장 "어떤 녀석은 실루엣이고 어떤 녀석들은 그냥 물음표" — 생물과 같은 꼴).
                if crate::item_view::has_view(d.item) {
                    let side = crate::render::ITEM_PX as f32
                        * (icon.w / crate::render::ITEM_PX as f32).floor().max(1.0);
                    let dest = Rect::new(
                        (icon.x + (icon.w - side) * 0.5).round(),
                        (icon.y + (icon.h - side) * 0.5).round(),
                        side,
                        side,
                    );
                    let t = get_time() as f32;
                    let spin = self.codex_view.yaw.unwrap_or(0.7 * (t * 0.8).sin());
                    self.renderer.draw_item_shadow(d.item, dest, spin);
                    self.codex_view.rect = Some(icon);
                } else {
                    self.ui.text_center(
                        "?",
                        icon.x + icon.w * 0.5,
                        icon.y + (icon.h - self.ui.px(1.6 * k)) * 0.5,
                        1.6 * k,
                        ui::BORDER,
                    );
                }
                lines.extend(self.pane_wrap(&w, s.codex_not_owned, DIM));
            }
            lines.push(Ln::Gap);
            lines.extend(self.pane_wrap(&w, &fill(s.codex_place, &[tr(d.place)]), PLACE));
            self.push_bounty_line(&w, &bounty, &mut lines);
            self.draw_pane(&w, tabs::HEAD_H - 8.0 + ITEM_PIC + 4.0, &lines);
        }
        let right = w.panel.x + w.panel.w;
        let mut pairs = vec![(Act::Walk, s.verb_choose)];
        if bounty.ready(&self.progress) {
            pairs.push((Act::Confirm, s.verb_take));
        }
        self.tab_foot(f, &w, scene, right, &pairs);
    }

    /// 오른쪽 칸 끝의 보상 줄(0265): 받을 수 있으면 호박빛, 받았으면 흐리게, 아직 못 연 칸에는 없다.
    fn push_bounty_line(&mut self, w: &super::frame::Window, b: &Bounty, lines: &mut Vec<Ln>) {
        let s = s();
        if !b.open {
            return;
        }
        let paid = b.paid(&self.progress);
        let reward = fill(s.journey_embers, &[&b.embers.to_string()]);
        let text = if paid {
            fill(s.codex_bounty_taken, &[&reward])
        } else {
            fill(s.journey_reward, &[&reward])
        };
        lines.push(Ln::Gap);
        lines.extend(self.pane_wrap(w, &text, if paid { DIM } else { AMBER }));
    }
}

/// 도감 한 칸의 보상(0265).
struct Bounty {
    /// 적혔다·얻었다.
    open: bool,
    /// 받은 기록 이름(`codex::paid_key`).
    key: String,
    embers: u32,
    name: Tr,
}

impl Bounty {
    fn paid(&self, p: &crate::save::Progress) -> bool {
        p.codex_paid.contains(&self.key)
    }
    fn ready(&self, p: &crate::save::Progress) -> bool {
        self.open && !self.paid(p)
    }
}

/// 도감의 물건 한 줄(장신구·덧옷 0176·무기 0191 — 같은 꼴로 보인다). 무기는 설명에 세계 안의 말이 들어 있어 따로 없다.
struct Thing {
    owned: bool,
    /// 보상을 받은 기록 이름(0265 — `codex::paid_key`).
    paid: String,
    name: Tr,
    desc: Tr,
    flavor: Option<Tr>,
    place: Tr,
    item: crate::item_view::Item,
}

impl App {
    fn codex_things(&self, tab: usize) -> Vec<Thing> {
        things(&self.progress, tab)
    }
}

/// 묶음 `tab`(장신구·덧옷·무기)의 물건들(표 순서 — 0278 R22: 기록만 받는 순수 함수, 시험이 붙는다).
fn things(p: &crate::save::Progress, tab: usize) -> Vec<Thing> {
    if tab == WEAPONS {
        // 맨손은 도감의 무기가 아니다(0265 — 사장 "무기중에 맨손은 빠져야지": 얻는 것이 아니라 무기 칸을 비운 것).
        return crate::weapons::IDS
            .iter()
            .filter(|&&id| id != crate::weapons::Id::Fists)
            .map(|&id| {
                let w = id.def();
                let paid = codex::paid_key("weapon", id.key());
                Thing {
                    // 팔거나 내려놓은 것도 도감에는 남는다(0266 — `codex_had`).
                    owned: p.weapons.contains(&id) || p.codex_had.contains(&paid),
                    paid,
                    name: w.name,
                    desc: w.desc,
                    flavor: None,
                    place: w.place,
                    item: crate::item_view::Item::Weapon(id),
                }
            })
            .collect();
    }
    if tab == CHARMS {
        crate::charms::TABLE
            .iter()
            .map(|d| Thing {
                owned: p.owned.contains(&d.id)
                    || p.codex_had.contains(&codex::paid_key("charm", d.key)),
                paid: codex::paid_key("charm", d.key),
                name: d.name,
                desc: d.desc,
                flavor: Some(d.flavor),
                place: codex::charm_place(d.id),
                item: crate::item_view::Item::Charm(d.id),
            })
            .collect()
    } else {
        let have = crate::garb::owned(&p.finds);
        crate::garb::TABLE
            .iter()
            .map(|d| Thing {
                owned: have.contains(&d.id),
                paid: codex::paid_key("garb", d.key),
                name: d.name,
                desc: d.desc,
                flavor: Some(d.flavor),
                place: d.place,
                item: crate::item_view::Item::Garb(d.id),
            })
            .collect()
    }
}

/// 묶음의 줄 수.
fn group_len(p: &crate::save::Progress, tab: usize) -> usize {
    match tab {
        FOES => ENTRIES.len(),
        CHARMS | GARB | WEAPONS => things(p, tab).len(),
        _ => 0,
    }
}

/// 묶음 `tab`의 `i`째 칸의 보상(0265).
fn bounty_at(p: &crate::save::Progress, tab: usize, i: usize) -> Bounty {
    if tab == FOES {
        let e = &ENTRIES[i.min(ENTRIES.len() - 1)];
        return Bounty {
            open: codex::seen(&p.codex, e.kind),
            key: codex::paid_key("foe", codex::kind_key(e.kind)),
            embers: codex::FOE_EMBERS,
            name: crate::species::def(e.kind).name,
        };
    }
    let things = things(p, tab);
    let t = &things[i.min(things.len() - 1)];
    Bounty {
        open: t.owned,
        key: t.paid.clone(),
        embers: codex::THING_EMBERS,
        name: t.name,
    }
}

/// 묶음에 받을 칸이 있나(여정은 아니다 — 그것은 `journey::any_ready`).
fn group_ready(p: &crate::save::Progress, tab: usize) -> bool {
    tab != JOURNEY && (0..group_len(p, tab)).any(|i| bounty_at(p, tab, i).ready(p))
}

/// 받을 도감 칸이 하나라도 있나.
fn any_ready(p: &crate::save::Progress) -> bool {
    (FOES..GROUPS).any(|t| group_ready(p, t))
}

/// 기록 탭에 들어올 때 여는 묶음: 받을 목표가 있으면 여정, 아니면 받을 칸이 있는 첫 묶음, 없으면 여정.
fn start_group(p: &crate::save::Progress) -> usize {
    if crate::journey::any_ready(p) {
        return JOURNEY;
    }
    (FOES..GROUPS)
        .find(|&t| group_ready(p, t))
        .unwrap_or(JOURNEY)
}

/// 추측 줄·세계 안의 말: 관찰보다 옅고 푸르스름하게(0129 장비 창과 같은 결).
const GUESS: Color = Color::new(0.55, 0.56, 0.62, 1.0);
/// 나오는 곳: 옅은 호박빛(길 안내 색의 결).
const PLACE: Color = Color::new(0.78, 0.66, 0.46, 1.0);

#[cfg(test)]
mod tests {
    use super::*;

    /// 0278 R22: 도감의 받을 칸은 기록만으로 센다 — 얻은(적힌) 칸이고 아직 안 받은 것, 받으면 빠지고, 팔고 내려놓아 가진 것에서
    /// 빠져도 도감의 기억(`codex_had`)으로 남는다. 기록 탭은 받을 것이 있는 첫 묶음에서 연다(없으면 여정).
    #[test]
    fn codex_bounties_count_from_the_record_and_the_tab_opens_on_the_first_group_with_one() {
        let mut p = crate::save::Progress::default();
        for t in FOES..GROUPS {
            assert!(!group_ready(&p, t), "group {t}");
        }
        assert!(!any_ready(&p));
        assert_eq!(start_group(&p), JOURNEY);
        assert_eq!(group_len(&p, FOES), ENTRIES.len());
        assert_eq!(group_len(&p, CHARMS), crate::charms::TABLE.len());
        assert_eq!(group_len(&p, GARB), crate::garb::TABLE.len());
        assert_eq!(group_len(&p, WEAPONS), crate::weapons::IDS.len() - 1);
        assert_eq!(group_len(&p, JOURNEY), 0);
        let ready = |p: &crate::save::Progress, t: usize| {
            (0..group_len(p, t))
                .filter(|&i| bounty_at(p, t, i).ready(p))
                .count()
        };
        // 장신구 하나를 얻었다: 장신구 묶음에 하나.
        let charm = crate::charms::TABLE[1].id;
        p.owned.push(charm);
        assert_eq!(ready(&p, CHARMS), 1);
        assert_eq!(start_group(&p), CHARMS);
        // 적 하나를 적었다: 적 묶음이 먼저(차례).
        let foe = ENTRIES[0].kind;
        p.codex.push(codex::kind_key(foe).to_string());
        assert_eq!(ready(&p, FOES), 1);
        assert_eq!(start_group(&p), FOES);
        // 받으면 빠진다.
        let b = bounty_at(&p, FOES, 0);
        assert_eq!(b.embers, codex::FOE_EMBERS);
        p.codex_paid.push(b.key.clone());
        assert_eq!(ready(&p, FOES), 0);
        assert_eq!(start_group(&p), CHARMS);
        // 팔아 가진 것에서 빠져도 기억으로 남는다(무기).
        let at = |p: &crate::save::Progress| {
            (0..group_len(p, WEAPONS))
                .find(|&i| {
                    things(p, WEAPONS)[i].item
                        == crate::item_view::Item::Weapon(crate::weapons::Id::Hammer)
                })
                .unwrap()
        };
        let i = at(&p);
        assert!(!bounty_at(&p, WEAPONS, i).open);
        p.codex_had
            .push(codex::paid_key("weapon", crate::weapons::Id::Hammer.key()));
        assert!(bounty_at(&p, WEAPONS, i).ready(&p));
        assert_eq!(bounty_at(&p, WEAPONS, i).embers, codex::THING_EMBERS);
        assert!(any_ready(&p));
    }
}
