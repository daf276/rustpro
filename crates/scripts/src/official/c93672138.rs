// Evil★Twin's Trouble Sunny
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 93672138;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(4);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(
            duel,
            |c, duel| c.is_set_card_full(duel, &[SET_EVIL_TWIN], Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

// In the Extra Deck and Graveyard beside Evil★Twin Lil-la (an Evil★Twin monster to Link Summon from and to send as cost) and Ki-sikil and Lil-la monsters in the Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, GRAVE),
    (36609518, MZONE),
    (36609518, MZONE),
    (81078880, MZONE),
    (54257392, GRAVE),
    (81078880, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_cost(duel, Cost::self_tribute);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, tgcost);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_KI_SIKIL, SET_LIL_LA]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sprescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (
        sg.filter_count(duel, |c, duel| c.is_set_card(duel, &[SET_KI_SIKIL]), None) < 2
            && sg.filter_count(duel, |c, duel| c.is_set_card(duel, &[SET_LIL_LA]), None) < 2,
        false,
    )
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_mzone_count(tp, Some(e.get_handler(duel).into())) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE).min(2);
    if ft < 1 {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    let sg = aux::select_unselect_group(duel, g, 1, ft as usize, &sprescon, tp, HINTMSG_SPSUMMON)?;
    if !sg.is_empty() {
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn tgcostfilter(c: CardId, duel: &Duel, ct: usize) -> bool {
    c.is_set_card(duel, &[SET_EVIL_TWIN])
        && c.is_able_to_grave_as_cost(duel)
        && c.is_monster(duel)
        && (!c.is_location(duel, LOCATION_MZONE) || (c.is_faceup(duel) && ct > 1))
}

fn tgcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let ct = duel
        .get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None)
        .len();
    if chk == 0 {
        return Ok(ct > 0
            && Cost::self_banish(duel, a, 0)?
            && duel.is_existing_matching_card(
                |c, duel| tgcostfilter(c, duel, ct),
                tp,
                LOCATION_HAND | LOCATION_DECK | LOCATION_MZONE,
                0,
                1,
                None,
            ));
    }
    Cost::self_banish(duel, a, 1)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let cg = duel.select_matching_card(
        tp,
        |c, duel| tgcostfilter(c, duel, ct),
        tp,
        LOCATION_HAND | LOCATION_DECK | LOCATION_MZONE,
        0,
        1,
        1,
        None,
    )?;
    duel.send_to_grave(cg, REASON_COST)?;
    Ok(false)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_matching_group(aux::TRUE, a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(g), 1, 0, 0);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}
