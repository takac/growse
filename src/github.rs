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
        Ok(format!(
            "{}#L{}",
            self.to_repo_url_with_path_and_branch().unwrap(),
            self.state.line_number.unwrap()
        ))
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
        let branch = &self.state.branch;

        let path = if self.state.current_dir == self.state.repo_dir {
            self.state.path.clone().ok_or("No path found")?
        } else {
            let repo_dir = &self.state.repo_dir;
            let current_dir = &self.state.current_dir;
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
            host, self.url.fullname, self.state.branch
        ))
    }

    fn to_repo_url(&self) -> Result<String, Box<dyn std::error::Error>> {
        let host = self.url.host.clone().ok_or("No host found")?;
        Ok(format!("https://{}/{}", host, self.url.fullname))
    }
}
