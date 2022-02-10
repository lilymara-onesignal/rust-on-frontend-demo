use rand::{thread_rng, Rng};
use serde_json::Value;
use std::time::Duration;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

fn main() {
    yew::start_app::<Root>();
}

#[derive(PartialEq, Clone, Debug)]
struct Pokemon {
    id: i32,
    name: String,
    image_src: String,
}

#[derive(PartialEq, Clone, Debug)]
enum FetchState<T> {
    Empty,
    Fetching,
    Done(T),
}

#[function_component(Root)]
fn root() -> Html {
    let pokemon = use_state_eq(|| FetchState::<Pokemon>::Empty);
    let guess = use_state_eq(|| Guess::None);

    let pokemon_inner = pokemon.clone();
    let guess_inner = guess.clone();
    let onclick = Callback::from(move |_| {
        let pokemon = pokemon_inner.clone();
        pokemon.set(FetchState::Fetching);
        guess_inner.set(Guess::None);

        let mut rng = thread_rng();
        let id: i32 = rng.gen_range(1..=100);

        spawn_local(async move {
            gloo::timers::future::sleep(Duration::from_millis(500)).await;

            let url = format!("https://pokeapi.co/api/v2/pokemon/{id}");
            let request = reqwest::get(url).await.unwrap();
            let text = request.text().await.unwrap();

            let v: Value = serde_json::from_str(&text).unwrap();

            let name = v["name"].as_str().unwrap().into();
            let image_src = v["sprites"]["front_default"].as_str().unwrap().into();

            pokemon.set(FetchState::Done(Pokemon {
                id,
                name,
                image_src,
            }))
        });
    });

    let button_disabled = *pokemon == FetchState::Fetching;

    html! {
        <div class="App">
            <button {onclick} disabled={button_disabled}>{"get pokemon"}</button>
            <ViewPokemon pokemon={(*pokemon).clone()} guess={guess.clone()} />
        </div>
    }
}

#[derive(PartialEq, Properties)]
struct ViewPokemonProps {
    pokemon: FetchState<Pokemon>,
    guess: UseStateHandle<Guess>,
}

#[derive(PartialEq, Clone)]
enum Guess {
    None,
    Correct(String),
    Incorrect(String),
}

#[function_component(ViewPokemon)]
fn view_pokemon(p: &ViewPokemonProps) -> Html {
    let pokemon = match &p.pokemon {
        FetchState::Done(p) => p,
        FetchState::Fetching => {
            return html! {
                <div>{"loading..."}</div>
            }
        }
        _ => return html! {},
    };

    let name = pokemon.name.clone();

    let input_ref = NodeRef::default();
    let input_ref_inner = input_ref.clone();
    let guess_inner = p.guess.clone();
    let onclick = Callback::from(move |_| {
        let input = input_ref_inner.cast::<HtmlInputElement>().unwrap();
        let name_guess = input.value().to_lowercase();

        if name_guess == name {
            guess_inner.set(Guess::Correct(name_guess));
        } else {
            guess_inner.set(Guess::Incorrect(name_guess));
        }
    });

    html! {
        <div>
            <div>
                <img src={pokemon.image_src.clone()} />
            </div>
            <input ref={input_ref.clone()} type="text" />
            <button {onclick}>{ "submit" }</button>
            <ViewGuess guess={(*p.guess).clone()} />
        </div>
    }
}

#[derive(PartialEq, Properties)]
struct ViewGuessProps {
    guess: Guess,
}

#[function_component(ViewGuess)]
fn view_guess(p: &ViewGuessProps) -> Html {
    let guess_text = match &p.guess {
        Guess::None => return html! {},
        Guess::Correct(good_guess) => format!("Yes, that's a {good_guess}!"),
        Guess::Incorrect(bad_guess) => format!("No, that's not a {bad_guess}"),
    };

    html! {
        <div>{ guess_text }</div>
    }
}
