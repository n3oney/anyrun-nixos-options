use fuzzy_matcher::FuzzyMatcher;

use std::{collections::HashMap, fs};

use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::*;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    prefix: Option<String>,
    options_path: String,
    min_score: Option<i64>,
    nixpkgs_url: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct DefaultOrExample {
    #[serde(rename = "_type")]
    #[allow(unused)]
    r#type: String,
    text: String,
}

#[derive(Deserialize, Debug)]
pub struct NixOSOption {
    declarations: Vec<String>,
    description: String,
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
    options: HashMap<String, NixOSOption>,
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

    let options = fs::read_to_string(&cfg.options_path).unwrap_or_else(|why| {
        panic!(
            "Error reading anyrun-nixos-options options.json file ({}).\n{}",
            cfg.options_path, why
        )
    });

    let options: HashMap<String, NixOSOption> = serde_json::from_str(&options).unwrap();

    State {
        config: cfg,
        options,
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
    let input = if let Some(input) =
        input.strip_prefix(&state.config.prefix.clone().unwrap_or(":nix".to_string()))
    {
        let trimmed = input.trim();
        trimmed.replace(" ", ".")
    } else {
        return RVec::new();
    };

    let mut matches = vec![];

    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default().smart_case();

    let mut entries = state
        .options
        .iter()
        .filter_map(|(key, query)| {
            let score = matcher.fuzzy_indices(&key, &input).unwrap_or((0, vec![]));

            if score.0 > state.config.min_score.unwrap_or(50) {
                Some((score, key, query))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    entries.sort_by(|a, b| (b.0).0.cmp(&(a.0).0));

    let md_url_regex = regex::Regex::new(r#"\[([^\[]+)\](\(.*\))"#).unwrap();
    let url_regex =
        regex::Regex::new(r#"&lt;([^\s\.]+\.[^\s]{2,}|www\.[^\s]+\.[^\s]{2,})&gt;"#).unwrap();

    let file_regex = regex::Regex::new(r#"\{file\}`(.+?)`"#).unwrap();
    let command_regex = regex::Regex::new(r#"\{command\}`(.+?)`"#).unwrap();
    let option_regex = regex::Regex::new(r#"\{option\}`(.+?)`"#).unwrap();
    let plain_url_regex = regex::Regex::new(r#"`(.+?)`"#).unwrap();

    for entry in entries {
        let encoded_desc = html_escape::encode_text(&entry.2.description);

        let url_parsed = url_regex.replace_all(&encoded_desc, |caps: &regex::Captures| {
            format!(r#"<span foreground="lightblue"><u>{}</u></span>"#, &caps[1])
        });

        let md_parsed = md_url_regex.replace_all(&url_parsed, |caps: &regex::Captures| {
            format!(r#"<span foreground="lightblue"><u>{}</u></span>"#, &caps[1])
        });

        let file_parsed = file_regex.replace_all(&md_parsed, |caps: &regex::Captures| {
            format!(r#"<span foreground="lightgreen">{}</span>"#, &caps[1])
        });

        let command_parsed = command_regex.replace_all(&file_parsed, |caps: &regex::Captures| {
            format!(r#"<span font_family="monospace">{}</span>"#, &caps[1])
        });

        let option_parsed = option_regex.replace_all(&command_parsed, |caps: &regex::Captures| {
            format!(
                r#"<span font_family="monospace" foreground="orange">{}</span>"#,
                &caps[1]
            )
        });

        let plain_url_parsed =
            plain_url_regex.replace_all(&option_parsed, |caps: &regex::Captures| {
                format!(r#"<span foreground="lightblue"><u>{}</u></span>"#, &caps[1])
            });

        let mut description = plain_url_parsed.trim().to_string();

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

        matches.push(Match {
            title: format!(r#"<span font_family="monospace">{}</span>"#, title).into(),
            description: ROption::RSome(description.trim().into()),
            icon: ROption::RNone,
            id: ROption::RNone,
            use_pango: true,
        });
    }

    matches.into()
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

    let value = state.options.get(&key_with_no_monospace.to_string());

    if let Some(value) = value {
        for url in &value.declarations {
            open::that(format!(
                "{}/{}",
                state
                    .config
                    .nixpkgs_url
                    .clone()
                    .unwrap_or("https://github.com/NixOS/nixpkgs/blob/nixos-unstable".to_owned()),
                url
            ))
            .ok();
        }
        HandleResult::Close
    } else {
        HandleResult::Refresh(false)
    }
}