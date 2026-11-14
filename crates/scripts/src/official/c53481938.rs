// Rokket Barrage
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 53481938;

pub const LISTED_SERIES: &[u16] = &[SET_ROKKET];

const LOCATION_HAND_DECK_GRAVE_REMOVED: u32 = LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE | LOCATION_REMOVED;

pub const LISTED_NAMES: &[u32] = &[ID];

// A copy in the hand to Special Summon and one on the field, with Rokket Synchron and Rokket Loader in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (48355999, DECK),
    (48355999, DECK),
    (25554552, DECK),
    (25554552, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, selfspcost);
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, selfspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn selfspcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_DARK) && c.is_race(duel, RACE_DRAGON) && !c.is_level(duel, &[5]) && c.is_able_to_grave_as_cost(duel)
}

fn selfspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, selfspcostfilter, LOCATION_DECK)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && c.is_attribute_except(duel, ATTRIBUTE_DARK)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    if !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return false;
    }
    let tg: Option<CardSet> = duel.get_chain_info(a.ev as u8, CHAININFO_TARGET_CARDS);
    tg.is_some_and(|tg| tg.is_contains(a.e.get_handler(duel))) && re.get_handler(duel).is_link_monster(duel) && re.is_monster_effect(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ROKKET])
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && !c.is_code(duel, ID)
        && (!c.is_location(duel, LOCATION_REMOVED) || c.is_faceup(duel))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_mzone_count(tp, Some(c.into())) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND_DECK_GRAVE_REMOVED, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND_DECK_GRAVE_REMOVED as i32);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mut ft = duel.get_mzone_count(tp, Some(c.into()));
    if c.is_relate_to_effect(duel, e) && duel.destroy(c, REASON_EFFECT)? > 0 && ft > 0 {
        let g = duel.get_matching_group(
            aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
            tp,
            LOCATION_HAND_DECK_GRAVE_REMOVED,
            0,
            None,
        );
        if g.is_empty() {
            return Ok(());
        }
        ft = ft.min(4);
        if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
            ft = 1;
        }
        let sg = aux::select_unselect_group(
            duel,
            g,
            1,
            ft as usize,
            &aux::dpcheck(|c, duel| i64::from(c.get_location(duel))),
            tp,
            HINTMSG_SPSUMMON,
        )?;
        if !sg.is_empty() {
            duel.break_effect()?;
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
