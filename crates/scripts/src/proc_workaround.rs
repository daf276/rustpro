// Port of proc_workaround.lua: the Card methods it replaces and the global effects it registers on load.
use engine::{Args, CardId, CardSet, Duel, EffectId, Script};

use crate::chain::DuelChain;
use crate::constant::*;
use crate::keys;
use crate::utility::{CardUtility, global_continuous};

pub trait CardWorkaround: Copy {
    fn is_relate_to_effect(self, duel: &Duel, e: EffectId) -> bool;
    fn is_able_to_hand(self, duel: &Duel) -> bool;
    fn is_able_to_deck(self, duel: &Duel) -> bool;
    fn is_able_to_extra(self, duel: &Duel) -> bool;
    fn is_able_to_grave(self, duel: &Duel) -> bool;
    fn is_able_to_remove(self, duel: &Duel) -> bool;
    fn is_able_to_remove_full(self, duel: &Duel, player: Option<u8>, position: u32, reason: u32) -> bool;
}

impl CardWorkaround for CardId {
    fn is_relate_to_effect(self, duel: &Duel, e: EffectId) -> bool {
        if !(duel.is_chain_solving() && e.is_activated(duel)) {
            return self.is_relate_to_effect_core(duel, e);
        }
        if duel.get_reason_effect() == Some(e) {
            return self.is_relate_to_chain(duel, 0);
        }
        for chain_link in 0..duel.get_current_chain() {
            if duel.get_chain_info::<Option<EffectId>>(chain_link, CHAININFO_TRIGGERING_EFFECT) == Some(e) {
                return self.is_relate_to_chain(duel, chain_link);
            }
        }
        self.is_relate_to_effect_core(duel, e)
    }

    fn is_able_to_hand(self, duel: &Duel) -> bool {
        !self.is_location(duel, LOCATION_HAND) && self.is_able_to_hand_core(duel)
    }

    fn is_able_to_deck(self, duel: &Duel) -> bool {
        !self.is_location(duel, LOCATION_DECK) && self.is_able_to_deck_core(duel)
    }

    fn is_able_to_extra(self, duel: &Duel) -> bool {
        !self.is_location(duel, LOCATION_EXTRA) && self.is_able_to_extra_core(duel)
    }

    fn is_able_to_grave(self, duel: &Duel) -> bool {
        !self.is_location(duel, LOCATION_GRAVE) && self.is_able_to_grave_core(duel)
    }

    fn is_able_to_remove(self, duel: &Duel) -> bool {
        self.is_able_to_remove_full(duel, None, POS_FACEUP, REASON_EFFECT)
    }

    fn is_able_to_remove_full(self, duel: &Duel, player: Option<u8>, position: u32, reason: u32) -> bool {
        let player = player.unwrap_or_else(|| duel.get_reason_player());
        !self.is_location(duel, LOCATION_REMOVED) && self.is_able_to_remove_core(duel, player, position, reason)
    }
}

pub trait DuelWorkaround {
    fn swap_sequence(&mut self, card1: CardId, card2: CardId) -> bool;
    fn move_to_field(&mut self, card: CardId, move_player: u8, player: u8, destination: u32, positions: u32, enable: bool) -> Script<bool>;
    fn is_damage_step(&self) -> bool;
    fn overlay(&mut self, target: CardId, materials: impl Into<CardSet>) -> Script<()>;
    fn overlay_full(&mut self, target: CardId, materials: impl Into<CardSet>, send_materials_to_grave: bool) -> Script<()>;
    fn get_target_group(
        &self,
        filter: impl Fn(CardId, &Duel) -> bool,
        player: u8,
        location1: u32,
        location2: u32,
        exception: Option<CardSet>,
    ) -> CardSet;
    // Duel.IsMainPhase(player), Duel.IsDrawPhase(player) and Duel.IsStandbyPhase(player): also the player's turn.
    fn is_main_phase_full(&self, player: Option<u8>) -> bool;
    fn is_draw_phase_full(&self, player: Option<u8>) -> bool;
    fn is_standby_phase_full(&self, player: Option<u8>) -> bool;
    fn get_release_group(&self, player: u8) -> CardSet;
    fn get_release_group_full(&self, player: u8, use_hand: bool, use_oppo: bool, reason: u32) -> CardSet;
    fn check_release_group_cost(
        &self,
        tp: u8,
        f: impl Fn(CardId, &Duel) -> bool,
        counts: (usize, usize),
        use_hand: bool,
        check: Option<ReleaseCheck>,
        ex: Option<CardSet>,
    ) -> bool;
    fn select_release_group_cost(
        &mut self,
        tp: u8,
        f: impl Fn(CardId, &Duel) -> bool,
        counts: (usize, usize),
        use_hand: bool,
        check: Option<ReleaseCheck>,
        ex: Option<CardSet>,
    ) -> Script<CardSet>;
}

impl DuelWorkaround for Duel {
    fn swap_sequence(&mut self, card1: CardId, card2: CardId) -> bool {
        let (sequence1, sequence2) = (card1.get_sequence(self), card2.get_sequence(self));
        self.swap_sequence_core(card1, card2);
        card1.is_on_field(self) && card2.is_on_field(self) && card1.is_sequence(self, sequence2) && card2.is_sequence(self, sequence1)
    }

    fn move_to_field(&mut self, card: CardId, move_player: u8, player: u8, destination: u32, positions: u32, enable: bool) -> Script<bool> {
        let reason_player = if self.get_reason_effect().is_some() {
            self.get_reason_player()
        } else {
            PLAYER_NONE
        };
        card.set_reason_player(self, reason_player);
        self.move_to_field_core(card, move_player, player, destination, positions, enable)
    }

    fn is_damage_step(&self) -> bool {
        let current_phase = self.get_current_phase();
        current_phase == PHASE_DAMAGE || current_phase == PHASE_DAMAGE_CAL
    }

    fn overlay(&mut self, target: CardId, materials: impl Into<CardSet>) -> Script<()> {
        self.overlay_full(target, materials, false)
    }

    // Duel.Overlay, which also raises EVENT_MOVE for the attached cards.
    fn overlay_full(&mut self, target: CardId, materials: impl Into<CardSet>, send_materials_to_grave: bool) -> Script<()> {
        let materials = materials.into();
        let mut eg = materials;
        let core_reason_effect = self.get_reason_effect();
        let (re, r, rp) =
            if self.is_chain_solving() || core_reason_effect.is_some_and(|e| !e.is_has_property(self, EFFECT_FLAG_CANNOT_DISABLE)) {
                (core_reason_effect, REASON_EFFECT, self.get_reason_player())
            } else {
                (None, REASON_RULE, PLAYER_NONE)
            };
        if !send_materials_to_grave {
            for material in materials {
                eg.merge(material.get_overlay_group(self));
            }
        }
        self.overlay_core(target, materials, send_materials_to_grave)?;
        for ec in eg {
            ec.set_reason_player(self, rp);
        }
        self.raise_event(eg, EVENT_MOVE, re, r, rp, 0, 0)
    }

    // Duel.GetTargetGroup: the matching cards the reason effect can target.
    fn get_target_group(
        &self,
        filter: impl Fn(CardId, &Duel) -> bool,
        player: u8,
        location1: u32,
        location2: u32,
        exception: Option<CardSet>,
    ) -> CardSet {
        let re = self.get_reason_effect();
        unported!(self, re.is_none());
        let g = self.get_matching_group(filter, player, location1, location2, exception);
        re.map_or(g, |re| g.filter(self, |c, duel| c.is_can_be_effect_target(duel, re), None))
    }

    fn is_main_phase_full(&self, player: Option<u8>) -> bool {
        self.is_main_phase() && player.is_none_or(|player| self.is_turn_player(player))
    }

    fn is_draw_phase_full(&self, player: Option<u8>) -> bool {
        self.get_current_phase() == PHASE_DRAW && player.is_none_or(|player| self.is_turn_player(player))
    }

    fn is_standby_phase_full(&self, player: Option<u8>) -> bool {
        self.get_current_phase() == PHASE_STANDBY && player.is_none_or(|player| self.is_turn_player(player))
    }

    fn get_release_group(&self, player: u8) -> CardSet {
        self.get_release_group_full(player, false, false, REASON_COST)
    }

    // Duel.GetReleaseGroup, which also offers the Extra Deck cards EFFECT_EXTRA_RELEASE_NONSUM allows to be released.
    fn get_release_group_full(&self, player: u8, use_hand: bool, use_oppo: bool, reason: u32) -> CardSet {
        let g = self.get_release_group_core(player, use_hand, use_oppo, reason);
        let exg = self.get_matching_group(
            |c, duel| c.is_has_effect(duel, EFFECT_EXTRA_RELEASE_NONSUM).is_some() && c.is_releasable_full(duel, reason, None),
            player,
            LOCATION_EXTRA,
            0,
            None,
        );
        unported!(self, !exg.is_empty());
        g
    }

    // Duel.CheckReleaseGroupCost with `maxc` given.
    fn check_release_group_cost(
        &self,
        tp: u8,
        f: impl Fn(CardId, &Duel) -> bool,
        counts: (usize, usize),
        use_hand: bool,
        check: Option<ReleaseCheck>,
        ex: Option<CardSet>,
    ) -> bool {
        let search = ReleaseSearch::new(self, tp, f, counts, use_hand, check, ex);
        search.mg.includes(search.mustg)
            && search
                .mg
                .is_exists(self, |c, duel| search.recursive(duel, c, CardSet::default(), 0), 1, None)
    }

    fn select_release_group_cost(
        &mut self,
        tp: u8,
        f: impl Fn(CardId, &Duel) -> bool,
        counts: (usize, usize),
        use_hand: bool,
        check: Option<ReleaseCheck>,
        ex: Option<CardSet>,
    ) -> Script<CardSet> {
        let search = ReleaseSearch::new(self, tp, f, counts, use_hand, check, ex);
        let mut sg = search.mustg;
        while sg.len() < search.maxc {
            let cg = search.mg.filter(self, |c, duel| search.recursive(duel, c, sg, sg.len()), Some(sg));
            if cg.is_empty() {
                break;
            }
            let cancel = search.goal(self, sg, sg.len()).0;
            self.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
            let Some(tc) = cg.select_unselect(self, Some(sg), tp, cancel, cancel, 1, 1)? else {
                break;
            };
            if search.mustg.is_empty() || !search.mustg.contains(tc) {
                sg.toggle(tc);
            }
        }
        unported!(self, !sg.intersection(search.exg).is_empty());
        Ok(sg)
    }
}

// The check (sg, tp, exg) -> (res, stop) of Duel.CheckReleaseGroupCost and Duel.SelectReleaseGroupCost.
pub type ReleaseCheck<'a> = &'a dyn Fn(&Duel, CardSet, u8, CardSet) -> (bool, bool);

// What the search of Duel.Check/SelectReleaseGroupCost shares: the releasable cards, extra releases and must releases.
struct ReleaseSearch<'a> {
    tp: u8,
    mg: CardSet,
    exg: CardSet,
    mustg: CardSet,
    minc: usize,
    maxc: usize,
    check: Option<ReleaseCheck<'a>>,
}

impl ReleaseSearch<'_> {
    fn new<'a>(
        duel: &Duel,
        tp: u8,
        f: impl Fn(CardId, &Duel) -> bool,
        (minc, maxc): (usize, usize),
        use_hand: bool,
        check: Option<ReleaseCheck<'a>>,
        ex: Option<CardSet>,
    ) -> ReleaseSearch<'a> {
        let mg = duel.get_release_group_full(tp, use_hand, false, REASON_COST).filter(duel, f, ex);
        let (g, exg) = mg.split(duel, |c, duel| release_cost_filter(c, duel, tp), None);
        let mustg = g.filter(
            duel,
            |c, duel| c.is_has_effect(duel, EFFECT_EXTRA_RELEASE).is_some() && c.is_controler(duel, 1 - tp),
            None,
        );
        unported!(duel, !(exg.is_empty() && mustg.is_empty()));
        ReleaseSearch {
            tp,
            mg,
            exg,
            mustg,
            minc,
            maxc,
            check,
        }
    }

    // aux.MakeSpecialCheck(check, tp, exg) applied to sg, with aux.ReleaseCheckSingleUse.
    fn special_check(&self, duel: &Duel, sg: CardSet) -> (bool, bool) {
        let (res, stop) = self.check.map_or((true, false), |check| check(duel, sg, self.tp, self.exg));
        let extra_releases = sg.intersection(self.exg).len();
        (res && extra_releases <= 1, stop || extra_releases > 1)
    }

    // aux.RelCheckGoal.
    fn goal(&self, duel: &Duel, sg: CardSet, ct: usize) -> (bool, bool) {
        if ct < self.minc || ct > self.maxc {
            return (false, ct > self.maxc);
        }
        let (res, stop) = self.special_check(duel, sg);
        (res && sg.includes(self.mustg), stop)
    }

    // aux.RelCheckRecursive, with sg and ct as they are before the card is added.
    fn recursive(&self, duel: &Duel, c: CardId, mut sg: CardSet, ct: usize) -> bool {
        sg.insert(c);
        let ct = ct + 1;
        let (mut res, stop) = self.goal(duel, sg, ct);
        if !res && !stop {
            res = ct < self.maxc && self.mg.is_exists(duel, |c, duel| self.recursive(duel, c, sg, ct), 1, Some(sg));
        }
        res
    }
}

fn release_cost_filter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let eff = c.is_has_effect(duel, EFFECT_EXTRA_RELEASE_NONSUM);
    !(c.is_controler(duel, 1 - tp) && eff.is_some_and(|eff| eff.check_count_limit(duel, tp)))
        && c.is_has_effect(duel, EFFECT_EXTRA_RELEASE).is_none()
}

pub fn load(duel: &mut Duel) {
    let hand_adjust_eff = global_continuous(duel, EVENT_DISCARD);
    hand_adjust_eff.set_count_limit(duel, 1, 0);
    hand_adjust_eff.set_condition(duel, |_, a| a.r & REASON_ADJUST > 0);
    hand_adjust_eff.set_promptless_operation(duel, track_chain_solving);
    duel.register_effect(hand_adjust_eff, 0);

    let sum_status_eff = global_continuous(duel, EVENT_CHANGE_POS);
    sum_status_eff.set_promptless_operation(duel, |duel, a| {
        for c in a.eg.unwrap_or_default().filter(duel, summon_status_filter, None) {
            c.set_status(duel, STATUS_SUMMON_TURN, false);
            c.set_status(duel, STATUS_FLIP_SUMMON_TURN, false);
            c.set_status(duel, STATUS_SPSUMMON_TURN, false);
            c.set_status(duel, STATUS_FORM_CHANGED, true);
        }
        Ok(())
    });
    duel.register_effect(sum_status_eff, 0);

    let set_turn_status_split = global_continuous(duel, EVENT_MSET);
    set_turn_status_split.set_promptless_operation(duel, |duel, a| {
        for c in a.eg.unwrap_or_default() {
            c.set_status(duel, STATUS_SUMMON_TURN, false);
            c.set_status(duel, STATUS_FORM_CHANGED, true);
        }
        Ok(())
    });
    duel.register_effect(set_turn_status_split, 0);

    let shuffle_eff = global_continuous(duel, EVENT_CHAIN_SOLVING);
    shuffle_eff.set_promptless_operation(duel, shuffle_on_resolution);
    duel.register_effect(shuffle_eff, 0);

    let tracking_eff = global_continuous(duel, EVENT_CHAIN_END);
    tracking_eff.set_promptless_operation(duel, |duel, _| {
        (0..2).for_each(|player| duel.set_script_state(keys::hand_shuffled(player), 0));
        Ok(())
    });
    duel.register_effect(tracking_eff, 0);

    let ns_eff = global_continuous(duel, EVENT_SUMMON_SUCCESS);
    ns_eff.set_promptless_operation(duel, |duel, a| {
        for sc in a.eg.unwrap_or_default() {
            let phase = duel.get_current_phase();
            sc.register_flag_effect_full(duel, 160214042, RESET_EVENT | RESETS_STANDARD, 0, 1, phase.into(), 0);
        }
        Ok(())
    });
    duel.register_effect(ns_eff, 0);
    let sp_eff = ns_eff.clone_effect(duel);
    sp_eff.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    duel.register_effect(sp_eff, 0);
}

fn track_chain_solving(duel: &mut Duel, _: &Args) -> Script<()> {
    let chain_solving_eff = global_continuous(duel, EVENT_CHAIN_SOLVING);
    chain_solving_eff.set_count_limit(duel, 1, 0);
    chain_solving_eff.set_promptless_operation(duel, |duel, a| {
        a.e.reset(duel);
        let cannot_act_eff = duel.global_effect();
        cannot_act_eff.set_type(duel, EFFECT_TYPE_FIELD);
        cannot_act_eff.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        cannot_act_eff.set_code(duel, EFFECT_CANNOT_ACTIVATE);
        cannot_act_eff.set_target_range(duel, 1, 1);
        cannot_act_eff.set_value(duel, 1);
        cannot_act_eff.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(cannot_act_eff, 0);
        Ok(())
    });
    chain_solving_eff.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(chain_solving_eff, 0);
    Ok(())
}

fn summon_status_filter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_status(duel, STATUS_SUMMON_TURN | STATUS_FLIP_SUMMON_TURN | STATUS_SPSUMMON_TURN)
}

fn check_opinfo(duel: &Duel, ev: u8, category: u64, rc: CardId) -> bool {
    let contains = |cards: Option<CardSet>| cards.is_some_and(|cards| cards.is_contains(rc));
    contains(duel.get_operation_info(ev, category).and_then(|info| info.op_cards))
        || contains(duel.get_possible_operation_info(ev, category).and_then(|info| info.op_cards))
}

fn shuffle_on_resolution(duel: &mut Duel, a: &Args) -> Script<()> {
    let ev = a.ev as u8;
    if duel.get_chain_info::<u32>(ev, CHAININFO_TRIGGERING_LOCATION) != LOCATION_HAND {
        return Ok(());
    }
    let Some(re) = a.re else { return Ok(()) };
    let rc = re.get_handler(duel);
    let player = rc.get_controler(duel);
    if duel.get_script_state(keys::hand_shuffled(player)) != 0 {
        return Ok(());
    }
    if !(rc.is_relate_to_effect(duel, re) && rc.is_location(duel, LOCATION_HAND)) {
        return Ok(());
    }
    let moves = [
        CATEGORY_SPECIAL_SUMMON,
        CATEGORY_SUMMON,
        CATEGORY_TOGRAVE,
        CATEGORY_DESTROY,
        CATEGORY_REMOVE,
        CATEGORY_TODECK,
        CATEGORY_TOEXTRA,
        CATEGORY_EQUIP,
        CATEGORY_RELEASE,
    ];
    if moves.into_iter().any(|category| check_opinfo(duel, ev, category, rc)) {
        return Ok(());
    }
    duel.shuffle_hand(player)?;
    // move_eff and the effect it registers keep `player` as their label, where the Lua closures capture it.
    let move_eff = duel.create_effect(rc);
    move_eff.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    move_eff.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    move_eff.set_code(duel, EVENT_MOVE);
    move_eff.set_label(duel, player.into());
    move_eff.set_promptless_operation(duel, |duel, a| {
        let player = a.e.get_label(duel) as u8;
        let eff = duel.create_effect(a.e.get_handler(duel));
        eff.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        eff.set_code(duel, EVENT_CHAIN_SOLVED);
        eff.set_label(duel, player.into());
        eff.set_promptless_operation(duel, |duel, a| {
            duel.shuffle_hand(a.e.get_label(duel) as u8)?;
            a.e.reset(duel);
            Ok(())
        });
        eff.set_reset(duel, RESET_CHAIN);
        duel.register_effect(eff, player);
        a.e.reset(duel);
        Ok(())
    });
    move_eff.set_reset(duel, RESET_CHAIN);
    rc.register_effect(duel, move_eff);
    duel.set_script_state(keys::hand_shuffled(player), 1);
    Ok(())
}
