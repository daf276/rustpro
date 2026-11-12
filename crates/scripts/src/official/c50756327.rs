// Crowley, the First Propheseer
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 50756327;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_SPELLCASTER, Some(lc), sumtype, tp)
    }
}

// Link Summoned from Mystical Elf (Spellcasters), with three different Spellbooks in the Deck and Dark Magician (Level 7) in the
// hand to Normal Summon without Tributes, which no check counts since it Summons other cards.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (15025844, MZONE),
    (15025844, MZONE),
    (15025844, MZONE),
    (11911336, DECK),
    (23314220, DECK),
    (89739383, DECK),
    (36996508, HAND),
    (36996508, HAND),
    (36996508, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_SUMMON_PROC);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_HAND, 0);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, ntcon);
    e2.set_target_card(duel, |duel, _, c| c.is_level_above(duel, 5) && c.is_race(duel, RACE_SPELLCASTER));
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SPELLBOOK]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let dg = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
        return Ok(dg.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= 3);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let mut g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= 3 {
        let mut sg1 = CardSet::default();
        for _ in 0..3 {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
            let sg = g.select(duel, tp, 1, 1, None)?;
            if let Some(tc) = sg.get_first() {
                let code = tc.get_code(duel);
                g.remove_matching(duel, |c, duel| c.is_code(duel, code), None);
            }
            sg1.merge(sg);
        }
        duel.confirm_cards(1 - tp, sg1)?;
        duel.shuffle_deck(tp)?;
        duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_ATOHAND);
        let cg = sg1.select(duel, 1 - tp, 1, 1, None)?;
        if let Some(tc) = cg.get_first() {
            duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn ntcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    a.min == 0 && duel.get_location_count(c.get_controler(duel), LOCATION_MZONE) > 0
}
