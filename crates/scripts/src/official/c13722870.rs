// Dark Flare Knight
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 13722870;
pub const LISTED_NAMES: &[u32] = &[CARD_DARK_MAGICIAN, CARD_FLAME_SWORDSMAN, 49217579];
pub const MATERIAL_SETCODE: &[u16] = &[SET_DARK_MAGICIAN];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(CARD_DARK_MAGICIAN), FusionMaterial::Code(CARD_FLAME_SWORDSMAN)];
}

// In the Extra Deck and on the field, Fusion Summoned by Polymerization with Dark Magician and Flame Swordsman, with Mirage
// Knight in the Deck to summon and the opponent's strong attackers to destroy it in battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (CARD_DARK_MAGICIAN, HAND),
    (CARD_FLAME_SWORDSMAN, MZONE),
    (49217579, DECK),
    (49217579, DECK),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(49217579),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_BATTLE_DESTROYED);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE));
    e2.set_target(
        duel,
        stock::operation_info_target::<CATEGORY_SPECIAL_SUMMON, { LOCATION_HAND | LOCATION_DECK }>,
    );
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, 49217579) && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, e, tp),
        tp,
        LOCATION_HAND | LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if let Some(sc) = g.get_first()
        && duel.special_summon(sc, 0, tp, tp, true, false, POS_FACEUP)? > 0
    {
        sc.complete_procedure(duel);
    }
    Ok(())
}
