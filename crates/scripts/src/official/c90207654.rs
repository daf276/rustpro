// Gullveig of the Nordic Ascendant
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 90207654;

pub const LISTED_SERIES: &[u16] = &[SET_NORDIC, SET_AESIR];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_NORDIC], Some(lc), sumtype, tp) && c.is_level_below(duel, 5)
    }
}

// Link Summoned from a Nordic monster (Valkyrie, Vanadis, Mara) with more Nordic monsters in the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (40844552, MZONE),
    (61777313, MZONE),
    (73417207, MZONE),
    (88283496, DECK),
    (2333365, DECK),
    (88283496, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_condition(duel, tgcon);
    e2.set_target_card(duel, |duel, e, c| {
        tgfilter(c, duel) && e.get_handler(duel).get_linked_group(duel).is_contains(c)
    });
    e2.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_CANNOT_BE_BATTLE_TARGET);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, tgcon);
    e3.set_value(duel, Value::Card(aux::imval1));
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_NORDIC]) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn spcheck(duel: &Duel, sg: CardSet, e: EffectId, tp: u8) -> bool {
    duel.get_mzone_count_full(tp, Some(sg), tp, LOCATION_REASON_TOFIELD, 0xff) >= sg.len() as i32
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, sg.len(), None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(CardId::is_able_to_remove, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, None);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 1, 3, &|duel, sg, _, _| {
            (spcheck(duel, sg, e, tp), false)
        }));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = duel.get_matching_group(CardId::is_able_to_remove, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, None);
    let ct = if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        1
    } else {
        3
    };
    let rg = aux::select_unselect_group(
        duel,
        g,
        1,
        ct,
        &|duel, sg, _, _| (spcheck(duel, sg, e, tp), false),
        tp,
        HINTMSG_REMOVE,
    )?;
    if !rg.is_empty() && duel.remove(rg, POS_FACEUP, REASON_EFFECT)? != 0 {
        let ct = (duel.get_operated_group().len() as i32)
            .min(duel.get_location_count(tp, LOCATION_MZONE))
            .max(0) as u8;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, ct, ct, None)?;
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_set_card(duel, &[SET_AESIR]));
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH);
    e2.set_code(duel, EFFECT_CANNOT_SUMMON);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    e2.set_target_range(duel, 1, 0);
    duel.register_effect(e2, tp);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_CANNOT_MSET);
    duel.register_effect(e3, tp);
    let e4 = duel.create_effect(c);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_reset(duel, RESET_PHASE | PHASE_END);
    e4.set_target_range(duel, 1, 0);
    duel.register_effect(e4, tp);
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_AESIR]) && c.is_monster(duel)
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_linked_group(duel).is_exists(duel, tgfilter, 1, None)
}
