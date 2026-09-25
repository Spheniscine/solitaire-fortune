use dioxus::prelude::*;

use crate::{components::{SkinTrait, VIDEO_GAMEPLAY, rem}, game::{Card, DECK_SIZE, GameState, NUM_TRUMPS, ScreenState, Suit, TRUMP_RANK_MAX, TRUMP_RANK_MIN}};

#[component]
fn Emph(children: Element) -> Element {
    rsx! {
        strong {
            color: "#ff0",
            {children}
        }
    }
}

#[component]
pub fn Help(mut game_state: Signal<GameState>) -> Element {
    let st = game_state.read();
    let skin = st.skin;

    let rank_text_trump = |rank: u8| {
        rsx! {
            span {
                font_size: "1.2em",
                {skin.render_rank(&Card { rank, suit: Suit::Trump })}
            }
        }
    };

    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; font-size: 3.7rem; color: #fff; padding: 4rem;",
            class: "help",

            div {
                text_align: "left",

                p {
                    margin_top: "0",
                    "The deck has {DECK_SIZE} cards:"

                    ul {
                        li {
                            "52 " Emph {"common"} " cards, equivalent to a standard deck of 13 ranks in each of 4 suits."
                        }
                        li {
                            "{NUM_TRUMPS} additional " Emph {"trump"} " cards, ranked from " {rank_text_trump(TRUMP_RANK_MIN)} "~" {rank_text_trump(TRUMP_RANK_MAX)} 
                            ", acting as a fifth suit."
                        }
                    }
                }

                p {
                    "Cards stack in the " Emph {"tableau"} " in " Emph {"incrementing or decrementing"} " order, " Emph {"by suit"} ". Only one card may be moved at a time."
                }

                p {
                    "To " Emph {"win the game"} ", move all cards to the foundations."
                    ul {
                        li { "Trump cards go to the top-left, up from " {rank_text_trump(TRUMP_RANK_MIN)} " and down from " {rank_text_trump(TRUMP_RANK_MAX)} " until they meet." }
                        li { "Common cards go to the top-right, in incrementing order by suit." }
                    }
                }

                p {
                    "There are three " Emph {"free cells"} " in the top-middle that may hold up to one card each, but each has a " Emph {"drawback"} ":"

                    ul {
                        li {
                            "The leftmost free cell will block trump cards from going to the foundations when filled."
                        }
                        li {
                            "The middle free cell may only be used once."
                        }
                        li {
                            "The rightmost free cell will block common cards from going to the foundations when filled."
                        }
                    }

                    "Cards can’t be moved between free cells."
                }

                p {
                    Emph{"Shortcut notes:"},
                    ul {
                        li {
                            Emph{"Supermoves:"},
                            " Multiple cards in sequence may be selected. They will stack in reverse order when moved, as if moved one by one. Cards
                            may also be moved to the foundations this way."
                        }

                        li {
                            Emph{"Double click:"},
                            " Double-clicking on a card or stack will try to move it to the foundations if possible."
                        }
                    }
                }

                div {
                    position: "absolute",
                    bottom: rem(2.),
                    width: "92rem",
                    display: "flex",
                    justify_content: "center",

                    a {
                        href: VIDEO_GAMEPLAY,
                        target: "_blank",
                        text_decoration: "none",
                        margin_right: rem(4.),
                        div {
                            width: rem(30.),
                            position: "relative",
                            class: "game-button",
                            "Example video"
                        }
                    }

                    div {
                        width: rem(30.),
                        position: "relative",
                        class: "game-button",
                        onclick: move |_| game_state.write().screen_state = ScreenState::Game,
                        "Back to game"
                    }
                }
            }
        }
    }
}