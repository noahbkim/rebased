// use ratatui_tree::TreeItem;
//
// use git2::{Commit, Oid, Repository};
// use git2::{DiffFile, Revwalk};
//
// use anyhow::Context;
// use ratatui::prelude::{Line, Style};
// use std::path::Path;
//
// pub fn find_commit_by_id(repo: &Repository, id: Oid) -> anyhow::Result<Commit<'_>> {
//     repo.find_commit(id)
//         .with_context(|| format!("failed to find commit {}", id))
// }
//
// pub fn find_id_by_short_name(repo: &Repository, short_name: &str) -> anyhow::Result<Oid> {
//     repo.resolve_reference_from_short_name(short_name)
//         .with_context(|| format!("failed to resolve commit {}", short_name))?
//         .target()
//         .ok_or_else(|| anyhow::format_err!("no target OID for reference {}", short_name))
// }
//
// pub fn find_merge_base<'repo>(
//     repo: &'repo Repository,
//     id1: Oid,
//     id2: Oid,
// ) -> anyhow::Result<Commit<'repo>> {
//     repo.merge_base(id1, id2)
//         .with_context(|| format!("failed to resolve merge base between {} and {}", id1, id2))?;
// }
//
// //    pub fn from_range(repo: &'repo Repository, tail: Oid, head: Oid) -> anyhow::Result<Self> {
// //         let mut stack = Self::new();
// //         let mut revwalk = repo
// //             .revwalk()
// //             .context("failed to construct a revision walk")?;
// //         revwalk
// //             .push(head)
// //             .context("failed to push {} onto the revision walk")?;
// //         revwalk
// //             .set_sorting(git2::Sort::TOPOLOGICAL)
// //             .context("failed to sort the revision walk topologically")?;
// //         for result in revwalk {
// //             let id = result.context("failed to retrieve commit from revwalk")?;
// //             if id == tail {
// //                 break;
// //             }
// //             let commit = repo
// //                 .find_commit(id)
// //                 .with_context(|| format!("failed to find commit {}", id))?;
// //             stack.commits.push(StackCommit::new(commit))
// //         }
// //         Ok(stack)
// //     }
//
// // pub fn load(repo: &Repository, tail: &str, head: &str) -> anyhow::Result<StackWalk<'_>> {
// //     let base_id =
// //     let head_id = repo
// //         .resolve_reference_from_short_name(head)
// //         .with_context(|| format!("failed to resolve destination reference {}", head))?
// //         .target()
// //         .ok_or(anyhow::format_err!(
// //             "no target OID for destination {}",
// //             head
// //         ))?;
//
// //     Ok(StackWalk {
// //         revwalk,
// //         merge_base_id,
// //     })
// // }
