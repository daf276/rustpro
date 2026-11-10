// Mementotlan Fusion
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 66518509;
const MEMENTOMICTLAN_TECUHTLICA_CREATION_KING: u32 = 14529511;
const MEMENTOTLAN_ANGWITCH: u32 = 54550967;
const MEMENTOTLAN_TATSUNOOTOSHIGO: u32 = 81677154;
const MEMENTOTLAN_BONE_PARTY: u32 = 80722024;
const DARK_HOLE: u32 = 53129443;

struct Params;

impl FusionSummonParams for Params {
    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let sg = duel
            .has_flag_effect(tp, ID)
            .then(|| duel.get_matching_group(aux::necro_valley_filter(extramatfilter), tp, LOCATION_GRAVE, 0, None));
        (sg, Some((checkmat, e.get_handler(duel))))
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk != 0 {
            duel.set_possible_operation_info(0, CATEGORY_TODECK, None, 1, a.tp, LOCATION_GRAVE as i32);
        }
        Ok(true)
    }

    const EXTRAOP: Option<FusionMaterialFn> = Some(extraop);
}

fn extraop(duel: &mut Duel, _: EffectId, _: CardId, _: u8, sg: &mut CardSet) -> Script<()> {
    let rg = sg.filter(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None);
    if !rg.is_empty() {
        duel.hint_selection_full(rg, true);
        duel.send_to_deck(rg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT + REASON_MATERIAL + REASON_FUSION)?;
        for c in rg {
            sg.remove(c);
        }
    }
    Ok(())
}

// In hand with Mementomictlan Tecuhtlica - Creation King in the Extra Deck, Fusion Summoned with Memento monsters from the
// hand and the Graveyard after the opponent's Dark Hole, and in the Graveyard to destroy a monster and search a Memento Trap.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (MEMENTOMICTLAN_TECUHTLICA_CREATION_KING, EXTRA),
    (MEMENTOTLAN_ANGWITCH, HAND),
    (MEMENTOTLAN_ANGWITCH, HAND),
    (MEMENTOTLAN_TATSUNOOTOSHIGO, MZONE),
    (MEMENTOTLAN_ANGWITCH, GRAVE),
    (MEMENTOTLAN_BONE_PARTY, DECK),
    (MEMENTOTLAN_BONE_PARTY, DECK),
    (DARK_HOLE, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummonedCode(MEMENTOMICTLAN_TECUHTLICA_CREATION_KING),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_count_limit(duel, 1, ID);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_DESTROY);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
    });
}

fn checkfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_previous_location(duel, LOCATION_MZONE) || (c.is_monster(duel) && !c.is_previous_location(duel, LOCATION_ONFIELD)))
        && c.is_reason(duel, REASON_EFFECT)
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = a.eg.unwrap_or_default().filter(duel, checkfilter, None);
    if g.is_empty() {
        return Ok(());
    }
    for p in 0..2 {
        if g.is_exists(duel, |c, duel| c.is_previous_controler(duel, p), 1, None) && !duel.has_flag_effect(p, ID) {
            duel.register_flag_effect(p, ID, RESET_PHASE + PHASE_END, 0, 1);
        }
    }
    Ok(())
}

fn extramatfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MEMENTO]) && c.is_able_to_deck(duel)
}

fn checkmat(duel: &Duel, _: CardId, _: u8, sg: CardSet, _: CardId) -> bool {
    sg.is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_MEMENTO]), 1, None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MEMENTO]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_MZONE, 0, None);
    if chk == 0 {
        return Ok(!g.is_empty() && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, LOCATION_MZONE as i32);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let desg = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    if duel.destroy(desg, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let hg = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !hg.is_empty() {
        duel.send_to_hand(hg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, hg)?;
    }
    Ok(())
}
