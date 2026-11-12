// Ancient Flamvell Deity
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 26304459;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_FIRE, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_race_full(duel, RACE_PYRO, Some(sc), sumtype, tp)
    }
}

// In the Extra Deck, Synchro Summoned with Flamvell Archer (a Level 3 FIRE Tuner) and Fireyarou (a Level 4 FIRE Pyro), with cards in the
// opponent's GY to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (54326448, MZONE),
    (54326448, MZONE),
    (71407486, MZONE),
    (71407486, MZONE),
    (5318639, GRAVE | OPPONENT),
    (5318639, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, remcon);
    e1.set_target(duel, remtg);
    e1.set_operation(duel, remop);
    c.register_effect(duel, e1);
}

fn remcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_synchro_summoned(duel)
}

fn remtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let location = if duel.is_player_affected_by_effect(1 - tp, CARD_SPIRIT_ELIMINATION).is_some() {
        LOCATION_MZONE | LOCATION_GRAVE
    } else {
        LOCATION_GRAVE
    };
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - tp, location as i32);
    Ok(false)
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, false, false)
}

fn remop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ht = duel.get_field_group_count(tp, 0, LOCATION_HAND);
    if ht == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let rg = duel.select_matching_card(tp, rmfilter, tp, 0, LOCATION_MZONE | LOCATION_GRAVE, 1, ht as u8, None)?;
    let c = e.get_handler(duel);
    if !rg.is_empty() {
        duel.remove(rg, POS_FACEUP, REASON_EFFECT)?;
        if c.is_facedown(duel) || !c.is_relate_to_effect(duel, e) {
            return Ok(());
        }
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, rg.len() as i32 * 200);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}
