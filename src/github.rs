use crate::*;

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
