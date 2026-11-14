// Wattkyuki
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 67752972;

pub const LISTED_SERIES: &[u16] = &[SET_WATT];
pub const LISTED_NAMES: &[u32] = &[ID];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_WATT], Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_race_full(duel, RACE_THUNDER, Some(sc), sumtype, tp)
    }
}

// Copies on the field to attack directly, a Watt Tuner in the Graveyard and Thunder monsters for the shuffle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (18407024, GRAVE),
    (27324313, MZONE),
    (2772236, EXTRA),
    (29765339, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DIRECT_ATTACK);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_DAMAGE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spsynccon);
    e2.set_target(duel, spsynctg);
    e2.set_operation(duel, spsyncop);
    c.register_effect(duel, e2);
}

fn spsynccon(duel: &Duel, a: &Args) -> bool {
    a.ep == 1 - a.tp && duel.get_attack_target().is_none()
}

fn gyfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_WATT]) && c.is_type(duel, TYPE_TUNER) && c.is_able_to_deck(duel)
}

fn mzfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_THUNDER) && !c.is_type(duel, TYPE_TUNER) && c.is_able_to_deck(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, matg: Option<CardSet>) -> bool {
    c.is_set_card(duel, &[SET_WATT])
        && c.is_type(duel, TYPE_SYNCHRO)
        && !c.is_code(duel, ID)
        && duel.get_location_count_from_ex(tp, tp, matg, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn rescon(duel: &Duel, sg: CardSet, e: EffectId, tp: u8) -> (bool, bool) {
    (
        sg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_MZONE), None) == 1
            && sg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None) == 1
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, Some(sg)), tp, LOCATION_EXTRA, 0, 1, None),
        false,
    )
}

fn spsynctg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g1 = duel.get_matching_group(gyfilter, tp, LOCATION_GRAVE, 0, None);
    let g2 = duel.get_matching_group(mzfilter, tp, LOCATION_MZONE, 0, None);
    if chk == 0 {
        return Ok(!g1.is_empty()
            && !g2.is_empty()
            && aux::select_unselect_group_check(duel, g1.union(g2), 2, 2, &|duel, sg, _, _| rescon(duel, sg, e, tp)));
    }
    duel.set_operation_info(0, CATEGORY_TODECK, None, 2, tp, (LOCATION_MZONE | LOCATION_GRAVE) as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spsyncop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g1 = duel.get_matching_group(gyfilter, tp, LOCATION_GRAVE, 0, None);
    let g2 = duel.get_matching_group(mzfilter, tp, LOCATION_MZONE, 0, None);
    if g1.is_empty() || g2.is_empty() {
        return Ok(());
    }
    let rg = aux::select_unselect_group(
        duel,
        g1.union(g2),
        2,
        2,
        &|duel, sg, _, _| rescon(duel, sg, e, tp),
        tp,
        HINTMSG_TODECK,
    )?;
    if rg.len() != 2 {
        return Ok(());
    }
    duel.hint_selection(rg);
    if duel.send_to_deck(rg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && duel
            .get_operated_group()
            .is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK | LOCATION_EXTRA), 1, None)
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, None), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        if let Some(sc) = g.get_first() {
            duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
