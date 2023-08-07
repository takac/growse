use clap::Parser;
use git2::{Remote, Repository};
use git_url_parse::GitUrl;
use regex::Regex;
use std::path::Path;
use url::Url;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    // Path to file in repo
    path: Option<String>,

    #[arg(short, long)]
    no_show: bool,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    branch: Option<String>,
    // use current branch
    // #[arg(short, long)]
    // current_branch: bool,
}

#[derive(Clone)]
struct GitOpenConfig {
    is_open_link: bool,
    verbose: bool,
    branch: Option<String>,
    path: Option<String>,
    line_number: Option<u32>,
    default_branch: Option<String>,
    current_dir: Option<String>,
    repo_dir: Option<String>,
}

fn main() {
    let cli = Cli::parse();

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

    let config = GitOpenConfig {
        is_open_link: !cli.no_show,
        verbose: cli.verbose,
        branch: cli.branch,
        path,
        line_number,
        default_branch: None,
        current_dir: Some(
            std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        ),
        repo_dir: None,
    };
    if let Err(e) = run(&config) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(config: &GitOpenConfig) -> Result<(), Box<dyn std::error::Error>> {
    let repo = Repository::open_from_env()?;
    // TODO list/choose remote!
    let remote = repo.find_remote("origin")?;
    let git_url = remote.url().ok_or("No url")?;

    let mut config = config.clone();
    config.default_branch = default_head(&repo, &remote, &config);
    config.repo_dir = Some(repo.path().parent().unwrap().to_str().unwrap().to_string());

    if config.verbose {
        println!("repo_dir: {:?}", config.repo_dir);
    }

    let link_url = remote_url_to_repo_url(git_url, &config)?;
    if config.is_open_link {
        open_link(&link_url)?;
    } else {
        println!("{}", link_url);
    }
    Ok(())
}

fn default_head(repo: &Repository, remote: &Remote, config: &GitOpenConfig) -> Option<String> {
    let default_branch = remote.default_branch();
    if let Ok(default_branch) = default_branch {
        if config.verbose {
            println!("default_branch: {:?}", config.default_branch);
        }
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
    None
}

fn remote_url_to_repo_url(
    url: &str,
    config: &GitOpenConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    let url = GitUrl::parse(url)?;
    let host = url.host.ok_or("No host found")?;

    // bitbucket
    if url.port == Some(7999) {
        let owner = url.owner.ok_or("No owner found")?;
        if config.branch.is_some() {
            let branch_name = format!("refs/heads/{}", config.branch.as_ref().unwrap());
            let new_url = Url::parse_with_params(
                &format!(
                    "https://{}/projects/{}/repos/{}/browse",
                    host, owner, url.name
                ),
                &[("at", branch_name)],
            )?;
            return Ok(new_url.as_str().to_string());
        } else {
            return Ok(format!(
                "https://{}/projects/{}/repos/{}",
                host, owner, url.name
            ));
        }
    }

    let parts = url.path.split('/').collect::<Vec<&str>>();

    if config.verbose {
        println!("path: {:?}", url.path);
        println!("parts: {:?}", parts);
    }

    if parts.len() == 3 && !parts[0].is_empty() {
        let organization = parts[0];
        let owner = url.owner.ok_or("No owner found")?;
        let new_url = format!("https://{}/{}/{}/{}", host, organization, owner, url.name);
        return Ok(new_url);
    }

    if config.path.is_some() {
        let branch = if config.branch.is_some() {
            config.branch.as_ref().unwrap()
        } else {
            let default_branch = &config.default_branch;
            if default_branch.is_some() {
                default_branch.as_ref().unwrap()
            } else {
                println!("Warning: No default branch found, defaulting to master");
                "master"
            }
        };
        let path = if config.current_dir == config.repo_dir {
            config.path.as_ref().unwrap().clone()
        } else {
            let repo_dir = config.repo_dir.as_ref().unwrap();
            let current_dir = config.current_dir.as_ref().unwrap();
            let offset_path = current_dir.strip_prefix(&format!("{}/", repo_dir)).unwrap();
            format!("{}/{}", offset_path, config.path.as_ref().unwrap())
        };
        if config.line_number.is_some() {
            let line = config.line_number.unwrap();
            Ok(format!(
                "https://{}/{}/blob/{}/{}#L{}",
                host, url.fullname, branch, path, line,
            ))
        } else {
            Ok(format!(
                "https://{}/{}/blob/{}/{}",
                host, url.fullname, branch, path,
            ))
        }
    } else if config.branch.is_some() {
        Ok(format!(
            "https://{}/{}/tree/{}",
            host,
            url.fullname,
            config.branch.as_ref().unwrap()
        ))
    } else {
        Ok(format!("https://{}/{}", host, url.fullname))
    }
}

fn open_link(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    open::that(url)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    static TEST_CONFIG: super::GitOpenConfig = super::GitOpenConfig {
        is_open_link: true,
        verbose: false,
        branch: None,
        path: None,
        default_branch: None,
        current_dir: None,
        repo_dir: None,
        line_number: None,
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
                super::remote_url_to_repo_url(url, &TEST_CONFIG).unwrap()
            );
            assert_eq!(
                expected,
                super::remote_url_to_repo_url(&format!("{}.git", url), &TEST_CONFIG).unwrap()
            );
        }
    }

    #[test]
    fn test_bb_repo_link() {
        let expected = "https://bitbucket.company.com/projects/takac/repos/git-open";
        let url = "ssh://git@bitbucket.company.com:7999/takac/git-open.git";
        assert_eq!(
            expected,
            super::remote_url_to_repo_url(url, &TEST_CONFIG).unwrap()
        );
    }

    #[test]
    fn test_gitlab_repo_link() {
        let expected = "https://gitlab.com/takac/side-project/git-open";
        let url = "git@gitlab.com:takac/side-project/git-open.git";
        assert_eq!(
            expected,
            super::remote_url_to_repo_url(url, &TEST_CONFIG).unwrap()
        );
    }

    #[test]
    fn test_simple_repo_link_with_branch() {
        let remote_urls = &[
            "ssh://git@github.com/takac/git-open",
            "https://github.com/takac/git-open",
            "git@github.com:takac/git-open",
        ];
        let config = super::GitOpenConfig {
            branch: Some("master".to_string()),
            path: None,
            default_branch: None,
            current_dir: None,
            repo_dir: None,
            ..TEST_CONFIG
        };

        for url in remote_urls {
            let expected = "https://github.com/takac/git-open/tree/master";
            assert_eq!(
                expected,
                super::remote_url_to_repo_url(url, &config).unwrap()
            );
            assert_eq!(
                expected,
                super::remote_url_to_repo_url(&format!("{}.git", url), &config).unwrap()
            );
        }
    }

    #[test]
    fn test_bb_repo_link_with_branch() {
        let config = super::GitOpenConfig {
            branch: Some("master".to_string()),
            path: None,
            default_branch: None,
            current_dir: None,
            repo_dir: None,
            ..TEST_CONFIG
        };

        let expected = "https://bitbucket.company.com/projects/takac/repos/git-open/browse?at=refs%2Fheads%2Fmaster";
        let url = "ssh://git@bitbucket.company.com:7999/takac/git-open.git";
        assert_eq!(
            expected,
            super::remote_url_to_repo_url(url, &config).unwrap()
        );
    }

    #[test]
    fn test_simple_repo_link_with_path() {
        let remote_urls = &[
            "ssh://git@github.com/takac/git-open",
            "https://github.com/takac/git-open",
            "git@github.com:takac/git-open",
        ];
        let config = super::GitOpenConfig {
            branch: None,
            path: Some("src/main.rs".to_string()),
            default_branch: Some("main".to_string()),
            current_dir: Some("/home/takac/git-open".to_string()),
            repo_dir: Some("/home/takac/git-open".to_string()),
            ..TEST_CONFIG
        };

        for url in remote_urls {
            let expected = "https://github.com/takac/git-open/blob/main/src/main.rs";
            assert_eq!(
                expected,
                super::remote_url_to_repo_url(url, &config).unwrap()
            );
            assert_eq!(
                expected,
                super::remote_url_to_repo_url(&format!("{}.git", url), &config).unwrap()
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
        let config = super::GitOpenConfig {
            branch: None,
            path: Some("main.rs".to_string()),
            default_branch: Some("main".to_string()),
            current_dir: Some("/home/takac/git-open/src".to_string()),
            repo_dir: Some("/home/takac/git-open".to_string()),
            ..TEST_CONFIG
        };

        for url in remote_urls {
            let expected = "https://github.com/takac/git-open/blob/main/src/main.rs";
            assert_eq!(
                expected,
                super::remote_url_to_repo_url(url, &config).unwrap()
            );
            assert_eq!(
                expected,
                super::remote_url_to_repo_url(&format!("{}.git", url), &config).unwrap()
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
        let config = super::GitOpenConfig {
            branch: None,
            path: Some("main.rs".to_string()),
            line_number: Some(10),
            default_branch: Some("main".to_string()),
            current_dir: Some("/home/takac/git-open/src".to_string()),
            repo_dir: Some("/home/takac/git-open".to_string()),
            ..TEST_CONFIG
        };

        for url in remote_urls {
            let expected = "https://github.com/takac/git-open/blob/main/src/main.rs#L10";
            assert_eq!(
                expected,
                super::remote_url_to_repo_url(url, &config).unwrap()
            );
        }
    }
}
