// Scarlight Red Dragon Archfiend
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 80666118;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub const LISTED_NAMES: &[u32] = &[CARD_RED_DRAGON_ARCHFIEND];
// In the Extra Deck, Synchro Summoned with Ash Blossom & Joyous Spring and Shapesnatch, and on the field with a weaker
// Special Summoned Effect Monster of each player.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (14558127, MZONE),
    (4035199, MZONE),
    (ID, MZONE),
    (14558127, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn name_change(duel: &mut Duel, c: CardId) -> EffectId {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e1.set_value(duel, CARD_RED_DRAGON_ARCHFIEND as i32);
    e1
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = name_change(duel, c);
    e1.set_description(duel, aux::stringid(ID, 0));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, atk: i32) -> bool {
    c.is_faceup(duel)
        && c.is_attack_below(duel, atk)
        && c.get_summon_type(duel) & SUMMON_TYPE_SPECIAL == SUMMON_TYPE_SPECIAL
        && c.is_type(duel, TYPE_EFFECT)
}

fn weaker(duel: &Duel, c: CardId, tp: u8) -> CardSet {
    let atk = c.get_attack(duel);
    duel.get_matching_group(|c, duel| filter(c, duel, atk), tp, LOCATION_MZONE, LOCATION_MZONE, Some(c.into()))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if chk == 0 {
        let atk = c.get_attack(duel);
        return Ok(duel.is_existing_matching_card(
            |c, duel| filter(c, duel, atk),
            tp,
            LOCATION_MZONE,
            LOCATION_MZONE,
            1,
            Some(c.into()),
        ));
    }
    let g = weaker(duel, c, tp);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, g.len() as i32 * 500);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if !c.is_relate_to_effect(duel, a.e) || c.is_facedown(duel) {
        return Ok(());
    }
    let g = weaker(duel, c, tp);
    let ct = duel.destroy(g, REASON_EFFECT)?;
    if ct > 0 {
        duel.break_effect()?;
        duel.damage(1 - tp, ct as u32 * 500, REASON_EFFECT)?;
    }
    Ok(())
}
