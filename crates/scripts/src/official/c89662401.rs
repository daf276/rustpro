// Salamangreat Fowl
use crate::prelude::*;
use crate::testing::{Check, HAND, OPPONENT, SZONE};

const ID: u32 = 89662401;

pub const LISTED_SERIES: &[u16] = &[SET_SALAMANGREAT];
pub const LISTED_NAMES: &[u32] = &[ID];

// Copies in the hand, with Salamangreat Gazelle in the hand to Normal Summon and send, and the opponent's Set Salamangreat Rage.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (26889158, HAND),
    (26889158, HAND),
    (26889158, HAND),
    (14934922, SZONE | OPPONENT),
    (14934922, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_cost(duel, cacost);
    e3.set_target(duel, catg);
    e3.set_operation(duel, caop);
    c.register_effect(duel, e3);
}

fn spcfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_SALAMANGREAT]) && c.is_controler(duel, tp) && c.is_faceup(duel) && !c.is_code(duel, ID)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| spcfilter(c, duel, a.tp), 1, None)
}

fn cacfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SALAMANGREAT])
        && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
        && c.is_able_to_grave_as_cost(duel)
}

fn cacost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, cacfilter, LOCATION_HAND | LOCATION_ONFIELD)
}

fn cafilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) && c.get_flag_effect(duel, ID) == 0
}

fn catg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_SZONE) && cafilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(cafilter, tp, 0, LOCATION_SZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEDOWN);
    duel.select_target(tp, cafilter, tp, 0, LOCATION_SZONE, 1, 1, None)?;
    Ok(false)
}

fn caop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_facedown(duel)
        && tc.is_relate_to_effect(duel, a.e)
    {
        tc.register_flag_effect_full(
            duel,
            ID,
            RESETS_STANDARD_PHASE_END,
            EFFECT_FLAG_CLIENT_HINT,
            1,
            0,
            aux::stringid(ID, 2),
        );
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CANNOT_TRIGGER);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
