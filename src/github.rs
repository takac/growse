use crate::*;

impl Repo for GitHub {
    fn is_host(&self) -> bool {
        self.url.host.as_ref().unwrap().contains("github")
    }

    fn to_repo_url_with_path_and_branch(&self) -> Result<String, Box<dyn std::error::Error>> {
        return self.to_repo_url_with_path();
    }

    fn to_repo_url_with_path_and_branch_and_line_number(
        &self,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = self.to_repo_url_with_path_and_branch().unwrap();
        let line_number = self.state.line_number.unwrap();

        Ok(format!("{url}#L{line_number}"))
    }

    fn to_repo_url_with_path_and_line_number(&self) -> Result<String, Box<dyn std::error::Error>> {
        let url = self.to_repo_url_with_path().unwrap();
        let line_number = self.state.line_number.unwrap();

        Ok(format!("{url}#L{line_number}"))
    }

    fn to_repo_url_with_path(&self) -> Result<String, Box<dyn std::error::Error>> {
        let branch = &self.state.branch;
        let fullname = &self.url.fullname;
        let host = self.url.host.clone().ok_or("No host found")?;
        let path = self.state.path.clone().ok_or("No path found")?;

        Ok(format!("https://{host}/{fullname}/blob/{branch}/{path}"))
    }

    fn to_repo_url_with_branch(&self) -> Result<String, Box<dyn std::error::Error>> {
        let host = self.url.host.clone().ok_or("No host found")?;
        let branch = &self.state.branch;
        let fullname = &self.url.fullname;

        Ok(format!("https://{host}/{fullname}/tree/{branch}"))
    }

    fn to_repo_url(&self) -> Result<String, Box<dyn std::error::Error>> {
        let host = self.url.host.clone().ok_or("No host found")?;
        let fullname = &self.url.fullname;

        Ok(format!("https://{host}/{fullname}"))
    }
}
