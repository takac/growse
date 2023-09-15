use crate::*;

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
        let branch_name = format!("refs/heads/{}", self.state.branch);
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
