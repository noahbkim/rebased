use git2::Repository;
use std::path::Path;

pub struct Engine {
    repository: Repository
}

impl Engine {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }

    pub fn open<P: AsRef<Path>>(repository_path: P) -> Result<Self, git2::Error> {
        Repository::open(repository_path).map(Self::new)
    }    
}
