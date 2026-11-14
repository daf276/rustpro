// Gouki Destroy Ogre
use super::c22510667::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 88406570;

pub const LISTED_SERIES: &[u16] = &[SET_GOUKI];
// Link Summoned from Gouki Guts and Gouki Machine Suprex, with Gouki monsters in your Graveyard and monsters in the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (7540107, MZONE),
    (29099860, MZONE),
    (7540107, MZONE),
    (7540107, GRAVE),
    (29099860, GRAVE),
    (7540107, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure<2, 0>>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, |duel, e, c| e.get_handler(duel).get_linked_group(duel).is_contains(c));
    e1.set_value(duel, Value::Reason(|_, _, _, r, _| r & REASON_BATTLE != 0));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn spfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8, zone: u32) -> bool {
    c.is_set_card(duel, &[SET_GOUKI])
        && !c.is_link_monster(duel)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, zone)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let zone = e.get_handler(duel).get_linked_zone(duel);
        return Ok(zone != 0
            && duel.get_location_count_full(1 - tp, LOCATION_MZONE, 1 - tp, LOCATION_REASON_TOFIELD, 0xff) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter1(c, duel, e, 1 - tp), 1 - tp, LOCATION_GRAVE, 0, 1, None)
            && duel.is_existing_matching_card(|c, duel| spfilter2(c, duel, e, tp, zone), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, PLAYER_ALL, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mut ft = duel.get_location_count_full(1 - tp, LOCATION_MZONE, 1 - tp, LOCATION_REASON_TOFIELD, 0xff);
    if ft > 0 {
        if duel.is_player_affected_by_effect(1 - tp, CARD_BLUEEYES_SPIRIT).is_some() {
            ft = 1;
        }
        if ft > 1 {
            ft = 2;
        }
        duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_SPSUMMON);
        let filter = aux::necro_valley_filter(|c, duel| spfilter1(c, duel, e, 1 - tp));
        let g1 = duel.select_matching_card(1 - tp, filter, 1 - tp, LOCATION_GRAVE, 0, 1, ft as u8, None)?;
        if !g1.is_empty() {
            let ct = duel.special_summon(g1, 0, 1 - tp, 1 - tp, false, false, POS_FACEUP)?;
            let zone = c.get_linked_zone_full(duel, tp);
            let mut ct = duel
                .get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, zone)
                .min(ct);
            if zone != 0 && ct > 0 && c.is_relate_to_effect(duel, e) {
                if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
                    ct = 1;
                }
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
                let filter = aux::necro_valley_filter(|c, duel| spfilter2(c, duel, e, tp, zone));
                let g2 = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE, 0, 1, ct as u8, None)?;
                if !g2.is_empty() {
                    duel.break_effect()?;
                    duel.special_summon_full(g2, 0, tp, tp, false, false, POS_FACEUP, zone)?;
                }
            }
        }
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_set_card(duel, &[SET_GOUKI]));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
