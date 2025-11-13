use leptos::*;

#[derive(Clone, Debug, PartialEq)]
struct Ingredient {
    id: usize,
    name: String,
    protein: f64,
    fat: f64,
    net_carbs: f64,
    servings: f64,
}

impl Ingredient {
    fn empty(id: usize) -> Self {
        Self {
            id,
            name: String::new(),
            protein: 0.0,
            fat: 0.0,
            net_carbs: 0.0,
            servings: 1.0,
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    let (ingredients, set_ingredients) = create_signal(vec![Ingredient::empty(0)]);
    let next_id = create_rw_signal(1usize);

    let add_ingredient = {
        let set_ingredients = set_ingredients;
        move |_| {
            let id = next_id.get_untracked();
            next_id.update(|value| *value += 1);
            set_ingredients.update(|items| items.push(Ingredient::empty(id)));
        }
    };

    let remove_ingredient = {
        let set_ingredients = set_ingredients;
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

    let totals = create_memo(move |_| {
        ingredients.with(|items| {
            let mut protein = 0.0;
            let mut fat = 0.0;
            let mut carbs = 0.0;
            for item in items {
                let servings = sanitize_quantity(item.servings);
                protein += sanitize_quantity(item.protein) * servings;
                fat += sanitize_quantity(item.fat) * servings;
                carbs += sanitize_quantity(item.net_carbs) * servings;
            }
            (protein, fat, carbs)
        })
    });

    let stylesheet = include_str!("./styles.css");

    view! {
        <style>{stylesheet}</style>
        <main class="app">
            <section class="app__header">
                <h1>P:E Recipe Calculator</h1>
                <p>
                    "Build a recipe from food labels, enter their per-serving macros, "
                    "and specify how many servings of each item you plan to use. "
                    "The calculator totals protein, fat, and net carbs, and "
                    "shows the overall protein efficiency ratio (protein ÷ fat+net carbs)."
                </p>
            </section>

            <section class="app__actions">
                <button class="primary" on:click=add_ingredient>
                    "+ Add food"
                </button>
            </section>

            <section class="app__ingredients">
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
                                        .map(|item| sanitize_quantity(item.protein) * sanitize_quantity(item.servings))
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
                                        .map(|item| sanitize_quantity(item.fat) * sanitize_quantity(item.servings))
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
                                        .map(|item| sanitize_quantity(item.net_carbs) * sanitize_quantity(item.servings))
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
                                            let ingredients = ingredients;
                                            move || {
                                                ingredients.with(|items| {
                                                    items
                                                        .iter()
                                                        .find(|item| item.id == id)
                                                        .map(|item| sanitize_quantity(item.protein))
                                                        .unwrap_or_default()
                                                })
                                            }
                                        },
                                        move |value| update_ingredient(set_ingredients, id, |item| item.protein = value),
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
                                                        .map(|item| sanitize_quantity(item.fat))
                                                        .unwrap_or_default()
                                                })
                                            }
                                        },
                                        move |value| update_ingredient(set_ingredients, id, |item| item.fat = value),
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
                                                        .map(|item| sanitize_quantity(item.net_carbs))
                                                        .unwrap_or_default()
                                                })
                                            }
                                        },
                                        move |value| update_ingredient(set_ingredients, id, |item| item.net_carbs = value),
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
                                                        .map(|item| sanitize_quantity(item.servings))
                                                        .unwrap_or(1.0)
                                                })
                                            }
                                        },
                                        move |value| update_ingredient(set_ingredients, id, |item| item.servings = value.max(0.0)),
                                    )}
                                </div>

                                <div class="card__summary">
                                    <p>{move || format!("Protein: {} g", format_number(per_recipe_protein()))}</p>
                                    <p>{move || format!("Fat: {} g", format_number(per_recipe_fat()))}</p>
                                    <p>{move || format!("Net carbs: {} g", format_number(per_recipe_carbs()))}</p>
                                </div>
                            </article>
                        }
                    }
                />
            </section>

            <section class="app__summary">
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
        </main>
    }
}

fn macro_input<V, F>(label: &'static str, value: V, on_change: F) -> impl IntoView
where
    V: Fn() -> f64 + 'static,
    F: Fn(f64) + 'static,
{
    view! {
        <label class="card__field">
            <span>{label}</span>
            <input
                class="number-input"
                type="number"
                min="0"
                step="0.1"
                prop:value=move || format_number(value())
                on:input=move |ev| {
                    let parsed = parse_number(leptos::event_target_value(&ev));
                    on_change(parsed);
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

fn parse_number(input: String) -> f64 {
    input.trim().parse::<f64>().unwrap_or(0.0)
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

pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}
