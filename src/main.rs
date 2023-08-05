use git2::Repository;
use std::env;

fn main() {
    println!("Hello, world!");
    // get current directory
    let current_dir = env::current_dir().unwrap();
    let repo = match Repository::open(current_dir) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };
    println!("Repo: {:?}", repo.path());

    repo.remotes().unwrap().iter().for_each(|remote| {
        println!("Remote: {:?}", remote);
        repo.find_remote(remote.unwrap()).unwrap().url().unwrap();
    });
}


fn remote_to_url(remote: &git2::Remote) -> String {
    let url = remote.url().unwrap();
    // let url = url.replace("
    url.to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic() {
        
        // origin = ssh://git@github.com/takac/git-open
        // origin = git@github.com:takac/git-open.git
        // origin = https://github.com/takac/git-open.git
        
// git_open_test "" "https://github.com/takac/git-open"
// git_open_test "-v" "https://github.com/takac/git-open"
// git_open_test "-b master" "https://github.com/takac/git-open/tree/master"
// git_open_test "-b bob" "https://github.com/takac/git-open/tree/bob"
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn another() {
        // panic!("Make this test fail");
    }
}
