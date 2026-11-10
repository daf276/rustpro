// Doomkaiser Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 6021033;
const PLAGUESPREADER: u32 = 33420078;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[PLAGUESPREADER])
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_race_full(duel, RACE_ZOMBIE, Some(sc), sumtype, tp)
    }
}

pub const MATERIAL: &[u32] = &[PLAGUESPREADER];
pub const LISTED_NAMES: &[u32] = &[PLAGUESPREADER];
// In the Extra Deck, Synchro Summoned from Plaguespreader Zombie (Level 2) and Zombino (Level 4), with Zombies in the opponent's Graveyard
// to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (PLAGUESPREADER, MZONE),
    (PLAGUESPREADER, MZONE),
    (14575467, MZONE),
    (14575467, MZONE),
    (14575467, GRAVE | OPPONENT),
    (14575467, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_LEAVE_FIELD);
    e2.set_operation(duel, desop);
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_ZOMBIE) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| filter(c, duel, e, tp), tp, 0, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| filter(c, duel, e, tp), tp, 0, LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e)
        && tc.is_race(duel, RACE_ZOMBIE)
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP_ATTACK)? != 0
    {
        if c.is_facedown(duel) || !c.is_relate_to_effect(duel, e) {
            return Ok(());
        }
        c.set_card_target(duel, tc);
        e.set_label_object_group(duel, Some(tc.into()));
        tc.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 0);
        c.register_flag_effect(duel, ID, RESET_EVENT | RESET_TURN_SET | RESET_TOFIELD, 0, 0);
    }
    Ok(())
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc =
        a.e.get_label_object(duel)
            .and_then(|e1| e1.get_label_object_group(duel))
            .and_then(|g| g.get_first());
    if let Some(tc) = tc
        && tc.get_flag_effect(duel, ID) != 0
        && a.e.get_handler(duel).get_flag_effect(duel, ID) != 0
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
