// T.G. Trident Launcher
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 50750868;

pub const LISTED_SERIES: &[u16] = &[SET_TG];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| mzfilter(c, duel, lc, sumtype, tp), 1, None)
    }
}

// T.G. Tuners (Tank Grub, Striker) to Link Summon with, and T.G. Warwolf and Striker in the hand, Deck and Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (74627016, MZONE),
    (74627016, MZONE),
    (1315120, MZONE),
    (1315120, MZONE),
    (293542, HAND),
    (293542, DECK),
    (293542, DECK),
    (1315120, GRAVE),
    (1315120, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_target_card(duel, tgtg);
    e2.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e2);
}

fn mzfilter(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_set_card_full(duel, &[SET_TG], Some(lc), sumtype, tp) && c.is_type_full(duel, TYPE_TUNER, Some(lc), sumtype, tp)
}

fn tgtg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    e.get_handler(duel).get_linked_group(duel).is_contains(c) && c.is_set_card(duel, &[SET_TG]) && c.is_type(duel, TYPE_SYNCHRO)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, zone: u32) -> bool {
    c.is_set_card(duel, &[SET_TG]) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, zone)
}

fn spcheck(duel: &Duel, sg: CardSet) -> (bool, bool) {
    (
        sg.get_class_count(duel, |c, duel| i64::from(c.get_location(duel))) == sg.len(),
        false,
    )
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let zone = e.get_handler(duel).get_linked_zone_full(duel, tp) & ZONES_MMZ;
        let ct = duel.get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, zone);
        let g = duel.get_matching_group(
            |c, duel| spfilter(c, duel, e, tp, zone),
            tp,
            LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
            0,
            None,
        );
        return Ok(ct > 2
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && aux::select_unselect_group_check(duel, g, 3, 3, &|duel, sg, _, _| spcheck(duel, sg)));
    }
    duel.set_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        None,
        3,
        tp,
        (LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE) as i32,
    );
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    let zone = c.get_linked_zone_full(duel, tp) & ZONES_MMZ;
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, zone));
    let sg = duel.get_matching_group(filter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE, 0, None);
    if sg.is_empty() {
        return Ok(());
    }
    let rg = aux::select_unselect_group(duel, sg, 3, 3, &|duel, sg, _, _| spcheck(duel, sg), tp, HINTMSG_SPSUMMON)?;
    duel.special_summon_full(rg, 0, tp, tp, true, false, POS_FACEUP_DEFENSE, zone)?;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_set_card(duel, &[SET_TG]));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
