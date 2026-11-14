// Blackwing - Nothung the Starlight
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 95040215;

pub const LISTED_SERIES: &[u16] = &[SET_BLACKWING];

// A Synchro Summon of any Tuner and 1+ non-Tuners; the Assault Blackwing monsters reuse it with their own maximum.
pub struct Procedure<const MAX: usize>;

impl<const MAX: usize> SynchroProcedure for Procedure<MAX> {
    const MIN2: usize = 1;
    const MAX2: usize = MAX;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned from Blizzard (a Level 2 Tuner) and Zephyros (Level 4), with monsters of the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (22835145, MZONE),
    (22835145, MZONE),
    (14785765, MZONE),
    (14785765, MZONE),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure<99>>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE + CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_EXTRA_SUMMON_COUNT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_BLACKWING]));
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - tp);
    duel.set_target_param(800);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 800);
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, None, 1, 1 - tp, 800);
    duel.set_operation_info(0, CATEGORY_DEFCHANGE, None, 1, 1 - tp, 800);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if duel.damage(p, d, REASON_EFFECT)? > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
        let tc = duel
            .select_matching_card(tp, |c, duel| c.is_faceup(duel), tp, 0, LOCATION_MZONE, 1, 1, None)?
            .get_first();
        if let Some(tc) = tc {
            duel.break_effect()?;
            let e1 = duel.create_effect(a.e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_value(duel, -800);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
            let e2 = e1.clone_effect(duel);
            e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
            tc.register_effect(duel, e2);
        }
    }
    Ok(())
}
