use std::time::Duration;

use rand::{Rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};

use crate::{components::LocalStorage, game::{Board, BoardPos, Card, DECK_SIZE, DepotRole, FREECELL_BLOCKS_COMMON, FREECELL_BLOCKS_TRUMP, FREECELL_SINGLE_USE, NUM_RANKS, NUM_TRUMPS, RANKS, Skin, Suit, TRUMP_RANK_MAX, TRUMP_RANK_MIN, TRUMP_RANKS}};

pub const ANIMATION_DURATION: Duration = Duration::from_millis(200);
pub type AnimationKey = u16;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum ActionRecord {
    Move { pos1: BoardPos, pos2: BoardPos },
    FreeCellSingleUsed, // when the single-use freecell is used
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ScreenState {
    #[default] Game, 
    Settings, Help,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GameState {
    pub board: Board,
    pub deal: Vec<Card>,
    #[serde(skip)]
    pub animation_key: AnimationKey, // used for syncing and to provide animator components with cycling keys
    pub history: Vec<ActionRecord>,
    pub undo_stack: Vec<usize>,
    pub already_won: bool,
    pub num_wins: i32,

    pub screen_state: ScreenState,

    pub allow_undo: bool,
    pub skin: Skin,
}

impl GameState {
    pub fn new_deal(rng: &mut impl Rng) -> Vec<Card> {
        let mut deck = Vec::with_capacity(DECK_SIZE);
        for rank in RANKS {
            for suit in Suit::iter_common() {
                deck.push(Card { rank, suit });
            }
        }
        for rank in TRUMP_RANKS {
            deck.push(Card { rank, suit: Suit::Trump })
        }

        deck.shuffle(rng);
        deck
    }

    pub fn init() -> Self {
        let mut res = Self {
            board: Board::empty(),
            deal: vec![],
            animation_key: 0,
            history: vec![],
            undo_stack: vec![],
            already_won: false,
            num_wins: 0,
            screen_state: ScreenState::Game,
            allow_undo: true,
            skin: Skin::default(),
        };

        res.new_game();
        res
    }

    pub fn new_game(&mut self) {
        let deal = Self::new_deal(&mut rand::rng());
        self.board = Board::from_deal(&deal);
        self.deal = deal;
        self.history.clear();
        self.undo_stack.clear();
        self.already_won = false;

        if !self.is_busy() { LocalStorage.save_game_state(&self); }
    }

    pub fn is_busy(&self) -> bool {
        self.is_acting()
    }

    pub fn is_acting(&self) -> bool {
        !self.board.animation_acts.is_empty()
    }

    pub fn undo_possible(&self) -> bool {
        self.allow_undo && !self.undo_stack.is_empty()
    }

    fn do_move_raw(&mut self, pos1: BoardPos, pos2: BoardPos) {
        self.board.do_move(pos1, pos2);
        self.history.push(ActionRecord::Move { pos1, pos2 })
    }

    pub fn can_stack(&self, back: Card, front: Card) -> bool {
        back.suit == front.suit && 
        back.rank.abs_diff(front.rank) == 1
    }

    pub fn can_select(&self, pos: BoardPos) -> bool {
        let depot = pos.depot_index;
        let ord = pos.card_index;

        if ord >= self.board.depots[depot].len() {
            return false;
        }
        let slice = &self.board.depots[depot][ord..];

        let Some(role) = DepotRole::role(depot) else { return false };
        match role {
            DepotRole::TrumpLow => false,
            DepotRole::TrumpHigh => false,
            DepotRole::TrumpLast => false,
            DepotRole::CommonHome => false,
            DepotRole::FreeCell => slice.len() <= 1,
            DepotRole::Tableau => slice.windows(2).all(|w| self.can_stack(w[0], w[1])),
        }
    }

    pub fn is_won(&self) -> bool {
        !self.board.depots[DepotRole::TrumpLast.id(0)].is_empty() &&
        DepotRole::CommonHome.range().all(|d| {
            self.board.depots[d].len() == NUM_RANKS
        })
    }

    fn move_intent(&mut self, mut pos1: BoardPos, pos2: BoardPos) -> bool {
        if pos1.depot_index == pos2.depot_index { return false; }
        let depot1 = &self.board.depots[pos1.depot_index];
        let depot2 = &self.board.depots[pos2.depot_index];
        let mut num_moved = depot1.len() - pos1.card_index;
        if pos2.card_index != depot2.len() { return false; }

        let Some(role) = DepotRole::role(pos2.depot_index) else { return false };
        let history_len = self.history.len();

        let Some(&front_card) = depot1.last() else { return false };

        let mut trump_last = false;
        match role {
            DepotRole::TrumpLow | DepotRole::TrumpHigh => {
                if !self.board.depots[FREECELL_BLOCKS_TRUMP].is_empty() { return false; }
                let ok = if let Some(&c) = depot2.last() {
                    self.can_stack(c, front_card)
                } else {
                    let rank = if role == DepotRole::TrumpLow {TRUMP_RANK_MIN} else {TRUMP_RANK_MAX};
                    front_card == Card { rank, suit: Suit::Trump }
                };
                if !ok { return false; }

                let num_trumps_sorted = self.board.depots[DepotRole::TrumpLow.id(0)].len() + 
                    self.board.depots[DepotRole::TrumpHigh.id(0)].len() + num_moved;
                if num_trumps_sorted == NUM_TRUMPS {
                    trump_last = true;
                    num_moved -= 1;
                    pos1.card_index += 1;
                }
            },
            DepotRole::TrumpLast => { return false; },
            DepotRole::CommonHome => {
                if !self.board.depots[FREECELL_BLOCKS_COMMON].is_empty() { return false; }
                let ok = depot2.last().is_none_or(|&c| self.can_stack(c, front_card));
                if !ok { return false; }
            },
            DepotRole::FreeCell => {
                if DepotRole::role(pos1.depot_index) == Some(DepotRole::FreeCell) { return false; }
                if pos2.depot_index == FREECELL_SINGLE_USE && self.board.freecell_single_used { return false; }
                if num_moved != 1 || !depot2.is_empty() { return false; }
            },
            DepotRole::Tableau => {
                let ok = depot2.last().is_none_or(|&c| self.can_stack(c, front_card));
                if !ok { return false; }
            },
        }

        if num_moved > 0 { self.do_move_raw(pos1, pos2); }
        if trump_last {
            pos1.card_index -= 1;
            self.do_move_raw(pos1, self.board.top_pos(DepotRole::TrumpLast.id(0)));
        }

        if pos1.depot_index == FREECELL_SINGLE_USE {
            self.board.freecell_single_used = true;
            self.history.push(ActionRecord::FreeCellSingleUsed);
        }

        self.undo_stack.push(history_len);
        true
    }

    pub fn onclick(&mut self, pos: BoardPos) {
        if self.is_busy() { return; }

        if let Some(src) = self.board.selected {
            if pos == src { 
                self.board.selected = None; 
                return;
            }
            if src.depot_index == pos.depot_index && self.can_select(pos) {
                self.board.selected = Some(pos);
                return;
            }

            let dest = BoardPos { depot_index: pos.depot_index, card_index: pos.card_index.wrapping_add(1) };
            self.move_intent(src, dest);
        } else {
            if self.can_select(pos) {
                self.board.selected = Some(pos);
            }
        }
    }

    fn try_sort(&mut self, pos: BoardPos, homes: impl IntoIterator<Item = usize>) {
        for dest in homes {
            let dest = self.board.top_pos(dest);
            if self.move_intent(pos, dest) {
                return;
            }
        }
    }

    pub fn ondoubleclick(&mut self, pos: BoardPos) {
        if self.is_busy() { return; }
        if !self.can_select(pos) { return; } // needed, or illegal stacks can still be moved this way!

        let Some(&card) = self.board.depots[pos.depot_index].get(pos.card_index) else { return };
        if card.suit == Suit::Trump {
            self.try_sort(pos, [DepotRole::TrumpLow.id(0), DepotRole::TrumpHigh.id(0)]);
        } else {
            self.try_sort(pos, DepotRole::CommonHome.range());
        }
    }

    pub fn advance_animations(&mut self, key: AnimationKey) {
        if key != self.animation_key { return; }
        self.animation_key = self.animation_key.wrapping_add(1);
        
        self.board.advance_actions();

        if self.is_won() {
            if !self.already_won {
                self.num_wins += 1;
                self.already_won = true;
            }
        } else {
            // self.check_auto_moves();
        }

        if !self.is_busy() { LocalStorage.save_game_state(&self); }
    }

    pub fn restart(&mut self) {
        if self.history.is_empty() || !self.undo_possible() { return; }
        self.board = Board::from_deal(&self.deal);
        self.history.clear();
        self.undo_stack.clear();

        if !self.is_busy() { LocalStorage.save_game_state(&self); }
    }

    pub fn undo(&mut self) {
        if self.is_busy() || !self.undo_possible() { return; }
        let Some(target_len) = self.undo_stack.pop() else {return};
        while self.history.len() > target_len {
            let rec = self.history.pop().unwrap();
            match rec {
                ActionRecord::Move { pos1, pos2 } => {
                    self.board.do_move(pos2, pos1)
                },
                ActionRecord::FreeCellSingleUsed => {
                    self.board.freecell_single_used = false;
                },
            }
            self.board.advance_actions(); // no animation, as repeated card moves on same card causes problems
        }

        LocalStorage.save_game_state(&self);
    }
}