use crate::*;

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
