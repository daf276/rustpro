// The Shallow Grave
use crate::prelude::*;
use crate::testing::{Check, GRAVE, OPPONENT, SZONE};

const ID: u32 = 43434803;

// Set, with Ojama Green in both Graveyards to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (12482652, GRAVE), (12482652, GRAVE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, tp, 0xff)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(|c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None)
                && duel.get_location_count(tp, LOCATION_MZONE) > 0
                && duel.is_existing_target(|c, duel| filter(c, duel, e, 1 - tp), 1 - tp, LOCATION_GRAVE, 0, 1, None)
                && duel.get_location_count_full(1 - tp, LOCATION_MZONE, 1 - tp, LOCATION_REASON_TOFIELD, 0xff) > 0,
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sg = duel.select_target(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_SPSUMMON);
    let og = duel.select_target(1 - tp, |c, duel| filter(c, duel, e, 1 - tp), 1 - tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    let (Some(sc), Some(oc)) = (sg.and_then(|g| g.get_first()), og.and_then(|g| g.get_first())) else {
        return Ok(false);
    };
    let g = CardSet::from_iter([sc, oc]);
    let owner = g.get_first().map_or(0, |c| i32::from(c.get_owner(duel)));
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g), 2, PLAYER_ALL, owner);
    e.set_label_object_group(duel, Some(sc.into()));
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(sc) = e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    let g: Option<CardSet> = duel.get_chain_info(0, CHAININFO_TARGET_CARDS);
    let Some(oc) = g.and_then(|g| g.into_iter().find(|&c| c != sc)) else {
        return Ok(());
    };
    if sc.is_relate_to_effect(duel, e) {
        duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEDOWN_DEFENSE)?;
    }
    if oc.is_relate_to_effect(duel, e) {
        duel.special_summon_step(oc, 0, 1 - tp, 1 - tp, false, false, POS_FACEDOWN_DEFENSE)?;
    }
    duel.special_summon_complete()?;
    Ok(())
}
