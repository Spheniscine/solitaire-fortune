use dioxus::prelude::*;
use glam::Vec2;

use crate::{components::{BASE_CARD_HEIGHT, BASE_CARD_WIDTH, CARD_BORDER_RADIUS_RATIO, CARD_FRAME_DEFAULT_COLOR, CardComponent, CardFrame, SkinTrait, rem}, game::{AnimationKey, Board, BoardPos, Card, DepotRole, FREECELL_BLOCKS_COMMON, FREECELL_BLOCKS_TRUMP, FREECELL_SINGLE_USE, NUM_DEPOTS, RANK_MIN, Skin, Suit, TRUMP_RANK_MAX, TRUMP_RANK_MIN}};

#[component]
pub fn BoardComponent(
    position: Vec2,
    board: Board,
    skin: Skin,
    #[props(default)]
    onclick: EventHandler<BoardPos>,
    #[props(default)]
    ondoubleclick: EventHandler<BoardPos>,

    #[props(default)]
    animation_key: AnimationKey,
    #[props(default)]
    is_won: bool,
) -> Element {
    let card_width = BASE_CARD_WIDTH;
    let card_height = BASE_CARD_HEIGHT;
    let spacer_x = 0.8f32;
    let spacer_y = 1f32;
    let start_y = 2f32;

    let pos_x = {
        let w = DepotRole::Tableau.number_of() as f32;
        let left = 50. - (w * card_width + (w-1.) * spacer_x) / 2.;
        move |i: usize| {
            left + (card_width + spacer_x) * i as f32
        }
    };

    let pos_y = |i: usize| start_y + (card_height + spacer_y) * i as f32;
    let column_card_offset = Vec2::new(0., card_height / 2.);

    let get_pos = |depot: usize, ord: usize| {
        let (role, index) = DepotRole::role_and_subindex(depot).unwrap();
        match role {
            DepotRole::TrumpLow => Vec2::new(pos_x(0), pos_y(0)),
            DepotRole::TrumpHigh => Vec2::new(pos_x(1), pos_y(0)),
            DepotRole::TrumpLast => Vec2::new(pos_x(0).midpoint(pos_x(1)), pos_y(0) - 1.),
            DepotRole::CommonHome => Vec2::new(pos_x(7 + index), pos_y(0)),
            DepotRole::FreeCell => Vec2::new(pos_x(3 + index), pos_y(0)),
            DepotRole::Tableau => Vec2::new(pos_x(index), pos_y(1)) + column_card_offset * ord as f32,
        }
    };

    let symbol2 = |text: &str| rsx! {
        span {
            font_family: "'Noto Sans Symbols 2'",
            position: "relative",
            top: "0.12em",
            {text}
        }
    };

    let dashed = |depot: usize| {
        depot == FREECELL_SINGLE_USE
    };

    let get_hint = |depot: usize| {
        let role = DepotRole::role(depot).unwrap();
        match role {
            DepotRole::TrumpLow => Some(skin.render_rank(&Card { rank: TRUMP_RANK_MIN, suit: Suit::Trump })),
            DepotRole::TrumpHigh => Some(skin.render_rank(&Card { rank: TRUMP_RANK_MAX, suit: Suit::Trump })),
            DepotRole::TrumpLast => None,
            DepotRole::CommonHome => Some(skin.render_rank(&Card { rank: RANK_MIN, suit: Suit::Spades })),
            DepotRole::FreeCell => if board.freecell_single_used && depot == FREECELL_SINGLE_USE {None} 
                else {Some(symbol2("✽"))},
            DepotRole::Tableau => Some(rsx!{}),
        }
    };

    let selected_height = if let Some(BoardPos { depot_index, card_index }) = board.selected {
        let d = if DepotRole::role(depot_index).unwrap() == DepotRole::Tableau {
            board.depots[depot_index].len() - card_index - 1
        } else {
            0
        };

        card_height + column_card_offset.y * d as f32
    } else {0.};

    let filled_color = |depot: usize| {
        if board.depots[depot].is_empty() {CARD_FRAME_DEFAULT_COLOR} else {"#ff0"}
    };
    let freecell_label_width = 4f32;

    let freecell_labels = {
        let pos1 = get_pos(FREECELL_BLOCKS_TRUMP, 0) - Vec2::new(spacer_x + freecell_label_width, 0.);
        let pos2 = get_pos(FREECELL_BLOCKS_COMMON, 0) + Vec2::new(card_width + spacer_x, 0.);

        let label = |pos: Vec2, arrow: &str, depot: usize| {
            rsx! {
                div {
                    style: "place-items: center;",
                    position: "absolute",
                    height: rem(card_height),
                    width: rem(freecell_label_width),
                    line_height: 1,
                    top: rem(pos.y),
                    left: rem(pos.x),
                    color: filled_color(depot),
                    font_size: rem(3.4),
                    display: "grid",
                    

                    span {
                        text_align: "center",
                        span {
                            font_family: "'Noto Emoji'",
                            "🔒"
                        } br {}
                        span {
                            font_family: "'Noto Sans Symbols 2'",
                            {arrow}
                        }
                    }
                }
            }
        };

        rsx! {
            {label(pos1, "⬅", FREECELL_BLOCKS_TRUMP)}
            {label(pos2, "⮕", FREECELL_BLOCKS_COMMON)}
        }
    };

    rsx! {
        div {
            position: "absolute",
            top: rem(position.y),
            left: rem(position.x),

            {freecell_labels}

            for depot in 0..NUM_DEPOTS {
                if let Some(hint) = get_hint(depot) {
                    CardFrame { 
                        position: get_pos(depot, 0),
                        width: card_width,
                        hint,
                        onclick: move |_| {
                            onclick.call(BoardPos::new(depot, !0))
                        },
                        dashed: dashed(depot),
                    }
                }

                for i in 0..board.depots[depot].len() {
                    if board.selected == Some(BoardPos::new(depot, i)) {
                        div {
                            position: "absolute",
                            top: rem(get_pos(depot, i).y),
                            left: rem(get_pos(depot, i).x),
                            width: rem(card_width),
                            height: rem(selected_height),
                            background_color: "#ff0",
                            border_radius: rem(card_width * CARD_BORDER_RADIUS_RATIO),
                            class: "selected-halo",
                        }
                    }

                    CardComponent { 
                        position: get_pos(depot, i),
                        width: card_width,
                        card: board.depots[depot][i],
                        // number_hint: if !is_face_up(depot) {i + 1},
                        skin,
                        onclick: move |_| {
                            onclick.call(BoardPos::new(depot, i))
                        },
                        ondoubleclick: move |_| {
                            ondoubleclick.call(BoardPos::new(depot, i))
                        },
                    }
                }
            }

            // {anims}

            if is_won {
                div {
                    position: "absolute",
                    top: rem(25.),
                    left: rem(17.5),
                    width: rem(59.),
                    background_color: "#505",
                    padding: rem(3.),
                    color: "#fff",
                    font_size: rem(7.),
                    border_radius: rem(2.),
                    text_align: "center",
                    "YOU WIN!",
                }
            }
        }
    }
}