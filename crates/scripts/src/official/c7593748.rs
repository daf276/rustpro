// Gimmick Puppet Gigantes Doll
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 7593748;

pub const LISTED_SERIES: &[u16] = &[SET_GIMMICK_PUPPET];
// On the field with Xyz Import to attach Sangan twice for the Control effect, and monsters of other Levels to change. No Level 4
// Gimmick Puppet is ported, so no Xyz Summon of this card is reached.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (2118022, MZONE),
    (2483611, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, _, _, _| c.is_set_card(duel, &[SET_GIMMICK_PUPPET]));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_CONTROL);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::detach_from_self::<2, 2>);
    e1.set_target(duel, cttg);
    e1.set_operation(duel, ctop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_LVCHANGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_target(duel, lvtg);
    e2.set_operation(duel, lvop);
    c.register_effect(duel, e2);
}

fn cttg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_able_to_change_controler(duel) && chkc.is_location(duel, LOCATION_MZONE));
    }
    if chk == 0 {
        return Ok(ft > 0 && duel.is_existing_target(CardId::is_able_to_change_controler, tp, 0, LOCATION_MZONE, 1, None));
    }
    let ct = ft.min(2) as u8;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g = duel.select_target(tp, CardId::is_able_to_change_controler, tp, 0, LOCATION_MZONE, 1, ct, None)?;
    let count = g.map_or(0, |g| g.len()) as u8;
    duel.set_operation_info(0, CATEGORY_CONTROL, g, count, 0, 0);
    Ok(false)
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let ge1 = duel.create_effect(c);
    ge1.set_type(duel, EFFECT_TYPE_FIELD);
    ge1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    ge1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    ge1.set_target_range(duel, 1, 0);
    ge1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_set_card(duel, &[SET_GIMMICK_PUPPET]));
    ge1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(ge1, tp);
    let ge2 = duel.create_effect(c);
    ge2.set_type(duel, EFFECT_TYPE_FIELD);
    ge2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    ge2.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    ge2.set_target_range(duel, LOCATION_MZONE, 0);
    ge2.set_target_card(duel, |duel, _, c| !c.is_type(duel, TYPE_XYZ));
    ge2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(ge2, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    let tg = duel.get_target_cards(e).unwrap_or_default();
    if !tg.is_empty() {
        let reason_player = duel.get_reason_player();
        duel.get_control_full(tg, tp, PHASE_END as u16, 1, 0xff, reason_player)?;
    }
    Ok(())
}

fn lvfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.has_level(duel) && !c.is_level(duel, &[8])
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(lvfilter, a.tp, LOCATION_MZONE, 0, 1, Some(c.into())));
    }
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(lvfilter, a.tp, LOCATION_MZONE, 0, None);
    let c = a.e.get_handler(duel);
    for tc in g {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, 8);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
