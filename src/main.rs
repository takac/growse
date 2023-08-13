use clap::Parser;
use git2::{Remote, Repository};
use git_url_parse::GitUrl;
use regex::Regex;
use serde::Deserialize;
// use std::collections::HashMap;
use std::path::Path;
use url::Url;

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    // Path to file in repo
    path: Option<String>,

    #[arg(short, long)]
    no_show: Option<bool>,

    #[arg(short, long)]
    verbose: Option<bool>,

    #[arg(short, long, env = "GROWSE_BRANCH")]
    branch: Option<String>,

    #[clap(short, long, env = "GROWSE_REMOTE")]
    remote: Vec<String>,

    #[clap(short, long, env = "GROWSE_CONFIG_FILE")]
    config_file: Option<String>,
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
    current_dir: Option<String>,
    repo_dir: Option<String>,
    remote_name: Option<String>,
    branch: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    // cli.try_into().unwrap();
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

    let mut state = GrowseState {
        path,
        line_number,
        remote_name: None,
        branch: None,
        current_dir: Some(
            std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        ),
        repo_dir: Some(repo.path().parent().unwrap().to_str().unwrap().to_string()),
    };
    let remote_name = remote_name(&repo, &state)?;
    let remote = repo.find_remote(&remote_name)?;
    let git_url = remote.url().ok_or("No url")?;

    state.branch = default_branch(&repo, &remote, &config);

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

fn remote_name(
    repo: &Repository,
    config: &GrowseState,
) -> Result<String, Box<dyn std::error::Error>> {
    let binding = repo.remotes()?;
    let remotes = binding.iter().collect::<Vec<Option<&str>>>();
    if config.remote_name.is_some() {
        let remote_c = config.remote_name.as_ref().unwrap();
        Ok(remote_c.to_string())
    } else if remotes.is_empty() {
        Err("No remote".into())
    } else if remotes.len() > 1 {
        // TODO config for preference?
        println!(
            "Warning: Multiple remotes: {:?}, choosing first {:?}",
            remotes, remotes[0]
        );
        Ok(remotes[0].unwrap().to_string())
    } else {
        Ok(remotes[0].unwrap().to_string())
    }
}

fn default_branch(repo: &Repository, remote: &Remote, config: &GrowseConfig) -> Option<String> {
    let default_branch = remote.default_branch();
    if let Ok(default_branch) = default_branch {
        return Some(default_branch.as_str().unwrap().to_string());
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
                return Some(short_name.to_str().unwrap().to_string());
            }
        } else if config.verbose {
            println!("Could not resolve reference: {:?}", remote_ref);
        }
    }
    // TODO fall back to master or main?? lookup local remote branches?
    None
}

#[derive(Clone)]
struct BitBucket {
    url: GitUrl,
    config: GrowseConfig,
    state: GrowseState,
}

#[derive(Clone)]
struct GitHub {
    url: GitUrl,
    config: GrowseConfig,
    state: GrowseState,
}

#[derive(Clone)]
struct GitLab {
    url: GitUrl,
    config: GrowseConfig,
    state: GrowseState,
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

impl RepoUrler for BitBucket {
    fn to_url(&self) -> Result<String, Box<dyn std::error::Error>> {
        if self.config.use_branch {
            if self.state.path.is_some() {
                self.to_repo_url_with_path()
            } else {
                self.to_repo_url_with_branch()
            }
        } else if self.state.path.is_some() {
            self.to_repo_url_with_path()
        } else {
            self.to_repo_url()
        }
    }
}

impl RepoUrler for GitHub {
    fn to_url(&self) -> Result<String, Box<dyn std::error::Error>> {
        if self.config.use_branch {
            if self.state.path.is_some() {
                self.to_repo_url_with_path()
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

impl RepoUrler for GitLab {
    fn to_url(&self) -> Result<String, Box<dyn std::error::Error>> {
        if self.config.use_branch {
            if self.state.path.is_some() {
                self.to_repo_url_with_path()
            } else {
                self.to_repo_url_with_branch()
            }
        } else if self.state.path.is_some() {
            self.to_repo_url_with_path()
        } else {
            self.to_repo_url()
        }
    }
}

impl Repo for BitBucket {
    fn is_host(&self) -> bool {
        self.url.host.as_ref().unwrap().contains("bitbucket") || self.url.port == Some(7999)
    }

    fn to_repo_url_with_path_and_branch(&self) -> Result<String, Box<dyn std::error::Error>> {
        Err("Not implemented".into())
    }

    fn to_repo_url_with_path(&self) -> Result<String, Box<dyn std::error::Error>> {
        Err("Not implemented".into())
    }

    fn to_repo_url_with_path_and_branch_and_line_number(
        &self,
    ) -> Result<String, Box<dyn std::error::Error>> {
        Err("Not implemented".into())
    }

    fn to_repo_url_with_path_and_line_number(&self) -> Result<String, Box<dyn std::error::Error>> {
        Err("Not implemented".into())
    }

    fn to_repo_url_with_branch(&self) -> Result<String, Box<dyn std::error::Error>> {
        let host = self.url.host.clone().ok_or("No host found")?;
        let owner = self.url.owner.clone().ok_or("No owner found")?;

        let branch_name = format!(
            "refs/heads/{}",
            self.state.branch.as_ref().ok_or("No branch found")?
        );

        let new_url = Url::parse_with_params(
            &format!(
                "https://{}/projects/{}/repos/{}/browse",
                host, owner, self.url.name
            ),
            &[("at", branch_name.as_str())],
        )?;

        Ok(new_url.to_string())
    }

    fn to_repo_url(&self) -> Result<String, Box<dyn std::error::Error>> {
        // https://opensource.ncsa.illinois.edu/bitbucket/scm/bd/bdcli.git
        // https://opensource.ncsa.illinois.edu/bitbucket/projects/bd/repos/bdcli/browse

        // https://bitbucket.gi.de/scm/dig/frontend.git
        // https://bitbucket.gi.de/projects/DIG/repos/frontend/browse

        let host = self.url.host.clone().ok_or("No host found")?;
        let owner = self.url.owner.clone().ok_or("No owner found")?;

        Ok(format!(
            "https://{}/projects/{}/repos/{}",
            host, owner, self.url.name
        ))
    }
}

// use proc_macro::TokenStream;
// use quote::quote;
// use syn;

// #[proc_macro_derive(RepoMacro)]
// pub fn to_repo_derive(input: TokenStream) -> TokenStream {
//     // Construct a representation of Rust code as a syntax tree
//     // that we can manipulate
//     let ast = syn::parse(input).unwrap();

//     // Build the trait implementation
//     impl_repo_macro(&ast)
// }

// fn impl_repo_macro(ast: &syn::DeriveInput) -> TokenStream {
//     let name = &ast.ident;
//     let gen = quote! {
//         impl RepoUrler for #name {
//             fn to_url(&self) -> Result<String, Box<dyn std::error::Error>> {
//                 if self.config.use_branch {
//                     if self.state.path.is_some() {
//                         self.to_repo_url_with_path()
//                     } else {
//                         self.to_repo_url_with_branch()
//                     }
//                 } else if self.state.path.is_some() {
//                     self.to_repo_url_with_path()
//                 } else {
//                     self.to_repo_url()
//                 }
//             }
//         }
//     };
//     gen.into()
// }

impl Repo for GitHub {
    fn is_host(&self) -> bool {
        self.url.host.as_ref().unwrap().contains("github")
    }
    //     if self.state.line_number.is_some() {
    //         let line = self.state.line_number.unwrap();
    //         Ok(format!(
    //             "https://{}/{}/blob/{}/{}#L{}",
    //             host, self.url.fullname, branch, path, line,
    //         ))

    fn to_repo_url_with_path_and_branch(&self) -> Result<String, Box<dyn std::error::Error>> {
        Err("Not implemented".into())
    }

    fn to_repo_url_with_path_and_branch_and_line_number(
        &self,
    ) -> Result<String, Box<dyn std::error::Error>> {
        Err("Not implemented".into())
    }

    fn to_repo_url_with_path_and_line_number(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(format!(
            "{}#L{}",
            self.to_repo_url_with_path().unwrap(),
            self.state.line_number.unwrap()
        ))
    }

    fn to_repo_url_with_path(&self) -> Result<String, Box<dyn std::error::Error>> {
        let host = self.url.host.clone().ok_or("No host found")?;
        let branch = self.state.branch.as_ref().unwrap();

        let path = if self.state.current_dir == self.state.repo_dir {
            self.state.path.as_ref().unwrap().clone()
        } else {
            let repo_dir = self.state.repo_dir.as_ref().unwrap();
            let current_dir = self.state.current_dir.as_ref().unwrap();
            let offset_path = current_dir.strip_prefix(&format!("{}/", repo_dir)).unwrap();
            format!("{}/{}", offset_path, self.state.path.as_ref().unwrap())
        };
        Ok(format!(
            "https://{}/{}/blob/{}/{}",
            host, self.url.fullname, branch, path,
        ))
    }

    fn to_repo_url_with_branch(&self) -> Result<String, Box<dyn std::error::Error>> {
        let host = self.url.host.clone().ok_or("No host found")?;

        Ok(format!(
            "https://{}/{}/tree/{}",
            host,
            self.url.fullname,
            self.state.branch.as_ref().unwrap()
        ))
    }

    fn to_repo_url(&self) -> Result<String, Box<dyn std::error::Error>> {
        // https://docs.er.kcl.ac.uk/CREATE/web/git/
        //  git@github.kcl.ac.uk:USERNAME/REPO.git
        let host = self.url.host.clone().ok_or("No host found")?;
        Ok(format!("https://{}/{}", host, self.url.fullname))
    }
}

impl Repo for GitLab {
    fn is_host(&self) -> bool {
        self.url.host.as_ref().unwrap().contains("gitlab")
    }

    fn to_repo_url_with_path_and_branch(&self) -> Result<String, Box<dyn std::error::Error>> {
        Err("Not implemented".into())
    }

    fn to_repo_url_with_path(&self) -> Result<String, Box<dyn std::error::Error>> {
        Err("Not implemented".into())
    }

    fn to_repo_url_with_path_and_branch_and_line_number(
        &self,
    ) -> Result<String, Box<dyn std::error::Error>> {
        Err("Not implemented".into())
    }

    fn to_repo_url_with_path_and_line_number(&self) -> Result<String, Box<dyn std::error::Error>> {
        Err("Not implemented".into())
    }

    fn to_repo_url_with_branch(&self) -> Result<String, Box<dyn std::error::Error>> {
        let host = self.url.host.clone().ok_or("No host found")?;
        let owner = self.url.owner.clone().ok_or("No owner found")?;
        let branch_name = format!("refs/heads/{}", self.state.branch.as_ref().unwrap());
        let new_url = Url::parse_with_params(
            &format!(
                "https://{}/{}/{}/-/tree/{}",
                host, owner, self.url.name, branch_name
            ),
            &[("at", branch_name)],
        )?;
        Ok(new_url.as_str().to_string())
    }

    fn to_repo_url(&self) -> Result<String, Box<dyn std::error::Error>> {
        if self.config.verbose {
            println!("gitlab_url_to_repo_url: {:?}", self.url);
        }
        // git@gitlab.com:gitlab-com/gl-infra/gitlab-dedicated/library/terraform/cloudwatch_log_export.git
        // https://gitlab.com/gitlab-com/gl-infra/gitlab-dedicated/library/terraform/cloudwatch_log_export.git
        // default head
        // https://gitlab.com/gitlab-com/gl-infra/gitlab-dedicated/library/terraform/cloudwatch_log_export
        // branch
        // https://gitlab.com/gitlab-com/gl-infra/gitlab-dedicated/library/terraform/cloudwatch_log_export/-/tree/1.1.0?ref_type=tags

        let host = self.url.host.clone().ok_or("No host found")?;
        let owner = self.url.owner.clone().ok_or("No owner found")?;
        let parts = self
            .url
            .path
            .trim_matches('/')
            .split('/')
            .collect::<Vec<&str>>();

        // FIXME this doesn't really work
        let organization = parts[0];
        // use path??
        let new_url = format!(
            "https://{}/{}/{}/{}",
            host, organization, owner, self.url.name
        );
        Ok(new_url)
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
    static TEST_STATE: GrowseState = GrowseState {
        branch: None,
        path: None,
        current_dir: None,
        repo_dir: None,
        line_number: None,
        remote_name: None,
    };

    #[test]
    fn test_simple_repo_link() {
        let remote_urls = &[
            "ssh://git@github.com/takac/git-open",
            "https://github.com/takac/git-open",
            "git@github.com:takac/git-open",
        ];
        for url in remote_urls {
            let expected = "https://github.com/takac/git-open";
            assert_eq!(
                expected,
                remote_url_to_repo_url(url, &TEST_STATE, &TEST_CONFIG).unwrap()
            );
            assert_eq!(
                expected,
                remote_url_to_repo_url(&format!("{}.git", url), &TEST_STATE, &TEST_CONFIG).unwrap()
            );
        }
    }

    fn default_test(expected_to_input: HashMap<&str, &str>) {
        for (expected, input) in expected_to_input {
            assert_eq!(
                expected,
                remote_url_to_repo_url(input, &TEST_STATE, &TEST_CONFIG).unwrap()
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
            branch: Some("master".to_string()),
            path: None,
            current_dir: None,
            repo_dir: None,
            remote_name: None,
            ..TEST_STATE
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
            branch: Some("master".to_string()),
            path: None,
            current_dir: None,
            repo_dir: None,
            remote_name: None,
            ..TEST_STATE
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
            branch: Some("main".to_string()),
            path: Some("src/main.rs".to_string()),
            current_dir: Some("/home/takac/git-open".to_string()),
            repo_dir: Some("/home/takac/git-open".to_string()),
            remote_name: None,
            ..TEST_STATE
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
            branch: Some("main".to_string()),
            path: Some("main.rs".to_string()),
            current_dir: Some("/home/takac/git-open/src".to_string()),
            repo_dir: Some("/home/takac/git-open".to_string()),
            remote_name: None,
            ..TEST_STATE
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
            branch: Some("main".to_string()),
            path: Some("main.rs".to_string()),
            line_number: Some(10),
            current_dir: Some("/home/takac/git-open/src".to_string()),
            repo_dir: Some("/home/takac/git-open".to_string()),
            remote_name: None,
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
