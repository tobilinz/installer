#![allow(non_snake_case)]
use dioxus::prelude::*;
use isahc::ReadResponseExt;

fn Header(cx: Scope) -> Element {
    // TODO(figure out how to make this from modpack_source)
    cx.render(rsx! {
        div {
            class: "header",
            div {
                class: "header-inner",
                h1 {
                    "Majestic Overhaul"
                }
            }
        }
    })
}

fn Spinner(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            class: "lds-ring",
            div {}
            div {}
            div {}
            div {}
        }
    })
}

#[derive(Props, PartialEq)]
struct CreditsProps {
    manifest: super::Manifest,
}

fn Credits(cx: Scope<CreditsProps>) -> Element {
    cx.render(rsx! {
        ul {
            for r#mod in &cx.props.manifest.mods {
                li {
                    "{r#mod.name} by "
                    for author in &r#mod.authors {
                        a {
                            href: "{author.link}",
                            if r#mod.authors.last().unwrap() == author {
                                author.name.to_string()
                            } else {
                                author.name.to_string() + ", "
                            }
                        }
                    }
                }
            }
            for shaderpack in &cx.props.manifest.shaderpacks {
                li {
                    "{shaderpack.name} by "
                    for author in &shaderpack.authors {
                        a {
                            href: "{author.link}",
                            if shaderpack.authors.last().unwrap() == author {
                                author.name.to_string()
                            } else {
                                author.name.to_string() + ", "
                            }
                        }
                    }
                }
            }
            for resourcepack in &cx.props.manifest.resourcepacks {
                li {
                    "{resourcepack.name} by "
                    for author in &resourcepack.authors {
                        a {
                            href: "{author.link}",
                            if resourcepack.authors.last().unwrap() == author {
                                author.name.to_string()
                            } else {
                                author.name.to_string() + ", "
                            }
                        }
                    }
                }
            }
        }
    })
}

#[derive(Props, PartialEq)]
struct SettingsProps<'a> {
    launcher_state: &'a UseState<super::Launcher>,
    settings: &'a UseState<bool>,
}

fn Settings<'a>(cx: Scope<'a, SettingsProps<'a>>) -> Element {
    // TODO(Launcher selection screen)
    cx.render(rsx! {
        div {
            class: "container",
            form {
                id: "launcher-select",
                onsubmit: move |event| {
                    let launcher = event.data.values["launcher-select"].split('-').collect::<Vec<_>>();
                    cx.props.launcher_state.set(match *launcher.first().unwrap() {
                        "vanilla" => super::Launcher::Vanilla(super::get_minecraft_folder()),
                        "multimc" => super::Launcher::MultiMC(super::get_multimc_folder(
                            launcher.last().expect("Invalid MultiMC!"),
                        )),
                        &_ => {
                            panic!("Invalid launcher!")
                        }
                    });
                    cx.props.settings.set(false);
                },
                select {
                    name: "launcher-select",
                    option {
                        value: "vanilla",
                        "Vanilla"
                    }
                    option {
                        value: "multimc-MultiMC",
                        "MultiMC"
                    }
                    option {
                        value: "multimc-PrismLauncher",
                        "Prism Launcher"
                    }
                    // TODO(Add other launchers support)
                    option {
                        value: "other",
                        "Other"
                    }
                }
                input {
                    r#type: "submit",
                    value: "Save",
                    class: "install-button"
                }
            }
        }
    })
}

#[derive(Props, PartialEq)]
struct VersionProps<'a> {
    modpack_source: String,
    modpack_branch: String,
    launcher: &'a super::Launcher,
}

fn Version<'a>(cx: Scope<'a, VersionProps<'a>>) -> Element<'a> {
    let modpack_source = (cx.props.modpack_source).to_owned();
    let modpack_branch = (cx.props.modpack_branch).to_owned();
    let launcher = (cx.props.launcher).to_owned();
    let profile = use_future(cx, (), |_| async move {
        super::init(&modpack_source, &modpack_branch, launcher).await
    })
    .value();

    // 'use_future's will always be 'None' on components first render
    if profile.is_none() {
        return cx.render(rsx! {
            div {
                class: "container",
                "Loading..."
            }
        });
    };

    // states can be turned into an Rc using .current() and can be made into an owned value by using .as_ref().to_owned()
    // TODO(Clean this up)
    let installer_profile = use_state(cx, || profile.unwrap().to_owned());
    let installing = use_state(cx, || false);
    let credits = use_state(cx, || false);
    let on_submit = move |event: FormEvent| {
        cx.spawn({
            let mut installer_profile = profile.unwrap().to_owned();
            installing.set(true);
            let installing = installing.clone();
            async move {
                for k in event.data.values.keys() {
                    if event.data.values[k] == "true" {
                        installer_profile.enabled_features.push(k.to_owned())
                    }
                }

                if !installer_profile.installed {
                    super::install(installer_profile).await;
                } else if installer_profile.update_available {
                    super::update(installer_profile).await;
                }
                installing.set(false);
            }
        });
    };
    // TODO(Split these renders into multiple components)
    if **installing {
        cx.render(rsx! {
            div {
                class: "version-container",
                div {
                    class: "subtitle-container",
                    h1 {
                        "{installer_profile.manifest.subtitle}"
                    }
                }
                div {
                    class: "version-inner-container",
                    div {
                        class: "container",
                        style: "justify-items: center;",
                        Spinner {}
                        p {
                            "Installing..."
                        }
                    }
                }
            }
        })
    } else if **credits {
        cx.render(rsx! {
            div {
                class: "version-container",
                div {
                    class: "subtitle-container",
                    h1 {
                        "{installer_profile.manifest.subtitle}"
                    }
                }
                div {
                    class: "version-inner-container",
                    div {
                        class: "container",
                        div {
                            class: "info-container",
                            div {
                                class: "button-container",
                                button {
                                    class: "credits-button",
                                    onclick: move |_| {
                                        credits.set(false);
                                    },
                                    "X"
                                }
                            }
                            div {
                                class: "credits",
                                Credits {
                                    manifest: installer_profile.manifest.clone()
                                }
                            }
                        }
                    }
                }
            }
        })
    } else {
        cx.render(rsx! {
            div {
                class: "version-container",
                form {
                    onsubmit: on_submit,
                    div {
                        class: "subtitle-container",
                        h1 {
                            "{installer_profile.manifest.subtitle}"
                        }
                    }
                    div {
                        class: "version-inner-container",
                        div {
                            class: "container",
                            div {
                                class: "info-container",
                                div {
                                    class: "button-container",
                                    button {
                                        class: "credits-button",
                                        onclick: move |_| {
                                            credits.set(true);
                                        },
                                        "i"
                                    }
                                }
                                div {
                                    class: "feature-list",
                                    for feat in &installer_profile.manifest.features {
                                        label {
                                            if feat.default {
                                                rsx!(input {
                                                    name: "{feat.id}",
                                                    r#type: "checkbox",
                                                    checked: "true"
                                                })
                                            } else {
                                                rsx!(input {
                                                    name: "{feat.id}",
                                                    r#type: "checkbox"
                                                })
                                            }
                                            "{feat.name}"
                                        }
                                    }
                                }
                            }
                            div {
                                class: "description",
                                // Yes this will allow modpacks to include any valid html including script tags
                                dangerous_inner_html: "{installer_profile.manifest.description}"
                            }
                        }
                        input {
                            r#type: "submit",
                            value: "Install",
                            class: "install-button"
                        }
                    }
                }
            }
        })
    }
}

pub fn App(cx: Scope) -> Element {
    let modpack_source = "Commander07/modpack-test/";
    let branches: Vec<super::GithubBranch> = serde_json::from_str(
        super::build_http_client()
            .get(super::GH_API.to_owned() + modpack_source + "branches")
            .expect("Failed to retrive branches!")
            .text()
            .unwrap()
            .as_str(),
    )
    .expect("Failed to parse branches!");
    let launcher: &UseState<super::Launcher> = use_state(cx, || {
        super::Launcher::Vanilla(super::get_minecraft_folder())
    });
    let settings: &UseState<bool> = use_state(cx, || false);
    let cog = String::from("data:image/png;base64,") + include_str!("assets/cog_icon.png.base64");
    // TODO(Speed up font)
    let style_css = include_str!("style.css");
    let style_css = style_css.replace(
        "Wynncraft_Game_Font.woff2.base64",
        include_str!("assets/Wynncraft_Game_Font.woff2.base64"),
    );
    if **settings {
        cx.render(rsx! {
            style { style_css }
            Header {}
            div {
                class: "fake-body",
                Settings {
                    launcher_state: launcher,
                    settings: settings
                }
            }
        })
    } else {
        cx.render(rsx! {
            style { style_css }
                button {
                    class: "settings-button",
                    onclick: move |_| {
                        settings.set(true);
                    },
                    img {
                        src: "{cog}",
                    }
                }
            Header {}
            div {
                class: "fake-body",
                for i in 0..branches.len() {
                    Version {
                        modpack_source: modpack_source.to_string(),
                        modpack_branch: branches[i].name.clone(),
                        launcher: &**launcher
                    }
                }
            }
        })
    }
}
