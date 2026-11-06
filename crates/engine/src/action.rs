use crate::buffer::ProgressiveBuffer;
use crate::constants::*;
use crate::duel::{Duel, Status};
use crate::processor::Unit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prompt {
    IdleCommand { player: u8 },
    BattleCommand { player: u8 },
    YesNo { player: u8 },
    // One of the options of MSG_SELECT_OPTION.
    Option { player: u8 },
    // Whether to activate the card's effect (MSG_SELECT_EFFECTYN), answered like YesNo.
    EffectYesNo { player: u8 },
    Chain { player: u8, forced: bool },
    // Picked one card at a time; `cancelable` as the message shows it, so also when min is 0.
    Card { player: u8, cancelable: bool, min: u8, max: u8 },
    // Like Card, but min counts release units (a double tribute counts 2) while max counts cards.
    Tribute { player: u8, cancelable: bool, min: u8, max: u8 },
    // One card per answer, picking or dropping it; ending the selection is a Cancel or else a Finish.
    UnselectCard { player: u8, cancelable: bool, finishable: bool },
    // `blocked` has a bit set per unavailable zone, as Action::Place numbers them.
    Place { player: u8, count: u8, blocked: u32 },
    Position { player: u8, positions: u8 },
}

// Kinds in the order of their response codes (playerop.cpp); indices point into the prompt's lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdleCommand {
    Summon(u16),
    SpecialSummon(u16),
    Reposition(u16),
    MonsterSet(u16),
    SpellSet(u16),
    Activate(u16),
    ToBattle,
    ToEnd,
    Shuffle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleCommand {
    Activate(u16),
    Attack(u16),
    ToMain2,
    ToEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    IdleCommand(IdleCommand),
    BattleCommand(BattleCommand),
    YesNo(bool),
    Option(u16),
    // None passes.
    Chain(Option<u16>),
    // Adds one card, by index, to the selection; the prompt resolves once `max` cards are picked.
    Card(u16),
    // Ends a selection of at least `min` cards.
    Finish,
    Cancel,
    // A zone: the own monster zones 0-6 and spell zones 8-15, the opponent's at +16.
    Place(u8),
    Position(u8),
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ActionError {
    #[error("no prompt is waiting for an answer")]
    NoPrompt,
    #[error("{0:?} is not a legal action")]
    Illegal(Action),
    #[error("response {0:02x?} does not answer the prompt")]
    Undecodable(Vec<u8>),
}

impl IdleCommand {
    fn from_response(kind: i32, index: u16) -> Option<Self> {
        Some(match kind {
            0 => Self::Summon(index),
            1 => Self::SpecialSummon(index),
            2 => Self::Reposition(index),
            3 => Self::MonsterSet(index),
            4 => Self::SpellSet(index),
            5 => Self::Activate(index),
            6 => Self::ToBattle,
            7 => Self::ToEnd,
            8 => Self::Shuffle,
            _ => return None,
        })
    }

    fn response(self) -> i32 {
        let (kind, index) = match self {
            Self::Summon(index) => (0, index),
            Self::SpecialSummon(index) => (1, index),
            Self::Reposition(index) => (2, index),
            Self::MonsterSet(index) => (3, index),
            Self::SpellSet(index) => (4, index),
            Self::Activate(index) => (5, index),
            Self::ToBattle => (6, 0),
            Self::ToEnd => (7, 0),
            Self::Shuffle => (8, 0),
        };
        i32::from(index) << 16 | kind
    }
}

impl BattleCommand {
    fn from_response(kind: i32, index: u16) -> Option<Self> {
        Some(match kind {
            0 => Self::Activate(index),
            1 => Self::Attack(index),
            2 => Self::ToMain2,
            3 => Self::ToEnd,
            _ => return None,
        })
    }

    fn response(self) -> i32 {
        let (kind, index) = match self {
            Self::Activate(index) => (0, index),
            Self::Attack(index) => (1, index),
            Self::ToMain2 => (2, 0),
            Self::ToEnd => (3, 0),
        };
        i32::from(index) << 16 | kind
    }
}

fn indices(count: usize) -> impl Iterator<Item = u16> {
    (0..count).map(|index| index as u16)
}

// Monster zone 7 does not exist; see SelectPlace in playerop.cpp.
const ZONES: u32 = 0xff7f_ff7f;
const POSITIONS: [u32; 4] = [POS_FACEUP_ATTACK, POS_FACEDOWN_ATTACK, POS_FACEUP_DEFENSE, POS_FACEDOWN_DEFENSE];

// The three response bytes of SelectPlace for a zone, and back.
fn place_response(player: u8, zone: u8) -> [u8; 3] {
    let location = if zone % 16 >= 8 { LOCATION_SZONE } else { LOCATION_MZONE };
    [if zone >= 16 { 1 - player } else { player }, location, zone % 8]
}

fn response_zone(player: u8, [select_player, location, sequence]: [u8; 3]) -> u8 {
    sequence % 8 + if location == LOCATION_SZONE { 8 } else { 0 } + if select_player == player { 0 } else { 16 }
}

impl Duel {
    pub fn prompt(&self) -> Option<Prompt> {
        match self.core.units.last()? {
            Unit::SelectIdleCmd(arg) if arg.step > 0 => Some(Prompt::IdleCommand { player: arg.playerid }),
            Unit::SelectBattleCmd(arg) if arg.step > 0 => Some(Prompt::BattleCommand { player: arg.playerid }),
            Unit::SelectYesNo(arg) if arg.step > 0 => Some(Prompt::YesNo { player: arg.playerid }),
            Unit::SelectOption(arg) if arg.step > 0 => Some(Prompt::Option { player: arg.playerid }),
            Unit::SelectEffectYesNo(arg) if arg.step > 0 => Some(Prompt::EffectYesNo { player: arg.playerid }),
            Unit::SelectChain(arg) if arg.step > 0 => Some(Prompt::Chain {
                player: arg.playerid,
                forced: arg.forced,
            }),
            Unit::SelectCard(arg) if arg.step > 0 => Some(Prompt::Card {
                player: arg.playerid,
                cancelable: arg.cancelable || arg.min == 0,
                min: arg.min,
                max: arg.max,
            }),
            Unit::SelectTributeP(arg) if arg.step > 0 => Some(Prompt::Tribute {
                player: arg.playerid,
                cancelable: arg.cancelable || arg.min == 0,
                min: arg.min,
                max: arg.max,
            }),
            Unit::SelectUnselectCard(arg) if arg.step > 0 => Some(Prompt::UnselectCard {
                player: arg.playerid,
                cancelable: arg.cancelable,
                finishable: arg.finishable,
            }),
            Unit::SelectPlace(arg) if arg.step > 0 => Some(Prompt::Place {
                player: arg.playerid,
                count: arg.count,
                blocked: arg.flag,
            }),
            Unit::SelectPosition(arg) if arg.step > 0 => Some(Prompt::Position {
                player: arg.playerid,
                positions: arg.positions,
            }),
            _ => None,
        }
    }

    pub fn legal_actions(&self) -> Vec<Action> {
        let mut actions = Vec::new();
        self.legal_actions_into(&mut actions);
        actions
    }

    // Replaces the contents of `actions`; reusing one buffer keeps a simulation loop free of allocations.
    pub fn legal_actions_into(&self, actions: &mut Vec<Action>) {
        actions.clear();
        let Some(prompt) = self.prompt() else { return };
        match prompt {
            Prompt::IdleCommand { player } => {
                let core = &self.core;
                let lists = [
                    &core.summonable_cards,
                    &core.spsummonable_cards,
                    &core.repositionable_cards,
                    &core.msetable_cards,
                    &core.ssetable_cards,
                ];
                let counts = lists.map(|list| list.len()).into_iter().chain([core.select_chains.len()]);
                for (count, kind) in counts.zip(0..) {
                    let commands = indices(count).filter_map(|index| IdleCommand::from_response(kind, index));
                    actions.extend(commands.map(Action::IdleCommand));
                }
                let flags = [
                    (IdleCommand::ToBattle, self.infos.phase == PHASE_MAIN1 && core.to_bp),
                    (IdleCommand::ToEnd, core.to_ep),
                    (
                        IdleCommand::Shuffle,
                        self.infos.can_shuffle && self.player[player].list_hand.len() > 1,
                    ),
                ];
                let commands = flags.into_iter().filter_map(|(command, legal)| legal.then_some(command));
                actions.extend(commands.map(Action::IdleCommand));
            }
            Prompt::BattleCommand { .. } => {
                let activations = indices(self.core.select_chains.len()).map(BattleCommand::Activate);
                let commands = activations.chain(indices(self.core.attackable_cards.len()).map(BattleCommand::Attack));
                let flags = [(BattleCommand::ToMain2, self.core.to_m2), (BattleCommand::ToEnd, self.core.to_ep)];
                actions.extend(
                    commands
                        .chain(flags.into_iter().filter_map(|(command, legal)| legal.then_some(command)))
                        .map(Action::BattleCommand),
                );
            }
            Prompt::YesNo { .. } | Prompt::EffectYesNo { .. } => actions.extend([Action::YesNo(false), Action::YesNo(true)]),
            Prompt::Option { .. } => actions.extend(indices(self.core.select_options.len()).map(Action::Option)),
            Prompt::Chain { forced, .. } => {
                let chains = indices(self.core.select_chains.len()).map(|index| Action::Chain(Some(index)));
                actions.extend((!forced).then_some(Action::Chain(None)).into_iter().chain(chains));
            }
            Prompt::Card { .. } | Prompt::Tribute { .. } => {
                let picks = indices(self.core.select_cards.len()).map(Action::Card);
                let ends = [Action::Finish, Action::Cancel];
                actions.extend(picks.chain(ends).filter(|&action| self.is_legal_pick(prompt, action)));
            }
            Prompt::UnselectCard {
                cancelable, finishable, ..
            } => {
                let end = if cancelable {
                    Some(Action::Cancel)
                } else {
                    finishable.then_some(Action::Finish)
                };
                let count = self.core.select_cards.len() + self.core.unselect_cards.len();
                actions.extend(indices(count).map(Action::Card).chain(end));
            }
            Prompt::Place { player, blocked, .. } => {
                let open = ZONES & !blocked & !self.picked_zones(player);
                actions.extend((0..32).filter(|&zone| open & (1 << zone) != 0).map(Action::Place));
            }
            Prompt::Position { positions, .. } => actions.extend(
                POSITIONS
                    .into_iter()
                    .filter(|&position| u32::from(positions) & position != 0)
                    .map(|position| Action::Position(position as u8)),
            ),
        }
    }

    // Whether `legal_actions` would list the action, without building the list.
    fn is_legal(&self, prompt: Prompt, action: Action) -> bool {
        let core = &self.core;
        let below = |index: u16, count: usize| usize::from(index) < count;
        match (prompt, action) {
            (Prompt::IdleCommand { player }, Action::IdleCommand(command)) => match command {
                IdleCommand::Summon(index) => below(index, core.summonable_cards.len()),
                IdleCommand::SpecialSummon(index) => below(index, core.spsummonable_cards.len()),
                IdleCommand::Reposition(index) => below(index, core.repositionable_cards.len()),
                IdleCommand::MonsterSet(index) => below(index, core.msetable_cards.len()),
                IdleCommand::SpellSet(index) => below(index, core.ssetable_cards.len()),
                IdleCommand::Activate(index) => below(index, core.select_chains.len()),
                IdleCommand::ToBattle => self.infos.phase == PHASE_MAIN1 && core.to_bp,
                IdleCommand::ToEnd => core.to_ep,
                IdleCommand::Shuffle => self.infos.can_shuffle && self.player[player].list_hand.len() > 1,
            },
            (Prompt::BattleCommand { .. }, Action::BattleCommand(command)) => match command {
                BattleCommand::Activate(index) => below(index, core.select_chains.len()),
                BattleCommand::Attack(index) => below(index, core.attackable_cards.len()),
                BattleCommand::ToMain2 => core.to_m2,
                BattleCommand::ToEnd => core.to_ep,
            },
            (Prompt::YesNo { .. } | Prompt::EffectYesNo { .. }, Action::YesNo(_)) => true,
            (Prompt::Option { .. }, Action::Option(index)) => below(index, core.select_options.len()),
            (Prompt::Chain { forced, .. }, Action::Chain(None)) => !forced,
            (Prompt::Chain { .. }, Action::Chain(Some(index))) => below(index, core.select_chains.len()),
            (Prompt::Card { .. } | Prompt::Tribute { .. }, _) => self.is_legal_pick(prompt, action),
            (Prompt::UnselectCard { .. }, Action::Card(index)) => below(index, core.select_cards.len() + core.unselect_cards.len()),
            (Prompt::UnselectCard { cancelable, .. }, Action::Cancel) => cancelable,
            (
                Prompt::UnselectCard {
                    cancelable, finishable, ..
                },
                Action::Finish,
            ) => !cancelable && finishable,
            (Prompt::Place { player, blocked, .. }, Action::Place(zone)) => {
                zone < 32 && (ZONES & !blocked & !self.picked_zones(player)) & (1 << zone) != 0
            }
            (Prompt::Position { positions, .. }, Action::Position(position)) => {
                POSITIONS.contains(&u32::from(position)) && positions & position != 0
            }
            _ => false,
        }
    }

    // Card and Tribute: any unpicked card while fewer than `max` are picked, Finish once enough are picked (min counts
    // release units for tributes), and Cancel if allowed.
    fn is_legal_pick(&self, prompt: Prompt, action: Action) -> bool {
        let (Prompt::Card { cancelable, min, max, .. } | Prompt::Tribute { cancelable, min, max, .. }) = prompt else {
            return false;
        };
        let picked = self.returns.at_u32(1) as usize;
        match action {
            Action::Card(index) => {
                picked < usize::from(max)
                    && usize::from(index) < self.core.select_cards.len()
                    && !self.picked_cards().any(|pick| pick == index)
            }
            Action::Finish => {
                let enough = if matches!(prompt, Prompt::Tribute { .. }) {
                    let cards = self
                        .picked_cards()
                        .filter_map(|index| self.core.select_cards.get(usize::from(index)));
                    cards.map(|&card| self.cards[card].release_param).sum::<u32>() >= u32::from(min)
                } else {
                    picked >= usize::from(min)
                };
                enough && picked < usize::from(max)
            }
            Action::Cancel => cancelable,
            _ => false,
        }
    }

    // The zones picked so far, kept in the response buffer as the SelectPlace response, as a mask of Place numbers.
    fn picked_zones(&self, player: u8) -> u32 {
        self.returns
            .data
            .as_chunks::<3>()
            .0
            .iter()
            .fold(0, |picked, &bytes| picked | 1 << response_zone(player, bytes))
    }

    // A card selection in progress lives in the response buffer, in its type-0 encoding (count, then u32 indices).
    fn picked_cards(&self) -> impl Iterator<Item = u16> + '_ {
        (0..self.returns.at_u32(1) as usize).map(|index| self.returns.at_u32(index + 2) as u16)
    }

    fn respond(&mut self, value: i32) {
        self.returns.clear();
        self.returns.set_i32(0, value);
    }

    // Answers the prompt and runs to the next one, except while a card selection still needs picks.
    pub fn apply(&mut self, action: Action) -> Result<Status, ActionError> {
        let prompt = self.prompt().ok_or(ActionError::NoPrompt)?;
        let legal = self.is_legal(prompt, action);
        debug_assert_eq!(legal, self.legal_actions().contains(&action), "{prompt:?} {action:?}");
        if !legal {
            return Err(ActionError::Illegal(action));
        }
        match action {
            Action::IdleCommand(command) => self.respond(command.response()),
            Action::BattleCommand(command) => self.respond(command.response()),
            Action::YesNo(yes) => self.respond(i32::from(yes)),
            Action::Option(index) => self.respond(i32::from(index)),
            Action::Chain(index) => self.respond(index.map_or(-1, i32::from)),
            Action::Card(index) if matches!(prompt, Prompt::UnselectCard { .. }) => {
                self.respond(1);
                self.returns.set_i32(1, i32::from(index));
            }
            Action::Card(index) => {
                let count = self.returns.at_u32(1);
                self.returns.set_i32(0, 0);
                self.returns.set_u32(count as usize + 2, u32::from(index));
                self.returns.set_u32(1, count + 1);
                if matches!(prompt, Prompt::Card { max, .. } | Prompt::Tribute { max, .. } if count + 1 < u32::from(max)) {
                    return Ok(Status::Awaiting);
                }
            }
            Action::Finish if matches!(prompt, Prompt::UnselectCard { .. }) => self.respond(-1),
            Action::Finish => {
                self.returns.set_i32(0, 0);
                self.returns.set_u32(1, self.returns.at_u32(1));
            }
            Action::Cancel => self.respond(-1),
            Action::Place(zone) => {
                let Prompt::Place { player, count, .. } = prompt else {
                    return Err(ActionError::Illegal(action));
                };
                self.returns.data.extend(place_response(player, zone));
                if self.returns.data.len() < 3 * usize::from(count) {
                    return Ok(Status::Awaiting);
                }
            }
            Action::Position(position) => self.respond(i32::from(position)),
        }
        Ok(self.run())
    }

    // Decodes ocgcore response bytes for the current prompt into the actions that give the same response.
    pub fn decode_response(&self, response: &[u8]) -> Result<Vec<Action>, ActionError> {
        let prompt = self.prompt().ok_or(ActionError::NoPrompt)?;
        let buffer = ProgressiveBuffer {
            data: response.iter().copied().collect(),
        };
        let value = buffer.at_i32(0);
        let (kind, index) = (value & 0xffff, (value >> 16) as u16);
        let undecodable = || ActionError::Undecodable(response.to_vec());
        Ok(match prompt {
            Prompt::IdleCommand { .. } => vec![Action::IdleCommand(
                IdleCommand::from_response(kind, index).ok_or_else(undecodable)?,
            )],
            Prompt::BattleCommand { .. } => vec![Action::BattleCommand(
                BattleCommand::from_response(kind, index).ok_or_else(undecodable)?,
            )],
            Prompt::YesNo { .. } | Prompt::EffectYesNo { .. } => vec![Action::YesNo(match value {
                0 => false,
                1 => true,
                _ => return Err(undecodable()),
            })],
            Prompt::Option { .. } => vec![Action::Option(u16::try_from(value).map_err(|_| undecodable())?)],
            Prompt::Chain { .. } => {
                vec![Action::Chain(if value == -1 {
                    None
                } else {
                    Some(u16::try_from(value).map_err(|_| undecodable())?)
                })]
            }
            Prompt::Card { max, .. } | Prompt::Tribute { max, .. } => {
                let picks: Vec<u16> = match value {
                    -1 => return Ok(vec![Action::Cancel]),
                    0 => (0..buffer.at_u32(1) as usize).map(|pick| buffer.at_u32(pick + 2) as u16).collect(),
                    1 => (0..buffer.at_u32(1) as usize).map(|pick| buffer.at_u16(pick + 4)).collect(),
                    2 => (0..buffer.at_u32(1) as usize)
                        .map(|pick| u16::from(buffer.at_u8(pick + 8)))
                        .collect(),
                    3 => indices(self.core.select_cards.len())
                        .filter(|&pick| buffer.bit_get(usize::from(pick) + 32))
                        .collect(),
                    _ => return Err(undecodable()),
                };
                let finish = (picks.len() < usize::from(max)).then_some(Action::Finish);
                picks.into_iter().map(Action::Card).chain(finish).collect()
            }
            Prompt::UnselectCard { cancelable, .. } => match value {
                -1 => vec![if cancelable { Action::Cancel } else { Action::Finish }],
                1 => vec![Action::Card(u16::try_from(buffer.at_i32(1)).map_err(|_| undecodable())?)],
                _ => return Err(undecodable()),
            },
            Prompt::Place { player, count, .. } => response
                .as_chunks::<3>()
                .0
                .iter()
                .take(usize::from(count))
                .map(|&bytes| Action::Place(response_zone(player, bytes)))
                .collect(),
            Prompt::Position { .. } => vec![Action::Position(u8::try_from(value).map_err(|_| undecodable())?)],
        })
    }
}
