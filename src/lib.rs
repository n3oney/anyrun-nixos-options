use fuzzy_matcher::FuzzyMatcher;
use serde_inline_default::serde_inline_default;
use std::{
    collections::HashMap,
    fs,
    sync::{Arc, Mutex},
};

use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::*;
use rayon::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename = "Config")]
pub struct AnyrunConfig {
    max_entries: Option<usize>,
}

#[serde_inline_default]
#[derive(Deserialize, Debug)]
pub struct Config {
    options: HashMap<String, Vec<String>>,
    #[serde_inline_default(0)]
    min_score: i64,
    #[serde_inline_default("https://github.com/NixOS/nixpkgs/blob/nixos-unstable".to_string())]
    nixpkgs_url: String,
}

#[derive(Deserialize, Debug)]
pub struct DefaultOrExample {
    #[serde(rename = "_type")]
    #[allow(unused)]
    r#type: String,
    text: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Declaration {
    NixOS(String),
    Nmd(NmdDeclaration),
}

#[derive(Deserialize, Debug)]
pub struct NmdDeclaration {
    // #[serde(rename = "channelPath")]
    // channel_path: String,
    // path: String,
    url: String,
}

#[derive(Deserialize, Debug)]
pub struct NixOSOption {
    declarations: Vec<Declaration>,
    description: Option<String>,
    default: Option<DefaultOrExample>,
    example: Option<DefaultOrExample>,
    #[allow(unused)]
    loc: Vec<String>,
    #[serde(rename = "readOnly")]
    #[allow(unused)]
    read_only: bool,
    r#type: String,
}

pub struct State {
    config: Config,
    options: HashMap<String, HashMap<String, NixOSOption>>,
    anyrun_cfg: AnyrunConfig,
}

#[init]
fn init(config_dir: RString) -> State {
    let content =
        fs::read_to_string(format!("{}/nixos-options.ron", config_dir)).unwrap_or_else(|why| {
            panic!(
                "Error reading anyrun-nixos-options config file ({}/nixos-options.ron).\n{}",
                config_dir, why
            )
        });

    let cfg: Config = ron::from_str(&content).unwrap();

    let anyrun_content = fs::read_to_string(format!("{}/config.ron", config_dir))
        .unwrap_or_else(|why| panic!("Error reading anyrun config file.\n{}", why));

    let anyrun_cfg: AnyrunConfig = ron::from_str(&anyrun_content).unwrap();

    if anyrun_cfg.max_entries.is_none() {
        println!("With the anyrun-nixos-options, it's recommended to set anyrun's `max_entries` to some small value.");
    }

    if let Some(max_entries) = anyrun_cfg.max_entries {
        if max_entries == 0 {
            println!("With the anyrun-nixos-options, it's recommended to set anyrun's `max_entries` to some small value.");
        }
    }

    let mut options: HashMap<String, HashMap<String, NixOSOption>> = HashMap::new();

    for option in &cfg.options {
        let mut hash_map: HashMap<String, NixOSOption> = HashMap::new();

        for path in option.1 {
            let raw_options = fs::read_to_string(path).unwrap_or_else(|why| {
                panic!(
                    "Error reading anyrun-nixos-options options file ({}).\n{}",
                    path, why
                )
            });

            let parsed_options: HashMap<String, NixOSOption> =
                serde_json::from_str(&raw_options).unwrap();

            hash_map.extend(parsed_options);
        }

        options.insert(option.0.clone(), hash_map);
    }

    State {
        config: cfg,
        options,
        anyrun_cfg,
    }
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "NixOS Options".into(),
        icon: "go-home".into(),
    }
}

#[get_matches]
fn get_matches(input: RString, state: &mut State) -> RVec<Match> {
    let results = Arc::new(Mutex::new(RVec::<Match>::new()));

    for option in &state.options {
        let prefix = option.0;

        let input = if let Some(input) = input.strip_prefix(prefix) {
            let trimmed = input.trim();
            trimmed.replace(" ", ".")
        } else {
            continue;
        };

        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default().smart_case();

        let mut entries = option
            .1
            .par_iter()
            .filter_map(|(key, query)| {
                let score = matcher.fuzzy_indices(&key, &input).unwrap_or((0, vec![]));

                if score.0 > state.config.min_score {
                    Some((score, key, query))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        entries.par_sort_unstable_by(|a, b| (b.0).0.cmp(&(a.0).0));

        if let Some(max_entries) = state.anyrun_cfg.max_entries {
            if max_entries > 0 {
                entries.truncate(max_entries);
            }
        }

        let md_url_regex = regex::Regex::new(r#"\[([^\[]+)\](\(.*\))"#).unwrap();
        let url_regex =
            regex::Regex::new(r#"&lt;([^\s\.]+\.[^\s]{2,}|www\.[^\s]+\.[^\s]{2,})&gt;"#).unwrap();

        let file_regex = regex::Regex::new(r#"\{file\}`(.+?)`"#).unwrap();
        let command_regex = regex::Regex::new(r#"\{command\}`(.+?)`"#).unwrap();
        let option_regex = regex::Regex::new(r#"\{option\}`(.+?)`"#).unwrap();
        let plain_url_regex = regex::Regex::new(r#"`(.+?)`"#).unwrap();

        let res = results.clone();

        entries.par_iter().for_each(move |entry| {
            let mut description = if let Some(desc) = &entry.2.description {
                let encoded_desc = html_escape::encode_text(desc);

                let url_parsed = url_regex.replace_all(&encoded_desc, |caps: &regex::Captures| {
                    format!(r#"<span foreground="lightblue"><u>{}</u></span>"#, &caps[1])
                });

                let md_parsed = md_url_regex.replace_all(&url_parsed, |caps: &regex::Captures| {
                    format!(r#"<span foreground="lightblue"><u>{}</u></span>"#, &caps[1])
                });

                let file_parsed = file_regex.replace_all(&md_parsed, |caps: &regex::Captures| {
                    format!(r#"<span foreground="lightgreen">{}</span>"#, &caps[1])
                });

                let command_parsed =
                    command_regex.replace_all(&file_parsed, |caps: &regex::Captures| {
                        format!(r#"<span font_family="monospace">{}</span>"#, &caps[1])
                    });

                let option_parsed =
                    option_regex.replace_all(&command_parsed, |caps: &regex::Captures| {
                        format!(
                            r#"<span font_family="monospace" foreground="orange">{}</span>"#,
                            &caps[1]
                        )
                    });

                let plain_url_parsed =
                    plain_url_regex.replace_all(&option_parsed, |caps: &regex::Captures| {
                        format!(r#"<span foreground="lightblue"><u>{}</u></span>"#, &caps[1])
                    });

                plain_url_parsed.trim().to_string()
            } else {
                "".to_string()
            };

            description.push_str(&format!(
                "\n\n<b>Type</b>: <span font_family=\"monospace\">{}</span>",
                html_escape::encode_text(&entry.2.r#type),
            ));

            if let Some(default) = &entry.2.default {
                description.push_str(&format!(
                    "\n<b>Default</b>:{}<span font_family=\"monospace\">{}</span>",
                    if default.text.contains("\n") {
                        "\n"
                    } else {
                        " "
                    },
                    html_escape::encode_text(&default.text)
                ))
            }

            if let Some(example) = &entry.2.example {
                description.push_str(&format!(
                    "\n<b>Example</b>:{}<span font_family=\"monospace\">{}</span>",
                    if example.text.contains("\n") {
                        "\n"
                    } else {
                        " "
                    },
                    html_escape::encode_text(&example.text)
                ))
            }

            let mut title = String::new();

            let mut iterator = entry.1.chars().enumerate().peekable();

            let mut is_red = false;

            while let Some((i, char)) = iterator.next() {
                if entry.0 .1.contains(&i) {
                    if !is_red {
                        title.push_str(&format!("<span weight=\"bold\" foreground=\"#db5a65\">"));
                        is_red = true;
                    }

                    title.push_str(&html_escape::encode_text(&char.to_string()));

                    if let Some(next) = iterator.peek() {
                        if !entry.0 .1.contains(&next.0) {
                            title.push_str("</span>");
                            is_red = false;
                        }
                    }
                } else {
                    title.push_str(&html_escape::encode_text(&char.to_string()));
                }
            }
            if is_red {
                title.push_str("</span>");
            }

            res.lock().unwrap().push(Match {
                title: format!(r#"<span font_family="monospace">{}</span>"#, title).into(),
                description: ROption::RSome(description.trim().into()),
                icon: ROption::RNone,
                id: ROption::RNone,
                use_pango: true,
            })
        });
    }

    Arc::try_unwrap(results)
        .unwrap_or_else(|_| panic!("Failed to get results"))
        .into_inner()
        .unwrap()
}

#[handler]
fn handler(selection: Match, state: &mut State) -> HandleResult {
    let span_regex =
        regex::Regex::new(r##"<span weight="bold" foreground="#db5a65">(.+?)</span>"##).unwrap();

    let key = span_regex
        .replace_all(&selection.title, |caps: &regex::Captures| {
            caps[1].to_string()
        })
        .to_string();

    let key_with_no_monospace = &key[30..key.len() - 7];

    for option in &state.options {
        let value = option.1.get(&key_with_no_monospace.to_string());

        if let Some(value) = value {
            for declaration in &value.declarations {
                let url = match declaration {
                    Declaration::NixOS(v) => v,
                    Declaration::Nmd(v) => &v.url,
                };

                let parsed_url = url::Url::parse(url);

                let final_url = match parsed_url {
                    Ok(_) => url.clone(),
                    Err(_) => format!("{}/{}", state.config.nixpkgs_url.clone(), url),
                };

                open::that(final_url).ok();
            }
            return HandleResult::Close;
        }
    }

    return HandleResult::Refresh(false);
}
