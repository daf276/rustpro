// Holactie the Creator of Light
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 10000040;

pub const LISTED_NAMES: &[u32] = &[CARD_SLIFER, CARD_OBELISK, CARD_RA];

// In the hand with the three Egyptian Gods (Slifer, Obelisk and Ra) on the field to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, DECK),
    (ID, DECK),
    (CARD_SLIFER, MZONE),
    (CARD_OBELISK, MZONE),
    (CARD_RA, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.add_must_be_special_summoned(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, selfspcon);
    e1.set_target(duel, selfsptg);
    e1.set_operation(duel, selfspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e2.set_code(duel, EFFECT_CANNOT_DISABLE_SPSUMMON);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_operation(duel, |duel, a| {
        let player = a.e.get_handler(duel).get_summon_player(duel);
        duel.win(player, WIN_REASON_CREATORGOD);
        Ok(())
    });
    c.register_effect(duel, e3);
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    (
        duel.get_mzone_count(tp, Some(sg)) > 0 && sg.get_class_count(duel, |c, duel| i64::from(c.get_original_code_rule(duel))) == 3,
        false,
    )
}

fn gods(duel: &Duel, tp: u8) -> CardSet {
    duel.get_release_group(tp).filter(
        duel,
        |c, duel| {
            [CARD_SLIFER, CARD_OBELISK, CARD_RA]
                .iter()
                .any(|&code| c.is_original_code_rule(duel, code))
        },
        None,
    )
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let g = gods(duel, tp);
    g.len() >= 3 && aux::select_unselect_group_check(duel, g, 3, 3, &|duel, sg, _, _| rescon(duel, sg, tp))
}

fn selfsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = gods(duel, tp);
    let rescon = |duel: &Duel, sg, _, _| rescon(duel, sg, tp);
    let sg = aux::select_unselect_group_full(duel, g, 3, 3, &rescon, tp, HINTMSG_RELEASE, None, true)?;
    if sg.len() == 3 {
        e.set_label_object_group(duel, Some(sg));
        return Ok(true);
    }
    Ok(false)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.release(g, REASON_COST)?;
    }
    Ok(())
}
