// Salamangreat Sunlight Wolf
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 87871125;

pub const LISTED_NAMES: &[u32] = &[ID];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        matfilter(c, duel, lc, sumtype, tp)
    }
}

pub fn matfilter(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp) && c.is_attribute_full(duel, ATTRIBUTE_FIRE, Some(lc), sumtype, tp)
}

// A copy on the field and one in the Extra Deck, with FIRE Effect Monsters (Link Apple, Jet Synchron) to Link Summon it from and FIRE monsters in the Graveyard to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (7925734, MZONE),
    (9742784, MZONE),
    (7925734, GRAVE),
    (9742784, GRAVE),
];
// The second effect needs a Reincarnation Link Summon, out of reach without a second copy used as material.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    aux::enable_check_reincarnation(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        aux::zptgroupcon(duel, a.eg.unwrap_or_default(), aux::TRUE, a.e.get_handler(duel))
    });
    e1.set_target(duel, thtg1);
    e1.set_operation(duel, thop1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, thcon2);
    e3.set_target(duel, thtg2);
    e3.set_operation(duel, thop2);
    c.register_effect(duel, e3);
}

fn thfilter1(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_FIRE) && c.is_able_to_hand(duel)
}

fn thtg1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter1, LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let tc = duel
        .select_matching_card(tp, thfilter1, tp, LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first();
    if let Some(tc) = tc
        && duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_HAND)
    {
        duel.confirm_cards(1 - tp, tc)?;
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        e1.set_code(duel, EFFECT_CANNOT_SUMMON);
        e1.set_target_range(duel, 1, 0);
        e1.set_target_card(duel, |duel, e, c| c.is_code(duel, e.get_label(duel) as u32));
        e1.set_label(duel, i64::from(tc.get_code(duel)));
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        duel.register_effect(e2, tp);
        let e3 = e1.clone_effect(duel);
        e3.set_code(duel, EFFECT_CANNOT_MSET);
        duel.register_effect(e3, tp);
    }
    Ok(())
}

fn thcon2(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reincarnation_summoned(duel) && c.is_link_summoned(duel)
}

fn thfilter2(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SALAMANGREAT]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter2, LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop2(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter2, LOCATION_GRAVE)
}
