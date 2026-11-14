// Meteonis Drytron
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 22398665;

pub const LISTED_SERIES: &[u16] = &[SET_DRYTRON];
// Drytron Meteonis Draconids to summon from Drytron monsters, which the Graveyard effect also needs face-up.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (69815951, HAND),
    (69815951, GRAVE),
    (97148796, HAND),
    (96026108, HAND),
    (60037599, MZONE),
    (22420202, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(69815951), Check::ResolvesEffect(aux::stringid(ID, 1))];

struct Params;

impl RitualParams for Params {
    const DESC: u64 = aux::stringid(ID, 0);
    const LOCATION: Option<u32> = Some(LOCATION_HAND | LOCATION_GRAVE);
    const REQUIREMENTFUNC: Option<fn(CardId, &Duel, CardId) -> u32> = Some(|c, duel, _| c.get_attack(duel) as u32);

    fn filter(c: CardId, duel: &Duel) -> bool {
        c.get_attack(duel) > 0 && c.is_ritual_monster(duel)
    }

    fn lv(c: CardId, duel: &Duel) -> u32 {
        c.get_attack(duel) as u32
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_race(duel, RACE_MACHINE) && c.get_attack(duel) > 0
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Ritual::add_proc_greater::<Params>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_DRYTRON]) && c.get_attack(duel) >= 1000
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && atkfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel) && duel.is_existing_target(atkfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, atkfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_faceup(duel)
        && tc.update_attack(duel, -1000, Some(RESETS_STANDARD_PHASE_END | RESET_OPPO_TURN), Some(c), None) == -1000
        && c.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
