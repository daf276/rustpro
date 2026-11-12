// Multiplying Kuriboh!
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 14965712;

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_MAGICIAN];

// In hand to Special Summon and on the field for the second effect, with Dark Magician in the Deck and Graveyard to add or Special
// Summon, and Elfnote Lucina and Sangan of the opponent to attack and activate monster effects.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (CARD_DARK_MAGICIAN, DECK),
    (CARD_DARK_MAGICIAN, GRAVE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1a.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1a.set_code(duel, EVENT_CHAINING);
    e1a.set_range(duel, LOCATION_HAND);
    e1a.set_count_limit(duel, 1, ID);
    e1a.set_condition(duel, |duel, a| {
        a.ep == 1 - a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel))
    });
    e1a.set_target(duel, stock::special_summon_self_target);
    e1a.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1b.set_condition(duel, attackcon);
    c.register_effect(duel, e1b);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(
        duel,
        CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON + CATEGORY_ATKCHANGE,
    );
    e2a.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2a.set_code(duel, EVENT_CHAINING);
    e2a.set_range(duel, LOCATION_MZONE);
    e2a.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e2a.set_condition(duel, |duel, a| {
        let location: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
        a.ep == 1 - a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel)) && location & LOCATION_MZONE > 0
    });
    e2a.set_target(duel, thsptg);
    e2a.set_operation(duel, thspop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e2b.set_condition(duel, attackcon);
    c.register_effect(duel, e2b);
}

fn attackcon(duel: &Duel, a: &Args) -> bool {
    duel.get_attacker().is_some_and(|attacker| attacker.is_controler(duel, 1 - a.tp))
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mmz_chk: bool) -> bool {
    (c.is_code(duel, CARD_DARK_MAGICIAN) || (c.is_attack(duel, &[300]) && c.is_defense(duel, &[200])))
        && (c.is_able_to_hand(duel) || (mmz_chk && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn thsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
        return Ok(duel.is_existing_matching_card(
            |c, duel| thspfilter(c, duel, e, tp, mmz_chk),
            tp,
            LOCATION_DECK | LOCATION_GRAVE,
            0,
            1,
            None,
        ));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn thspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let filter = aux::necro_valley_filter(|c, duel| thspfilter(c, duel, e, tp, mmz_chk));
    let Some(sc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    let res = aux::to_hand_or_else_result(
        duel,
        sc,
        tp,
        |_, duel| mmz_chk && sc.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        |duel| duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP).map(|count| count > 0),
        aux::stringid(ID, 3),
    )?;
    if !res {
        return Ok(());
    }
    if sc.is_location(duel, LOCATION_HAND) {
        duel.shuffle_hand(tp)?;
    }
    let (opp_card, relation_chk) = if !duel.is_damage_step() {
        let opp_card = a.re.map(|re| re.get_handler(duel));
        let relation_chk =
            a.re.zip(opp_card)
                .is_some_and(|(re, opp_card)| opp_card.is_relate_to_effect(duel, re));
        (opp_card, relation_chk)
    } else {
        let opp_card = duel.get_attacker();
        (opp_card, opp_card.is_some_and(|opp_card| opp_card.is_relate_to_battle(duel)))
    };
    unported!(duel, opp_card.is_none());
    let Some(opp_card) = opp_card else { return Ok(()) };
    if relation_chk
        && opp_card.is_faceup(duel)
        && opp_card.is_controler(duel, 1 - tp)
        && !opp_card.is_attack(duel, &[0])
        && duel.select_yes_no(tp, aux::stringid(ID, 4))?
    {
        duel.break_effect()?;
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, 0);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        opp_card.register_effect(duel, e1);
    }
    Ok(())
}
