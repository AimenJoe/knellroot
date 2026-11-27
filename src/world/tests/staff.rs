//! 지팡이(0282 — 마법이 먼저, 사장 "지팡이는 이제 타격이 덜 화려해야" · "완드처럼 바꾼다"): RB의 마법 탄이 주된 공격(칸이 많고 빨리
//! 찬다 — 완드의 저주탄과 같은 꼴), 근접은 수수한 3연타(검보다 덜 낸다 — 붙은 적을 밀어내 쏠 자리를 연다). 지팡이다움은 속성
//! 게이지(주특성 `Art::Channel` · 보조특성 · 스킬 트리)에 그대로 있다.
use super::*;
use crate::weapons::{
    Id as W, Look, Shape, BARE_STAFF_EXTRA, MAGIC_TOSS, STAFF_CHARGES, STAFF_REFILL,
};

/// RB 칸: 옛 지팡이는 다른 종류와 같은 셋 · 차는 배 1(3초에 하나)이었다 — 이제 칸이 많고(`STAFF_CHARGES`) 빨리 찬다(`STAFF_REFILL`).
/// 월드의 칸 수(HUD · 쉼터가 읽는 `cast_charges`)도 그것이고, 빈 칸에서 다섯이 다 차는 것이 옛 셋이 다 차던 것(9초)보다 빠르다.
#[test]
fn the_staffs_bolt_holds_more_charges_and_refills_faster_than_before() {
    let t = Tuning::default();
    let toss = W::Staff.toss().expect("the staff has a toss");
    assert_eq!(toss, MAGIC_TOSS);
    assert_eq!((toss.charges, toss.refill), (STAFF_CHARGES, STAFF_REFILL));
    assert!(
        toss.charges > crate::tuning::CAST_CHARGES,
        "more than three"
    );
    assert!(toss.refill < 1.0, "fills faster than one per cast cooldown");
    let mut w = World::new("test", 30.0);
    w.set_weapon(W::Staff);
    assert_eq!(w.cast_charges(), STAFF_CHARGES);
    w.player.ranged.mana = 0.0;
    let fill = STAFF_CHARGES * t[P::CastCooldown] * STAFF_REFILL;
    assert!(
        fill < crate::tuning::CAST_CHARGES * t[P::CastCooldown],
        "{fill}"
    );
    for _ in 0..((fill + 0.1) / TICK_DT).round() as usize {
        idle(&mut w);
        w.step(&Actions::default(), &t);
    }
    assert!((w.player.ranged.mana - STAFF_CHARGES).abs() < 1e-3);
    // 두 손으로 엮는 손(빈 손의 결)은 그 위에 하나 더.
    w.set_open_hand(true);
    assert_eq!(w.cast_charges(), STAFF_CHARGES + BARE_STAFF_EXTRA);
    // 다른 근접 종류는 그대로 셋.
    for id in [
        W::Sword,
        W::Hammer,
        W::Spear,
        W::Daggers,
        W::Greatsword,
        W::Scythe,
    ] {
        let toss = id.toss().unwrap();
        assert_eq!(
            (toss.charges, toss.refill),
            (crate::tuning::CAST_CHARGES, 1.0),
            "{id:?}"
        );
    }
}

/// 마법 탄은 그대로 땅을 따라 흘러 꿰뚫는다(완드의 곧은 저주탄과 갈린다 — 속성도 싣지 않는다: 지팡이의 속성은 스킬에). 모은 탄은
/// 피해가 아니라 크기 · 거리 · 자세 피해 · 휘청 — 칸이 빨리 차는 지팡이에서 모은 탄이 곱절을 내면 마구 쏘기보다 늘 세다(완드와 같은 까닭).
#[test]
fn the_staffs_bolt_still_flows_and_pierces_and_charging_buys_reach_not_damage() {
    let (bolt, charged) = W::Staff.bolts();
    assert_eq!((bolt.look, charged.look), (Look::Magic, Look::Magic));
    assert!(!bolt.straight() && !bolt.thrown() && !bolt.smites() && !bolt.is_shot());
    assert!(bolt.pierce && charged.pierce);
    assert_eq!(bolt.dose.1, 0.0);
    assert_eq!(charged.dose.1, 0.0);
    assert_eq!(charged.damage, bolt.damage);
    assert!(charged.range > bolt.range && charged.scale > bolt.scale);
    assert!(charged.poise > bolt.poise && charged.stagger > 0.0);
    // 완드와 칸의 꼴은 같다 — 다른 것은 탄(흐르는 마법 vs 곧은 저주)과 종류의 결.
    let wand = W::Wand.toss().unwrap();
    assert_eq!(wand.charges, STAFF_CHARGES);
    assert_ne!(wand.bolt.look, bolt.look);
}

/// 근접은 수수하다: 3연타(옛 4연타의 넷째 — 두 손 둘레 내려치기 — 는 걷었다), 판정은 모두 앞 부채꼴(둘레 치기 없음), 타마다 10, 검보다
/// 짧은 사이에 덜 낸다(검의 끝 타 20이 없다 · 같은 시간의 피해도 검 아래), 넉백은 검의 평타보다 크고 끝 타가 가장 크다(밀어낸다).
/// 강공격도 둘레로 밀어낼 뿐 피해는 평타 하나만큼. RB가 주된 공격이라는 종류의 성질(`Quirks::rb_main`)은 완드와 지팡이만.
#[test]
fn the_staff_swings_three_plain_pushing_hits_that_deal_less_than_the_sword() {
    let s = W::Sword.def();
    let k = W::Staff.def();
    assert_eq!(k.combo.len(), 3);
    assert!(
        k.combo.iter().all(|h| h.shape == Shape::Arc),
        "no spin finisher"
    );
    assert!(k.combo.iter().all(|h| h.damage == 10));
    let sum = |c: &[weapons::Hit]| c.iter().map(|h| h.damage).sum::<i32>();
    assert!(sum(k.combo) < sum(s.combo));
    assert!(k.combo.iter().all(|h| h.knock > s.combo[0].knock));
    assert!(
        k.combo[2].knock > k.combo[0].knock,
        "the last hit pushes most"
    );
    assert!(k.heavy.unwrap().damage <= k.combo[0].damage);
    assert_eq!(k.heavy.unwrap().shape, Shape::Circle, "a push all around");
    let (staff, _) = super::ranged::dps(W::Staff, 20.0, true, 1.2);
    let (sword, _) = super::ranged::dps(W::Sword, 20.0, true, 1.2);
    assert!(staff < sword, "staff {staff:.1}/s vs sword {sword:.1}/s");
    let main: Vec<weapons::Class> = weapons::CLASSES
        .iter()
        .filter(|c| c.quirks.rb_main)
        .map(|c| c.class)
        .collect();
    assert_eq!(main, [weapons::Class::Staff, weapons::Class::Wand]);
}
