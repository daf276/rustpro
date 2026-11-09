// Relinquished Anima
use super::c64631466::{adcon, eqcon, eqfilter};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 94259633;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.get_level(duel) == 1 && !c.is_type_full(duel, TYPE_TOKEN, Some(lc), sumtype, tp)
    }
}

// On the field and in the Extra Deck, with Level 1 Normal Monsters (Mokey Mokey, Key Mace) to equip and to Link Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (27288416, MZONE),
    (27288416, MZONE),
    (1929294, MZONE),
    (1929294, 0),
    (27288416, MZONE | OPPONENT),
    (27288416, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, eqcon::<ID>);
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop2);
    c.register_effect(duel, e1);
    aux::add_er_equip_limit(duel, c, Some(eqcon::<ID>), e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_condition(duel, adcon::<ID>);
    e2.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e2);
}

fn eqfilter2(c: CardId, duel: &Duel, tp: u8, lg: CardSet) -> bool {
    c.is_faceup(duel) && (c.is_controler(duel, tp) || c.is_able_to_change_controler(duel)) && lg.is_contains(c)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let lg = a.e.get_handler(duel).get_linked_group(duel);
    let filter = |c, duel: &Duel| eqfilter2(c, duel, tp, lg);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(
            duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_EQUIP, g, 1, 0, 0);
    Ok(false)
}

fn eqop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_facedown(duel) {
        return Ok(());
    }
    if tc.is_relate_to_effect(duel, e) && tc.is_monster(duel) && eqcon::<ID>(duel, a) {
        c.equip_by_effect_and_limit_register(duel, e, tp, tc, Some(ID), None)?;
    }
    Ok(())
}

fn atkval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    let Some(tc) = e
        .get_handler(duel)
        .get_equip_group(duel)
        .filter(duel, eqfilter::<ID>, None)
        .get_first()
    else {
        return 0;
    };
    let atk = tc.get_text_attack(duel);
    if tc.get_original_type(duel) & TYPE_MONSTER == 0 || atk < 0 {
        0
    } else {
        atk
    }
}
