// Number 18: Heraldry Patriarch
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 23649496;
pub const XYZ_NUMBER: u32 = 18;

// Two copies on the field, one destroyed by the other with Sangan pairs of the opponent's, with Xyz Import to attach, and Heraldic Beast Eale in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (82315772, DECK),
    (82315772, DECK),
    (82315772, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
}

fn same_code_filter(code: u32) -> impl Fn(CardId, &Duel) -> bool {
    aux::faceup_filter(move |c, duel| c.is_code(duel, code))
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel)
        && duel.is_existing_matching_card(
            same_code_filter(c.get_code(duel)),
            0,
            LOCATION_MZONE,
            LOCATION_MZONE,
            1,
            Some(c.into()),
        )
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(cfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    if let Some(tc) = duel
        .select_matching_card(tp, cfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?
        .get_first()
    {
        let code = tc.get_code(duel);
        let dg = duel.get_matching_group(same_code_filter(code), tp, LOCATION_MZONE, LOCATION_MZONE, Some(tc.into()));
        duel.destroy(dg, REASON_EFFECT)?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        e1.set_target_range(duel, 0, 1);
        e1.set_label(duel, i64::from(code));
        e1.set_target_spsummon(duel, |duel, e, c, _, _, _, _, _| c.is_code(duel, e.get_label(duel) as u32));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
        for code in [EFFECT_CANNOT_SUMMON, EFFECT_CANNOT_FLIP_SUMMON] {
            let clone = e1.clone_effect(duel);
            clone.set_code(duel, code);
            c.register_effect(duel, clone);
        }
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_HERALDIC_BEAST]) && c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 2, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 2, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(tgfilter, tp, LOCATION_DECK, 0, None);
    if g.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let sg = g.select(duel, tp, 2, 2, None)?;
        duel.send_to_grave(sg, REASON_EFFECT)?;
    }
    Ok(())
}
