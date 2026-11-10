// Sprind the Irondash Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 1906812;
const KITT: u32 = 45484331;

pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(CARD_ALBAZ),
        FusionMaterial::Filter(|c, duel, _, _, _| {
            c.is_location(duel, LOCATION_MZONE) && c.is_type(duel, TYPE_EFFECT) && c.is_status(duel, STATUS_SPSUMMON_TURN)
        }),
    ];
}

// On the field with Raigeki on the opponent's side to destroy it, and Springans Kitt in the Deck to add or Special Summon; a
// second copy in the Extra Deck for the Fusion Summon with Fallen of Albaz and Springans Kitt.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (CARD_ALBAZ, HAND),
    (KITT, HAND),
    (KITT, DECK),
    (KITT, DECK),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, seqtg);
    e1.set_operation(duel, seqop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
}

fn seqtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0);
    }
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn seqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) || c.is_controler(duel, 1 - tp) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOZONE);
    if let Some(seq) = duel.select_disable_field(tp, 1, LOCATION_MZONE as u16, 0, 0)? {
        duel.move_sequence(c, seq.trailing_zeros() as u8)?;
        let dg = c.get_column_group(duel).filter(duel, CardId::is_faceup, Some(c.into()));
        if !dg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
            duel.break_effect()?;
            duel.destroy(dg, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, ft: i32) -> bool {
    c.is_monster(duel)
        && (c.is_set_card(duel, &[SET_SPRINGANS]) || c.is_code(duel, CARD_ALBAZ))
        && (c.is_able_to_hand(duel) || (c.is_can_be_special_summoned(duel, e, 0, tp, false, false) && ft > 0))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| thfilter(c, duel, e, tp, ft), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, |c, duel| thfilter(c, duel, e, tp, ft), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        aux::to_hand_or_else(
            duel,
            tc,
            tp,
            |tc, duel| tc.is_can_be_special_summoned(duel, e, 0, tp, false, false) && ft > 0,
            |duel| duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP).map(drop),
            aux::stringid(ID, 3),
        )?;
    }
    Ok(())
}
