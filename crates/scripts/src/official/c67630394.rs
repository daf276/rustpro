// Full Armored Dark Knight Lancer
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 67630394;

pub const LISTED_SERIES: &[u16] = &[SET_XYZ];

const XYZ_IMPORT: u32 = 14602126;
const AXE_OF_DESPAIR: u32 = 40619825;
const GAUNTLET_LAUNCHER: u32 = 15561463;

pub const LISTED_NAMES: &[u32] = &[ID];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 3;
    const ALTERF: Option<AlterFilter> = Some(xyzfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        duel.get_flag_effect(tp, ID) == 0
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        Ok(true)
    }
}

// On the field with Axe of Despair in the hand to equip, Xyz Import in the Graveyard to add, and Sangan on the opponent's
// side to attach; and a second copy in the Extra Deck to be Xyz Summoned on Gauntlet Launcher.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (GAUNTLET_LAUNCHER, MZONE),
    (AXE_OF_DESPAIR, HAND),
    (AXE_OF_DESPAIR, HAND),
    (XYZ_IMPORT, GRAVE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Card(|duel, _, c| ((c.get_overlay_count(duel) + c.get_equip_count(duel)) * 300) as i32),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_EQUIP);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, atchcon);
    e3.set_target(duel, atchtg);
    e3.set_operation(duel, atchop);
    c.register_effect(duel, e3);
}

fn xyzfilter(c: CardId, duel: &Duel, tp: u8, xyzc: CardId) -> bool {
    c.is_faceup(duel) && c.is_rank(duel, &[5, 6]) && c.is_type_full(duel, TYPE_XYZ, Some(xyzc), SUMMON_TYPE_XYZ as u64, tp)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_XYZ]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn atchconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let target = c.get_equip_target(duel);
    unported!(duel, target.is_none());
    target.is_some_and(|target| target.is_controler(duel, tp))
}

fn atchcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| atchconfilter(c, duel, tp), 1, None)
}

fn atchfilter(c: CardId, duel: &Duel) -> bool {
    !c.is_type(duel, TYPE_TOKEN) && c.is_able_to_change_controler(duel)
}

fn atchtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_type(duel, TYPE_XYZ) && duel.is_existing_matching_card(atchfilter, a.tp, 0, LOCATION_MZONE, 1, Some(c.into())));
    }
    Ok(false)
}

fn atchop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTACH);
    let g = duel.select_matching_card(tp, atchfilter, tp, 0, LOCATION_MZONE, 1, 1, Some(c.into()))?;
    if let Some(tc) = g.get_first()
        && !tc.is_immune_to_effect(duel, a.e)
    {
        duel.overlay_full(c, tc, true)?;
    }
    Ok(())
}
