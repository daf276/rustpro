// Superheavy Samurai Battleball
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 9402966;

// Copies on the field, with Swordsman of Landstar (Level 3) on the opponent's side to send with it for Superheavy Samurai Swordmaster
// Musashi (Level 5) in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (75988594, EXTRA),
    (3573512, MZONE | OPPONENT),
    (3573512, MZONE | OPPONENT),
    (3573512, MZONE | OPPONENT),
    (3573512, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, sccon);
    e1.set_target(duel, sctg);
    e1.set_operation(duel, scop);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) || !c.is_set_card(duel, &[SET_SUPERHEAVY_SAMURAI])
}

fn sccon(duel: &Duel, a: &Args) -> bool {
    !duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None)
        && !duel.is_existing_matching_card(CardId::is_spell_trap, a.tp, LOCATION_GRAVE, 0, 1, None)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: u32) -> bool {
    c.is_faceup(duel)
        && c.get_level(duel) > 0
        && duel.is_existing_matching_card(
            |sc, duel| scfilter(sc, duel, e, tp, lv + c.get_original_level(duel)),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            None,
        )
}

fn scfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: u32) -> bool {
    c.is_set_card(duel, &[SET_SUPERHEAVY_SAMURAI])
        && c.is_level(duel, &[lv])
        && c.is_type(duel, TYPE_SYNCHRO)
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_SYNCHRO, tp, false, false)
}

fn sctg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let c = e.get_handler(duel);
    let lv = c.get_original_level(duel);
    if chk == 0 {
        let pg = aux::get_must_be_material_group(duel, tp, Some(CardSet::default()), tp, None, None, REASON_SYNCHRO);
        return Ok(pg.is_empty() && duel.is_existing_target(|c, duel| filter(c, duel, e, tp, lv), tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let mut g = duel
        .select_target(tp, |c, duel| filter(c, duel, e, tp, lv), tp, 0, LOCATION_MZONE, 1, 1, None)?
        .unwrap_or_default();
    g.insert(c);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(g), 2, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn scop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let tc = duel.get_first_target();
    let pg = aux::get_must_be_material_group(duel, tp, Some(CardSet::default()), tp, None, None, REASON_SYNCHRO);
    let Some(tc) = tc.filter(|tc| c.is_relate_to_effect(duel, e) && tc.is_relate_to_effect(duel, e) && pg.is_empty()) else {
        return Ok(());
    };
    let g = CardSet::from_iter([c, tc]);
    if duel.send_to_grave(g, REASON_EFFECT)? == 2
        && c.get_level(duel) > 0
        && c.is_location(duel, LOCATION_GRAVE)
        && tc.get_level(duel) > 0
        && tc.is_location(duel, LOCATION_GRAVE)
    {
        let lv = c.get_level(duel) + tc.get_level(duel);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = duel.select_matching_card(tp, |c, duel| scfilter(c, duel, e, tp, lv), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        if let Some(sc) = sg.get_first() {
            duel.break_effect()?;
            duel.special_summon(sc, SUMMON_TYPE_SYNCHRO, tp, tp, false, false, POS_FACEUP)?;
            sc.complete_procedure(duel);
        }
    }
    Ok(())
}
