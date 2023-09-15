mod bitbucket;
mod github;
mod gitlab;

use clap::CommandFactory;
use clap::*;
use clap_complete::*;
use git2::{Remote, Repository};
use git_url_parse::GitUrl;
use regex::Regex;
use serde::Deserialize;
use std::io;
// use std::collections::HashMap;
use duplicate::duplicate;
use std::path::Path;
use url::Url;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    // Path to file in repo
    #[arg(value_hint = ValueHint::FilePath)]
    path: Option<String>,

    #[arg(short, long)]
    no_show: Option<bool>,

    #[arg(short, long)]
    verbose: Option<bool>,

    #[arg(short, long, env = "GROWSE_BRANCH")]
    branch: Option<String>,

    #[arg(short, long, env = "GROWSE_REMOTE")]
    remote: Option<String>,

    #[arg(short, long, env = "GROWSE_CONFIG_FILE")]
    config_file: Option<String>,

    #[arg(long, value_name = "SHELL", value_parser, hide = true)]
    completion: Option<Shell>,
}

// TODO XDG_CONFIG_HOME
const DEFAULT_CONFIG_FILE: &str = "./config/growse.toml";

#[derive(Debug, Deserialize, Clone)]
struct GrowseConfigFile {
    growse: GrowseConfig,
}

#[derive(Debug, Deserialize, Clone)]
struct GrowseConfig {
    // branch: Option<String>,
    // remote_priority: Option<Vec<String>>,
    #[serde(default)]
    use_branch: bool,
    #[serde(default)]
    no_show: bool,
    #[serde(default)]
    verbose: bool,
    // #[serde(default)]
    // hosts: Option<HashMap<String, String>>,
}

#[derive(Clone)]
struct GrowseState {
    path: Option<String>,
    line_number: Option<u32>,
    current_dir: String,
    repo_dir: String,
    // remote_name: String,
    branch: String,
}

trait RepoUrler {
    fn to_url(&self) -> Result<String, Box<dyn std::error::Error>>;
}

trait Repo {
    fn is_host(&self) -> bool;
    fn to_repo_url(&self) -> Result<String, Box<dyn std::error::Error>>;
    fn to_repo_url_with_branch(&self) -> Result<String, Box<dyn std::error::Error>>;
    fn to_repo_url_with_path(&self) -> Result<String, Box<dyn std::error::Error>>;
    fn to_repo_url_with_path_and_branch(&self) -> Result<String, Box<dyn std::error::Error>>;
    fn to_repo_url_with_path_and_line_number(&self) -> Result<String, Box<dyn std::error::Error>>;
    fn to_repo_url_with_path_and_branch_and_line_number(
        &self,
    ) -> Result<String, Box<dyn std::error::Error>>;
}

duplicate! {
    [ name; [BitBucket]; [GitHub]; [GitLab] ]
    pub struct name {
        url: GitUrl,
        config: GrowseConfig,
        state: GrowseState,
    }
    impl RepoUrler for name {
        fn to_url(&self) -> Result<String, Box<dyn std::error::Error>> {
            if self.config.use_branch {
                if self.state.path.is_some() {
                    if self.state.line_number.is_some() {
                        self.to_repo_url_with_path_and_branch_and_line_number()
                    } else {
                        self.to_repo_url_with_path_and_branch()
                    }
                } else {
                    self.to_repo_url_with_branch()
                }
            } else if self.state.path.is_some() {
                if self.state.line_number.is_some() {
                    self.to_repo_url_with_path_and_line_number()
                } else {
                    self.to_repo_url_with_path()
                }
            } else {
                self.to_repo_url()
            }
        }
    }
}

fn remote_url_to_repo_url(
    url: &str,
    state: &GrowseState,
    config: &GrowseConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    let url = GitUrl::parse(url)?;

    let github = GitHub {
        url: url.clone(),
        config: config.clone(),
        state: state.clone(),
    };
    let gitlab = GitLab {
        url: url.clone(),
        config: config.clone(),
        state: state.clone(),
    };
    let bitbucket = BitBucket {
        url,
        config: config.clone(),
        state: state.clone(),
    };

    if github.is_host() {
        github.to_url()
    } else if gitlab.is_host() {
        gitlab.to_url()
    } else if bitbucket.is_host() {
        bitbucket.to_url()
    } else {
        panic!("Unknown host")
    }
}

fn main() {
    let cli = Cli::parse();

    if let Some(shell) = cli.completion {
        let mut c = Cli::command();
        let name = c.get_name().to_string();
        generate(shell, &mut c, name, &mut io::stdout());
        std::process::exit(0);
    }
    if let Err(e) = run(&cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn merge_config_cli(cli: &Cli, config: &GrowseConfig) -> GrowseConfig {
    let mut config = config.clone();
    config.use_branch = cli.branch.is_some();
    if cli.no_show.is_some() {
        config.no_show = cli.no_show.unwrap();
    }
    if cli.verbose.is_some() {
        config.verbose = cli.verbose.unwrap();
    }
    config
}

fn config(cli: &Cli) -> Result<GrowseConfig, Box<dyn std::error::Error>> {
    // use given config file
    if let Some(config_file) = cli.config_file.as_ref() {
        let config: GrowseConfigFile =
            toml::from_str(&std::fs::read_to_string(cli.config_file.clone().unwrap())?)?;
        if Path::new(config_file).exists() {
            Ok(merge_config_cli(cli, &config.growse))
        } else {
            Err(format!("Config file {} not found", config_file).into())
        }
    } else {
        // lookup default config file
        if Path::new(DEFAULT_CONFIG_FILE).exists() {
            let config: GrowseConfigFile =
                toml::from_str(&std::fs::read_to_string(cli.config_file.clone().unwrap())?)?;
            Ok(merge_config_cli(cli, &config.growse))
        } else {
            Ok(GrowseConfig {
                use_branch: cli.branch.is_some(),
                no_show: cli.no_show.unwrap_or(false),
                verbose: cli.verbose.unwrap_or(false),
            })
        }
    }
}

fn run(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let config = config(cli)?;

    // TODO check if file exists locally??
    let (path, line_number) = if let Some(path) = cli.path.as_deref() {
        let re = Regex::new(r"(.*?)((:)(\d+))?$").unwrap();
        let captures = re.captures(path).unwrap();
        let path = captures.get(1).unwrap().as_str().to_string();
        if let Some(line_number) = captures.get(4) {
            let x = line_number.as_str().parse::<u32>().unwrap();
            (Some(path), Some(x))
        } else {
            (Some(path), None)
        }
    } else {
        (None, None)
    };

    let repo = Repository::open_from_env()?;

    let remote_name = if cli.remote.is_none() {
        default_remote(&repo)
    } else {
        // TODO Manually check remote exists in repo??
        cli.remote.clone().unwrap()
    };
    let remote = repo.find_remote(&remote_name)?;

    let branch = if cli.branch.is_some() {
        cli.branch.clone().unwrap()
    } else {
        default_branch(&repo, &remote, &config)
    };

    let git_url = remote.url().ok_or("No url found for remote")?;

    let state = GrowseState {
        path,
        line_number,
        // remote_name,
        branch,
        current_dir: std::env::current_dir()?.to_str().unwrap().to_string(),
        repo_dir: repo
            .path()
            .parent()
            .ok_or("No parent found")?
            .to_string_lossy()
            .to_string(),
    };

    println!("branch: {:?}", state.branch);

    if config.verbose {
        println!("repo_dir: {:?}", state.repo_dir);
    }

    let link_url = remote_url_to_repo_url(git_url, &state, &config)?;

    if !config.no_show {
        open_link(&link_url)?;
    } else {
        println!("{}", link_url);
    }
    Ok(())
}

fn default_remote(repo: &Repository) -> String {
    return "origin".to_string();
}

// fn remote_name(
//     repo: &Repository,
//     config: &GrowseState,
// ) -> Result<String, Box<dyn std::error::Error>> {
//     let binding = repo.remotes()?;
//     let remotes = binding.iter().collect::<Vec<Option<&str>>>();
//     if config.remote_name.is_some() {
//         let remote_c = config.remote_name.as_ref().unwrap();
//         Ok(remote_c.to_string())
//     } else if remotes.is_empty() {
//         Err("No remote".into())
//     } else if remotes.len() > 1 {
//         // TODO config for preference?
//         println!(
//             "Warning: Multiple remotes: {:?}, choosing first {:?}",
//             remotes, remotes[0]
//         );
//         Ok(remotes[0].unwrap().to_string())
//     } else {
//         Ok(remotes[0].unwrap().to_string())
//     }
// }

fn default_branch(repo: &Repository, remote: &Remote, config: &GrowseConfig) -> String {
    let default_branch = remote.default_branch();
    if let Ok(default_branch) = default_branch {
        return default_branch.as_str().unwrap().to_string();
    } else {
        let remote_ref = format!("refs/remotes/{}/HEAD", remote.name().unwrap());
        let reference = repo.resolve_reference_from_short_name(&remote_ref);
        if let Ok(reference) = reference {
            let remote_ref_prefix = format!("refs/remotes/{}/", remote.name().unwrap());
            let name = reference.name();
            let short_name = Path::new(name.unwrap())
                .strip_prefix(remote_ref_prefix)
                .unwrap();
            if config.verbose {
                println!("reference name: {:?}", name);
                println!("resolved: {:?}", short_name);
            }
            if short_name != Path::new("HEAD") {
                return short_name.to_str().unwrap().to_string();
            }
        } else if config.verbose {
            println!("Could not resolve reference: {:?}", remote_ref);
        }
    }
    // TODO fall back to master or main?? lookup local remote branches?
    "master".to_string()
}

fn open_link(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    open::that(url)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    static TEST_CONFIG: GrowseConfig = GrowseConfig {
        verbose: true,
        no_show: false,
        use_branch: false,
        // remote_priority: Some(vec![]),
    };
    fn generate_test_state() -> GrowseState {
        GrowseState {
            branch: "master".to_string(),
            line_number: None,
            path: None,
            current_dir: "/home/takac/git-open".to_string(),
            repo_dir: "/home/takac/git-open".to_string(),
            // remote_name: "origin".to_string(),
        }
    }
    // static TEST_STATE: GrowseState = GrowseState {
    //     branch: "master".to_string(),
    //     line_number: None,
    //     path: None,
    //     current_dir: "/home/takac/git-open".to_string(),
    //     repo_dir: "/home/takac/git-open".to_string(),
    //     remote_name: "origin".to_string(),
    // };

    #[test]
    fn test_simple_repo_link() {
        let remote_urls = &[
            "ssh://git@github.com/takac/git-open",
            "https://github.com/takac/git-open",
            "git@github.com:takac/git-open",
        ];
        let test_state = generate_test_state();
        for url in remote_urls {
            let expected = "https://github.com/takac/git-open";
            assert_eq!(
                expected,
                remote_url_to_repo_url(url, &test_state, &TEST_CONFIG).unwrap()
            );
            assert_eq!(
                expected,
                remote_url_to_repo_url(&format!("{}.git", url), &test_state, &TEST_CONFIG).unwrap()
            );
        }
    }

    fn default_test(expected_to_input: HashMap<&str, &str>) {
        for (expected, input) in expected_to_input {
            assert_eq!(
                expected,
                remote_url_to_repo_url(input, &generate_test_state(), &TEST_CONFIG).unwrap()
            );
        }
    }

    #[test]
    fn test_bb_repo_link() {
        default_test(HashMap::from([
            (
                "https://bitbucket.company.com/projects/takac/repos/git-open",
                "ssh://git@bitbucket.company.com:7999/takac/git-open.git",
            ),
            (
                "https://bitbucket.gi.de/projects/dig/repos/frontend",
                "https://bitbucket.gi.de/scm/dig/frontend.git",
            ),
        ]));
    }

    #[test]
    fn test_gitlab_repo_link() {
        default_test(HashMap::from([(
            "https://gitlab.com/takac/side-project/git-open",
            "git@gitlab.com:takac/side-project/git-open.git",
        )]));
    }

    #[test]
    fn test_simple_repo_link_with_branch() {
        let remote_urls = &[
            "ssh://git@github.com/takac/git-open",
            "https://github.com/takac/git-open",
            "git@github.com:takac/git-open",
        ];
        let state = GrowseState {
            branch: "master".to_string(),
            ..generate_test_state()
        };

        let config = GrowseConfig {
            use_branch: true,
            ..TEST_CONFIG
        };

        for url in remote_urls {
            let expected = "https://github.com/takac/git-open/tree/master";
            assert_eq!(
                expected,
                remote_url_to_repo_url(url, &state, &config).unwrap()
            );
            assert_eq!(
                expected,
                remote_url_to_repo_url(&format!("{}.git", url), &state, &config).unwrap()
            );
        }
    }

    #[test]
    fn test_bb_repo_link_with_branch() {
        let state = GrowseState {
            branch: "master".to_string(),
            ..generate_test_state()
        };
        let config = GrowseConfig {
            use_branch: true,
            ..TEST_CONFIG
        };

        let expected = "https://bitbucket.company.com/projects/takac/repos/git-open/browse?at=refs%2Fheads%2Fmaster";
        let url = "ssh://git@bitbucket.company.com:7999/takac/git-open.git";
        assert_eq!(
            expected,
            remote_url_to_repo_url(url, &state, &config).unwrap()
        );
    }

    #[test]
    fn test_simple_repo_link_with_path() {
        let remote_urls = &[
            "ssh://git@github.com/takac/git-open",
            "https://github.com/takac/git-open",
            "git@github.com:takac/git-open",
        ];
        let state = GrowseState {
            branch: "main".to_string(),
            path: Some("src/main.rs".to_string()),
            ..generate_test_state()
        };

        for url in remote_urls {
            let expected = "https://github.com/takac/git-open/blob/main/src/main.rs";
            assert_eq!(
                expected,
                remote_url_to_repo_url(url, &state, &TEST_CONFIG).unwrap()
            );
            assert_eq!(
                expected,
                remote_url_to_repo_url(&format!("{}.git", url), &state, &TEST_CONFIG).unwrap()
            );
        }
    }

    #[test]
    fn test_simple_repo_link_with_path_not_at_root() {
        let remote_urls = &[
            "ssh://git@github.com/takac/git-open",
            "https://github.com/takac/git-open",
            "git@github.com:takac/git-open",
        ];
        let state = GrowseState {
            branch: "main".to_string(),
            path: Some("main.rs".to_string()),
            ..generate_test_state()
        };

        for url in remote_urls {
            let expected = "https://github.com/takac/git-open/blob/main/src/main.rs";
            assert_eq!(
                expected,
                remote_url_to_repo_url(url, &state, &TEST_CONFIG).unwrap()
            );
            assert_eq!(
                expected,
                remote_url_to_repo_url(&format!("{}.git", url), &state, &TEST_CONFIG).unwrap()
            );
        }
    }

    #[test]
    fn test_simple_repo_link_with_path_line_nos() {
        let remote_urls = &[
            "ssh://git@github.com/takac/git-open",
            "https://github.com/takac/git-open",
            "git@github.com:takac/git-open",
        ];
        let state = GrowseState {
            branch: "main".to_string(),
            path: Some("main.rs".to_string()),
            line_number: Some(10),
            ..generate_test_state()
        };

        for url in remote_urls {
            let expected = "https://github.com/takac/git-open/blob/main/src/main.rs#L10";
            assert_eq!(
                expected,
                remote_url_to_repo_url(url, &state, &TEST_CONFIG).unwrap()
            );
        }
    }

    #[test]
    fn test_repo_link_with_remote() {
        let remote_urls = &[
            "ssh://git@github.com/takac/git-open",
            "https://github.com/takac/git-open",
            "git@github.com:takac/git-open",
        ];
        let state = GrowseState {
            branch: "main".to_string(),
            path: Some("main.rs".to_string()),
            line_number: Some(10),
            current_dir: "/home/takac/git-open/src".to_string(),
            repo_dir: "/home/takac/git-open".to_string(),
            // remote_name: "origin".to_string(),
        };

        for url in remote_urls {
            let expected = "https://github.com/takac/git-open/blob/main/src/main.rs#L10";
            assert_eq!(
                expected,
                remote_url_to_repo_url(url, &state, &TEST_CONFIG).unwrap()
            );
        }
    }

    #[test]
    fn test_load_config() {
        let config: GrowseConfigFile = toml::from_str(
            r#"
            [growse]
            verbose = true
            "#,
        )
        .unwrap();
        // config.growse.into
        assert!(config.growse.verbose);
    }
}
