// Yuki-Onna, the Absolute Zero Mayakashi
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT, REMOVED};
use engine::constants::MSG_CHAIN_DISABLED;

const ID: u32 = 2645637;

pub const LISTED_NAMES: &[u32] = &[ID];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_ZOMBIE, Some(lc), sumtype, tp)
    }
}

// Copies in the Extra Deck, Link Summoned from Zombies (Bone Mouse), with Monster Reborn and Zombies in the Graveyard to Special Summon from there,
// and the opponent's Ash Blossom and Yuki-Musume (banished) to chain.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (21239280, MZONE),
    (21239280, MZONE),
    (83764718, HAND),
    (83764718, HAND),
    (21239280, GRAVE),
    (21239280, GRAVE),
    (72700231, REMOVED | OPPONENT),
    (14558127, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_CHAIN_DISABLED)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_code(duel, ID), LOCATION_ONFIELD);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_CHAIN_SOLVING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, discon);
    e1.set_operation(duel, |duel, a| {
        duel.negate_effect(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 2, ID);
    e2.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        a.eg.is_some_and(|eg| eg.is_exists(duel, |tc, duel| tc.is_summon_location(duel, LOCATION_GRAVE), 1, Some(c.into())))
    });
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_condition(duel, atkchaincon);
    c.register_effect(duel, e3);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    if a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) {
        return false;
    }
    let loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    a.rp == 1 - a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel)) && loc == LOCATION_REMOVED
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.has_non_zero_attack(duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(!c.has_flag_effect(duel, ID)
            && duel.is_existing_target(CardId::has_non_zero_attack, tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(c.into())));
    }
    c.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD | RESET_CHAIN, 0, 1);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(
        tp,
        CardId::has_non_zero_attack,
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        1,
        1,
        Some(c.into()),
    )?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, 0, 0);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) && tc.get_attack(duel) > 0 {
        let c = e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, 0);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        if !tc.is_attack(duel, &[0]) {
            return Ok(());
        }
        tc.negate_effects(duel, c, 0, false);
    }
    Ok(())
}

fn atkchaincon(duel: &Duel, a: &Args) -> bool {
    let loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    loc == LOCATION_GRAVE && a.re.is_some_and(|re| re.is_monster_effect(duel))
}
