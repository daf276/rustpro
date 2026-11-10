// Metamorphosis
use crate::prelude::*;
use crate::testing::{EXTRA, HAND, MZONE};

const ID: u32 = 46411259;
const TRIBUTE_LEVEL: u8 = 0;

// In hand, with Dark Paladin (Level 8 Fusion) in the Extra Deck and Level 8 Normal Monsters to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (98502113, EXTRA), (47484352, MZONE), (10949074, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_label(duel, 0);
    e1.set_cost(duel, |duel, a, _| {
        a.e.set_label(duel, 100);
        Ok(true)
    });
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, lv: u32, e: EffectId, tp: u8, mc: Option<CardId>) -> bool {
    c.is_fusion_monster(duel)
        && c.is_level(duel, &[lv])
        && duel.get_location_count_from_ex(tp, tp, mc.map(Into::into), c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn costfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let lv = c.get_level(duel);
    lv > 0 && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, lv, e, tp, Some(c)), tp, LOCATION_EXTRA, 0, 1, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        if e.get_label(duel) != 100 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        return Ok(duel.check_release_group_cost(tp, |c, duel| costfilter(c, duel, e, tp), (1, 1), false, None, None));
    }
    let rg = duel.select_release_group_cost(tp, |c, duel| costfilter(c, duel, e, tp), (1, 1), false, None, None)?;
    if let Some(tribute) = rg.get_first() {
        e.set_chain_data(duel, TRIBUTE_LEVEL, i64::from(tribute.get_level(duel)));
    }
    duel.release(rg, REASON_COST)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let lv = e.get_chain_data(duel, TRIBUTE_LEVEL).unwrap_or(0) as u32;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, lv, e, tp, None), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
