// Shiranui Spectralsword
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE};

const ID: u32 = 36630403;

// In the Graveyard with Zombino (Level 4 Zombie) to banish with it for Doomkaiser Dragon (Level 6 Zombie
// Synchro Monster) in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (14575467, GRAVE),
    (14575467, GRAVE),
    (6021033, EXTRA),
    (6021033, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(6021033), Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, aux::exccon);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter1(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: u32) -> bool {
    let clv = c.get_level(duel);
    clv > 0
        && !c.is_type(duel, TYPE_TUNER)
        && c.is_race(duel, RACE_ZOMBIE)
        && c.is_able_to_remove(duel)
        && duel.is_existing_matching_card(|c, duel| filter2(c, duel, e, tp, lv + clv), tp, LOCATION_EXTRA, 0, 1, None)
}

fn filter2(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: u32) -> bool {
    c.get_level(duel) == lv
        && c.is_race(duel, RACE_ZOMBIE)
        && c.is_type(duel, TYPE_SYNCHRO)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let lv = c.get_level(duel);
    let filter = |c, duel: &Duel| filter1(c, duel, e, tp, lv);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel
            .is_player_affected_by_effect(e.get_handler_player(duel), CARD_SPIRIT_ELIMINATION)
            .is_none()
            && c.is_able_to_remove(duel)
            && duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let mut g = duel
        .select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?
        .unwrap_or_default();
    g.insert(c);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), 2, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !c.is_relate_to_effect(duel, e) || !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let lv = c.get_level(duel) + tc.get_level(duel);
    let g = CardSet::from_iter([c, tc]);
    if duel.remove(g, POS_FACEUP, REASON_EFFECT)? == 2 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = duel.select_matching_card(tp, |c, duel| filter2(c, duel, e, tp, lv), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        if !sg.is_empty() {
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
