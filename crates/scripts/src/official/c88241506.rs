// Maiden with Eyes of Blue
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 88241506;

pub const LISTED_NAMES: &[u32] = &[CARD_BLUEEYES_W_DRAGON];

// Copies in the hand and on the field for the opponent's attackers (Elfnote Lucina, Kagemusha of the Blue Flame) and Mage Power to
// target, with Blue-Eyes White Dragon in the Deck, hand and Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (CARD_BLUEEYES_W_DRAGON, DECK),
    (CARD_BLUEEYES_W_DRAGON, DECK),
    (CARD_BLUEEYES_W_DRAGON, HAND),
    (CARD_BLUEEYES_W_DRAGON, GRAVE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (15401633, HAND | OPPONENT),
    (15401633, HAND | OPPONENT),
    (15401633, HAND | OPPONENT),
    (83746708, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, natg);
    e1.set_operation(duel, naop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_BECOME_TARGET);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.eg.is_some_and(|eg| eg.is_contains(a.e.get_handler(duel))));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CARD_BLUEEYES_W_DRAGON) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn natg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_POSITION, Some(c.into()), 1, 0, 0);
    duel.set_possible_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        None,
        1,
        a.tp,
        (LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE) as i32,
    );
    Ok(false)
}

fn naop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if duel.negate_attack()? && c.is_relate_to_effect(duel, e) {
        duel.change_position_full(
            c,
            [POS_FACEUP_DEFENSE, POS_FACEUP_DEFENSE, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK],
            false,
        )?;
        if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
            return Ok(());
        }
        let g = duel.get_matching_group(
            aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
            tp,
            LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
            0,
            None,
        );
        if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.break_effect()?;
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g1 = g.select(duel, tp, 1, 1, None)?;
            duel.special_summon(g1, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(
        duel,
        a,
        chk,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(
        duel,
        a,
        aux::necro_valley_filter(|c, duel| spfilter(c, duel, a.e, a.tp)),
        LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
        POS_FACEUP,
    )
}
