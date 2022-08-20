use std::{
    fs::{self, DirEntry},
    io,
    path::Path,
    time::Duration,
};

use clap::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};
// fn ser_duration<S: Serializer>(dur: &Duration, s: S) -> Result<S::Ok, S::Error> {
//     s.serialize_u64(dur.as_secs())
// }

// fn deser_duration<D: Deserializer>(d: D) -> Result<S::Ok, S::Error> {
//     d.deserialize_u64(visitor)
// }

#[derive(Serialize, Deserialize, Debug)]
enum IngredientUnit {
    Cup,
    Tablespoon,
    Teaspoon,
    Grams,
    Kilograms,
    Millilitre,
    Number,
    Tins,
    Inch,
}

#[derive(Serialize, Deserialize, Debug)]
struct IngredientAmount {
    unit: IngredientUnit,
    number: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Ingredient {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<IngredientAmount>,
}

#[derive(Serialize, Deserialize, Debug)]
struct IngredientGroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    ingredients: Vec<Ingredient>,
}
#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
struct Recipe {
    name: String,
    #[serde_as(as = "DurationSeconds<u64>")]
    prep_time: Duration,
    #[serde_as(as = "DurationSeconds<u64>")]
    cook_time: Duration,
    serves: i32,
    ingredient_groups: Vec<IngredientGroup>,
    steps: Vec<String>,
}

#[derive(Debug)]
struct RecipeIntro {
    name: String,
    prep_time: Duration,
    cook_time: Duration,
    serves: i32,
}

fn parse_begin_recipe(recipe: &str) -> RecipeIntro {
    let re = Regex::new(
        r"\\beginrecipie\{(?P<name>[^\}]*)\}\{(?P<prep_time_mins>\d*)\}\{(?P<cook_time_mins>\d*)\}\{(?P<serves>\d*)\}",
    )
    .unwrap();
    println!("{}", recipe);
    let found = re.captures(recipe).unwrap();

    RecipeIntro {
        name: found.name("name").unwrap().as_str().to_string(),
        prep_time: Duration::from_secs(
            found
                .name("prep_time_mins")
                .unwrap()
                .as_str()
                .parse()
                .unwrap(),
        ) * 60,
        cook_time: Duration::from_secs(
            found
                .name("cook_time_mins")
                .unwrap()
                .as_str()
                .parse()
                .unwrap(),
        ) * 60,
        serves: found.name("serves").unwrap().as_str().parse().unwrap(),
    }
}

fn parse_ingredient(item: &str) -> Ingredient {
    let re = Regex::new(r"\\grams\{([\d\.]*)\} (.*)").unwrap();
    if let Some(k) = re.captures(item) {
        return Ingredient {
            name: k.get(2).unwrap().as_str().trim_end().to_string(),
            amount: Some(IngredientAmount {
                unit: IngredientUnit::Grams,
                number: k.get(1).unwrap().as_str().parse::<f32>().unwrap(),
            }),
        };
    }
    let re = Regex::new(r"\\kg\{([\d\.]*)\} (.*)").unwrap();
    if let Some(k) = re.captures(item) {
        return Ingredient {
            name: k.get(2).unwrap().as_str().trim_end().to_string(),
            amount: Some(IngredientAmount {
                unit: IngredientUnit::Kilograms,
                number: k.get(1).unwrap().as_str().parse::<f32>().unwrap(),
            }),
        };
    }
    let re = Regex::new(r"\\tablespoons\{([\d\.]*)\} (.*)").unwrap();
    if let Some(k) = re.captures(item) {
        return Ingredient {
            name: k.get(2).unwrap().as_str().trim_end().to_string(),
            amount: Some(IngredientAmount {
                unit: IngredientUnit::Tablespoon,
                number: k.get(1).unwrap().as_str().parse::<f32>().unwrap(),
            }),
        };
    }
    let re = Regex::new(r"\\teaspoons\{([\d\.]*)\} (.*)").unwrap();
    if let Some(k) = re.captures(item) {
        return Ingredient {
            name: k.get(2).unwrap().as_str().trim_end().to_string(),
            amount: Some(IngredientAmount {
                unit: IngredientUnit::Teaspoon,
                number: k.get(1).unwrap().as_str().parse::<f32>().unwrap(),
            }),
        };
    }
    let re = Regex::new(r"\\ml\{([\d\.]*)\} (.*)").unwrap();
    if let Some(k) = re.captures(item) {
        return Ingredient {
            name: k.get(2).unwrap().as_str().trim_end().to_string(),
            amount: Some(IngredientAmount {
                unit: IngredientUnit::Millilitre,
                number: k.get(1).unwrap().as_str().parse::<f32>().unwrap(),
            }),
        };
    }
    let re = Regex::new(r"\\cups\{([\d\.]*)\} (.*)").unwrap();
    if let Some(k) = re.captures(item) {
        return Ingredient {
            name: k.get(2).unwrap().as_str().trim_end().to_string(),
            amount: Some(IngredientAmount {
                unit: IngredientUnit::Cup,
                number: k.get(1).unwrap().as_str().parse::<f32>().unwrap(),
            }),
        };
    }
    let re = Regex::new(r"^(\d) tins (.*)").unwrap();
    if let Some(k) = re.captures(item) {
        return Ingredient {
            name: k.get(2).unwrap().as_str().trim_end().to_string(),
            amount: Some(IngredientAmount {
                unit: IngredientUnit::Tins,
                number: k.get(1).unwrap().as_str().parse::<f32>().unwrap(),
            }),
        };
    }
    let re = Regex::new(r"^(\d+) (.*)").unwrap();
    if let Some(k) = re.captures(item) {
        return Ingredient {
            name: k.get(2).unwrap().as_str().trim_end().to_string(),
            amount: Some(IngredientAmount {
                unit: IngredientUnit::Number,
                number: k.get(1).unwrap().as_str().parse::<f32>().unwrap(),
            }),
        };
    }
    return Ingredient {
        name: item.trim_end().to_string(),
        amount: None,
    };
}

fn parse_ingredients_list(ingredient_list: &str) -> Vec<Ingredient> {
    let re = Regex::new(r"\\item (.*)").unwrap();
    re.captures_iter(ingredient_list)
        .map(|c| parse_ingredient(c.get(1).unwrap().as_str()))
        .collect()
}

fn parse_ingredients(recipe: &str) -> Vec<IngredientGroup> {
    let main_re = Regex::new(r"\\begin\{main\}([\S\s]*)\\end\{main\}").unwrap();
    let main_ingredients = main_re.find(recipe).unwrap().as_str();

    let mut ret = vec![IngredientGroup {
        name: None,
        ingredients: parse_ingredients_list(main_ingredients),
    }];

    let sub_re =
        Regex::new(r"\\begin\{subingredient\}\{([^\}]*)\}([\S\s]*?)\\end\{subingredient\}")
            .unwrap();
    sub_re.captures_iter(recipe).for_each(|c| {
        ret.push(IngredientGroup {
            name: Some(c.get(1).unwrap().as_str().to_string()),
            ingredients: parse_ingredients_list(c.get(2).unwrap().as_str()),
        })
    });

    ret
}

fn parse_steps(recipe: &str) -> Vec<String> {
    let re = Regex::new(r"\\begin\{recipe\}([\s\S]*)\\end\{recipe\}").unwrap();
    let steps = re.find(recipe).unwrap().as_str();
    let steps_re = Regex::new(r"\\step\{([^\}]*)\}").unwrap();
    steps_re
        .captures_iter(steps)
        .map(|s| s.get(1).unwrap().as_str().to_string())
        .collect()
}

fn parse_recipe(recipe: &str) -> Recipe {
    let intro = parse_begin_recipe(recipe);

    Recipe {
        name: intro.name,
        prep_time: intro.prep_time,
        cook_time: intro.cook_time,
        serves: intro.serves,
        ingredient_groups: parse_ingredients(recipe),
        steps: parse_steps(recipe),
    }
}

fn ingredient_amount_to_latex(amount: &IngredientAmount) -> String {
    let number = amount.number;
    match amount.unit {
        IngredientUnit::Cup => format!("\\cups{{{number}}}"),
        IngredientUnit::Grams => format!("\\grams{{{number}}}"),
        IngredientUnit::Kilograms => format!("\\kg{{{number}}}"),
        IngredientUnit::Millilitre => format!("\\ml{{{number}}}"),
        IngredientUnit::Number => format!("{number}"),
        IngredientUnit::Tablespoon => format!("\\tablespoons{{{number}}}"),
        IngredientUnit::Teaspoon => format!("\\teaspoons{{{number}}}"),
        IngredientUnit::Tins => format!("{number} tins"),
        IngredientUnit::Inch => format!("{number} inches"),
    }
}

fn ingredient_group_to_latex(group: &IngredientGroup) -> String {
    let items = group
        .ingredients
        .iter()
        .map(|s| -> String {
            if let Some(amount) = &s.amount {
                return format!(
                    "        \\item {description} {name}",
                    name = s.name,
                    description = ingredient_amount_to_latex(&amount)
                );
            }
            format!("        \\item {name}", name = s.name)
        })
        .collect::<Vec<String>>()
        .join("\n");
    if let Some(name) = &group.name {
        return format!(
            "    \\begin{{subingredient}}{{{name}}}\n{items}\n    \\end{{subingredient}}"
        );
    }
    return format!("    \\begin{{main}}\n{items}\n    \\end{{main}}");
}
fn recipe_to_latex(recipe: Recipe) -> String {
    let begin_recipe = format!(
        "\\beginrecipie{{{name}}}{{{prep}}}{{{cook}}}{{{serves}}}",
        name = recipe.name,
        prep = recipe.prep_time.as_secs() / 60,
        cook = recipe.cook_time.as_secs() / 60,
        serves = recipe.serves
    );
    let groups = recipe
        .ingredient_groups
        .iter()
        .map(|s| -> String { ingredient_group_to_latex(s) })
        .collect::<Vec<String>>()
        .join("\n");
    let ingredients = format!("\\begin{{ingredient}}\n{groups}\n\\end{{ingredient}}");
    let steps = recipe
        .steps
        .iter()
        .map(|s| -> String { format!("    \\step{{{s}}}") })
        .collect::<Vec<String>>()
        .join("\n");
    let recipe = format!("\\begin{{recipe}}\n{steps}\n\\end{{recipe}}");
    return format!("{begin_recipe}\n{ingredients}\n{recipe}");
}

fn visit_dirs(dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path)?;
            } else {
                convert_to_json(entry);
            }
        }
    }
    Ok(())
}

fn convert_to_json(entry: DirEntry) {
    let contents = fs::read_to_string(entry.path()).expect("Something went wrong reading the file");
    let mut path = entry.path();
    path.set_extension("json");

    if let Ok(rel) = path.strip_prefix("../recipies") {
        let mut out = Path::new("./out").to_path_buf();
        out.push(rel);
        let mut o2 = out.clone();
        let out_path = out.as_path();

        println!("{:#?}{:#?}", out_path, rel);
        let dir = Path::new("./out").to_path_buf();
        o2.pop();
        fs::create_dir_all(o2).unwrap();

        fs::write(
            out_path,
            serde_json::to_string_pretty(&parse_recipe(&contents)).unwrap(),
        )
        .unwrap();
    }
}

fn generate_latex_pages(dir: &Path, from_path: &Path, to_path: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                generate_latex_pages(&path, &from_path, &to_path)?;
            } else {
                convert_to_latex(entry, from_path, to_path);
            }
        }
    }
    Ok(())
}

fn convert_to_latex(entry: DirEntry, from_path: &Path, to_path: &Path) {
    let entry_path = entry.path();
    let contents = fs::read_to_string(entry.path()).expect("Something went wrong reading the file");
    let recipe: Recipe = serde_json::from_str(&contents).unwrap();

    let relative_path = entry_path.strip_prefix(from_path).unwrap();
    let out_path = to_path.join(relative_path).with_extension("tex");
    fs::create_dir_all(out_path.parent().unwrap()).unwrap();
    fs::write(out_path, recipe_to_latex(recipe)).unwrap();
}

fn generate_markdown_pages(dir: &Path, from_path: &Path, to_path: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                generate_markdown_pages(&path, &from_path, &to_path)?;
            } else {
                convert_to_markdown(entry, from_path, to_path);
            }
        }
    }
    Ok(())
}

fn convert_to_markdown(entry: DirEntry, from_path: &Path, to_path: &Path) {
    let entry_path = entry.path();
    let contents = fs::read_to_string(entry.path()).expect("Something went wrong reading the file");
    let recipe: Recipe = serde_json::from_str(&contents).unwrap();

    let relative_path = entry_path.strip_prefix(from_path).unwrap();
    let out_path = to_path.join(relative_path).with_extension("md");
    fs::create_dir_all(out_path.parent().unwrap()).unwrap();
    fs::write(out_path, recipe_to_markdown(recipe)).unwrap();
}

fn recipe_to_markdown(recipe: Recipe) -> String {
    let begin_recipe = format!(
        "# {name}\n\nPrep Time: {prep} min\n\nCook Time: {cook} min\n\nServes: {serves}",
        name = recipe.name,
        prep = recipe.prep_time.as_secs() / 60,
        cook = recipe.cook_time.as_secs() / 60,
        serves = recipe.serves
    );
    let groups = recipe
        .ingredient_groups
        .iter()
        .map(|s| -> String { ingredient_group_to_markdown(s) })
        .collect::<Vec<String>>()
        .join("\n\n");
    let ingredients = format!("## Ingredients\n\n{groups}");
    let steps = recipe
        .steps
        .iter()
        .map(|s| -> String { format!("- {s}") })
        .collect::<Vec<String>>()
        .join("\n");
    let recipe = format!("## Method\n\n{steps}");
    return format!("{begin_recipe}\n\n{ingredients}\n\n{recipe}\n");
}

fn ingredient_group_to_markdown(group: &IngredientGroup) -> String {
    let items = group
        .ingredients
        .iter()
        .map(|s| -> String {
            if let Some(amount) = &s.amount {
                return format!(
                    "- {description} {name}",
                    name = s.name,
                    description = ingredient_amount_to_markdown(&amount)
                );
            }
            format!("- {name}", name = s.name)
        })
        .collect::<Vec<String>>()
        .join("\n");
    if let Some(name) = &group.name {
        return format!("### {name}\n\n{items}");
    }
    return format!("{items}");
}

fn ingredient_amount_to_markdown(amount: &IngredientAmount) -> String {
    let number = amount.number;
    match amount.unit {
        IngredientUnit::Cup => format!("{number} cups"),
        IngredientUnit::Grams => format!("{number} g"),
        IngredientUnit::Kilograms => format!("{number} kg"),
        IngredientUnit::Millilitre => format!("{number} ml"),
        IngredientUnit::Number => format!("{number}"),
        IngredientUnit::Tablespoon => format!("{number} tbsp"),
        IngredientUnit::Teaspoon => format!("{number} tsp"),
        IngredientUnit::Tins => format!("{number} tins"),
        IngredientUnit::Inch => format!("{number} inches"),
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap(long, value_parser)]
    from: String,

    /// Number of times to greet
    #[clap(long, value_parser)]
    to_markdown: Option<String>,

    /// Number of times to greet
    #[clap(long, value_parser)]
    to_latex: Option<String>,
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args);

    if let Some(to) = args.to_markdown {
        let from_path = Path::new(&args.from);
        let to_path = Path::new(&to);
        fs::remove_dir_all(to_path).ok();
        generate_markdown_pages(from_path, from_path, to_path).unwrap();
        println!("{}, {}", from_path.display(), to_path.display())
    }

    if let Some(to) = args.to_latex {
        let from_path = Path::new(&args.from);
        let to_path = Path::new(&to);
        fs::remove_dir_all(to_path).ok();
        generate_latex_pages(from_path, from_path, to_path).unwrap();
        println!("{}, {}", from_path.display(), to_path.display())
    }

    // let in_file = "src/lasagne.tex";
    // //println!("In file {}", filename);

    // let contents = fs::read_to_string(in_file).expect("Something went wrong reading the file");

    // let out_file = "src/lasagne.json";
    // fs::write(
    //     out_file,
    //     serde_json::to_string(&parse_recipe(&contents)).unwrap(),
    // )
    // .unwrap();

    // let contents = fs::read_to_string(out_file).expect("Something went wrong reading the file");
    // let from_json: Recipe = serde_json::from_str(&contents).unwrap();
    // //println!("from_json:\n{:#?}", recipe_to_latex(from_json));
    // let out_file_2 = "src/lasagne2.tex";
    // fs::write(out_file_2, recipe_to_latex(from_json)).unwrap();

    // visit_dirs(Path::new("..//recipies")).unwrap();
}
