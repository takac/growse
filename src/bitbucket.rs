use crate::*;

impl Repo for BitBucket {
    fn is_host(&self) -> bool {
        self.url.host.as_ref().unwrap().contains("bitbucket") || self.url.port == Some(7999)
    }

    fn to_repo_url_with_path_and_branch(&self) -> Result<String, Box<dyn std::error::Error>> {
        let host = self.url.host.clone().ok_or("No host found")?;
        let owner = self.url.owner.clone().ok_or("No owner found")?;

        let branch_name = format!("refs/heads/{}", self.state.branch);
        let name = self.url.name.clone();
        let path = self.state.path.clone().ok_or("No path found")?;

        let new_url = Url::parse_with_params(
            &format!("https://{host}/projects/{owner}/repos/{name}/browse/{path}"),
            &[("at", branch_name.as_str())],
        )?;

        Ok(new_url.to_string())
    }

    fn to_repo_url_with_path(&self) -> Result<String, Box<dyn std::error::Error>> {
        let repo_url = self.to_repo_url()?;
        let path = self.state.path.clone().ok_or("No path found")?;
        Ok(format!("{repo_url}/browse/{path}"))
    }

    fn to_repo_url_with_path_and_branch_and_line_number(
        &self,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let host = self.url.host.clone().ok_or("No host found")?;
        let owner = self.url.owner.clone().ok_or("No owner found")?;

        let branch_name = format!("refs/heads/{}", self.state.branch);
        let name = self.url.name.clone();
        let path = self.state.path.clone().ok_or("No path found")?;
        let line_number = self.state.line_number.ok_or("No line number found")?;

        let mut new_url = Url::parse_with_params(
            &format!("https://{host}/projects/{owner}/repos/{name}/browse/{path}"),
            &[("at", branch_name.as_str())],
        )?;
        new_url.set_fragment(Some(line_number.to_string().as_str()));

        Ok(new_url.to_string())
    }

    fn to_repo_url_with_path_and_line_number(&self) -> Result<String, Box<dyn std::error::Error>> {
        let host = self.url.host.clone().ok_or("No host found")?;
        let owner = self.url.owner.clone().ok_or("No owner found")?;

        let name = self.url.name.clone();
        let path = self.state.path.clone().ok_or("No path found")?;
        let line_number = self.state.line_number.ok_or("No line number found")?;

        let mut new_url = Url::parse(&format!(
            "https://{host}/projects/{owner}/repos/{name}/browse/{path}"
        ))?;
        new_url.set_fragment(Some(line_number.to_string().as_str()));

        Ok(new_url.to_string())
    }

    fn to_repo_url_with_branch(&self) -> Result<String, Box<dyn std::error::Error>> {
        let host = self.url.host.clone().ok_or("No host found")?;
        let owner = self.url.owner.clone().ok_or("No owner found")?;

        let branch_name = format!("refs/heads/{}", self.state.branch);
        let name = self.url.name.clone();

        let new_url = Url::parse_with_params(
            &format!("https://{host}/projects/{owner}/repos/{name}/browse"),
            &[("at", branch_name.as_str())],
        )?;

        Ok(new_url.to_string())
    }

    fn to_repo_url(&self) -> Result<String, Box<dyn std::error::Error>> {
        let host = self.url.host.clone().ok_or("No host found")?;
        let owner = self.url.owner.clone().ok_or("No owner found")?;
        let name = self.url.name.clone();

        Ok(format!("https://{host}/projects/{owner}/repos/{name}"))
    }
}
