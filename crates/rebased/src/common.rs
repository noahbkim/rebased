use anyhow::Context;
use git2::Commit;
use git2::Diff;
use git2::Repository;

pub fn diff<'repo>(
    repository: &'repo Repository,
    commit: &Commit<'repo>,
) -> anyhow::Result<Diff<'repo>> {
    if commit.parent_count() != 1 {
        return Err(anyhow::format_err!(
            "commit {} has {} parents",
            commit.id(),
            commit.parent_count()
        ));
    }

    Ok(repository.diff_tree_to_tree(
        Some(
            &commit
                .parent(0)
                .context("failed to retrieve commit parent")?
                .tree()
                .context("failed to retrieve commit tree")?,
        ),
        Some(&commit.tree().context("failed to retrieve commit tree")?),
        None,
    )?)
}
