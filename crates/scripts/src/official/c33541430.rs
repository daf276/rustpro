// Flower Cardian Moonflowerviewing
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 33541430;

pub const LISTED_SERIES: &[u16] = &[SET_FLOWER_CARDIAN];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 2;
    const MAX2: usize = 2;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Synchro Summoned with Cherry Inmato (Level 2 Tuner) and two Mushroom Man (Level 2 Normal Monsters), with Flower Cardian Pine
// in the Deck to draw and Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (525110, MZONE),
    (525110, MZONE),
    (14181608, MZONE),
    (14181608, MZONE),
    (14181608, MZONE),
    (81752019, DECK),
    (81752019, DECK),
    (81752019, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DRAW + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_count_limit(duel, 1, 0);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_cost(duel, drcost);
    e2.set_target(duel, stock::draw_target::<1>);
    e2.set_operation(duel, drop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_SYNCHRO_MATERIAL_CUSTOM);
    e3.set_synchro_material_operation(duel, synop);
    c.register_effect(duel, e3);
}

fn drcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_code(duel, EFFECT_SKIP_DP);
    e1.set_reset(duel, RESET_PHASE | PHASE_DRAW | RESET_SELF_TURN);
    duel.register_effect(e1, a.tp);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if duel.draw(p, d, REASON_EFFECT)? != 0 {
        let Some(tc) = duel.get_operated_group().get_first() else {
            return Ok(());
        };
        duel.confirm_cards(1 - tp, tc)?;
        duel.break_effect()?;
        if tc.is_monster(duel) && tc.is_set_card(duel, &[SET_FLOWER_CARDIAN]) {
            if duel.get_location_count(tp, LOCATION_MZONE) > 0
                && duel.select_yes_no(tp, aux::stringid(ID, 1))?
                && duel.special_summon_step(tc, 0, tp, tp, true, false, POS_FACEUP)?
            {
                let e1 = duel.create_effect(e.get_handler(duel));
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_code(duel, EFFECT_DIRECT_ATTACK);
                e1.set_value(duel, 1);
                e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
                tc.register_effect_full(duel, e1, true);
            }
            duel.special_summon_complete()?;
        }
        duel.shuffle_hand(tp)?;
    }
    Ok(())
}

pub fn synop(duel: &Duel, _: EffectId, _: CardSet, _: CardSet, sg: CardSet, lv: u32, sc: CardId, _: u8) -> (bool, bool) {
    let res = sg.check_with_sum_equal(duel, |c, duel| c.get_synchro_level(duel, sc), lv, sg.len(), sg.len())
        || sg.check_with_sum_equal(duel, |_, _| 2, lv, sg.len(), sg.len());
    (res, true)
}
