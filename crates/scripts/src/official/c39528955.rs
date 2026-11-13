// Crusadia Spatha
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 39528955;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(99);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(
            duel,
            |c, duel| c.is_set_card_full(duel, &[SET_CRUSADIA], Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

// Copies in the Extra Deck, Link Summoned from Crusadia Arboria, with monsters on both sides to move.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (91646304, MZONE), (91646304, MZONE), (91646304, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(super::c45002991::atkval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_target_card(duel, super::c45002991::atklimit);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| {
        aux::zptgroupcon(
            duel,
            a.eg.unwrap_or_default(),
            |c, duel| c.is_type(duel, TYPE_EFFECT),
            a.e.get_handler(duel),
        )
    });
    e3.set_target(duel, seqtg);
    e3.set_operation(duel, seqop);
    c.register_effect(duel, e3);
}

fn seqfilter(c: CardId, duel: &Duel) -> bool {
    let tp = c.get_controler(duel);
    c.is_faceup(duel) && c.get_sequence(duel) < 5 && duel.get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_CONTROL, 0xff) > 0
}

fn seqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && seqfilter(chkc, duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(seqfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 1));
    duel.select_target(tp, seqfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, Some(c.into()))?;
    Ok(false)
}

fn seqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    let ttp = tc.get_controler(duel);
    if !tc.is_relate_to_effect(duel, e)
        || tc.is_immune_to_effect(duel, e)
        || duel.get_location_count_full(ttp, LOCATION_MZONE, ttp, LOCATION_REASON_CONTROL, 0xff) <= 0
    {
        return Ok(());
    }
    let (i, p1, p2) = if tc.is_controler(duel, tp) {
        (0, LOCATION_MZONE, 0)
    } else {
        (16, 0, LOCATION_MZONE)
    };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOZONE);
    if let Some(zone) = duel.select_disable_field(tp, 1, p1 as u16, p2 as u16, 0)? {
        duel.move_sequence(tc, (zone.trailing_zeros() - i) as u8)?;
    }
    Ok(())
}
