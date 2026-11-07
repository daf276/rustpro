// Ports of processor_unit.h, processor_visit.cpp and processor.cpp. Steps keep the core's numbering; the chain units
// are in chain.rs.
use crate::card::{Card, CardId, CardSet, LocInfo, SendtoParam};
use crate::chain::{AddChain, Execute, SolveChain, SolveContinuous};
use crate::constants::*;
use crate::duel::Duel;
use crate::effect::{Chain, Container, EffectId, Value};
use crate::field::PerPlayer;
use crate::operations::{
    ChangePos, Damage, Destroy, DestroyReplace, DiscardDeck, Draw, Equip, FlipSummon, GetControl, MonsterSet, MoveToField,
    OperationReplace, PayLpCost, Recover, Release, RemoveCounter, RemoveOverlay, SelectTribute, SendTo, SpSummon, SpSummonRule,
    SpSummonStep, SpellSet, SpellSetGroup, SummonRule, TossCoin, TossDice, XyzOverlay,
};
use crate::script::Procedure;

// P::restart: the step counter wraps to 0 when the processor increments it.
pub const RESTART: u16 = u16::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OcgStatus {
    End,
    Awaiting,
    Continue,
}

#[derive(Debug, Clone, Default)]
pub struct Adjust {
    pub step: u16,
}

#[derive(Debug, Clone, Default)]
pub struct Turn {
    pub step: u16,
    pub turn_player: u8,
    pub has_performed_second_battle_phase: bool,
}

#[derive(Debug, Clone, Default)]
pub struct RefreshLoc {
    pub step: u16,
    pub previously_disabled_locations: u32,
}

#[derive(Debug, Clone, Default)]
pub struct Startup {
    pub step: u16,
    // The turn the duel starts with: the first player's first turn, or the one a position starts at.
    pub turn: Turn,
}

#[derive(Debug, Clone, Default)]
pub struct SelectBattleCmd {
    pub step: u16,
    pub playerid: u8,
}

#[derive(Debug, Clone, Default)]
pub struct SelectIdleCmd {
    pub step: u16,
    pub playerid: u8,
}

#[derive(Debug, Clone, Default)]
pub struct SelectEffectYesNo {
    pub step: u16,
    pub playerid: u8,
    pub description: u64,
    pub card: CardId,
}

#[derive(Debug, Clone, Default)]
pub struct SelectYesNo {
    pub step: u16,
    pub playerid: u8,
    pub description: u64,
}

#[derive(Debug, Clone, Default)]
pub struct SelectOption {
    pub step: u16,
    pub playerid: u8,
}

#[derive(Debug, Clone, Default)]
pub struct AnnounceNumber {
    pub step: u16,
    pub playerid: u8,
}

#[derive(Debug, Clone, Default)]
pub struct AnnounceRace {
    pub step: u16,
    pub playerid: u8,
    pub count: u8,
    pub available: u64,
}

#[derive(Debug, Clone, Default)]
pub struct AnnounceAttribute {
    pub step: u16,
    pub playerid: u8,
    pub count: u8,
    pub available: u32,
}

#[derive(Debug, Clone, Default)]
pub struct AnnounceCard {
    pub step: u16,
    pub playerid: u8,
}

#[derive(Debug, Clone, Default)]
pub struct SortCard {
    pub step: u16,
    pub playerid: u8,
    pub is_chain: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SortDeck {
    pub step: u16,
    pub sort_player: u8,
    pub target_player: u8,
    pub count: u16,
    pub bottom: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SelectCard {
    pub step: u16,
    pub playerid: u8,
    pub cancelable: bool,
    pub min: u8,
    pub max: u8,
}

// SelectDisField, the disable_field variant, comes with EFFECT_DISABLE_FIELD.
#[derive(Debug, Clone, Default)]
pub struct SelectPlace {
    pub step: u16,
    pub playerid: u8,
    pub count: u8,
    pub flag: u32,
}

#[derive(Debug, Clone, Default)]
pub struct SelectPosition {
    pub step: u16,
    pub playerid: u8,
    pub positions: u8,
    pub code: u32,
}

#[derive(Debug, Clone, Default)]
pub struct SelectTributeP {
    pub step: u16,
    pub playerid: u8,
    pub cancelable: bool,
    pub min: u8,
    pub max: u8,
}

// The sum_param of the cards to choose from and of the must cards, which the core keeps on each card.
#[derive(Debug, Clone, Default)]
pub struct SelectSum {
    pub step: u16,
    pub playerid: u8,
    pub acc: i32,
    pub min: i32,
    pub max: i32,
    pub params: Vec<(CardId, u32)>,
}

#[derive(Debug, Clone, Default)]
pub struct SelectUnselectCard {
    pub step: u16,
    pub playerid: u8,
    pub cancelable: bool,
    pub min: u8,
    pub max: u8,
    pub finishable: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SelectChain {
    pub step: u16,
    pub playerid: u8,
    pub spe_count: u8,
    pub forced: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PointEvent {
    pub step: u16,
    pub skip_trigger: bool,
    pub skip_freechain: bool,
    pub skip_new: bool,
}

#[derive(Debug, Clone, Default)]
pub struct QuickEffect {
    pub step: u16,
    pub skip_freechain: bool,
    pub is_opponent: bool,
    pub priority_player: u8,
}

#[derive(Debug, Clone, Default)]
pub struct IdleCommand {
    pub step: u16,
    pub phase_to_change_to: u8,
    pub card_to_reposition: Option<CardId>,
}

#[derive(Debug, Clone, Default)]
pub struct PhaseEvent {
    pub step: u16,
    pub phase: u16,
    pub is_opponent: bool,
    pub priority_passed: bool,
}

// `forced_attack` (ForcedBattle, from Duel.ForceAttack), the damage change effect and must_attack_map
// (EFFECT_MUST_ATTACK_MONSTER) come with the cards that use them.
// Duel.CalculateDamage: the damage step of one attacker against one target, run apart from the Battle Phase's attack.
#[derive(Debug, Clone, Default)]
pub struct DamageStep {
    pub step: u16,
    pub attacker: Option<CardId>,
    pub attack_target: Option<CardId>,
    pub new_attack: bool,
    pub backup_phase: u16,
    pub cards_destroyed_by_battle: Option<CardSet>,
}

#[derive(Debug, Clone, Default)]
pub struct BattleCommand {
    pub step: u16,
    pub phase_to_change_to: u16,
    pub forced_attack_done: bool,
    pub is_replaying_attack: bool,
    pub attack_announce_failed: bool,
    pub repeat_battle_phase: bool,
    pub second_battle_phase_is_optional: bool,
    pub previous_point_event_had_any_trigger_to_resolve: bool,
    pub reason_player: u8,
    pub cards_destroyed_by_battle: Option<CardSet>,
    pub reason_card: Option<CardId>,
}

// Relay duels are unsupported, so no player ever needs recharging.
#[derive(Debug, Clone, Default)]
pub struct RefreshRelay {
    pub step: u16,
}

#[derive(Debug, Clone)]
pub enum Unit {
    Adjust(Adjust),
    Turn(Turn),
    RefreshLoc(RefreshLoc),
    Startup(Startup),
    SelectBattleCmd(SelectBattleCmd),
    SelectIdleCmd(SelectIdleCmd),
    SelectEffectYesNo(SelectEffectYesNo),
    SelectYesNo(SelectYesNo),
    SelectOption(SelectOption),
    AnnounceNumber(AnnounceNumber),
    AnnounceRace(AnnounceRace),
    AnnounceAttribute(AnnounceAttribute),
    AnnounceCard(AnnounceCard),
    SortCard(SortCard),
    SortDeck(SortDeck),
    SelectCard(SelectCard),
    SelectUnselectCard(SelectUnselectCard),
    SelectSum(SelectSum),
    SelectChain(SelectChain),
    SelectPlace(SelectPlace),
    SelectPosition(SelectPosition),
    SelectTributeP(SelectTributeP),
    SelectTribute(SelectTribute),
    QuickEffect(QuickEffect),
    IdleCommand(IdleCommand),
    PhaseEvent(PhaseEvent),
    PointEvent(PointEvent),
    BattleCommand(BattleCommand),
    DamageStep(DamageStep),
    Destroy(Destroy),
    DestroyReplace(DestroyReplace),
    OperationReplace(OperationReplace),
    Release(Release),
    SendTo(SendTo),
    MoveToField(MoveToField),
    ChangePos(ChangePos),
    SummonRule(SummonRule),
    FlipSummon(FlipSummon),
    MonsterSet(MonsterSet),
    SpellSet(SpellSet),
    SpSummon(SpSummon),
    SpSummonStep(SpSummonStep),
    SpSummonRule(SpSummonRule),
    SpellSetGroup(SpellSetGroup),
    RemoveCounter(RemoveCounter),
    RemoveOverlay(RemoveOverlay),
    XyzOverlay(XyzOverlay),
    Draw(Draw),
    DiscardDeck(DiscardDeck),
    Equip(Equip),
    GetControl(GetControl),
    TossCoin(TossCoin),
    TossDice(TossDice),
    Damage(Damage),
    Recover(Recover),
    PayLpCost(PayLpCost),
    Execute(Execute),
    AddChain(AddChain),
    SolveChain(SolveChain),
    SolveContinuous(SolveContinuous),
    RefreshRelay(RefreshRelay),
}

// A unit's step counter, which `run_steps` advances.
trait Steps {
    fn step(&mut self) -> &mut u16;
}

macro_rules! units {
    ($($name:ident),*) => {
        $(impl From<$name> for Unit {
            fn from(unit: $name) -> Self {
                Unit::$name(unit)
            }
        }

        impl Steps for $name {
            fn step(&mut self) -> &mut u16 {
                &mut self.step
            }
        })*
    };
}

units!(
    Adjust,
    Turn,
    RefreshLoc,
    Startup,
    SelectBattleCmd,
    SelectIdleCmd,
    SelectEffectYesNo,
    SelectYesNo,
    SelectOption,
    AnnounceNumber,
    AnnounceRace,
    AnnounceAttribute,
    AnnounceCard,
    SortCard,
    SortDeck,
    SelectCard,
    SelectUnselectCard,
    SelectSum,
    SelectChain,
    SelectPlace,
    SelectPosition,
    SelectTributeP,
    SelectTribute,
    QuickEffect,
    IdleCommand,
    PhaseEvent,
    PointEvent,
    BattleCommand,
    DamageStep,
    Destroy,
    DestroyReplace,
    OperationReplace,
    Release,
    SendTo,
    MoveToField,
    ChangePos,
    SummonRule,
    FlipSummon,
    MonsterSet,
    SpellSet,
    SpSummon,
    SpSummonStep,
    SpSummonRule,
    SpellSetGroup,
    RemoveCounter,
    RemoveOverlay,
    XyzOverlay,
    Draw,
    DiscardDeck,
    Equip,
    GetControl,
    TossCoin,
    TossDice,
    Damage,
    Recover,
    PayLpCost,
    Execute,
    AddChain,
    SolveChain,
    SolveContinuous,
    RefreshRelay
);

impl Duel {
    pub(crate) fn emplace_process(&mut self, unit: impl Into<Unit>) {
        self.core.subunits.push(unit.into());
    }

    // field::process in processor_visit.cpp. `units` is core.units, taken out by `run` so the front unit runs in place,
    // or the stack of a script's nested run.
    pub(crate) fn process_unit(&mut self, units: &mut Vec<Unit>) -> OcgStatus {
        if !self.pending_resets.get_mut().is_empty() {
            self.reset_pending();
        }
        if !self.core.subunits.is_empty() {
            self.core.subunits.reverse();
            units.append(&mut self.core.subunits);
        }
        let Some(unit) = units.last_mut() else {
            return OcgStatus::End;
        };
        let (done, needs_answer) = match unit {
            Unit::Adjust(arg) => self.run_steps(arg, Self::process_adjust, false),
            Unit::Turn(arg) => self.run_steps(arg, Self::process_turn, false),
            Unit::RefreshLoc(arg) => self.run_steps(arg, Self::process_refresh_loc, false),
            Unit::Startup(arg) => self.run_steps(arg, Self::process_startup, false),
            Unit::SelectBattleCmd(arg) => self.run_steps(arg, Self::process_select_battle_cmd, true),
            Unit::SelectIdleCmd(arg) => self.run_steps(arg, Self::process_select_idle_cmd, true),
            Unit::SelectEffectYesNo(arg) => self.run_steps(arg, Self::process_select_effect_yes_no, true),
            Unit::SelectYesNo(arg) => self.run_steps(arg, Self::process_select_yes_no, true),
            Unit::SelectOption(arg) => self.run_steps(arg, Self::process_select_option, true),
            Unit::AnnounceNumber(arg) => self.run_steps(arg, Self::process_announce_number, true),
            Unit::AnnounceRace(arg) => self.run_steps(arg, Self::process_announce_race, true),
            Unit::AnnounceAttribute(arg) => self.run_steps(arg, Self::process_announce_attribute, true),
            Unit::AnnounceCard(arg) => self.run_steps(arg, Self::process_announce_card, true),
            Unit::SortCard(arg) => self.run_steps(arg, Self::process_sort_card, true),
            Unit::SortDeck(arg) => self.run_steps(arg, Self::process_sort_deck, false),
            Unit::SelectCard(arg) => self.run_steps(arg, Self::process_select_card, true),
            Unit::SelectUnselectCard(arg) => self.run_steps(arg, Self::process_select_unselect_card, true),
            Unit::SelectSum(arg) => self.run_steps(arg, Self::process_select_sum, true),
            Unit::SelectChain(arg) => self.run_steps(arg, Self::process_select_chain, true),
            Unit::SelectPlace(arg) => self.run_steps(arg, Self::process_select_place, true),
            Unit::SelectPosition(arg) => self.run_steps(arg, Self::process_select_position, true),
            Unit::SelectTributeP(arg) => self.run_steps(arg, Self::process_select_tribute_p, true),
            Unit::SelectTribute(arg) => self.run_steps(arg, Self::process_select_tribute, false),
            Unit::QuickEffect(arg) => self.run_steps(arg, Self::process_quick_effect, false),
            Unit::IdleCommand(arg) => self.run_steps(arg, Self::process_idle_command, false),
            Unit::PhaseEvent(arg) => self.run_steps(arg, Self::process_phase_event, false),
            Unit::PointEvent(arg) => self.run_steps(arg, Self::process_point_event, false),
            Unit::BattleCommand(arg) => self.run_steps(arg, Self::process_battle_command, false),
            Unit::DamageStep(arg) => self.run_steps(arg, Self::process_damage_step, false),
            Unit::Destroy(arg) => self.run_steps(arg, Self::process_destroy, false),
            Unit::DestroyReplace(arg) => self.run_steps(arg, Self::process_destroy_replace, false),
            Unit::OperationReplace(arg) => self.run_steps(arg, Self::process_operation_replace, false),
            Unit::Release(arg) => self.run_steps(arg, Self::process_release, false),
            Unit::SendTo(arg) => self.run_steps(arg, Self::process_send_to, false),
            Unit::MoveToField(arg) => self.run_steps(arg, Self::process_move_to_field, false),
            Unit::ChangePos(arg) => self.run_steps(arg, Self::process_change_pos, false),
            Unit::SummonRule(arg) => self.run_steps(arg, Self::process_summon_rule, false),
            Unit::FlipSummon(arg) => self.run_steps(arg, Self::process_flip_summon, false),
            Unit::MonsterSet(arg) => self.run_steps(arg, Self::process_monster_set, false),
            Unit::SpellSet(arg) => self.run_steps(arg, Self::process_spell_set, false),
            Unit::SpSummon(arg) => self.run_steps(arg, Self::process_sp_summon, false),
            Unit::SpSummonStep(arg) => self.run_steps(arg, Self::process_sp_summon_step, false),
            Unit::SpSummonRule(arg) => self.run_steps(arg, Self::process_sp_summon_rule, false),
            Unit::SpellSetGroup(arg) => self.run_steps(arg, Self::process_spell_set_group, false),
            Unit::RemoveCounter(arg) => self.run_steps(arg, Self::process_remove_counter, false),
            Unit::RemoveOverlay(arg) => self.run_steps(arg, Self::process_remove_overlay, false),
            Unit::XyzOverlay(arg) => self.run_steps(arg, Self::process_xyz_overlay, false),
            Unit::Draw(arg) => self.run_steps(arg, Self::process_draw, false),
            Unit::DiscardDeck(arg) => self.run_steps(arg, Self::process_discard_deck, false),
            Unit::Equip(arg) => self.run_steps(arg, Self::process_equip, false),
            Unit::GetControl(arg) => self.run_steps(arg, Self::process_get_control, false),
            Unit::TossCoin(arg) => self.run_steps(arg, Self::process_toss_coin, false),
            Unit::TossDice(arg) => self.run_steps(arg, Self::process_toss_dice, false),
            Unit::Damage(arg) => self.run_steps(arg, Self::process_damage, false),
            Unit::Recover(arg) => self.run_steps(arg, Self::process_recover, false),
            Unit::PayLpCost(arg) => self.run_steps(arg, Self::process_pay_lp_cost, false),
            Unit::Execute(arg) => self.run_steps(arg, Self::process_execute, false),
            Unit::AddChain(arg) => self.run_steps(arg, Self::process_add_chain, false),
            Unit::SolveChain(arg) => self.run_steps(arg, Self::process_solve_chain, false),
            Unit::SolveContinuous(arg) => self.run_steps(arg, Self::process_solve_continuous, false),
            Unit::RefreshRelay(arg) => (arg.step >= 2, false),
        };
        if done {
            units.truncate(units.len() - 1);
            return OcgStatus::Continue;
        }
        if let Unit::RefreshRelay(arg) = unit {
            arg.step += 1;
        }
        // A script function's prompt, shown on top of the unit that ran the function.
        if let Some(prompt) = self.core.presented.take() {
            units.push(*prompt);
            return OcgStatus::Awaiting;
        }
        if needs_answer { OcgStatus::Awaiting } else { OcgStatus::Continue }
    }

    // Runs steps of the front unit until it ends, prompts, or emplaces a unit that then runs first: what the core's
    // loop of one step per call does, without dispatching again for each step. Not inlined, so that every unit's
    // loop is a function of its own instead of one process_unit holding all of them.
    #[inline(never)]
    fn run_steps<U: Steps>(&mut self, arg: &mut U, process: impl Fn(&mut Self, &mut U) -> bool, needs_answer: bool) -> (bool, bool) {
        loop {
            if process(self, arg) {
                break (true, needs_answer);
            }
            *arg.step() = arg.step().wrapping_add(1);
            if needs_answer || !self.core.subunits.is_empty() || self.result.is_some() || self.core.presented.is_some() {
                break (false, needs_answer);
            }
            if self.core.replay.interrupted {
                break (false, false);
            }
        }
    }

    // Flip effects of the damage step wait until damage calculation, then trigger before the others.
    fn release_delayed_flips(&mut self) {
        let core = &mut self.core;
        core.flip_delayed = false;
        core.new_fchain.splice(0..0, std::mem::take(&mut core.new_fchain_b));
        core.new_ochain.splice(0..0, std::mem::take(&mut core.new_ochain_b));
    }

    // A chain window's candidate the core collects with only its effect set.
    fn push_candidate(&mut self, effect: EffectId) {
        self.core.select_chains.push(Chain::new(effect));
    }

    // core.spe_effect[p]: the number of optional trigger effects and activate/quick effects with hints.
    fn process_phase_event(&mut self, arg: &mut PhaseEvent) -> bool {
        let phase = arg.phase;
        let turn_player = self.infos.turn_player;
        match arg.step {
            0 => {
                let skip = |duel: &Self, code| duel.core.force_turn_end || duel.is_player_affected_by_effect(turn_player, code).is_some();
                if (phase == PHASE_DRAW && skip(self, EFFECT_SKIP_DP))
                    || (phase == PHASE_STANDBY && skip(self, EFFECT_SKIP_SP))
                    || (phase == PHASE_BATTLE_START && skip(self, EFFECT_SKIP_BP))
                    || (phase == PHASE_BATTLE && skip(self, EFFECT_SKIP_BP))
                    || (phase == PHASE_END && skip(self, EFFECT_SKIP_EP))
                {
                    arg.step = 24;
                    return false;
                }
                let phase_event = EVENT_PHASE + u32::from(phase);
                self.nil_event.event_code = phase_event;
                self.nil_event.event_player = turn_player;
                let check_player = if arg.is_opponent { 1 - turn_player } else { turn_player };
                self.core.select_chains.clear();
                let nil_event = self.nil_event;
                let (mut tf_count, mut to_count, mut fc_count, mut cn_count) = (0, 0, 0, 0);
                for effect in self.field_effect_range(Container::TriggerF, phase_event) {
                    self.set_activate_location(effect);
                    if self.is_activateable(effect, check_player, &nil_event, false, false, false, false, false) {
                        self.effects.set_id(effect, self.next_field_id());
                        self.push_candidate(effect);
                        tf_count += 1;
                    }
                }
                // The continuous effects may change while their conditions run (Rescue Cat).
                for effect in self.field_effect_range(Container::Continuous, phase_event) {
                    if self.get_handler_player(effect) == check_player
                        && self.is_activateable(effect, check_player, &nil_event, false, false, false, false, false)
                    {
                        self.effects.set_id(effect, self.next_field_id());
                        self.push_candidate(effect);
                        cn_count += 1;
                    }
                }
                // Effects taking control non-permanently last until the End Phase, not until the end of the turn.
                for effect in self.field_effects.pheff.clone() {
                    let (code, reset_flag, reset_count) = {
                        let peffect = &self.effects[effect];
                        (peffect.code, peffect.reset_flag, peffect.reset_count)
                    };
                    let pid = self.get_handler_player(effect);
                    let own_turn = (reset_flag & RESET_SELF_TURN != 0 && pid == turn_player)
                        || (reset_flag & RESET_OPPO_TURN != 0 && pid != turn_player);
                    if code != EFFECT_SET_CONTROL
                        || reset_flag & u32::from(phase) == 0
                        || pid != check_player
                        || !own_turn
                        || reset_count != 1
                    {
                        continue;
                    }
                    let handler = self.get_handler(effect);
                    if self.get_value(effect, handler) != i32::from(self.cards[handler].current.controler) {
                        continue;
                    }
                    self.push_candidate(effect);
                    cn_count += 1;
                }
                self.core.spe_effect[check_player] = 0;
                if !self.core.hand_adjusted {
                    for effect in self.field_effect_range(Container::TriggerO, phase_event) {
                        self.set_activate_location(effect);
                        if self.is_activateable(effect, check_player, &nil_event, false, false, false, false, false) {
                            self.effects.set_id(effect, self.next_field_id());
                            self.push_candidate(effect);
                            to_count += 1;
                            self.core.spe_effect[check_player] += 1;
                        }
                    }
                    self.core.hint_timing[turn_player] = match phase {
                        PHASE_DRAW => TIMING_DRAW_PHASE,
                        PHASE_STANDBY => TIMING_STANDBY_PHASE,
                        PHASE_BATTLE_START => TIMING_BATTLE_START,
                        PHASE_BATTLE => TIMING_BATTLE_END,
                        _ => TIMING_END_PHASE,
                    };
                    for (container, cteffect) in [(Container::Activate, true), (Container::QuickO, false)] {
                        for effect in self.field_effect_range(container, EVENT_FREE_CHAIN) {
                            self.set_activate_location(effect);
                            if !self.is_chainable(effect, check_player)
                                || !self.is_activateable(effect, check_player, &nil_event, false, false, false, false, false)
                            {
                                continue;
                            }
                            self.effects.set_id(effect, self.next_field_id());
                            self.push_candidate(effect);
                            if self.check_hint_timing(effect) || (cteffect && self.check_cteffect_hint(effect, check_player)) {
                                self.core.spe_effect[check_player] += 1;
                            }
                            fc_count += 1;
                        }
                    }
                    for effect in self.field_effect_range(Container::Continuous, EVENT_FREE_CHAIN) {
                        if self.get_handler_player(effect) == check_player
                            && self.is_activateable(effect, check_player, &nil_event, false, false, false, false, false)
                        {
                            self.effects.set_id(effect, self.next_field_id());
                            self.push_candidate(effect);
                            fc_count += 1;
                        }
                    }
                }
                if self.core.select_chains.is_empty() {
                    self.returns.set_i32(0, -1);
                    arg.step = 1;
                } else if tf_count == 0 && cn_count == 1 && to_count == 0 && fc_count == 0 {
                    self.returns.set_i32(0, 0);
                    arg.step = 1;
                } else {
                    let hint = match self.infos.phase {
                        PHASE_DRAW => 20,
                        PHASE_STANDBY => 21,
                        PHASE_BATTLE_START => 28,
                        PHASE_BATTLE => 25,
                        _ => 26,
                    };
                    self.trace.message(MSG_HINT).u8(HINT_EVENT).u8(check_player).u64(hint);
                    if tf_count == 0 && to_count == 1 && fc_count == 0 && cn_count == 0 {
                        let effect = self
                            .core
                            .select_chains
                            .first()
                            .map(|chain| chain.triggering_effect)
                            .unwrap_or_default();
                        let handler = self.get_handler(effect);
                        self.field_select_effect_yes_no(check_player, 0, handler);
                    } else {
                        let spe_count = self.core.spe_effect[check_player] as u8;
                        self.emplace_process(SelectChain {
                            playerid: check_player,
                            spe_count,
                            forced: tf_count + cn_count > 0,
                            ..SelectChain::default()
                        });
                        arg.step = 1;
                    }
                }
                false
            }
            1 => {
                self.returns.set_i32(0, self.returns.at_i32(0) - 1);
                false
            }
            2 => {
                let selected = self.returns.at_i32(0);
                if selected == -1 {
                    if arg.priority_passed {
                        arg.step = 19;
                    } else {
                        arg.priority_passed = true;
                        arg.is_opponent = !arg.is_opponent;
                        arg.step = RESTART;
                    }
                    return false;
                }
                arg.priority_passed = false;
                let Some(mut chain) = self.take_select_chain(selected as usize) else {
                    return true;
                };
                let effect = chain.triggering_effect;
                let effect_type = self.effects[effect].effect_type;
                if effect_type & EFFECT_TYPE_ACTIONS == 0 {
                    self.remove_registered_effect(effect);
                    self.adjust_all();
                    arg.step = 3;
                } else if effect_type & EFFECT_TYPE_CONTINUOUS == 0 {
                    let check_player = if arg.is_opponent { 1 - turn_player } else { turn_player };
                    let handler = self.get_handler(effect);
                    chain.flag = 0;
                    chain.chain_id = self.next_field_id() as u16;
                    chain.evt = self.nil_event;
                    self.set_triggering_state(&mut chain, handler);
                    chain.triggering_player = check_player;
                    self.core.new_chains.push(chain);
                    self.cards[handler].set_status(STATUS_CHAINING, true);
                    self.dec_count(effect, check_player);
                    self.core.select_chains.clear();
                    self.emplace_process(AddChain::default());
                    let priority_player = if self.is_flag(DUEL_INVERTED_QUICK_PRIORITY) {
                        check_player
                    } else {
                        1 - check_player
                    };
                    self.emplace_process(QuickEffect {
                        priority_player,
                        ..QuickEffect::default()
                    });
                    self.infos.priorities = PerPlayer::default();
                } else {
                    self.core.select_chains.clear();
                    let nil_event = self.nil_event;
                    self.solve_continuous(self.get_handler_player(effect), effect, nil_event);
                    arg.step = 3;
                }
                false
            }
            3 => {
                self.core.chain_limit.clear();
                self.clear_chaining_status();
                self.emplace_process(SolveChain::default());
                arg.step = RESTART;
                false
            }
            4 => {
                self.adjust_instant();
                self.emplace_process(PointEvent::default());
                arg.step = RESTART;
                false
            }
            20 => {
                if phase != PHASE_END {
                    arg.step = 24;
                    return false;
                }
                let limit = 6;
                unported!(self, self.filter_player_effect(turn_player, EFFECT_HAND_LIMIT, true));
                let hand = self.player[turn_player].list_hand.clone();
                let hand_size = hand.len() as i32;
                if hand_size <= limit || self.is_flag(DUEL_NO_HAND_LIMIT) {
                    arg.step = 24;
                    return false;
                }
                self.core.select_cards = hand.into_iter().collect();
                self.trace.message(MSG_HINT).u8(HINT_SELECTMSG).u8(turn_player).u64(501);
                let to_discard = (hand_size - limit) as u8;
                self.emplace_process(SelectCard {
                    playerid: turn_player,
                    min: to_discard,
                    max: to_discard,
                    ..SelectCard::default()
                });
                false
            }
            21 => {
                if !self.return_cards.list.is_empty() {
                    let cards = self.return_cards.list.iter().copied().collect();
                    let reason = REASON_RULE + REASON_DISCARD + REASON_ADJUST;
                    self.send_to(
                        cards,
                        None,
                        reason,
                        turn_player,
                        PLAYER_NONE,
                        LOCATION_GRAVE,
                        0,
                        POS_FACEUP as u8,
                        false,
                    );
                }
                false
            }
            22 => {
                self.core.hand_adjusted = true;
                self.emplace_process(PointEvent::default());
                arg.step = RESTART;
                arg.is_opponent = false;
                arg.priority_passed = false;
                false
            }
            25 => {
                self.core.hint_timing[turn_player] = 0;
                self.reset_phase(phase);
                self.adjust_all();
                false
            }
            26 => {
                self.core.quick_f_chain.clear();
                self.core.instant_event.clear();
                self.core.point_event.clear();
                self.core.delayed_activate_event.clear();
                self.core.full_event.clear();
                true
            }
            _ => true,
        }
    }

    // The chains that just were built stop being STATUS_CHAINING before they solve.
    fn clear_chaining_status(&mut self) {
        for index in 0..self.core.current_chain.len() {
            if let Some(chain) = self.core.current_chain.get(index) {
                let handler = self.get_handler(chain.triggering_effect);
                self.cards[handler].set_status(STATUS_CHAINING, false);
            }
        }
    }

    // core.ignition_priority_chains: the obsolete ignition effect ruling of step 8.
    fn process_point_event(&mut self, arg: &mut PointEvent) -> bool {
        let turn_player = self.infos.turn_player;
        match arg.step {
            0 => {
                self.core.select_chains.clear();
                let core = &mut self.core;
                core.point_event.append(&mut core.instant_event);
                if arg.skip_trigger {
                    arg.step = 7;
                    return false;
                }
                let core = &mut self.core;
                core.new_fchain.append(&mut core.new_fchain_s);
                std::mem::swap(&mut core.new_fchain, &mut core.new_fchain_s);
                core.new_ochain.append(&mut core.new_ochain_s);
                std::mem::swap(&mut core.new_ochain, &mut core.new_ochain_s);
                core.full_event.append(&mut core.delayed_activate_event);
                self.core.delayed_quick.clear();
                std::mem::swap(&mut self.core.delayed_quick_tmp, &mut self.core.delayed_quick);
                self.core.current_player = turn_player;
                arg.step = 1;
                false
            }
            1 => false,
            2 => {
                self.collect_triggers(true);
                if self.core.select_chains.is_empty() {
                    self.returns.set_i32(0, -1);
                } else {
                    self.segoc_first_trigger();
                    if self.core.select_chains.len() == 1 {
                        self.returns.set_i32(0, 0);
                    } else {
                        let playerid = self.core.current_player;
                        self.emplace_process(SelectChain {
                            playerid,
                            spe_count: 0x7f,
                            forced: true,
                            ..SelectChain::default()
                        });
                    }
                }
                false
            }
            3 => {
                let selected = self.returns.at_i32(0);
                if selected == -1 {
                    if self.core.new_fchain_s.is_empty() {
                        self.core.current_player = turn_player;
                    } else {
                        self.core.current_player = 1 - turn_player;
                        arg.step = 1;
                    }
                    return false;
                }
                let Some(chain) = self.take_select_chain(selected as usize) else {
                    return true;
                };
                self.chain_trigger(chain);
                self.core.new_fchain_s.retain(|other| other.chain_id != chain.chain_id);
                arg.step = 1;
                false
            }
            4 => {
                self.collect_triggers(false);
                if self.core.select_chains.is_empty() {
                    self.returns.set_i32(0, -2);
                    arg.step = 5;
                    return false;
                }
                if self.is_flag(DUEL_TCG_SEGOC_NONPUBLIC) {
                    self.core.new_ochain_h.clear();
                }
                self.segoc_first_trigger();
                let playerid = self.core.current_player;
                if self.core.select_chains.len() == 1 && self.core.current_chain.is_empty() {
                    let effect = self
                        .core
                        .select_chains
                        .first()
                        .map(|chain| chain.triggering_effect)
                        .unwrap_or_default();
                    let handler = self.get_handler(effect);
                    self.field_select_effect_yes_no(playerid, 221, handler);
                } else {
                    self.emplace_process(SelectChain {
                        playerid,
                        spe_count: 0x7f,
                        ..SelectChain::default()
                    });
                    arg.step = 5;
                }
                false
            }
            5 => {
                self.returns.set_i32(0, self.returns.at_i32(0) - 1);
                false
            }
            6 => {
                let selected = self.returns.at_i32(0);
                if selected == -2 || (selected == -1 && !self.is_flag(DUEL_TCG_SEGOC_FIRSTTRIGGER)) {
                    let mut chains = std::mem::take(&mut self.core.select_chains);
                    for chain in chains.drain(..) {
                        self.effects[chain.triggering_effect].active_type = 0;
                        self.core.new_ochain_s.retain(|other| other.chain_id != chain.chain_id);
                    }
                    self.core.select_chains = chains;
                    if self.core.new_ochain_s.is_empty() {
                        self.core.current_player = turn_player;
                        arg.step = 6;
                    } else {
                        self.core.current_player = 1 - turn_player;
                        arg.step = 3;
                    }
                    return false;
                }
                if selected == -1 {
                    let discarded = self.core.select_chains.first().map(|chain| chain.event_id);
                    let current_player = self.core.current_player;
                    self.core
                        .new_ochain_s
                        .retain(|chain| !(Some(chain.event_id) == discarded && chain.triggering_player == current_player));
                    arg.step = 3;
                    return false;
                }
                let Some(chain) = self.take_select_chain(selected as usize) else {
                    return true;
                };
                self.chain_trigger(chain);
                self.core.new_ochain_s.retain(|other| other.chain_id != chain.chain_id);
                self.core.new_ochain_h.retain(|other| other.chain_id != chain.chain_id);
                arg.step = 3;
                false
            }
            7 => {
                self.core.select_chains.clear();
                false
            }
            8 => {
                let main_phase = self.infos.phase == PHASE_MAIN1 || self.infos.phase == PHASE_MAIN2;
                if arg.skip_freechain
                    || !(self.is_flag(DUEL_OCG_OBSOLETE_IGNITION) || self.is_flag(DUEL_TCG_FAST_EFFECT_IGNITION))
                    || !main_phase
                {
                    return false;
                }
                unported!(
                    self,
                    self.field_effects.registry.all(Container::Ignition, None).map(|(_, effect)| effect)
                );
                false
            }
            9 => {
                self.infos.priorities = PerPlayer::default();
                match self.core.current_chain.last() {
                    None => {
                        if !self.core.hand_adjusted {
                            let priority_player = if self.is_flag(DUEL_INVERTED_QUICK_PRIORITY) {
                                1 - turn_player
                            } else {
                                turn_player
                            };
                            self.emplace_process(QuickEffect {
                                skip_freechain: arg.skip_freechain,
                                priority_player,
                                ..QuickEffect::default()
                            });
                        }
                    }
                    Some(chain) => {
                        let priority_player = 1 - chain.triggering_player;
                        self.emplace_process(QuickEffect {
                            skip_freechain: arg.skip_freechain,
                            priority_player,
                            ..QuickEffect::default()
                        });
                    }
                }
                false
            }
            10 => {
                self.core.new_ochain_h.clear();
                self.core.full_event.clear();
                self.core.delayed_quick.clear();
                self.core.chain_limit.clear();
                if self.core.current_chain.is_empty() {
                    self.core.point_event.clear();
                    self.reset_chain();
                    self.returns.set_i32(0, 0);
                } else {
                    self.clear_chaining_status();
                    self.emplace_process(SolveChain {
                        skip_trigger: arg.skip_trigger,
                        skip_freechain: arg.skip_freechain,
                        skip_new: arg.skip_new,
                        ..SolveChain::default()
                    });
                }
                true
            }
            30 | 32 => {
                let check_player = if arg.step == 30 { turn_player } else { 1 - turn_player };
                self.nil_event.event_code = EVENT_FREE_CHAIN;
                self.core.select_chains.clear();
                self.core.spe_effect[check_player] = 0;
                let nil_event = self.nil_event;
                for effect in self.field_effect_range(Container::Continuous, EVENT_FREE_CHAIN) {
                    if self.get_handler_player(effect) == check_player
                        && self.is_activateable(effect, check_player, &nil_event, false, false, false, false, false)
                    {
                        self.push_candidate(effect);
                        self.core.spe_effect[check_player] += 1;
                    }
                }
                if self.core.select_chains.is_empty() {
                    arg.step = if arg.step == 30 { 31 } else { RESTART };
                } else {
                    let spe_count = self.core.spe_effect[check_player] as u8;
                    self.emplace_process(SelectChain {
                        playerid: check_player,
                        spe_count,
                        ..SelectChain::default()
                    });
                }
                false
            }
            31 | 33 => {
                let selected = self.returns.at_i32(0);
                if selected == -1 {
                    if arg.step == 33 {
                        arg.step = RESTART;
                    }
                    return false;
                }
                let Some(chain) = self.take_select_chain(selected as usize) else {
                    return true;
                };
                self.core.select_chains.clear();
                let effect = chain.triggering_effect;
                let nil_event = self.nil_event;
                self.solve_continuous(self.get_handler_player(effect), effect, nil_event);
                arg.step = if arg.step == 31 { 29 } else { 31 };
                false
            }
            _ => true,
        }
    }

    // PointEvent steps 3 and 6: the chosen trigger joins the chain.
    fn chain_trigger(&mut self, chain: Chain) {
        let effect = chain.triggering_effect;
        let handler = self.get_handler(effect);
        self.cards[handler].set_status(STATUS_CHAINING, true);
        self.dec_count(effect, chain.triggering_player);
        self.core.new_chains.push(chain);
        self.emplace_process(AddChain::default());
    }

    fn process_quick_effect(&mut self, arg: &mut QuickEffect) -> bool {
        let priority = arg.priority_player;
        match arg.step {
            0 => {
                self.core.select_chains.clear();
                unported!(self, self.core.quick_f_chain.iter().map(|&(effect, _)| effect));
                self.returns.set_i32(0, -1);
                false
            }
            1 => {
                if self.returns.at_i32(0) == -1 {
                    if !self.core.quick_f_chain.is_empty() {
                        arg.is_opponent = true;
                        arg.step = RESTART;
                    } else if let Some(chain) = self.core.new_chains.last() {
                        let priority_player = 1 - chain.triggering_player;
                        self.emplace_process(AddChain::default());
                        self.emplace_process(QuickEffect {
                            priority_player,
                            ..QuickEffect::default()
                        });
                        self.infos.priorities = PerPlayer::default();
                        return true;
                    }
                    return false;
                }
                unported!(self, self.core.select_chains.first().map(|chain| chain.triggering_effect));
                arg.step = RESTART;
                false
            }
            2 => {
                if !self.core.ignition_priority_chains.is_empty() {
                    std::mem::swap(&mut self.core.select_chains, &mut self.core.ignition_priority_chains);
                }
                let mut index = 0;
                while let Some(event) = self.core.point_event.iter().chain(&self.core.instant_event).nth(index).copied() {
                    index += 1;
                    for container in [Container::Activate, Container::QuickO] {
                        for effect in self.field_effect_range(container, event.event_code) {
                            self.set_activate_location(effect);
                            if !self.effects[effect].is_flag(EFFECT_FLAG_DELAY)
                                && self.is_chainable(effect, priority)
                                && self.is_activateable(effect, priority, &event, false, false, false, false, false)
                            {
                                let chain = self.chain_candidate(effect, event, priority);
                                self.core.select_chains.push(chain);
                            }
                        }
                    }
                }
                for index in 0..self.core.new_ochain_h.len() {
                    let Some(mut chain) = self.core.new_ochain_h.get(index).copied() else {
                        break;
                    };
                    let effect = chain.triggering_effect;
                    let handler = self.get_handler(effect);
                    let peffect = &self.effects[effect];
                    if !peffect.is_flag(EFFECT_FLAG_FIELD_ONLY)
                        && peffect.effect_type & EFFECT_TYPE_FIELD != 0
                        && peffect.range & u16::from(LOCATION_HAND) != 0
                        && self.cards[handler].current.location == LOCATION_HAND
                    {
                        let controler = self.cards[handler].current.controler;
                        if !self.is_has_relation(handler, effect, chain.chain_id) && self.is_condition_check(effect, controler, &chain.evt)
                        {
                            self.create_relation(handler, effect, chain.chain_id);
                        }
                        self.set_activate_location(effect);
                        chain.triggering_player = controler;
                        self.set_triggering_state(&mut chain, handler);
                        if let Some(stored) = self.core.new_ochain_h.get_mut(index) {
                            *stored = chain;
                        }
                    }
                    if chain.triggering_player == priority
                        && !self.cards[handler].is_status(STATUS_CHAINING)
                        && ((chain.triggering_location == u16::from(LOCATION_HAND) && self.cards[handler].is_position(POS_FACEDOWN))
                            || chain.triggering_location == u16::from(LOCATION_DECK))
                        && self.is_has_relation(handler, effect, chain.chain_id)
                        && self.is_chainable(effect, priority)
                        && self.is_activateable(effect, priority, &chain.evt, true, false, false, false, false)
                        && self.check_spself_from_hand_trigger(&chain)
                    {
                        self.core.select_chains.push(chain);
                    }
                }
                for index in 0..self.core.full_event.len() {
                    let Some(event) = self.core.full_event.get(index).copied() else {
                        break;
                    };
                    for effect in self.field_effect_range(Container::Activate, event.event_code) {
                        self.set_activate_location(effect);
                        if self.effects[effect].is_flag(EFFECT_FLAG_DELAY)
                            && self.is_chainable(effect, priority)
                            && self.is_activateable(effect, priority, &event, false, false, false, false, false)
                        {
                            let chain = self.chain_candidate(effect, event, priority);
                            self.core.select_chains.push(chain);
                        }
                    }
                }
                for (effect, event) in self.core.delayed_quick.clone() {
                    self.set_activate_location(effect);
                    if self.is_chainable(effect, priority)
                        && self.is_activateable(effect, priority, &event, true, false, false, false, false)
                    {
                        let chain = self.chain_candidate(effect, event, priority);
                        self.core.select_chains.push(chain);
                    }
                }
                self.core.spe_effect[priority] = self.core.select_chains.len() as i32;
                if !arg.skip_freechain {
                    self.nil_event.event_code = EVENT_FREE_CHAIN;
                    let nil_event = self.nil_event;
                    for (container, cteffect) in [(Container::Activate, true), (Container::QuickO, false)] {
                        for effect in self.field_effect_range(container, EVENT_FREE_CHAIN) {
                            self.set_activate_location(effect);
                            if !self.is_chainable(effect, priority)
                                || !self.is_activateable(effect, priority, &nil_event, false, false, false, false, false)
                            {
                                continue;
                            }
                            let chain = self.chain_candidate(effect, nil_event, priority);
                            self.core.select_chains.push(chain);
                            if self.check_hint_timing(effect) || (cteffect && self.check_cteffect_hint(effect, priority)) {
                                self.core.spe_effect[priority] += 1;
                            }
                        }
                    }
                }
                let hint_timing = self.core.hint_timing;
                if !self.core.current_chain.is_empty() || (hint_timing[0] | hint_timing[1]) & TIMING_ATTACK != 0 {
                    self.core.spe_effect[priority] = self.core.select_chains.len() as i32;
                }
                let spe_count = self.core.spe_effect[priority] as u8;
                self.emplace_process(SelectChain {
                    playerid: priority,
                    spe_count,
                    ..SelectChain::default()
                });
                false
            }
            3 => {
                let selected = self.returns.at_i32(0);
                if !self.core.select_chains.is_empty() && selected != -1 {
                    let Some(chain) = self.take_select_chain(selected as usize) else {
                        return true;
                    };
                    let effect = chain.triggering_effect;
                    self.core
                        .delayed_quick
                        .retain(|&(other, event)| (other, event) != (effect, chain.evt));
                    self.core.new_chains.push(chain);
                    let handler = self.get_handler(effect);
                    self.cards[handler].set_status(STATUS_CHAINING, true);
                    self.dec_count(effect, priority);
                    self.emplace_process(AddChain::default());
                    self.emplace_process(QuickEffect {
                        priority_player: 1 - priority,
                        ..QuickEffect::default()
                    });
                    self.infos.priorities = PerPlayer::default();
                } else {
                    self.infos.priorities[priority] = 1;
                    if self.infos.priorities[0] == 0 || self.infos.priorities[1] == 0 {
                        self.emplace_process(QuickEffect {
                            step: 1,
                            skip_freechain: arg.skip_freechain,
                            priority_player: 1 - priority,
                            ..QuickEffect::default()
                        });
                    } else {
                        self.core.hint_timing[0] &= TIMING_DAMAGE_STEP | TIMING_DAMAGE_CAL;
                        self.core.hint_timing[1] &= TIMING_DAMAGE_STEP | TIMING_DAMAGE_CAL;
                    }
                }
                self.core.select_chains.clear();
                true
            }
            _ => true,
        }
    }

    // IdleCommand step 1 and BattleCommand step 1: an activation chosen from the command builds a chain (the
    // continuous effects solve at once). Returns whether it was a continuous effect.
    fn activate_from_command(&mut self, selected: usize) -> bool {
        let turn_player = self.infos.turn_player;
        let Some(mut chain) = self.take_select_chain(selected) else {
            return false;
        };
        let effect = chain.triggering_effect;
        if self.effects[effect].effect_type & EFFECT_TYPE_CONTINUOUS != 0 {
            self.core.select_chains.clear();
            let nil_event = self.nil_event;
            self.solve_continuous(self.get_handler_player(effect), effect, nil_event);
            return true;
        }
        let handler = self.get_handler(effect);
        chain.flag = 0;
        chain.chain_id = self.next_field_id() as u16;
        chain.evt = crate::effect::Event {
            event_code: self.effects[effect].code,
            event_player: PLAYER_NONE,
            reason_player: PLAYER_NONE,
            ..crate::effect::Event::default()
        };
        self.set_triggering_state(&mut chain, handler);
        chain.triggering_player = turn_player;
        self.core.new_chains.push(chain);
        self.cards[handler].set_status(STATUS_CHAINING, true);
        self.dec_count(effect, turn_player);
        self.core.select_chains.clear();
        self.emplace_process(AddChain::default());
        self.emplace_process(QuickEffect {
            priority_player: 1 - turn_player,
            ..QuickEffect::default()
        });
        self.infos.priorities = PerPlayer::default();
        false
    }

    // The free chain effects the turn player can activate from the idle and battle commands.
    fn collect_command_activations(&mut self, battle: bool) {
        let turn_player = self.infos.turn_player;
        let nil_event = self.nil_event;
        for effect in self.field_effect_range(Container::Activate, EVENT_FREE_CHAIN) {
            self.set_activate_location(effect);
            if self.is_activateable(effect, turn_player, &nil_event, false, false, false, false, false)
                && (!battle || self.get_speed(effect) > 1)
            {
                self.push_candidate(effect);
            }
        }
        for effect in self.field_effect_range(Container::QuickO, EVENT_FREE_CHAIN) {
            self.set_activate_location(effect);
            if self.is_activateable(effect, turn_player, &nil_event, false, false, false, false, false) {
                self.push_candidate(effect);
            }
        }
        for effect in self.field_effect_range(Container::Continuous, EVENT_FREE_CHAIN) {
            if self.get_handler_player(effect) == turn_player
                && self.is_activateable(effect, turn_player, &nil_event, false, false, false, false, false)
            {
                self.push_candidate(effect);
            }
        }
        if !battle {
            let ignition: crate::effect::EffectSet = self
                .field_effects
                .registry
                .all(Container::Ignition, None)
                .map(|(_, effect)| effect)
                .collect();
            for effect in ignition {
                self.set_activate_location(effect);
                if self.is_activateable(effect, turn_player, &nil_event, false, false, false, false, false) {
                    self.push_candidate(effect);
                }
            }
        }
    }

    fn process_idle_command(&mut self, arg: &mut IdleCommand) -> bool {
        let turn_player = self.infos.turn_player;
        let phase = self.infos.phase;
        let selected = (self.returns.at_i32(0) >> 16) as usize;
        match arg.step {
            0 => {
                self.core.select_chains.clear();
                self.nil_event.event_code = EVENT_FREE_CHAIN;
                self.core.to_bp = true;
                self.core.to_ep = true;
                if (!self.is_flag(DUEL_ATTACK_FIRST_TURN)
                    && self.infos.turn_id == 1
                    && self.is_player_affected_by_effect(turn_player, EFFECT_BP_FIRST_TURN).is_none())
                    || phase == PHASE_MAIN2
                    || self.is_player_affected_by_effect(turn_player, EFFECT_CANNOT_BP).is_some()
                    || self.core.force_turn_end
                {
                    self.core.to_bp = false;
                }
                if phase == PHASE_MAIN1 {
                    let must_attack = self.player[turn_player]
                        .list_mzone
                        .into_iter()
                        .flatten()
                        .any(|card| self.is_capable_attack(card) && self.is_affected_by_effect(card, EFFECT_MUST_ATTACK).is_some());
                    if self.core.to_bp && (must_attack || self.is_player_affected_by_effect(turn_player, EFFECT_CANNOT_EP).is_some()) {
                        self.core.to_ep = false;
                    }
                }
                let skip = |duel: &Self, code| duel.is_player_affected_by_effect(turn_player, code).is_some();
                if (phase == PHASE_MAIN1 && skip(self, EFFECT_SKIP_M1))
                    || (phase == PHASE_MAIN2 && skip(self, EFFECT_SKIP_M2))
                    || self.core.force_turn_end
                {
                    if self.core.to_bp && self.core.to_ep {
                        self.core.select_options = [80, 81].into_iter().collect();
                        self.emplace_process(SelectOption {
                            playerid: turn_player,
                            ..SelectOption::default()
                        });
                        arg.step = 11;
                    } else {
                        arg.phase_to_change_to = if self.core.to_bp { 6 } else { 7 };
                        arg.step = 10;
                        self.reset_phase(phase);
                        self.adjust_all();
                    }
                    return false;
                }
                if phase == PHASE_MAIN2 && self.core.skip_m2 {
                    self.core.skip_m2 = false;
                    self.returns.set_i32(0, 7);
                    return false;
                }
                self.collect_command_activations(false);
                self.core.summonable_cards.clear();
                let mzone = self.player[turn_player].list_mzone;
                for card in self.player[turn_player]
                    .list_hand
                    .clone()
                    .into_iter()
                    .chain(mzone.into_iter().flatten())
                {
                    if self.is_can_be_summoned(card, turn_player, false, None, 0, 0x1f) {
                        self.core.summonable_cards.push(card);
                    }
                }
                self.core.spsummonable_cards.clear();
                for effect in self.filter_field_effect_list::<64>(EFFECT_SPSUMMON_PROC) {
                    let card = self.get_handler(effect);
                    let controler = self.cards[card].current.controler;
                    if self.check_count_limit(effect, controler)
                        && controler == turn_player
                        && self.is_special_summonable(
                            card,
                            turn_player,
                            0,
                            Procedure {
                                card,
                                ..Procedure::default()
                            },
                        )
                    {
                        self.core.spsummonable_cards.push(card);
                    }
                }
                for effect in self.filter_field_effect(EFFECT_SPSUMMON_PROC_G) {
                    let card = self.get_handler(effect);
                    if !self.check_count_limit(effect, turn_player)
                        || (self.cards[card].current.controler != turn_player && !self.effects[effect].is_flag(EFFECT_FLAG_BOTH_SIDE))
                    {
                        continue;
                    }
                    if self.check_procedure_condition(card, effect) {
                        self.core.spsummonable_cards.push(card);
                    }
                }
                self.core.repositionable_cards.clear();
                for card in mzone.into_iter().flatten() {
                    let target = &self.cards[card];
                    if (target.is_position(POS_FACEUP | POS_FACEDOWN_ATTACK) && self.is_capable_change_position(card, turn_player))
                        || (target.is_position(POS_FACEDOWN) && self.is_can_be_flip_summoned(card, turn_player))
                    {
                        self.core.repositionable_cards.push(card);
                    }
                }
                self.core.msetable_cards.clear();
                self.core.ssetable_cards.clear();
                for card in self.player[turn_player].list_hand.clone() {
                    if self.is_setable_mzone(card, turn_player, false, None, 0, 0x1f) {
                        self.core.msetable_cards.push(card);
                    }
                    if self.is_setable_szone(card, turn_player, false) {
                        self.core.ssetable_cards.push(card);
                    }
                }
                self.emplace_process(SelectIdleCmd {
                    playerid: turn_player,
                    ..SelectIdleCmd::default()
                });
                false
            }
            1 => {
                let response = self.returns.at_i32(0);
                let (kind, index) = (response & 0xffff, response >> 16);
                match kind {
                    5 => {
                        if self.activate_from_command(index as usize) {
                            arg.step = 2;
                        }
                        false
                    }
                    0..=4 => {
                        arg.step = kind as u16 + 4;
                        false
                    }
                    8 => {
                        arg.step = RESTART;
                        self.shuffle(turn_player, LOCATION_HAND);
                        self.infos.can_shuffle = false;
                        false
                    }
                    _ => {
                        arg.step = 9;
                        self.trace.message(MSG_HINT).u8(HINT_EVENT).u8(1 - turn_player).u64(23);
                        self.core.select_chains.clear();
                        self.core.hint_timing[turn_player] = TIMING_MAIN_END;
                        self.emplace_process(QuickEffect {
                            priority_player: 1 - turn_player,
                            ..QuickEffect::default()
                        });
                        self.infos.priorities[turn_player] = 1;
                        self.infos.priorities[1 - turn_player] = 0;
                        arg.phase_to_change_to = kind as u8;
                        false
                    }
                }
            }
            2 => {
                self.core.chain_limit.clear();
                self.clear_chaining_status();
                self.emplace_process(SolveChain::default());
                arg.step = RESTART;
                false
            }
            3 => {
                self.adjust_instant();
                self.emplace_process(PointEvent::default());
                arg.step = RESTART;
                false
            }
            5 => {
                let Some(&target) = self.core.summonable_cards.get(selected) else {
                    return true;
                };
                self.core.summon_cancelable = true;
                self.field_summon(turn_player, target, None, false, 0, 0x1f);
                arg.step = RESTART;
                false
            }
            6 => {
                let Some(&target) = self.core.spsummonable_cards.get(selected) else {
                    return true;
                };
                self.core.summon_cancelable = true;
                self.special_summon_rule(turn_player, target, 0, None);
                arg.step = RESTART;
                false
            }
            7 => {
                let Some(&target) = self.core.repositionable_cards.get(selected) else {
                    return true;
                };
                let card = &self.cards[target];
                if card.is_position(POS_FACEUP_ATTACK) || card.is_position(POS_FACEUP_DEFENSE) {
                    let npos = if card.is_position(POS_FACEUP_ATTACK) {
                        POS_FACEUP_DEFENSE
                    } else {
                        POS_FACEUP_ATTACK
                    };
                    self.reposition(target, npos);
                } else if card.is_position(POS_FACEDOWN_ATTACK) {
                    arg.card_to_reposition = Some(target);
                    let mut positions = 0;
                    if self.is_capable_change_position(target, turn_player) {
                        positions |= POS_FACEDOWN_DEFENSE;
                    }
                    if self.is_can_be_flip_summoned(target, turn_player) {
                        positions |= POS_FACEUP_ATTACK;
                    }
                    let code = self.cards[target].data.code;
                    self.emplace_process(SelectPosition {
                        playerid: turn_player,
                        code,
                        positions: positions as u8,
                        ..SelectPosition::default()
                    });
                    arg.step = 12;
                    return false;
                } else {
                    let sumplayer = card.current.controler;
                    self.emplace_process(FlipSummon {
                        step: 0,
                        sumplayer,
                        target,
                    });
                }
                self.cards[target].set_status(STATUS_FORM_CHANGED, true);
                arg.step = RESTART;
                false
            }
            8 => {
                let Some(&target) = self.core.msetable_cards.get(selected) else {
                    return true;
                };
                self.core.summon_cancelable = true;
                self.mset(self.cards[target].current.controler, target, None, false, 0, 0x1f);
                arg.step = RESTART;
                false
            }
            9 => {
                let Some(&target) = self.core.ssetable_cards.get(selected) else {
                    return true;
                };
                let controler = self.cards[target].current.controler;
                self.emplace_process(SpellSet {
                    step: 0,
                    setplayer: controler,
                    toplayer: controler,
                    target,
                    reason_effect: None,
                });
                arg.step = RESTART;
                false
            }
            10 => {
                self.core.chain_limit.clear();
                if !self.core.current_chain.is_empty() {
                    self.clear_chaining_status();
                    self.emplace_process(SolveChain::default());
                    arg.step = RESTART;
                    return false;
                }
                self.reset_phase(phase);
                self.adjust_all();
                false
            }
            11 => {
                self.returns.set_i32(0, i32::from(arg.phase_to_change_to));
                self.infos.can_shuffle = true;
                true
            }
            12 => {
                arg.phase_to_change_to = if self.returns.at_i32(0) == 0 { 6 } else { 7 };
                self.reset_phase(phase);
                self.adjust_all();
                arg.step = 10;
                false
            }
            13 => {
                let Some(target) = arg.card_to_reposition else { return true };
                if self.returns.at_i32(0) == POS_FACEUP_ATTACK as i32 {
                    let sumplayer = self.cards[target].current.controler;
                    self.emplace_process(FlipSummon {
                        step: 0,
                        sumplayer,
                        target,
                    });
                } else {
                    self.reposition(target, POS_FACEDOWN_DEFENSE);
                }
                self.cards[target].set_status(STATUS_FORM_CHANGED, true);
                arg.step = RESTART;
                false
            }
            _ => true,
        }
    }

    // IdleCommand's change of position by the turn player, followed by an adjust and a chain window.
    fn reposition(&mut self, target: CardId, npos: u32) {
        self.core.phase_action = true;
        self.field_change_position(target, None, self.infos.turn_player, npos, 0, false);
        self.adjust_all();
        self.emplace_process(PointEvent::default());
    }

    // Steps 0, 1 and 40-42 run the Battle Step's command loop, process_attack the attacks in between. Step 2 resolves
    // a chain activated from the command, steps 14 and 15 a continuous effect.
    fn process_battle_command(&mut self, arg: &mut BattleCommand) -> bool {
        let turn_player = self.infos.turn_player;
        match arg.step {
            0 => {
                self.core.select_chains.clear();
                self.nil_event.event_code = EVENT_FREE_CHAIN;
                if !self.core.chain_attack {
                    self.core.chain_attacker_id = 0;
                    self.core.chain_attack_target = None;
                }
                self.core.attack_player = false;
                self.core.attacker = None;
                self.core.attack_target = None;
                let skip_bp = self.is_player_affected_by_effect(turn_player, EFFECT_SKIP_BP);
                if skip_bp.is_some() || self.core.force_turn_end {
                    arg.step = 41;
                    arg.phase_to_change_to = 2;
                    let bp_twice = self.is_player_affected_by_effect(turn_player, EFFECT_BP_TWICE);
                    unported!(self, bp_twice);
                    arg.repeat_battle_phase = bp_twice.is_some();
                    if self.core.force_turn_end || skip_bp.is_some_and(|effect| matches!(self.effects[effect].value, Value::Int(0))) {
                        self.reset_phase(PHASE_BATTLE_STEP);
                        self.adjust_all();
                        self.infos.phase = PHASE_BATTLE;
                        self.emplace_process(PhaseEvent {
                            phase: PHASE_BATTLE,
                            ..PhaseEvent::default()
                        });
                    } else {
                        self.core.hint_timing[turn_player] = 0;
                        self.reset_phase(PHASE_BATTLE);
                        self.adjust_all();
                    }
                    return false;
                }
                self.collect_command_activations(true);
                self.core.attackable_cards.clear();
                let mut must_attack = false;
                if self
                    .is_player_affected_by_effect(turn_player, EFFECT_CANNOT_ATTACK_ANNOUNCE)
                    .is_none()
                {
                    for card in self.player[turn_player].list_mzone.into_iter().flatten() {
                        if !self.is_capable_attack_announce(card, turn_player) {
                            continue;
                        }
                        let chain_attack = self.core.chain_attack && self.core.chain_attacker_id == self.cards[card].fieldid;
                        if self.field_get_attack_target(card, chain_attack, true).is_empty() && self.cards[card].direct_attackable == 0 {
                            continue;
                        }
                        self.core.attackable_cards.push(card);
                        // EFFECT_FIRST_ATTACK narrows the list.
                        unported!(self, self.is_affected_by_effect(card, EFFECT_FIRST_ATTACK));
                        must_attack |= self.is_affected_by_effect(card, EFFECT_MUST_ATTACK).is_some();
                    }
                }
                self.core.to_m2 = !self.is_flag(DUEL_NO_MAIN_PHASE_2);
                self.core.to_ep = true;
                if must_attack || self.is_player_affected_by_effect(turn_player, EFFECT_CANNOT_M2).is_some() || self.core.force_turn_end {
                    self.core.to_m2 = false;
                }
                if must_attack {
                    self.core.to_ep = false;
                }
                self.core.attack_cancelable = true;
                self.core.attack_cost_paid = 0;
                self.emplace_process(SelectBattleCmd {
                    playerid: turn_player,
                    ..SelectBattleCmd::default()
                });
                false
            }
            1 => {
                let response = self.returns.at_i32(0);
                let (kind, index) = (response & 0xffff, response >> 16);
                if arg.forced_attack_done || (kind != 0 && kind != 1) {
                    arg.step = 39;
                    arg.phase_to_change_to = kind as u16;
                    self.trace.message(MSG_HINT).u8(HINT_EVENT).u8(1 - turn_player).u64(29);
                    self.core.select_chains.clear();
                    self.core.hint_timing[turn_player] = TIMING_BATTLE_STEP_END;
                    self.emplace_process(QuickEffect {
                        priority_player: 1 - turn_player,
                        ..QuickEffect::default()
                    });
                    self.infos.priorities[turn_player] = 1;
                    self.infos.priorities[1 - turn_player] = 0;
                    return false;
                }
                if kind == 0 {
                    if self.activate_from_command(index as usize) {
                        arg.step = 14;
                    }
                    return false;
                }
                if kind == 1 {
                    let Some(&attacker) = usize::try_from(index).ok().and_then(|index| self.core.attackable_cards.get(index)) else {
                        return true;
                    };
                    arg.step = 2;
                    arg.attack_announce_failed = false;
                    self.core.attacker = Some(attacker);
                    let card = &mut self.cards[attacker];
                    card.set_status(STATUS_ATTACK_CANCELED, false);
                    card.attack_controler = card.current.controler;
                    self.core.pre_field[0] = card.fieldid_r;
                    if self.core.chain_attack && self.core.chain_attacker_id != card.fieldid {
                        self.core.chain_attack = false;
                        self.core.chain_attacker_id = 0;
                    }
                    // The attack costs of EFFECT_ATTACK_COST run as core.tpchain here and in step 14.
                    unported!(self, self.filter_player_effect(turn_player, EFFECT_ATTACK_COST, false));
                    unported!(self, self.filter_effect(attacker, EFFECT_ATTACK_COST, true));
                }
                false
            }
            2 => {
                self.core.chain_limit.clear();
                self.clear_chaining_status();
                self.emplace_process(SolveChain::default());
                arg.step = RESTART;
                false
            }
            15 => {
                self.adjust_instant();
                self.emplace_process(PointEvent::default());
                arg.step = RESTART;
                false
            }
            3..=39 => {
                let Some(attacker) = self.core.attacker else { return true };
                self.process_attack(arg, attacker)
            }
            40 => {
                self.core.chain_limit.clear();
                if !self.core.current_chain.is_empty() {
                    self.clear_chaining_status();
                    self.emplace_process(SolveChain::default());
                    arg.step = RESTART;
                    return false;
                }
                self.reset_phase(PHASE_BATTLE_STEP);
                self.adjust_all();
                false
            }
            41 => {
                arg.second_battle_phase_is_optional = true;
                let bp_twice = self.filter_player_effect(turn_player, EFFECT_BP_TWICE, false);
                unported!(self, bp_twice.iter().copied());
                arg.repeat_battle_phase = !bp_twice.is_empty();
                self.infos.phase = PHASE_BATTLE;
                self.emplace_process(PhaseEvent {
                    phase: PHASE_BATTLE,
                    ..PhaseEvent::default()
                });
                self.adjust_all();
                false
            }
            42 => {
                self.core.attacker = None;
                self.core.attack_target = None;
                // The optional second Battle Phase prompt (SelectYesNo, then step 43) needs EFFECT_BP_TWICE, phase 2.
                self.returns.set_i32(0, i32::from(arg.phase_to_change_to));
                self.returns.set_i32(1, i32::from(arg.repeat_battle_phase));
                true
            }
            _ => true,
        }
    }

    // One attack, from its declaration to the end of its damage step. Steps 14-16 (attack costs, continuous effects,
    // EFFECT_MUST_ATTACK_MONSTER) and core.effect_damage_step (Duel.CalculateDamage) need effects (phase 2).
    fn process_attack(&mut self, arg: &mut BattleCommand, attacker: CardId) -> bool {
        let turn_player = self.infos.turn_player;
        let attack_target = self.core.attack_target;
        let canceled = self.cards[attacker].is_status(STATUS_ATTACK_CANCELED);
        match arg.step {
            3 => {
                arg.is_replaying_attack = false;
                if canceled {
                    arg.attack_announce_failed = true;
                    arg.step = 6;
                } else if self.core.attack_cost_paid != 1 && !self.core.attack_cancelable {
                    arg.step = RESTART;
                }
                false
            }
            4 => {
                self.core.attack_player = false;
                self.return_cards.clear();
                self.core.select_cards = self.field_get_attack_target(attacker, self.core.chain_attack, true);
                // EFFECT_PATRICIAN_OF_DARKNESS hands the choices to the opponent.
                let patrician = self.is_player_affected_by_effect(turn_player, EFFECT_PATRICIAN_OF_DARKNESS);
                if self.cards[attacker].direct_attackable != 0 {
                    if self.core.select_cards.is_empty() {
                        self.returns.set_i32(0, -2);
                        arg.step = 5;
                        return false;
                    }
                    unported!(self, patrician);
                    self.emplace_process(SelectYesNo {
                        playerid: turn_player,
                        description: 31,
                        ..SelectYesNo::default()
                    });
                    return false;
                }
                if self.core.select_cards.is_empty() {
                    arg.attack_announce_failed = true;
                    arg.step = 6;
                    return false;
                }
                unported!(self, patrician);
                self.select_attack_target();
                arg.step = 5;
                false
            }
            5 => {
                if self.returns.at_i32(0) != 0 {
                    self.returns.set_i32(0, -2);
                } else if !self.core.select_cards.is_empty() {
                    unported!(self, self.is_player_affected_by_effect(turn_player, EFFECT_PATRICIAN_OF_DARKNESS));
                    self.select_attack_target();
                } else {
                    arg.attack_announce_failed = true;
                    arg.step = 6;
                }
                false
            }
            6 => {
                if self.return_cards.canceled {
                    arg.step = if arg.is_replaying_attack { 12 } else { RESTART };
                    return false;
                }
                let target = if self.returns.at_i32(0) == -2 {
                    None
                } else {
                    self.return_cards.list.first().copied()
                };
                self.core.attack_target = target;
                self.core.pre_field[1] = target.map_or(0, |target| self.cards[target].fieldid_r);
                false
            }
            7 => {
                if !arg.is_replaying_attack {
                    // Phase 2: check_card_counter counts the attack for the activity counters of scripts.
                    self.core.phase_action = true;
                    self.core.attack_state_count[turn_player] += 1;
                    self.cards[attacker].attack_announce_count += 1;
                }
                if arg.attack_announce_failed {
                    self.cards[attacker].announce_count += 1;
                    self.core.chain_attack = false;
                    arg.step = RESTART;
                }
                false
            }
            8 => {
                self.core.attack_cancelable = true;
                let target_location = attack_target.map_or(LocInfo::default(), |target| self.cards.get_info_location(target));
                self.trace
                    .message(MSG_ATTACK)
                    .loc(self.cards.get_info_location(attacker))
                    .loc(target_location);
                if let Some(target) = attack_target {
                    self.field_raise_single_event(target, EVENT_BE_BATTLE_TARGET, None, 0, 0, 1 - turn_player, 0);
                    self.field_raise_event(Some(target.into()), EVENT_BE_BATTLE_TARGET, None, 0, 0, 1 - turn_player, 0);
                }
                self.core.attack_rollback = false;
                self.core.opp_mzone = self.opponent_field_ids();
                if !arg.is_replaying_attack {
                    self.field_raise_single_event(attacker, EVENT_ATTACK_ANNOUNCE, None, 0, 0, turn_player, 0);
                    self.field_raise_event(Some(attacker.into()), EVENT_ATTACK_ANNOUNCE, None, 0, 0, turn_player, 0);
                }
                let card = &mut self.cards[attacker];
                card.attack_controler = card.current.controler;
                self.core.pre_field[0] = card.fieldid_r;
                self.process_single_event();
                self.process_instant_event();
                self.core.hint_timing[turn_player] = TIMING_ATTACK;
                self.emplace_process(PointEvent::default());
                false
            }
            9 => {
                if self.is_player_affected_by_effect(turn_player, EFFECT_SKIP_BP).is_some() || canceled || self.core.attack_rollback {
                    arg.step = 10;
                    return false;
                }
                self.hint_both(24);
                self.core.hint_timing = PerPlayer([TIMING_BATTLE_PHASE; 2]);
                self.emplace_process(PointEvent {
                    step: 30,
                    ..PointEvent::default()
                });
                false
            }
            10 => {
                if self.returns.at_i32(0) != 0 {
                    arg.step = 8;
                } else {
                    self.adjust_all();
                }
                false
            }
            11 => {
                let mut canceled = canceled;
                if self.is_affected_by_effect(attacker, EFFECT_ATTACK_DISABLED).is_some() {
                    self.reset(attacker, EFFECT_ATTACK_DISABLED, RESET_CODE);
                    self.trace.message(MSG_ATTACK_DISABLED);
                    self.cards[attacker].set_status(STATUS_ATTACK_CANCELED, true);
                    canceled = true;
                }
                if self.is_player_affected_by_effect(turn_player, EFFECT_SKIP_BP).is_some() || canceled {
                    arg.step = 12;
                    return false;
                }
                if !self.core.attack_rollback {
                    self.cards[attacker].announce_count += 1;
                    self.add_announced_card(attacker, attack_target);
                    self.attack_all_target_check();
                    arg.step = 18;
                    return false;
                }
                let targets = self.field_get_attack_target(attacker, self.core.chain_attack, true);
                if targets.is_empty() && self.cards[attacker].direct_attackable == 0 {
                    arg.step = 12;
                    return false;
                }
                if self.is_flag(DUEL_STORE_ATTACK_REPLAYS) && !self.core.chain_attack {
                    self.returns.set_i32(0, 0);
                } else if self.is_affected_by_effect(attacker, EFFECT_MUST_ATTACK).is_none() {
                    self.emplace_process(SelectYesNo {
                        playerid: turn_player,
                        description: 30,
                        ..SelectYesNo::default()
                    });
                } else {
                    self.returns.set_i32(0, 1);
                    self.core.attack_cancelable = false;
                }
                false
            }
            12 => {
                if self.returns.at_i32(0) != 0 {
                    arg.is_replaying_attack = true;
                    arg.attack_announce_failed = false;
                    arg.step = 3;
                }
                false
            }
            13 => {
                let replays_stored = self.is_flag(DUEL_STORE_ATTACK_REPLAYS) && !self.core.chain_attack;
                if self.cards[attacker].fieldid_r == self.core.pre_field[0] && (!replays_stored || canceled) {
                    self.cards[attacker].announce_count += 1;
                    self.add_announced_card(attacker, attack_target);
                    self.attack_all_target_check();
                }
                self.core.chain_attack = false;
                arg.step = RESTART;
                self.reset_phase(PHASE_DAMAGE);
                self.adjust_all();
                false
            }
            19 => {
                self.infos.phase = PHASE_DAMAGE;
                self.core.chain_attack = false;
                arg.is_replaying_attack = false;
                self.core.damage_calculated = false;
                self.core.selfdes_disabled = true;
                self.core.flip_delayed = true;
                let card = &mut self.cards[attacker];
                card.attack_controler = card.current.controler;
                card.attacked_count += 1;
                self.core.pre_field = [card.fieldid_r, 0];
                if let Some(target) = attack_target {
                    let card = &mut self.cards[target];
                    card.attack_controler = card.current.controler;
                    self.core.pre_field[1] = card.fieldid_r;
                }
                self.core.battled_count[turn_player] += 1;
                self.adjust_all();
                false
            }
            20 => {
                self.trace.message(MSG_DAMAGE_STEP_START);
                self.raise_battle_events(EVENT_BATTLE_START, attacker, attack_target);
                let triggers = !self.core.new_fchain.is_empty() || !self.core.new_ochain.is_empty();
                arg.previous_point_event_had_any_trigger_to_resolve = false;
                if !self.is_flag(DUEL_6_STEP_BATLLE_STEP) || triggers {
                    arg.previous_point_event_had_any_trigger_to_resolve = triggers;
                    self.battle_window(40, false);
                }
                false
            }
            21 => {
                if canceled {
                    arg.step = 33;
                    return false;
                }
                let Some(target) = attack_target else { return false };
                for card in [attacker, target] {
                    self.cards[card].temp.position = self.cards[card].current.position;
                }
                if self.cards[target].is_position(POS_FACEDOWN) {
                    self.field_change_position(target, None, PLAYER_NONE, self.cards[target].current.position >> 1, 0, true);
                    self.adjust_all();
                }
                false
            }
            22 => {
                if let Some(target) = attack_target.filter(|&target| self.cards[target].temp.position & POS_FACEDOWN != 0) {
                    self.core.pre_field[1] = self.cards[target].fieldid_r;
                }
                self.raise_battle_events(EVENT_BATTLE_CONFIRM, attacker, attack_target);
                let triggers = !self.core.new_fchain.is_empty() || !self.core.new_ochain.is_empty();
                if !self.is_flag(DUEL_6_STEP_BATLLE_STEP) || !arg.previous_point_event_had_any_trigger_to_resolve || triggers {
                    self.core.hint_timing[turn_player] = TIMING_DAMAGE_STEP;
                    self.battle_window(41, false);
                }
                false
            }
            23 => {
                if canceled {
                    arg.step = 33;
                    return false;
                }
                self.infos.phase = PHASE_DAMAGE_CAL;
                self.adjust_all();
                false
            }
            24 => {
                self.calculate_battle_damage();
                self.raise_battle_events(EVENT_PRE_DAMAGE_CALCULATE, attacker, attack_target);
                self.core.hint_timing[turn_player] = TIMING_DAMAGE_CAL;
                self.battle_window(42, false);
                false
            }
            25 => {
                if canceled {
                    self.reset_phase(PHASE_DAMAGE_CAL);
                    self.adjust_all();
                    self.infos.phase = PHASE_DAMAGE;
                    arg.step = 33;
                }
                false
            }
            // Duel.CalculateDamage enters here.
            26 => {
                let (aa, ad) = (self.get_attack(attacker), self.get_defense(attacker));
                let pa = self.cards[attacker].current.controler;
                self.cards[attacker].set_status(STATUS_BATTLE_RESULT | STATUS_BATTLE_DESTROYED, false);
                let mut defender = (LocInfo::default(), 0, 0);
                if let Some(target) = attack_target {
                    defender = (
                        self.cards.get_info_location(target),
                        self.get_attack(target),
                        self.get_defense(target),
                    );
                    self.cards[target].set_status(STATUS_BATTLE_RESULT | STATUS_BATTLE_DESTROYED, false);
                    if pa != self.cards[target].current.controler {
                        self.cards[attacker].set_status(STATUS_OPPO_BATTLE, true);
                        self.cards[target].set_status(STATUS_OPPO_BATTLE, true);
                    }
                }
                let (reason_card, mut battle_destroyed) = self.calculate_battle_damage();
                let battlers = [(Some(attacker), attack_target), (attack_target, Some(attacker))];
                for ((card, other), destroyed) in battlers.into_iter().zip(&mut battle_destroyed) {
                    let (Some(card), Some(other)) = (card, other) else { continue };
                    if !*destroyed {
                        continue;
                    }
                    match self.is_affected_by_effect_target(card, EFFECT_INDESTRUCTABLE_BATTLE, other) {
                        Some(effect) => {
                            let code = self.cards[self.effects[effect].owner].data.code;
                            self.trace.message(MSG_HINT).u8(HINT_CARD).u8(0).u64(u64::from(code));
                            *destroyed = false;
                        }
                        None => self.cards[card].set_status(STATUS_BATTLE_RESULT, true),
                    }
                }
                let [attacker_destroyed, target_destroyed] = battle_destroyed.map(u8::from);
                let (target_location, da, dd) = defender;
                let mut message = self.trace.message(MSG_BATTLE);
                message
                    .loc(self.cards.get_info_location(attacker))
                    .u32(aa as u32)
                    .u32(ad as u32)
                    .u8(attacker_destroyed);
                message.loc(target_location).u32(da as u32).u32(dd as u32).u8(target_destroyed);
                drop(message);
                arg.reason_card = reason_card;
                if let Some(card) = reason_card {
                    arg.reason_player = self.cards[card].current.controler;
                }
                for p in [turn_player, 1 - turn_player] {
                    let amount = self.core.battle_damage[p] as u32;
                    let Some(card) = reason_card.filter(|_| amount != 0) else {
                        continue;
                    };
                    let reason_player = self.cards[card].current.controler;
                    for battler in [Some(attacker), attack_target].into_iter().flatten() {
                        self.field_raise_single_event(battler, EVENT_PRE_BATTLE_DAMAGE, None, 0, reason_player, p, amount);
                    }
                    self.field_raise_event(Some(card.into()), EVENT_PRE_BATTLE_DAMAGE, None, 0, reason_player, p, amount);
                }
                self.process_single_event();
                self.process_instant_event();
                self.core.damage_calculated = true;
                false
            }
            27 => {
                self.infos.phase = PHASE_DAMAGE;
                self.core.hint_timing[turn_player] = 0;
                self.core.chain_attack = false;
                for p in 0..2 {
                    let amount = self.core.battle_damage[p] as u32;
                    if amount != 0 {
                        self.field_damage(None, REASON_BATTLE, arg.reason_player, arg.reason_card, p, amount);
                    }
                }
                self.reset_phase(PHASE_DAMAGE_CAL);
                self.adjust_all();
                false
            }
            28 => {
                let mut destroyed = CardSet::default();
                let [attacker_field, target_field] = self.core.pre_field;
                let battlers = [
                    (Some(attacker), attack_target, attacker_field),
                    (attack_target, Some(attacker), target_field),
                ];
                for (card, other, field) in battlers {
                    let (Some(card), Some(other)) = (card, other) else { continue };
                    let target = &self.cards[card];
                    if !target.is_status(STATUS_BATTLE_RESULT) || target.current.location != LOCATION_MZONE || target.fieldid_r != field {
                        continue;
                    }
                    destroyed.insert(card);
                    let other_controler = self.cards[other].current.controler;
                    let target = &mut self.cards[card];
                    target.temp.reason = target.current.reason;
                    target.temp.reason_card = target.current.reason_card;
                    target.temp.reason_effect = target.current.reason_effect;
                    target.temp.reason_player = target.current.reason_player;
                    target.current.reason_effect = None;
                    target.current.reason = REASON_BATTLE;
                    target.current.reason_card = Some(other);
                    target.current.reason_player = other_controler;
                    let mut destination = u32::from(LOCATION_GRAVE);
                    if let Some(redirect) = self.is_affected_by_effect(other, EFFECT_BATTLE_DESTROY_REDIRECT)
                        && self.cards[card].data.card_type & TYPE_MONSTER != 0
                    {
                        destination = self.get_value(redirect, card) as u32;
                    }
                    let target = &mut self.cards[card];
                    target.sendto_param = SendtoParam {
                        playerid: target.owner,
                        position: POS_FACEUP as u8,
                        location: destination as u8,
                        sequence: (destination >> 16) as u8,
                    };
                    target.set_status(STATUS_DESTROY_CONFIRMED, true);
                }
                for card in [Some(attacker), attack_target].into_iter().flatten() {
                    self.cards[card].set_status(STATUS_BATTLE_RESULT, false);
                }
                self.core.battle_destroy_rep.clear();
                self.core.desrep_chain.clear();
                if !destroyed.is_empty() {
                    let (targets, reason) = (destroyed, REASON_BATTLE);
                    self.emplace_process(Destroy {
                        step: 10,
                        reason_player: PLAYER_NONE,
                        reason,
                        targets,
                        reason_effect: None,
                        replacing: false,
                    });
                    arg.cards_destroyed_by_battle = Some(destroyed);
                }
                false
            }
            29 => {
                if let Some(destroyed) = &mut arg.cards_destroyed_by_battle {
                    *destroyed = self.core.battle_destroyed;
                }
                if !self.core.battle_destroy_rep.is_empty() {
                    let rep = self.core.battle_destroy_rep;
                    self.field_destroy(
                        rep,
                        None,
                        REASON_EFFECT | REASON_REPLACE,
                        PLAYER_NONE,
                        PLAYER_NONE,
                        LOCATION_GRAVE,
                        0,
                    );
                }
                if !self.core.desrep_chain.is_empty() {
                    self.emplace_process(OperationReplace {
                        step: 15,
                        replace_effect: None,
                        target: None,
                        is_destroy: false,
                    });
                }
                self.adjust_all();
                false
            }
            30 => {
                for card in arg.cards_destroyed_by_battle.unwrap_or_default().iter() {
                    self.cards[card].set_status(STATUS_BATTLE_DESTROYED, true);
                    self.cards[card].set_status(STATUS_DESTROY_CONFIRMED, false);
                    self.filter_disable_related_cards(card);
                }
                self.core.selfdes_disabled = false;
                self.adjust_all();
                if self.is_flag(DUEL_6_STEP_BATLLE_STEP) {
                    if self.core.effect_damage_step == 0 {
                        self.core.hint_timing[turn_player] = TIMING_DAMAGE_CAL;
                        self.battle_window(45, false);
                    } else {
                        self.field_break_effect(true);
                    }
                }
                false
            }
            // Runs into step 32; Duel.CalculateDamage hands over to its own damage step and starts step 32 there.
            31 => {
                self.release_delayed_flips();
                self.field_raise_single_event(attacker, EVENT_BATTLED, None, 0, PLAYER_NONE, 0, 0);
                let mut battled = CardSet::from(attacker);
                if let Some(target) = attack_target {
                    battled.insert(target);
                    self.field_raise_single_event(target, EVENT_BATTLED, None, 0, PLAYER_NONE, 0, 1);
                }
                self.field_raise_event(Some(battled), EVENT_BATTLED, None, 0, PLAYER_NONE, 0, 0);
                self.process_single_event();
                self.process_instant_event();
                if self.core.effect_damage_step != 0 {
                    if let Some(Unit::DamageStep(damage_step)) = self.core.reserved.as_deref_mut() {
                        damage_step.cards_destroyed_by_battle = arg.cards_destroyed_by_battle;
                    }
                    return true;
                }
                arg.step = 32;
                self.battled_window();
                false
            }
            32 => {
                self.battled_window();
                false
            }
            33 => {
                if let Some(mut destroyed) = arg.cards_destroyed_by_battle {
                    let [attacker_field, target_field] = self.core.pre_field;
                    for card in arg.cards_destroyed_by_battle.unwrap_or_default().iter() {
                        let target = &self.cards[card];
                        let battled = target.fieldid_r == attacker_field || target.fieldid_r == target_field;
                        if target.current.location != LOCATION_MZONE || !battled {
                            destroyed.remove(card);
                        }
                    }
                    let (targets, reason) = (destroyed, REASON_BATTLE);
                    self.emplace_process(Destroy {
                        step: 3,
                        reason_player: PLAYER_NONE,
                        reason,
                        targets,
                        reason_effect: None,
                        replacing: false,
                    });
                }
                self.adjust_all();
                false
            }
            34 => {
                arg.cards_destroyed_by_battle = None;
                self.core.damage_calculated = true;
                self.core.selfdes_disabled = false;
                self.release_delayed_flips();
                let (mut destroying, mut destroyed) = (CardSet::default(), CardSet::default());
                // The event value tells the attacker (0) from the target (1).
                let value = |card| u32::from(Some(card) == attack_target);
                for (card, by) in [(Some(attacker), attack_target), (attack_target, Some(attacker))] {
                    let (Some(card), Some(by)) = (card, by) else { continue };
                    let reason = self.cards[card].current.reason;
                    if !self.cards[card].is_status(STATUS_BATTLE_DESTROYED) || reason & REASON_BATTLE == 0 {
                        continue;
                    }
                    let reason_player = self.cards[by].current.controler;
                    self.field_raise_single_event(by, EVENT_BATTLE_DESTROYING, None, reason, reason_player, 0, value(by));
                    self.field_raise_single_event(card, EVENT_BATTLE_DESTROYED, None, reason, reason_player, 0, value(card));
                    self.field_raise_single_event(card, EVENT_DESTROYED, None, reason, reason_player, 0, value(card));
                    destroying.insert(by);
                    destroyed.insert(card);
                }
                if !destroying.is_empty() {
                    self.field_raise_event(Some(destroying), EVENT_BATTLE_DESTROYING, None, 0, 0, 0, 0);
                }
                if !destroyed.is_empty() {
                    self.field_raise_event(Some(destroyed), EVENT_BATTLE_DESTROYED, None, 0, 0, 0, 0);
                    self.field_raise_event(Some(destroyed), EVENT_DESTROYED, None, 0, 0, 0, 0);
                }
                for card in [Some(attacker), attack_target].into_iter().flatten() {
                    self.cards[card].set_status(STATUS_BATTLE_DESTROYED, false);
                }
                self.raise_battle_events(EVENT_DAMAGE_STEP_END, attacker, attack_target);
                self.battle_window(44, true);
                arg.step = 38;
                false
            }
            39 => {
                for card in [Some(attacker), attack_target].into_iter().flatten() {
                    self.cards[card].set_status(STATUS_OPPO_BATTLE, false);
                }
                arg.step = RESTART;
                self.infos.phase = PHASE_BATTLE_STEP;
                self.trace.message(MSG_DAMAGE_STEP_END);
                self.reset_phase(PHASE_DAMAGE);
                self.adjust_all();
                self.core.effect_damage_step != 0
            }
            _ => true,
        }
    }

    // Duel.CalculateDamage's DamageStep: steps 0 and 1 run the damage calculation (BattleCommand step 26) apart from the
    // attack, then the unit waits in core.reserved for the chain it was called in to end, and step 2 runs the rest of
    // the damage step (BattleCommand step 32).
    pub(crate) fn process_damage_step(&mut self, arg: &mut DamageStep) -> bool {
        match arg.step {
            0 => {
                if self.core.effect_damage_step != 0 && !arg.new_attack {
                    return true;
                }
                self.core.effect_damage_step = 1;
                std::mem::swap(&mut self.core.attacker, &mut arg.attacker);
                std::mem::swap(&mut self.core.attack_target, &mut arg.attack_target);
                arg.backup_phase = self.infos.phase;
                let (attacker, attack_target) = (self.core.attacker, self.core.attack_target);
                let on_field = |duel: &Self, card: CardId| duel.cards[card].current.location == LOCATION_MZONE;
                let Some(attacker) =
                    attacker.filter(|&attacker| on_field(self, attacker) && attack_target.is_none_or(|target| on_field(self, target)))
                else {
                    arg.step = 2;
                    return false;
                };
                // The attack counters of a new attack come with the cards that call it with new_attack.
                unported!(self, arg.new_attack.then_some(attacker));
                self.add_announced_card(attacker, attack_target);
                self.attack_all_target_check();
                let target_location = attack_target.map_or(LocInfo::default(), |target| self.cards.get_info_location(target));
                self.trace
                    .message(MSG_ATTACK)
                    .loc(self.cards.get_info_location(attacker))
                    .loc(target_location);
                self.infos.phase = PHASE_DAMAGE;
                self.trace.message(MSG_DAMAGE_STEP_START);
                self.core.pre_field[0] = self.cards[attacker].fieldid_r;
                self.cards[attacker].attacked_count += 1;
                self.core.pre_field[1] = 0;
                if let Some(target) = attack_target {
                    self.core.pre_field[1] = self.cards[target].fieldid_r;
                    if self.cards[target].is_position(POS_FACEDOWN) {
                        self.field_change_position(target, None, PLAYER_NONE, self.cards[target].current.position >> 1, 0, true);
                        self.adjust_all();
                    }
                }
                false
            }
            1 => {
                self.infos.phase = PHASE_DAMAGE_CAL;
                self.emplace_process(BattleCommand {
                    step: 26,
                    ..BattleCommand::default()
                });
                arg.step = 2;
                self.core.reserved = Some(Box::new(Unit::DamageStep(arg.clone())));
                true
            }
            2 => {
                self.core.effect_damage_step = 2;
                self.emplace_process(BattleCommand {
                    step: 32,
                    cards_destroyed_by_battle: arg.cards_destroyed_by_battle,
                    ..BattleCommand::default()
                });
                false
            }
            3 => {
                std::mem::swap(&mut self.core.attacker, &mut arg.attacker);
                std::mem::swap(&mut self.core.attack_target, &mut arg.attack_target);
                for card in [self.core.attacker, self.core.attack_target].into_iter().flatten() {
                    self.cards[card].set_status(STATUS_ATTACK_CANCELED, true);
                }
                self.core.effect_damage_step = 0;
                self.infos.phase = arg.backup_phase;
                true
            }
            _ => true,
        }
    }

    fn battled_window(&mut self) {
        self.core.hint_timing[0] |= TIMING_BATTLED;
        self.core.hint_timing[1] |= TIMING_BATTLED;
        self.battle_window(43, false);
    }

    fn select_attack_target(&mut self) {
        let (player, cancelable) = (self.infos.turn_player, self.core.attack_cancelable);
        self.trace.message(MSG_HINT).u8(HINT_SELECTMSG).u8(player).u64(549);
        self.emplace_process(SelectCard {
            playerid: player,
            cancelable,
            min: 1,
            max: 1,
            ..SelectCard::default()
        });
    }

    fn hint_both(&mut self, hint: u64) {
        for player in 0..2 {
            self.trace.message(MSG_HINT).u8(HINT_EVENT).u8(player).u64(hint);
        }
    }

    // One of the damage step's chain windows, announced to both players by `hint`; `skip_new` (also from
    // DUEL_SINGLE_CHAIN_IN_DAMAGE_SUBSTEP) keeps a solved chain from opening another free chain window.
    fn battle_window(&mut self, hint: u64, skip_new: bool) {
        self.hint_both(hint);
        let skip_new = skip_new || self.is_flag(DUEL_SINGLE_CHAIN_IN_DAMAGE_SUBSTEP);
        self.emplace_process(PointEvent {
            skip_new,
            ..PointEvent::default()
        });
    }

    // Raises the event on the attacker (value 0), the target (value 1) and the field, then processes them.
    fn raise_battle_events(&mut self, code: u32, attacker: CardId, attack_target: Option<CardId>) {
        self.field_raise_single_event(attacker, code, None, 0, 0, 0, 0);
        if let Some(target) = attack_target {
            self.field_raise_single_event(target, code, None, 0, 0, 0, 1);
        }
        self.field_raise_event(None, code, None, 0, 0, 0, 0);
        self.process_single_event();
        self.process_instant_event();
    }

    // Sets core.battle_damage and returns the card dealing it and whether the attacker and the target are destroyed.
    fn calculate_battle_damage(&mut self) -> (Option<CardId>, [bool; 2]) {
        self.core.battle_damage = PerPlayer::default();
        let Some(attacker) = self.core.attacker else {
            return (None, [false; 2]);
        };
        let mut a = self.get_attack(attacker) as u32;
        let pa = self.cards[attacker].current.controler;
        if self.cards[attacker].is_position(POS_FACEUP_DEFENSE)
            && self
                .is_affected_by_effect(attacker, EFFECT_DEFENSE_ATTACK)
                .is_some_and(|effect| self.get_value(effect, attacker) != 0)
        {
            a = self.get_defense(attacker) as u32;
        }
        unported!(self, self.is_affected_by_effect(attacker, EFFECT_CHANGE_BATTLE_STAT));
        let (mut damp, mut reason_card, mut destroyed, mut pierce) = (0, None, [false; 2], false);
        match self.core.attack_target {
            Some(target) => {
                unported!(self, self.is_affected_by_effect(target, EFFECT_CHANGE_BATTLE_STAT));
                let attack_position = self.cards[target].is_position(POS_ATTACK);
                let d = (if attack_position {
                    self.get_attack(target)
                } else {
                    self.get_defense(target)
                }) as u32;
                if a > d {
                    if attack_position {
                        damp = self.cards[target].current.controler;
                        self.core.battle_damage[damp] = (a - d) as i32;
                        reason_card = Some(attacker);
                    } else {
                        let pierces = self.filter_effect(attacker, EFFECT_PIERCE, true);
                        if !pierces.is_empty() {
                            pierce = true;
                            let mut players = [false; 2];
                            for &effect in &pierces {
                                let player = 1 - self.get_handler_player(effect);
                                if let Some(damaged) = players.get_mut(usize::from(player)) {
                                    *damaged = true;
                                }
                            }
                            for (player, damaged) in [0, 1].into_iter().zip(players) {
                                if damaged {
                                    self.core.battle_damage[player] = (a - d) as i32;
                                }
                            }
                            unported!(
                                self,
                                pierces
                                    .iter()
                                    .filter(|&&effect| self.get_value(effect, attacker) as u32 == DOUBLE_DAMAGE)
                            );
                            if players != [true, true] {
                                damp = if players[0] { 0 } else { 1 };
                                unported!(
                                    self,
                                    [attacker, target]
                                        .into_iter()
                                        .filter_map(|card| self.is_affected_by_effect(card, EFFECT_BOTH_BATTLE_DAMAGE))
                                );
                            }
                            for card in [attacker, target] {
                                let codes = [EFFECT_REFLECT_BATTLE_DAMAGE, EFFECT_ALSO_BATTLE_DAMAGE];
                                unported!(self, codes.into_iter().filter_map(|code| self.is_affected_by_effect(card, code)));
                                unported!(self, self.filter_effect(card, EFFECT_CHANGE_BATTLE_DAMAGE, true));
                            }
                            for p in 0..2 {
                                let codes = [EFFECT_REFLECT_BATTLE_DAMAGE, EFFECT_ALSO_BATTLE_DAMAGE];
                                unported!(
                                    self,
                                    codes.into_iter().filter_map(|code| self.is_player_affected_by_effect(p, code))
                                );
                                unported!(self, self.filter_player_effect(p, EFFECT_CHANGE_BATTLE_DAMAGE, false));
                            }
                            let pd = self.cards[target].current.controler;
                            if self.is_affected_by_effect(attacker, EFFECT_NO_BATTLE_DAMAGE).is_some()
                                || self
                                    .is_affected_by_effect_target(target, EFFECT_AVOID_BATTLE_DAMAGE, attacker)
                                    .is_some()
                                || self.is_player_affected_by_effect(pd, EFFECT_AVOID_BATTLE_DAMAGE).is_some()
                            {
                                self.core.battle_damage[pd] = 0;
                            }
                            if self.is_affected_by_effect(target, EFFECT_NO_BATTLE_DAMAGE).is_some()
                                || self
                                    .is_affected_by_effect_target(attacker, EFFECT_AVOID_BATTLE_DAMAGE, target)
                                    .is_some()
                                || self.is_player_affected_by_effect(1 - pd, EFFECT_AVOID_BATTLE_DAMAGE).is_some()
                            {
                                self.core.battle_damage[1 - pd] = 0;
                            }
                            reason_card = Some(attacker);
                        }
                    }
                    destroyed = [false, true];
                } else if a < d {
                    damp = pa;
                    self.core.battle_damage[damp] = (d - a) as i32;
                    reason_card = Some(target);
                    destroyed = [attack_position, false];
                } else if attack_position && (a != 0 || self.is_flag(DUEL_0_ATK_DESTROYED)) {
                    destroyed = [true, true];
                }
            }
            None if a != 0 => {
                damp = 1 - pa;
                self.core.battle_damage[damp] = a as i32;
                reason_card = Some(attacker);
            }
            None => {}
        }
        if let Some(reason) = reason_card.filter(|_| !pierce) {
            unported!(self, self.is_affected_by_effect(reason, EFFECT_BATTLE_DAMAGE_TO_EFFECT));
            let dam_card = if reason == attacker {
                self.core.attack_target
            } else {
                Some(attacker)
            };
            let card_codes = [EFFECT_BOTH_BATTLE_DAMAGE, EFFECT_REFLECT_BATTLE_DAMAGE, EFFECT_ALSO_BATTLE_DAMAGE];
            for card in [Some(reason), dam_card].into_iter().flatten() {
                unported!(
                    self,
                    card_codes.into_iter().filter_map(|code| self.is_affected_by_effect(card, code))
                );
            }
            for p in [damp, 1 - damp] {
                let player_codes = [EFFECT_REFLECT_BATTLE_DAMAGE, EFFECT_ALSO_BATTLE_DAMAGE];
                unported!(
                    self,
                    player_codes
                        .into_iter()
                        .filter_map(|code| self.is_player_affected_by_effect(p, code)),
                );
            }
            self.change_battle_damage(reason, dam_card, damp);
            if self.is_affected_by_effect(reason, EFFECT_NO_BATTLE_DAMAGE).is_some()
                || dam_card.is_some_and(|card| {
                    self.is_affected_by_effect_target(card, EFFECT_AVOID_BATTLE_DAMAGE, reason)
                        .is_some()
                })
                || self.is_player_affected_by_effect(damp, EFFECT_AVOID_BATTLE_DAMAGE).is_some()
            {
                self.core.battle_damage[damp] = 0;
            }
            if dam_card.is_some_and(|card| self.is_affected_by_effect(card, EFFECT_NO_BATTLE_DAMAGE).is_some())
                || self.is_avoiding_battle_damage(reason, dam_card)
                || self.is_player_affected_by_effect(1 - damp, EFFECT_AVOID_BATTLE_DAMAGE).is_some()
            {
                self.core.battle_damage[1 - damp] = 0;
            }
        }
        if self.core.battle_damage[damp] == 0 && self.core.battle_damage[1 - damp] == 0 {
            reason_card = None;
        }
        (reason_card, destroyed)
    }

    // card.is_affected_by_effect(EFFECT_AVOID_BATTLE_DAMAGE, dam_card) with a dam_card that may be nil, which only an
    // effect whose value is an integer can be told apart for.
    fn is_avoiding_battle_damage(&self, reason_card: CardId, dam_card: Option<CardId>) -> bool {
        let found = self.is_affected_by_effect_target(reason_card, EFFECT_AVOID_BATTLE_DAMAGE, dam_card.unwrap_or(reason_card));
        unported!(
            self,
            found.filter(|&e| dam_card.is_none() && !matches!(self.effects[e].value, Value::Int(_)))
        );
        found.is_some()
    }

    // The EFFECT_CHANGE_BATTLE_DAMAGE effects of the cards and players, in id order, double, halve or set the damage.
    fn change_battle_damage(&mut self, reason_card: CardId, dam_card: Option<CardId>, damp: u8) {
        let mut effects = self.filter_effect(reason_card, EFFECT_CHANGE_BATTLE_DAMAGE, false);
        if let Some(dam_card) = dam_card {
            effects.extend(self.filter_effect(dam_card, EFFECT_CHANGE_BATTLE_DAMAGE, false));
        }
        effects.extend(self.filter_player_effect(damp, EFFECT_CHANGE_BATTLE_DAMAGE, false));
        effects.extend(self.filter_player_effect(1 - damp, EFFECT_CHANGE_BATTLE_DAMAGE, false));
        effects.sort_by_key(|&effect| self.effects.id(effect));
        for p in 0..2 {
            let (mut double_dam, mut half_dam, mut dam_value) = (false, false, -1);
            for &effect in &effects {
                let player_target = self.effects[effect].is_flag(EFFECT_FLAG_PLAYER_TARGET);
                unported!(self, (player_target && self.is_target_player(effect, p)).then_some(effect));
                let val = if player_target {
                    -1
                } else {
                    self.get_battle_damage_value(effect, p, reason_card)
                };
                if val == i64::from(DOUBLE_DAMAGE) {
                    double_dam = true;
                } else if val == i64::from(HALF_DAMAGE) {
                    half_dam = true;
                } else if val > 0 {
                    dam_value = val as i32;
                } else if val == 0 {
                    dam_value = 0;
                    break;
                }
            }
            let damage = &mut self.core.battle_damage[p];
            if double_dam != half_dam {
                *damage = if double_dam { *damage * 2 } else { *damage / 2 };
            }
            if dam_value >= 0 && *damage > 0 {
                *damage = dam_value;
            }
        }
    }

    fn process_turn(&mut self, arg: &mut Turn) -> bool {
        let turn_player = arg.turn_player;
        match arg.step {
            0 => {
                self.announced_cards.clear();
                for p in 0..2 {
                    for card in self.player[p].list_mzone.into_iter().flatten() {
                        let card = &mut self.cards[card];
                        let turn_status =
                            STATUS_SUMMON_TURN | STATUS_FLIP_SUMMON_TURN | STATUS_SPSUMMON_TURN | STATUS_SET_TURN | STATUS_FORM_CHANGED;
                        card.set_status(turn_status, false);
                        card.attack_announce_count = 0;
                        card.announce_count = 0;
                        card.attacked_count = 0;
                        card.attack_all_target = 1;
                    }
                    for card in self.player[p].list_szone.into_iter().flatten() {
                        self.cards[card].set_status(STATUS_SET_TURN, false);
                    }
                    let core = &mut self.core;
                    for count in [
                        &mut core.summon_state_count,
                        &mut core.normalsummon_state_count,
                        &mut core.flipsummon_state_count,
                        &mut core.spsummon_state_count,
                        &mut core.spsummon_state_count_rst,
                        &mut core.attack_state_count,
                        &mut core.battle_phase_count,
                        &mut core.battled_count,
                        &mut core.extra_summon,
                    ] {
                        count[p] = 0;
                    }
                    core.summon_count[p] = 0;
                    core.spsummon_once_map[p].clear();
                }
                self.effects.delete(&self.core.reseted_effects);
                self.core.reseted_effects.clear();
                self.core.effect_count_code.clear();
                self.emplace_process(RefreshRelay::default());
                false
            }
            1 => {
                self.core.force_turn_end = false;
                self.core.spsummon_rst = false;
                for index in 0..self.field_effects.rechargeable.len() {
                    if let Some(&effect) = self.field_effects.rechargeable.get(index)
                        && !self.effects[effect].is_flag(EFFECT_FLAG_NO_TURN_RESET)
                    {
                        self.recharge(effect);
                    }
                }
                for counter in self.core.custom_counters.iter_mut() {
                    counter.amount = [0; 2];
                }
                unported!(self, self.field_effects.spsummon_count_eff.iter().copied());
                self.infos.turn_id += 1;
                self.infos.turn_id_by_player[turn_player] += 1;
                self.infos.turn_player = turn_player;
                self.trace.message(MSG_NEW_TURN).u8(turn_player);
                if self.is_player_affected_by_effect(turn_player, EFFECT_SKIP_TURN).is_some() {
                    arg.step = 18;
                    self.reset_phase(PHASE_DRAW);
                    self.reset_phase(PHASE_STANDBY);
                    self.reset_phase(PHASE_END);
                    self.adjust_all();
                    return false;
                }
                self.infos.phase = PHASE_DRAW;
                self.core.phase_action = false;
                self.core.hand_adjusted = false;
                self.field_raise_event(None, EVENT_PHASE_START + u32::from(PHASE_DRAW), None, 0, 0, turn_player, 0);
                self.process_instant_event();
                self.adjust_all();
                false
            }
            2 => {
                self.clear_new_chains();
                if self.is_player_affected_by_effect(turn_player, EFFECT_SKIP_DP).is_some() || self.core.force_turn_end {
                    arg.step = 3;
                    self.reset_phase(PHASE_DRAW);
                    self.adjust_all();
                    return false;
                }
                self.trace.message(MSG_NEW_PHASE).u16(self.infos.phase);
                self.field_raise_event(None, EVENT_PREDRAW, None, 0, 0, turn_player, 0);
                self.process_instant_event();
                self.trace.message(MSG_HINT).u8(HINT_EVENT).u8(turn_player).u64(27);
                if !self.core.new_fchain.is_empty() || !self.core.new_ochain.is_empty() {
                    self.emplace_process(PointEvent {
                        skip_freechain: true,
                        ..PointEvent::default()
                    });
                }
                false
            }
            3 => {
                if self.is_flag(DUEL_1ST_TURN_DRAW) || self.infos.turn_id > 1 {
                    let mut count = self.get_draw_count(turn_player);
                    if count > 0 && self.is_flag(DUEL_DRAW_UNTIL_5) {
                        count = count.max(5 - self.player[turn_player].list_hand.len() as i32);
                    }
                    if count > 0 {
                        self.field_draw(None, REASON_RULE, turn_player, turn_player, count as u16);
                        self.emplace_process(PointEvent::default());
                    }
                }
                self.emplace_process(PhaseEvent {
                    phase: PHASE_DRAW,
                    ..PhaseEvent::default()
                });
                false
            }
            4 | 8 => false,
            5 => {
                self.infos.phase = PHASE_STANDBY;
                self.core.phase_action = false;
                self.clear_new_chains();
                if self.is_flag(DUEL_NO_STANDBY_PHASE)
                    || self.is_player_affected_by_effect(turn_player, EFFECT_SKIP_SP).is_some()
                    || self.core.force_turn_end
                {
                    arg.step = 6;
                    self.reset_phase(PHASE_STANDBY);
                    self.adjust_all();
                    return false;
                }
                self.trace.message(MSG_NEW_PHASE).u16(self.infos.phase);
                self.field_raise_event(None, EVENT_PHASE_START + u32::from(PHASE_STANDBY), None, 0, 0, turn_player, 0);
                self.process_instant_event();
                false
            }
            6 => {
                // EVENT_PHASE_START + PHASE_STANDBY is a special case (c89642993).
                let standby_start = EVENT_PHASE_START + u32::from(PHASE_STANDBY);
                if !self.core.new_fchain.is_empty()
                    || !self.core.new_ochain.is_empty()
                    || self.core.instant_event.last().is_none_or(|event| event.event_code != standby_start)
                {
                    self.emplace_process(PointEvent::default());
                }
                self.emplace_process(PhaseEvent {
                    phase: PHASE_STANDBY,
                    ..PhaseEvent::default()
                });
                false
            }
            7 => {
                self.infos.phase = PHASE_MAIN1;
                self.core.phase_action = false;
                self.field_raise_event(None, EVENT_PHASE_START + u32::from(PHASE_MAIN1), None, 0, 0, turn_player, 0);
                self.process_instant_event();
                self.adjust_all();
                false
            }
            9 => {
                self.clear_new_chains();
                self.trace.message(MSG_NEW_PHASE).u16(self.infos.phase);
                self.emplace_process(IdleCommand::default());
                false
            }
            10 => {
                if self.returns.at_i32(0) == 7 {
                    arg.step = 15;
                    return false;
                }
                self.infos.phase = PHASE_BATTLE_START;
                self.clear_new_chains();
                self.core.phase_action = false;
                self.core.battle_phase_count[turn_player] = self.core.battle_phase_count[turn_player].wrapping_add(1);
                self.trace.message(MSG_NEW_PHASE).u16(self.infos.phase);
                // Shows that the Battle Phase is entered and skipped.
                if self.is_player_affected_by_effect(self.infos.turn_player, EFFECT_SKIP_BP).is_some() || self.core.force_turn_end {
                    arg.step = 15;
                    self.reset_phase(PHASE_BATTLE_START);
                    self.reset_phase(PHASE_BATTLE_STEP);
                    self.reset_phase(PHASE_BATTLE);
                    self.adjust_all();
                    return false;
                }
                self.field_raise_event(None, EVENT_PHASE_START + u32::from(PHASE_BATTLE_START), None, 0, 0, turn_player, 0);
                self.process_instant_event();
                self.adjust_all();
                false
            }
            11 => {
                if !self.core.new_fchain.is_empty() || !self.core.new_ochain.is_empty() {
                    self.emplace_process(PointEvent::default());
                }
                self.emplace_process(PhaseEvent {
                    phase: PHASE_BATTLE_START,
                    ..PhaseEvent::default()
                });
                false
            }
            12 => {
                self.infos.phase = PHASE_BATTLE_STEP;
                self.clear_new_chains();
                self.core.phase_action = false;
                self.core.chain_attack = false;
                self.emplace_process(BattleCommand {
                    reason_player: PLAYER_NONE,
                    ..BattleCommand::default()
                });
                false
            }
            13 => {
                if !arg.has_performed_second_battle_phase && self.returns.at_i32(1) != 0 {
                    arg.has_performed_second_battle_phase = true;
                    arg.step = 9;
                    for p in 0..2 {
                        for card in self.player[p].list_mzone.into_iter().flatten() {
                            let card = &mut self.cards[card];
                            card.attack_announce_count = 0;
                            card.announce_count = 0;
                            card.attacked_count = 0;
                        }
                    }
                    self.announced_cards.clear();
                    return false;
                }
                arg.has_performed_second_battle_phase = false;
                if self.is_flag(DUEL_NO_MAIN_PHASE_2) {
                    arg.step = 15;
                    self.adjust_all();
                    return false;
                }
                self.core.skip_m2 = self.returns.at_i32(0) == 3 || self.core.force_turn_end;
                self.infos.phase = PHASE_MAIN2;
                self.core.phase_action = false;
                self.field_raise_event(None, EVENT_PHASE_START + u32::from(PHASE_MAIN2), None, 0, 0, turn_player, 0);
                self.process_instant_event();
                self.adjust_all();
                false
            }
            14 | 17 => {
                if !self.core.new_fchain.is_empty() || !self.core.new_ochain.is_empty() {
                    self.emplace_process(PointEvent::default());
                }
                false
            }
            15 => {
                self.clear_new_chains();
                self.trace.message(MSG_NEW_PHASE).u16(self.infos.phase);
                self.infos.can_shuffle = true;
                self.emplace_process(IdleCommand::default());
                false
            }
            16 => {
                self.infos.phase = PHASE_END;
                self.core.phase_action = false;
                if self.is_player_affected_by_effect(self.infos.turn_player, EFFECT_SKIP_EP).is_some() {
                    arg.step = 18;
                    self.reset_phase(PHASE_END);
                    self.adjust_all();
                    return false;
                }
                self.trace.message(MSG_NEW_PHASE).u16(self.infos.phase);
                self.field_raise_event(None, EVENT_PHASE_START + u32::from(PHASE_END), None, 0, 0, turn_player, 0);
                self.process_instant_event();
                self.adjust_all();
                false
            }
            18 => {
                self.clear_new_chains();
                self.emplace_process(PhaseEvent {
                    phase: PHASE_END,
                    ..PhaseEvent::default()
                });
                false
            }
            19 => {
                self.field_raise_event(None, EVENT_TURN_END, None, 0, 0, turn_player, 0);
                self.process_instant_event();
                self.adjust_all();
                false
            }
            20 => {
                self.clear_new_chains();
                arg.step = RESTART;
                arg.turn_player = 1 - arg.turn_player;
                false
            }
            _ => true,
        }
    }

    fn clear_new_chains(&mut self) {
        self.core.new_fchain.clear();
        self.core.new_ochain.clear();
        self.core.quick_f_chain.clear();
        self.core.delayed_quick_tmp.clear();
    }

    // Writes MSG_WIN for a player at 0 LP or decked out; Adjust step 1 and break_effect share it.
    pub(crate) fn check_win(&mut self) {
        let immune = |duel: &Self, p: u8, code| duel.is_player_affected_by_effect(p, code).is_some();
        let (lp, overdraw) = ([self.player[0].lp <= 0, self.player[1].lp <= 0], self.core.overdraw.0);
        let (mut winp, mut rea) = (5, 1);
        if lp[0] && !lp[1] && !immune(self, 0, EFFECT_CANNOT_LOSE_LP) {
            (winp, rea) = (1, 1);
        }
        if overdraw[0] && !overdraw[1] && !immune(self, 0, EFFECT_CANNOT_LOSE_DECK) {
            (winp, rea) = (1, 2);
        }
        if lp[1] && !lp[0] && !immune(self, 1, EFFECT_CANNOT_LOSE_LP) {
            (winp, rea) = (0, 1);
        }
        if overdraw[1] && !overdraw[0] && !immune(self, 1, EFFECT_CANNOT_LOSE_DECK) {
            (winp, rea) = (0, 2);
        }
        for (lost, code, reason) in [(lp, EFFECT_CANNOT_LOSE_LP, 1), (overdraw, EFFECT_CANNOT_LOSE_DECK, 2)] {
            if lost[0] && lost[1] && !(immune(self, 0, code) && immune(self, 1, code)) {
                winp = if immune(self, 0, code) {
                    0
                } else if immune(self, 1, code) {
                    1
                } else {
                    PLAYER_NONE
                };
                rea = reason;
            }
        }
        if winp != 5 {
            self.win(winp, rea);
        } else if self.core.win_player != 5 {
            self.win(self.core.win_player, self.core.win_reason);
        }
    }

    pub(crate) fn process_sort_deck(&mut self, arg: &mut SortDeck) -> bool {
        let (target_player, bottom) = (arg.target_player, arg.bottom);
        let count = usize::from(arg.count).min(self.player[target_player].list_main.len());
        match arg.step {
            0 => {
                let list = &self.player[target_player].list_main;
                self.core.select_cards = if bottom {
                    list.iter().take(count).copied().collect()
                } else {
                    list.iter().rev().take(count).copied().collect()
                };
                self.emplace_process(SortCard {
                    playerid: arg.sort_player,
                    is_chain: false,
                    ..SortCard::default()
                });
                false
            }
            1 => {
                if self.returns.at_u8(0) == u8::MAX || count == 0 {
                    return true;
                }
                let mut sorted = vec![CardId::default(); self.core.select_cards.len()];
                for index in 0..count {
                    if let (Some(slot), Some(&card)) = (
                        sorted.get_mut(usize::from(self.returns.at_u8(index))),
                        self.core.select_cards.get(index),
                    ) {
                        *slot = card;
                    }
                }
                for index in 0..count {
                    let Some(&card) = sorted.get(if bottom { index } else { count - index - 1 }) else {
                        continue;
                    };
                    let from = self.cards.get_info_location(card);
                    let list = &mut self.player[target_player].list_main;
                    list.remove(self.cards[card].current.sequence as usize);
                    if bottom {
                        list.insert(0, card);
                    } else {
                        list.push(card);
                    }
                    self.reset_sequence(target_player, LOCATION_DECK);
                    let to = self.cards.get_info_location(card);
                    self.trace
                        .message(MSG_MOVE)
                        .u32(0)
                        .loc(from)
                        .loc(to)
                        .u32(self.cards[card].current.reason);
                }
                if self.core.global_flag & GLOBALFLAG_DECK_REVERSE_CHECK != 0
                    && let Some(&top) = self.player[target_player].list_main.last()
                {
                    let (code, position) = (self.cards[top].data.code, self.cards[top].current.position);
                    if self.core.deck_reversed || position == POS_FACEUP_DEFENSE {
                        self.trace.message(MSG_DECK_TOP).u8(target_player).u32(0).u32(code).u32(position);
                    }
                }
                true
            }
            _ => true,
        }
    }

    fn win(&mut self, player: u8, reason: u8) {
        self.trace.message(MSG_WIN).u8(player).u8(reason);
        self.result.get_or_insert((player, reason));
        self.core.overdraw = PerPlayerBool::default();
        self.core.win_player = 5;
        self.core.win_reason = 0;
    }

    // Steps 3-5, 8 and 13 adjust trap monsters, control, brainwashing, granted effects and deck reversal, which come
    // with the cards that cause them.
    fn process_adjust(&mut self, arg: &mut Adjust) -> bool {
        match arg.step {
            0 => {
                self.core.re_adjust = false;
                false
            }
            1 => {
                if !self.core.force_turn_end {
                    self.check_win();
                }
                false
            }
            2 => {
                let mut turn_player = self.infos.turn_player;
                for _ in 0..2 {
                    let player = &self.player[turn_player];
                    for card in player.list_mzone.into_iter().chain(player.list_szone).flatten() {
                        self.add_to_disable_check_list(card);
                    }
                    turn_player = 1 - turn_player;
                }
                self.adjust_disable_check_list();
                self.emplace_process(RefreshLoc::default());
                false
            }
            5 => {
                arg.step = 7;
                false
            }
            9 => {
                if self.core.selfdes_disabled {
                    arg.step = 10;
                    return false;
                }
                self.adjust_self_destroy_set();
                false
            }
            10 => {
                let mut destroy_set = CardSet::default();
                let mut turn_player = self.infos.turn_player;
                for _ in 0..2 {
                    for card in self.player[turn_player].list_szone.into_iter().take(5).flatten() {
                        if let Some(target) = self.cards[card].equiping_target
                            && self.is_affected_by_effect_target(card, EFFECT_EQUIP_LIMIT, target).is_none()
                        {
                            destroy_set.insert(card);
                        }
                    }
                    turn_player = 1 - turn_player;
                }
                if !destroy_set.is_empty() {
                    self.core.re_adjust = true;
                    self.field_destroy(destroy_set, None, REASON_RULE, PLAYER_NONE, PLAYER_NONE, LOCATION_GRAVE, 0);
                }
                false
            }
            11 => {
                let mut turn_player = self.infos.turn_player;
                let mut pos_adjust = CardSet::default();
                for _ in 0..2 {
                    for card in self.player[turn_player].list_mzone.into_iter().flatten() {
                        let card_type = self.cards[card].data.card_type;
                        if (card_type & TYPE_LINK != 0 && card_type & TYPE_MONSTER != 0)
                            || self.is_affected_by_effect(card, EFFECT_CANNOT_CHANGE_POS_E).is_some()
                        {
                            continue;
                        }
                        let Some(&effect) = self.filter_effect(card, EFFECT_SET_POSITION, true).last() else {
                            continue;
                        };
                        let pos = self.get_value(effect, card) as u32;
                        if pos & 0xff != self.cards[card].current.position {
                            pos_adjust.insert(card);
                            self.cards[card].position_param = pos;
                            let just_pos = self.cards[card].is_status(STATUS_JUST_POS);
                            self.cards[card].set_status(STATUS_CONTINUOUS_POS, just_pos);
                        } else {
                            self.cards[card].set_status(STATUS_CONTINUOUS_POS, false);
                        }
                        self.cards[card].set_status(STATUS_JUST_POS, false);
                    }
                    turn_player = 1 - turn_player;
                }
                if !pos_adjust.is_empty() {
                    self.core.re_adjust = true;
                    self.emplace_process(ChangePos {
                        step: 0,
                        reason_player: PLAYER_NONE,
                        enable: true,
                        oppo_selection: false,
                        reason_effect: None,
                        targets: pos_adjust,
                        to_grave_set: CardSet::default(),
                    });
                }
                false
            }
            12 => {
                for p in 0..2 {
                    for &card in &self.player[p].list_hand {
                        let public = self.is_affected_by_effect(card, EFFECT_PUBLIC).is_some();
                        if !public && self.cards[card].is_position(POS_FACEUP) {
                            self.core.shuffle_hand_check[p] = true;
                        }
                        self.cards[card].current.position = if public { POS_FACEUP } else { POS_FACEDOWN };
                    }
                }
                let turn_player = self.infos.turn_player;
                if self.core.shuffle_hand_check[turn_player] {
                    self.shuffle(turn_player, LOCATION_HAND);
                }
                if self.core.shuffle_hand_check[1 - turn_player] {
                    self.shuffle(1 - turn_player, LOCATION_HAND);
                }
                false
            }
            15 => {
                self.field_raise_event(None, EVENT_ADJUST, None, 0, PLAYER_NONE, PLAYER_NONE, 0);
                self.process_instant_event();
                false
            }
            16 => {
                if self.core.re_adjust {
                    arg.step = RESTART;
                    return false;
                }
                for p in 0..2 {
                    if self.core.shuffle_hand_check[p] {
                        self.shuffle(p, LOCATION_HAND);
                    }
                }
                for p in 0..2 {
                    if self.core.shuffle_deck_check[p] {
                        self.shuffle(p, LOCATION_DECK);
                    }
                }
                true
            }
            14 => {
                self.adjust_attack();
                false
            }
            4 => {
                // core.last_control_changed_id is read only with EFFECT_REMOVE_BRAINWASHING.
                let turn_player = self.infos.turn_player;
                let (mut adjust_set, mut reason_cards) = (PerPlayer::<CardSet>::default(), CardSet::default());
                for p in 0..2 {
                    for card in self.player[p].list_mzone.into_iter().flatten() {
                        let (reference, effect) = self.refresh_control_status(card);
                        if self.cards[card].current.controler == reference
                            || self.is_affected_by_effect(card, EFFECT_CANNOT_CHANGE_CONTROL).is_some()
                        {
                            continue;
                        }
                        adjust_set[p].insert(card);
                        if let Some(effect) = effect
                            && (self.effects[effect].effect_type & EFFECT_TYPE_SINGLE == 0 || self.effects[effect].condition.is_some())
                        {
                            reason_cards.insert(self.get_handler(effect));
                        }
                    }
                }
                if !adjust_set[0].is_empty() || !adjust_set[1].is_empty() {
                    self.core.re_adjust = true;
                    for (targets, playerid) in [
                        (adjust_set[1 - turn_player], turn_player),
                        (adjust_set[turn_player], 1 - turn_player),
                    ] {
                        self.emplace_process(GetControl {
                            chose_player: PLAYER_NONE,
                            targets,
                            playerid,
                            zone: 0xff,
                            ..GetControl::default()
                        });
                    }
                    for card in reason_cards {
                        let readjusts = self.core.readjust_map.entry(card).or_default();
                        *readjusts += 1;
                        if *readjusts > 3 {
                            self.field_destroy(card.into(), None, REASON_RULE, PLAYER_NONE, PLAYER_NONE, LOCATION_GRAVE, 0);
                        }
                    }
                }
                false
            }
            3 | 8 | 13 => false,
            _ => true,
        }
    }

    // Adjust step 14: cancels the attack when the attacker can no longer attack or a battling monster left, and marks
    // a replay when the opponent's monsters changed.
    fn adjust_attack(&mut self) {
        let Some(attacker) = self.core.attacker else { return };
        let unstoppable = self.is_affected_by_effect(attacker, EFFECT_UNSTOPPABLE_ATTACK).is_some();
        if !unstoppable && self.cards[attacker].is_status(STATUS_ATTACK_CANCELED) {
            return;
        }
        let battle_step = self.infos.phase != PHASE_DAMAGE && self.infos.phase != PHASE_DAMAGE_CAL;
        let [attacker_field, target_field] = self.core.pre_field;
        let card = &self.cards[attacker];
        let left = |card: &Card, field| {
            card.current.location != LOCATION_MZONE || card.current.controler != card.attack_controler || card.fieldid_r != field
        };
        let canceled = if battle_step {
            !self.is_capable_attack(attacker) || card.current.controler != card.attack_controler || card.fieldid_r != attacker_field
        } else {
            left(card, attacker_field)
                || (card.current.position & POS_DEFENSE != 0 && self.is_affected_by_effect(attacker, EFFECT_DEFENSE_ATTACK).is_none())
                || self
                    .core
                    .attack_target
                    .is_some_and(|target| left(&self.cards[target], target_field))
        };
        if canceled {
            self.cards[attacker].set_status(STATUS_ATTACK_CANCELED, true);
        } else if battle_step
            && !self.core.attack_rollback
            && (self.opponent_field_ids() != self.core.opp_mzone || !self.confirm_attack_target())
        {
            self.core.attack_rollback = true;
        }
    }

    // Steps 1-6 apply the EFFECT_DISABLE_FIELD and EFFECT_USE_EXTRA_*ZONE effects step 0 collects, which come with
    // the cards that register them.
    fn process_refresh_loc(&mut self, arg: &mut RefreshLoc) -> bool {
        match arg.step {
            0 => {
                if self.is_flag(DUEL_3_COLUMNS_FIELD) {
                    self.player[0].used_location |= 0x1111;
                    self.player[1].used_location |= 0x1111;
                }
                arg.previously_disabled_locations = (self.player[0].disabled_location & 0xffff) | (self.player[1].disabled_location << 16);
                self.player[0].disabled_location = 0;
                self.player[1].disabled_location = 0;
                for code in [EFFECT_DISABLE_FIELD, EFFECT_USE_EXTRA_MZONE, EFFECT_USE_EXTRA_SZONE] {
                    unported!(self, self.filter_field_effect(code));
                }
                false
            }
            1 | 3 | 5 => {
                arg.step += 1;
                false
            }
            7 => {
                let cross = |disabled: u32| (((disabled >> 5) & 1) << 6) | (((disabled >> 6) & 1) << 5);
                self.player[0].disabled_location |= cross(self.player[1].disabled_location);
                self.player[1].disabled_location |= cross(self.player[0].disabled_location);
                let disabled = self.player[0].disabled_location | (self.player[1].disabled_location << 16);
                if disabled != arg.previously_disabled_locations {
                    self.trace.message(MSG_FIELD_DISABLED).u32(disabled);
                }
                true
            }
            _ => true,
        }
    }

    fn process_startup(&mut self, arg: &mut Startup) -> bool {
        match arg.step {
            0 => {
                self.core.shuffle_hand_check = PerPlayerBool::default();
                self.core.shuffle_deck_check = PerPlayerBool::default();
                self.field_raise_event(None, EVENT_STARTUP, None, 0, 0, 0, 0);
                self.process_instant_event();
                false
            }
            1 => {
                for p in 0..2 {
                    self.core.shuffle_hand_check[p] = false;
                    self.core.shuffle_deck_check[p] = false;
                    let start_count = self.player[p].start_count;
                    if start_count > 0 {
                        self.field_draw(None, REASON_RULE, PLAYER_NONE, p, start_count as u16);
                    }
                }
                self.emplace_process(std::mem::take(&mut arg.turn));
                true
            }
            _ => true,
        }
    }
}

type PerPlayerBool = PerPlayer<bool>;
