use clap::Parser;
use git2::Repository;
use git_url_parse::GitUrl;
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
}

struct GitOpenConfig {
    is_open_link: bool,
    verbose: bool,
    branch: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    if let Some(path) = cli.path.as_deref() {
        println!("Value for path: {path}");
    }
    let config = GitOpenConfig {
        is_open_link: !cli.no_show,
        verbose: cli.verbose,
        branch: cli.branch,
    };
    if let Err(e) = run(&config) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(config: &GitOpenConfig) -> Result<(), Box<dyn std::error::Error>> {
    let repo = Repository::open_from_env()?;
    let remote = repo.find_remote("origin")?;
    let git_url = remote.url().ok_or("No url")?;
    let link_url = remote_url_to_repo_url(git_url, config)?;

    if config.is_open_link {
        open_link(&link_url)?;
    } else {
        println!("{}", link_url);
    }
    Ok(())
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
    if config.branch.is_some() {
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
            ..TEST_CONFIG
        };

        let expected = "https://bitbucket.company.com/projects/takac/repos/git-open/browse?at=refs%2Fheads%2Fmaster";
        let url = "ssh://git@bitbucket.company.com:7999/takac/git-open.git";
        assert_eq!(
            expected,
            super::remote_url_to_repo_url(url, &config).unwrap()
        );
    }
}
