use git2::Repository;
use git_url_parse::GitUrl;

fn main() {
    // println!("Hello, world!");
    match run() {
        Ok(_) => {}
        Err(e) => println!("Error: {}", e),
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let repo = Repository::open_from_env()?;
    let remote = repo.find_remote("origin")?;
    let git_url = remote.url().ok_or("No url")?;
    let link_url = remote_url_to_repo_url(git_url)?;
    println!("{}", link_url);
    open_link(&link_url)?;
    Ok(())
}

fn remote_url_to_repo_url(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = GitUrl::parse(url)?;
    let host = url.host.ok_or("No host found")?;

    // bibucket
    if url.port == Some(7999) {
        let owner = url.owner.ok_or("No owner found")?;
        let new_url = format!("https://{}/projects/{}/repos/{}", host, owner, url.name);
        return Ok(new_url);
    }

    println!("path: {:?}", url.path);
    let parts = url.path.split('/').collect::<Vec<&str>>();
    println!("parts: {:?}", parts);
    if parts.len() == 3 && !parts[0].is_empty() {
        let organization = parts[0];
        let owner = url.owner.ok_or("No owner found")?;
        let new_url = format!("https://{}/{}/{}/{}", host, organization, owner, url.name);
        return Ok(new_url);
    }

    let new_url = format!("https://{}/{}", host, url.fullname);
    Ok(new_url)
}

fn open_link(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    open::that(url)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_simple_repo_link() {
        let remote_urls = &[
            "ssh://git@github.com/takac/git-open",
            "https://github.com/takac/git-open",
            "git@github.com:takac/git-open",
        ];

        for url in remote_urls {
            let expected = "https://github.com/takac/git-open";
            assert_eq!(expected, super::remote_url_to_repo_url(url).unwrap());
            assert_eq!(
                expected,
                super::remote_url_to_repo_url(&format!("{}.git", url)).unwrap()
            );
        }
    }

    #[test]
    fn test_bb_repo_link() {
        let expected = "https://bitbucket.company.com/projects/takac/repos/git-open";
        let url = "ssh://git@bitbucket.company.com:7999/takac/git-open.git";
        assert_eq!(expected, super::remote_url_to_repo_url(url).unwrap());
    }

    #[test]
    fn test_gitlab_repo_link() {
        let expected = "https://gitlab.com/takac/side-project/git-open";
        let url = "git@gitlab.com:takac/side-project/git-open.git";
        assert_eq!(expected, super::remote_url_to_repo_url(url).unwrap());
    }
}
