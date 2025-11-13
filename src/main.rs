use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use web_sys::window;

#[derive(Clone, Debug, PartialEq)]
struct Ingredient {
    id: usize,
    name: String,
    protein: String,
    fat: String,
    net_carbs: String,
    servings: String,
}

impl Ingredient {
    fn empty(id: usize) -> Self {
        Self {
            id,
            name: String::new(),
            protein: String::new(),
            fat: String::new(),
            net_carbs: String::new(),
            servings: "1".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct RowSnapshot {
    name: String,
    per_protein: f64,
    per_fat: f64,
    per_carbs: f64,
    servings: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct RecipePayload {
    name: Option<String>,
    ingredients: Vec<IngredientPayload>,
}

#[derive(Debug, Serialize, Deserialize)]
struct IngredientPayload {
    id: usize,
    name: String,
    protein: f64,
    fat: f64,
    net_carbs: f64,
    servings: f64,
}

#[component]
pub fn App() -> impl IntoView {
    let (initial_ingredients, initial_name) =
        load_recipe_from_url().unwrap_or_else(|| (vec![Ingredient::empty(0)], String::new()));
    let initial_next_id = initial_ingredients
        .iter()
        .map(|ingredient| ingredient.id)
        .max()
        .map(|max_id| max_id + 1)
        .unwrap_or(1);

    let (ingredients, set_ingredients) = create_signal(initial_ingredients);
    let next_id = create_rw_signal(initial_next_id);
    let (recipe_name, set_recipe_name) = create_signal(initial_name);

    let add_ingredient = {
        move |_| {
            let id = next_id.get_untracked();
            next_id.update(|value| *value += 1);
            set_ingredients.update(|items| items.push(Ingredient::empty(id)));
        }
    };

    let remove_ingredient = {
        move |id: usize| {
            set_ingredients.update(|items| {
                items.retain(|item| item.id != id);
                if items.is_empty() {
                    let new_id = next_id.get_untracked();
                    next_id.update(|value| *value += 1);
                    items.push(Ingredient::empty(new_id));
                }
            });
        }
    };

    let print_recipe = |_| {
        if let Some(win) = window() {
            let _ = win.print();
        }
    };

    create_effect({
        let ingredients = ingredients;
        let recipe_name = recipe_name;
        move |_| {
            let current = ingredients.get();
            let name = recipe_name.get();
            if let Some(encoded) = encode_recipe(&current, &name) {
                let target_hash = format!("#recipe={encoded}");
                if let Some(win) = window() {
                    let location = win.location();
                    if location.hash().unwrap_or_default() != target_hash {
                        if let Ok(history) = win.history() {
                            let _ = history.replace_state_with_url(
                                &JsValue::NULL,
                                "",
                                Some(&format!(
                                    "{}{}{}",
                                    location.pathname().unwrap_or_default(),
                                    location.search().unwrap_or_default(),
                                    target_hash
                                )),
                            );
                        } else {
                            let _ = location.set_hash(&target_hash);
                        }
                    }
                }
            }
        }
    });

    let totals = create_memo(move |_| {
        ingredients.with(|items| {
            let mut total_protein = 0.0;
            let mut total_fat = 0.0;
            let mut total_carbs = 0.0;
            for item in items {
                let servings = parse_quantity(&item.servings);
                total_protein += parse_quantity(&item.protein) * servings;
                total_fat += parse_quantity(&item.fat) * servings;
                total_carbs += parse_quantity(&item.net_carbs) * servings;
            }
            (total_protein, total_fat, total_carbs)
        })
    });

    let stylesheet = include_str!("./styles.css");

    view! {
        <style>{stylesheet}</style>
        <main class="app">
            <section class="app__header screen-only">
                <h1>"P:E Diet Recipe Calculator"</h1>
                <p>
                    "The "
                    <a href="https://thepediet.com/" target="_blank">"P:E Diet"</a>
                    " focuses on maximizing protein and reducing energy (fat and net carbs). "
                    "This site provides a convenient way to calculate these ratios."
                </p>
                <p>
                    "Build a recipe from food labels, enter their per-serving macros, "
                    "and specify how many servings of each item you plan to use. "
                    "The calculator totals protein, fat, and net carbs, and "
                    "shows the overall protein efficiency ratio (protein ÷ fat+net carbs)."
                </p>
                <p>
                    "Provided by "
                    <a href="https://www.snoyman.com/" target="_blank">Michael Snoyman</a>
                    ". This project is open source, code is available at "
                    <a href="https://github.com/snoyberg/pedietcalc" target="_blank">
                        <code>"github:snoyberg/pedietcalc"</code>
                    </a>
                    "."
                </p>
                <label class="recipe-name-field">
                    <span>"Recipe name (optional)"</span>
                    <input
                        class="recipe-name-input"
                        type="text"
                        placeholder="e.g. High-protein chili"
                        prop:value=move || recipe_name.get()
                        on:input=move |ev| {
                            set_recipe_name.set(event_target_value(&ev));
                        }
                    />
                </label>
                </section>

                <section class="app__actions screen-only">
                    <div class="button-row">
                        <button class="primary" on:click=add_ingredient>
                            "+ Add food"
                        </button>
                        <button class="secondary" on:click=print_recipe>
                            "Print recipe"
                        </button>
                    </div>
                </section>

            <section class="app__ingredients screen-only">
                <For
                    each=move || ingredients.get()
                    key=|ingredient: &Ingredient| ingredient.id
                    children=move |ingredient: Ingredient| {
                        let id = ingredient.id;
                            let per_recipe_protein = {
                                let ingredients = ingredients;
                                move || {
                                    ingredients.with(|items| {
                                        items
                                            .iter()
                                            .find(|item| item.id == id)
                                            .map(|item| parse_quantity(&item.protein) * parse_quantity(&item.servings))
                                            .unwrap_or_default()
                                    })
                                }
                            };
                            let per_recipe_fat = {
                                let ingredients = ingredients;
                                move || {
                                    ingredients.with(|items| {
                                        items
                                            .iter()
                                            .find(|item| item.id == id)
                                            .map(|item| parse_quantity(&item.fat) * parse_quantity(&item.servings))
                                            .unwrap_or_default()
                                    })
                                }
                            };
                            let per_recipe_carbs = {
                                let ingredients = ingredients;
                                move || {
                                    ingredients.with(|items| {
                                        items
                                            .iter()
                                            .find(|item| item.id == id)
                                            .map(|item| parse_quantity(&item.net_carbs) * parse_quantity(&item.servings))
                                            .unwrap_or_default()
                                    })
                                }
                            };

                        view! {
                            <article class="ingredient-card">
                                <div class="card__header">
                                    <input
                                        class="text-input"
                                        type="text"
                                        placeholder="Ingredient name"
                                        prop:value=move || {
                                            ingredients.with(|items| {
                                                items
                                                    .iter()
                                                    .find(|item| item.id == id)
                                                    .map(|item| item.name.clone())
                                                    .unwrap_or_default()
                                            })
                                        }
                                        on:input=move |ev| {
                                            let value = leptos::event_target_value(&ev);
                                            update_ingredient(set_ingredients, id, |item| item.name = value);
                                        }
                                    />
                                    <button
                                        class="ghost"
                                        disabled=move || ingredients.with(|items| items.len() <= 1)
                                        on:click=move |_| remove_ingredient(id)
                                    >
                                        "Remove"
                                    </button>
                                </div>

                                <div class="card__grid">
                                    {macro_input(
                                        "Protein (g per serving)",
                                            {
                                                move || {
                                                    ingredients.with(|items| {
                                                        items
                                                            .iter()
                                                            .find(|item| item.id == id)
                                                            .map(|item| item.protein.clone())
                                                            .unwrap_or_default()
                                                    })
                                                }
                                            },
                                            move |value| {
                                                update_ingredient(set_ingredients, id, |item| item.protein = value);
                                            },
                                        )}
                                        {macro_input(
                                            "Fat (g per serving)",
                                            {
                                                let ingredients = ingredients;
                                                move || {
                                                    ingredients.with(|items| {
                                                        items
                                                            .iter()
                                                            .find(|item| item.id == id)
                                                            .map(|item| item.fat.clone())
                                                            .unwrap_or_default()
                                                    })
                                                }
                                            },
                                            move |value| {
                                                update_ingredient(set_ingredients, id, |item| item.fat = value);
                                            },
                                        )}
                                        {macro_input(
                                            "Net carbs (g per serving)",
                                            {
                                                let ingredients = ingredients;
                                                move || {
                                                    ingredients.with(|items| {
                                                        items
                                                            .iter()
                                                            .find(|item| item.id == id)
                                                            .map(|item| item.net_carbs.clone())
                                                            .unwrap_or_default()
                                                    })
                                                }
                                            },
                                            move |value| {
                                                update_ingredient(set_ingredients, id, |item| item.net_carbs = value);
                                            },
                                        )}
                                        {macro_input(
                                            "Servings used in recipe",
                                            {
                                                let ingredients = ingredients;
                                                move || {
                                                    ingredients.with(|items| {
                                                        items
                                                            .iter()
                                                            .find(|item| item.id == id)
                                                            .map(|item| item.servings.clone())
                                                            .unwrap_or_else(|| "1".to_string())
                                                    })
                                                }
                                            },
                                            move |value| {
                                                update_ingredient(set_ingredients, id, |item| item.servings = value);
                                            },
                                        )}
                                    </div>

                                    <div class="card__summary">
                                        <p>{move || format!("Protein: {} g", format_number(per_recipe_protein()))}</p>
                                        <p>{move || format!("Fat: {} g", format_number(per_recipe_fat()))}</p>
                                        <p>{move || format!("Net carbs: {} g", format_number(per_recipe_carbs()))}</p>
                                        <p>{move || {
                                            let protein = per_recipe_protein();
                                            let fat = per_recipe_fat();
                                            let carbs = per_recipe_carbs();
                                            format!("P:E ratio: {}", format_ratio((protein, fat, carbs)))
                                        }}</p>
                                    </div>
                                </article>
                            }
                        }
                />
            </section>

            <section class="app__summary screen-only">
                <h2>Totals</h2>
                <ul>
                    <li>
                        <span>Total protein</span>
                        <strong>{
                            move || {
                                let (protein, _, _) = totals.get();
                                format!("{} g", format_number(protein))
                            }
                        }</strong>
                    </li>
                    <li>
                        <span>Total fat</span>
                        <strong>{
                            move || {
                                let (_, fat, _) = totals.get();
                                format!("{} g", format_number(fat))
                            }
                        }</strong>
                    </li>
                    <li>
                        <span>Total net carbs</span>
                        <strong>{
                            move || {
                                let (_, _, carbs) = totals.get();
                                format!("{} g", format_number(carbs))
                            }
                        }</strong>
                    </li>
                    <li class="highlight">
                        <span>P:E ratio</span>
                        <strong>{move || format_ratio(totals.get())}</strong>
                    </li>
                </ul>
            </section>

            <section class="print-report print-only">
                <h1>
                    {move || {
                        let name = recipe_name.get();
                        if name.trim().is_empty() {
                            "Recipe breakdown".to_string()
                        } else {
                            name
                        }
                    }}
                </h1>
                <table>
                    <thead>
                        <tr>
                            <th>Ingredient</th>
                            <th>Per serving (g)</th>
                            <th>Servings used</th>
                            <th>In recipe (g)</th>
                            <th>P:E ratio</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For
                            each=move || ingredients.get()
                            key=|ingredient: &Ingredient| ingredient.id
                            children=move |ingredient: Ingredient| {
                                let id = ingredient.id;
                                let row_data = create_memo({
                                    let ingredients = ingredients;
                                    move |_| {
                                        ingredients.with(|items| {
                                            items
                                                .iter()
                                                .find(|item| item.id == id)
                                                .map(|item| RowSnapshot {
                                                    name: if item.name.trim().is_empty() {
                                                        "Unnamed ingredient".to_string()
                                                    } else {
                                                        item.name.clone()
                                                    },
                                                    per_protein: parse_quantity(&item.protein),
                                                    per_fat: parse_quantity(&item.fat),
                                                    per_carbs: parse_quantity(&item.net_carbs),
                                                    servings: parse_quantity(&item.servings),
                                                })
                                                .unwrap_or_default()
                                        })
                                    }
                                });

                                view! {
                                    <tr>
                                        <td>{move || row_data.get().name.clone()}</td>
                                        <td>{move || {
                                            let row = row_data.get();
                                            format!(
                                                "P {} / F {} / C {}",
                                                format_number(row.per_protein),
                                                format_number(row.per_fat),
                                                format_number(row.per_carbs)
                                            )
                                        }}</td>
                                        <td>{move || format_number(row_data.get().servings)}</td>
                                        <td>{move || {
                                            let row = row_data.get();
                                            format!(
                                                "P {} / F {} / C {}",
                                                format_number(row.per_protein * row.servings),
                                                format_number(row.per_fat * row.servings),
                                                format_number(row.per_carbs * row.servings)
                                            )
                                        }}</td>
                                        <td>{move || {
                                            let row = row_data.get();
                                            format_ratio((
                                                row.per_protein * row.servings,
                                                row.per_fat * row.servings,
                                                row.per_carbs * row.servings,
                                            ))
                                        }}</td>
                                    </tr>
                                }
                            }
                        />
                    </tbody>
                </table>

                <div class="print-report__totals">
                    <div>
                        <span>Total protein</span>
                        <strong>{
                            move || {
                                let (protein, _, _) = totals.get();
                                format!("{} g", format_number(protein))
                            }
                        }</strong>
                    </div>
                    <div>
                        <span>Total fat</span>
                        <strong>{
                            move || {
                                let (_, fat, _) = totals.get();
                                format!("{} g", format_number(fat))
                            }
                        }</strong>
                    </div>
                    <div>
                        <span>Total net carbs</span>
                        <strong>{
                            move || {
                                let (_, _, carbs) = totals.get();
                                format!("{} g", format_number(carbs))
                            }
                        }</strong>
                    </div>
                    <div>
                        <span>P:E ratio</span>
                        <strong>{move || format_ratio(totals.get())}</strong>
                    </div>
                </div>
            </section>
        </main>
    }
}

fn macro_input<V, F>(label: &'static str, value: V, on_change: F) -> impl IntoView
where
    V: Fn() -> String + 'static,
    F: Fn(String) + 'static,
{
    view! {
        <label class="card__field">
            <span>{label}</span>
            <input
                class="number-input"
                type="text"
                inputmode="decimal"
                prop:value=value
                on:input=move |ev| {
                    let new_value = leptos::event_target_value(&ev);
                    on_change(new_value);
                }
            />
        </label>
    }
}

fn update_ingredient<F>(set_ingredients: WriteSignal<Vec<Ingredient>>, id: usize, updater: F)
where
    F: FnOnce(&mut Ingredient),
{
    set_ingredients.update(|items| {
        if let Some(item) = items.iter_mut().find(|item| item.id == id) {
            updater(item);
        }
    });
}

fn parse_quantity(raw: &str) -> f64 {
    sanitize_quantity(raw.trim().parse::<f64>().unwrap_or(0.0))
}

fn sanitize_quantity(value: f64) -> f64 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn format_number(value: f64) -> String {
    if value.abs() < 0.005 {
        "0.00".to_string()
    } else {
        format!("{value:.2}")
    }
}

fn format_ratio(totals: (f64, f64, f64)) -> String {
    let energy = totals.1 + totals.2;
    if energy <= f64::MIN_POSITIVE {
        "—".to_string()
    } else {
        format!("{:.2}", totals.0 / energy)
    }
}

fn encode_recipe(ingredients: &[Ingredient], name: &str) -> Option<String> {
    let trimmed_name = name.trim();
    let payload = RecipePayload {
        name: if trimmed_name.is_empty() {
            None
        } else {
            Some(trimmed_name.to_string())
        },
        ingredients: ingredients
            .iter()
            .map(|ingredient| IngredientPayload {
                id: ingredient.id,
                name: ingredient.name.clone(),
                protein: parse_quantity(&ingredient.protein),
                fat: parse_quantity(&ingredient.fat),
                net_carbs: parse_quantity(&ingredient.net_carbs),
                servings: parse_quantity(&ingredient.servings),
            })
            .collect(),
    };

    serde_json::to_vec(&payload)
        .ok()
        .map(|bytes| URL_SAFE_NO_PAD.encode(bytes))
}

fn decode_recipe(encoded: &str) -> Option<RecipePayload> {
    let raw = URL_SAFE_NO_PAD.decode(encoded.as_bytes()).ok()?;
    serde_json::from_slice(&raw).ok()
}

fn load_recipe_from_url() -> Option<(Vec<Ingredient>, String)> {
    let window = window()?;
    let location = window.location();
    let hash = location.hash().ok()?;
    let trimmed = hash.strip_prefix('#').unwrap_or(&hash);
    let encoded = trimmed.strip_prefix("recipe=")?;
    let payload = decode_recipe(encoded)?;
    let mut ingredients = payload
        .ingredients
        .into_iter()
        .map(Ingredient::from)
        .collect::<Vec<_>>();
    if ingredients.is_empty() {
        ingredients.push(Ingredient::empty(0));
    }
    let name = payload.name.unwrap_or_default();
    Some((ingredients, name))
}

impl From<IngredientPayload> for Ingredient {
    fn from(payload: IngredientPayload) -> Self {
        Self {
            id: payload.id,
            name: payload.name,
            protein: format_input_value(payload.protein),
            fat: format_input_value(payload.fat),
            net_carbs: format_input_value(payload.net_carbs),
            servings: format_input_value(payload.servings),
        }
    }
}

fn format_input_value(value: f64) -> String {
    if value.abs() < 0.005 {
        String::new()
    } else {
        format!("{value:.2}")
    }
}

pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}
