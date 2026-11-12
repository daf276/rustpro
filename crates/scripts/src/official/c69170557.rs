// Number C40: Gimmick Puppet of Dark Strings
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 69170557;
pub const XYZ_NUMBER: u32 = 40;
const COUNTER_STRING: u16 = 0x1024;

// On the field with Xyz Import to attach Sangan, which Dark Strings detaches to place String Counters on, and in the Extra Deck, Xyz Summoned with
// three Level 9 monsters (Shiina, Twin Tempests of Celestial Thunder) to destroy the monsters with them.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (12197223, MZONE),
    (12197223, MZONE),
    (12197223, MZONE),
    (12197223, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(9);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_DRAW + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_COUNTER);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, cttg);
    e2.set_operation(duel, ctop);
    c.register_effect(duel, e2);
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.get_counter(duel, COUNTER_STRING) != 0
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_matching_group(desfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(desfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if duel.destroy(g, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let og = duel
        .get_operated_group()
        .filter(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None);
    if duel.draw(tp, 1, REASON_EFFECT)? != 0 && !og.is_empty() {
        duel.break_effect()?;
        let matk = og
            .get_max_group(duel, |c, duel| c.get_base_attack(duel).into())
            .map_or(0, |(_, atk)| atk);
        if matk > 0 {
            duel.damage(1 - tp, matk as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn cttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(CardId::is_faceup, a.tp, 0, LOCATION_MZONE, 1, None))
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(CardId::is_faceup, a.tp, 0, LOCATION_MZONE, None);
    for tc in g {
        tc.add_counter(duel, COUNTER_STRING, 1);
    }
    Ok(())
}
