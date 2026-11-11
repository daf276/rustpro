// Chaos Mystic Box
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 75983808;

pub const LISTED_NAMES: &[u32] = &[CARD_LIGHT_AND_DARKNESS_RITUAL];

// In the hand and Set, against Mage Power and Mind Control targeting our monsters and Mystical Space Typhoons destroying the Set one, with Black Luster
// Soldier - Soldier of Light and Darkness in the hand and Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (58932615, MZONE),
    (58932615, MZONE),
    (70405001, HAND),
    (70405001, DECK),
    (70405001, DECK),
    (83746708, HAND | OPPONENT),
    (37520316, HAND | OPPONENT),
    (5318639, HAND | OPPONENT),
    (5318639, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(70405001),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_condition(duel, thdescon);
    e1.set_target(duel, thdestg);
    e1.set_operation(duel, thdesop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_condition(duel, |duel, a| {
        a.rp == 1 - a.tp && a.e.get_handler(duel).is_previous_controler(duel, a.tp)
    });
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn target_cards(duel: &Duel, ev: u32) -> Option<CardSet> {
    duel.get_chain_info(ev as u8, CHAININFO_TARGET_CARDS)
}

fn thdescon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    if !(a.rp == 1 - tp && a.re.is_some_and(|re| re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET))) {
        return false;
    }
    target_cards(duel, a.ev).is_some_and(|tg| {
        tg.is_exists(
            duel,
            |c, duel| c.is_controler(duel, tp) && c.is_on_field(duel) && c.is_able_to_hand(duel),
            1,
            Some(a.e.get_handler(duel).into()),
        )
    })
}

fn thdestg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, Some(a.e.get_handler(duel).into())));
    }
    let g = duel.get_field_group(tp, LOCATION_ONFIELD, LOCATION_ONFIELD);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_ONFIELD as i32);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn handspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, code: u32) -> bool {
    c.is_monster(duel)
        && c.lists_code(duel, CARD_LIGHT_AND_DARKNESS_RITUAL)
        && !c.is_code(duel, code)
        && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn thdesop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let re = a.re;
    let tg = target_cards(duel, a.ev).unwrap_or_default().filter(
        duel,
        |c, duel| re.is_some_and(|re| c.is_relate_to_effect(duel, re)) && c.is_controler(duel, tp),
        None,
    );
    if tg.is_empty() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let Some(sc) = tg.filter_select(duel, tp, CardId::is_able_to_hand, 1, 1, None)?.get_first() else {
        return Ok(());
    };
    if sc.is_faceup(duel) {
        duel.hint_selection(sc);
    } else {
        duel.confirm_cards(1 - tp, sc)?;
    }
    if duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? > 0 && sc.is_location(duel, LOCATION_HAND) {
        duel.shuffle_hand(sc.get_controler(duel))?;
        let c = e.get_handler(duel);
        let exc = c.is_relate_to_effect(duel, e).then(|| c.into());
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, exc)?;
        if g.is_empty() {
            return Ok(());
        }
        duel.hint_selection(g);
        let code = sc.get_code(duel);
        if duel.destroy(g, REASON_EFFECT)? > 0
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| handspfilter(c, duel, e, tp, code), tp, LOCATION_HAND, 0, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let sg = duel.select_matching_card(tp, |c, duel| handspfilter(c, duel, e, tp, code), tp, LOCATION_HAND, 0, 1, 1, None)?;
            if !sg.is_empty() {
                duel.break_effect()?;
                duel.special_summon(sg, 0, tp, tp, true, false, POS_FACEUP)?;
            }
        }
    }
    Ok(())
}

fn ritualspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_ritual_monster(duel)
        && c.lists_code(duel, CARD_LIGHT_AND_DARKNESS_RITUAL)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, true)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(
        duel,
        a,
        chk,
        |c, duel| ritualspfilter(c, duel, a.e, a.tp),
        LOCATION_HAND | LOCATION_DECK,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| ritualspfilter(c, duel, e, tp),
        tp,
        LOCATION_HAND | LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, true, POS_FACEUP)?;
    }
    Ok(())
}
