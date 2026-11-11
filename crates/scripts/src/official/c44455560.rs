// Fire King Courtier Ulcanix
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, OPPONENT};

const ID: u32 = 44455560;

pub const LISTED_NAMES: &[u32] = &[ID, 23015896];
// Two in hand with Legendary Fire King Ponix to destroy, Sacred Fire King Garunix in the Deck to search, High Avatar Garunix in the Deck to summon, and Raigeki on the other side.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (90681088, HAND),
    (66431519, DECK),
    (66431519, DECK),
    (23015896, DECK),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_LVCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_reason(duel, REASON_DESTROY));
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_FIRE) && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_FIRE) && c.is_race(duel, RACES_BEAST_BWARRIOR_WINGB) && c.is_able_to_hand(duel) && !c.is_code(duel, ID)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    !c.is_public(duel) || c.is_monster(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(desfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, Some(c.into()))
                && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None),
        );
    }
    let dg = duel.get_matching_group(desfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, Some(c.into()));
    let destroyed = if duel.is_existing_matching_card(filter, tp, LOCATION_HAND, 0, 1, None) {
        None
    } else {
        Some(dg)
    };
    duel.set_operation_info(0, CATEGORY_DESTROY, destroyed, 1, tp, (LOCATION_HAND | LOCATION_MZONE) as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let exc = c.is_relate_to_effect(duel, e).then(|| c.into());
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let desg = duel.select_matching_card(tp, desfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, exc)?;
    if desg.is_empty() || duel.destroy(desg, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    if duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, sc)?;
    duel.shuffle_hand(tp)?;
    let lvl = sc.get_level(duel);
    if c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && c.has_level(duel)
        && !c.is_level(duel, &[lvl])
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.break_effect()?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, lvl as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, 23015896) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP_DEFENSE)
}
