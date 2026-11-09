// Topologic Trisbaena
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 72529749;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned from two Sangans, then Monster Reborn on a Sangan in the Graveyard to Special Summon to a zone it points to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, GRAVE),
    (83764718, HAND),
    (83764718, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| {
        aux::zptgroupcon(duel, a.eg.unwrap_or_default(), aux::TRUE, a.e.get_handler(duel))
    });
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let mut g = aux::zptgroup(duel, a.eg.unwrap_or_default(), aux::TRUE, a.e.get_handler(duel));
    let tg = g.filter(duel, |c, duel| c.is_location(duel, LOCATION_MZONE), None);
    duel.set_target_card(tg);
    g.merge(duel.get_matching_group(CardId::is_spell_trap, a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None));
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let mut g = duel.get_target_cards(a.e).unwrap_or_default();
    g.merge(duel.get_matching_group(CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None));
    if duel.remove(g, POS_FACEUP, REASON_EFFECT)? != 0 {
        let ct = duel
            .get_operated_group()
            .filter_count(duel, |c, duel| c.is_controler(duel, 1 - tp), None);
        duel.damage(1 - tp, ct as u32 * 500, REASON_EFFECT)?;
    }
    Ok(())
}
