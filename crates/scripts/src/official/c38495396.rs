// Constellar Ptolemy M7
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 38495396;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(6);
    const CT: usize = 2;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 1);

    fn op(duel: &mut Duel, e: EffectId, _: u8, _: CardId) -> Script<bool> {
        e.get_handler(duel)
            .register_flag_effect(duel, ID, RESET_EVENT | (RESETS_STANDARD_PHASE_END & !RESET_TOFIELD), 0, 1);
        Ok(true)
    }
}

// In the Extra Deck, Xyz Summoned with two Level 6 Frostosaurus, or on Constellar Ptolemy Ω7.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (6195332, MZONE), (6631034, MZONE), (6631034, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e1);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, xyzc: CardId) -> bool {
    let sumtype = SUMMON_TYPE_XYZ as u64;
    c.is_faceup(duel)
        && c.is_set_card_full(duel, &[SET_CONSTELLAR], Some(xyzc), sumtype, tp)
        && !c.is_summon_code(duel, Some(xyzc), sumtype, tp, &[ID])
        && c.is_type_full(duel, TYPE_XYZ, Some(xyzc), sumtype, tp)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let locations = LOCATION_MZONE | LOCATION_GRAVE;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, locations) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(
            a.e.get_handler(duel).get_flag_effect(duel, ID) == 0 && duel.is_existing_target(thfilter, tp, locations, locations, 1, None)
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, thfilter, tp, locations, locations, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}
