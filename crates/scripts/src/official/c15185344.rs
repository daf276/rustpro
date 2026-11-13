// Swickelavee the Brilliantly Crowned Heavy Cavalry
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 15185344;
const COUNTER_CROWN: u16 = 0x21c;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_CROWN];

// In the Extra Deck, Xyz Summoned with Level 3 Beasts (Nekogal #1, Burglar, Dark Gray and Armored Rat), with the opponent's
// Sanganss to destroy by battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (1761063, MZONE),
    (6297941, MZONE),
    (9159938, MZONE),
    (16246527, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
    const ALTERF: Option<AlterFilter> = Some(altxyzmat);
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        duel.get_flag_effect(tp, ID) == 0
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        Ok(duel
            .register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, EFFECT_FLAG_OATH, 1)
            .is_some())
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.enable_counter_permit(duel, COUNTER_CROWN);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_CANNOT_BE_XYZ_MATERIAL);
    e0.set_value(duel, 1);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_COUNTER + CATEGORY_ATKCHANGE + CATEGORY_TOEXTRA + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLE_DESTROYING);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_relate_to_battle(duel));
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn altxyzmat(c: CardId, duel: &Duel, tp: u8, xyz: CardId) -> bool {
    c.is_race_full(duel, RACE_BEAST, Some(xyz), SUMMON_TYPE_XYZ as u64, tp) && c.is_attack_below(duel, 2000) && c.is_faceup(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    let c = e.get_handler(duel);
    let crown_counters = c.get_counter(duel, COUNTER_CROWN);
    duel.set_operation_info(0, CATEGORY_COUNTER, Some(c.into()), 1, tp, COUNTER_CROWN.into());
    if crown_counters < 3 {
        duel.set_operation_info(
            0,
            CATEGORY_ATKCHANGE,
            Some(c.into()),
            1,
            tp,
            if crown_counters == 1 { 400 } else { 600 },
        );
    }
    if crown_counters == 2 {
        duel.set_operation_info(0, CATEGORY_TOEXTRA, Some(c.into()), 1, tp, 0);
        duel.set_operation_info(0, CATEGORY_DRAW, None, 1, tp, 3);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && c.add_counter(duel, COUNTER_CROWN, 1) {
        let crown_counters = c.get_counter(duel, COUNTER_CROWN);
        if crown_counters == 1 {
            c.update_attack(duel, 400, None, None, None);
        } else if crown_counters == 2 {
            c.update_attack(duel, 600, None, None, None);
        } else if crown_counters == 3
            && duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)? > 0
            && c.is_location(duel, LOCATION_EXTRA)
        {
            duel.draw(tp, 3, REASON_EFFECT)?;
        }
    }
    Ok(())
}
