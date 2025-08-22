use git2::TreeWalkMode;
use git2::{DiffFormat, DiffOptions, Repository};
use std::env;
use std::path::Path;

pub(crate) fn get_git_diff() -> Vec<std::string::String> {
    let repo = get_repo().expect("Opening repo error get_git_diff");

    let mut diff_opts = DiffOptions::new();
    let old_tree = repo
        .head()
        .expect("Failed to get HEAD")
        .peel_to_tree()
        .expect("Head is not a tree");

    let mut diff_data: Vec<String> = vec![];

    repo.diff_tree_to_index(
        Some(&old_tree),
        Some(&repo.index().expect("Failed to index files")),
        Some(&mut diff_opts),
    )
    .expect("Error creating diff")
    .print(DiffFormat::Patch, |_d, _h, l| {
        let content = str::from_utf8(l.content())
            .expect("Content is not utf-8")
            .to_string();
        diff_data.push(format!("{}:{}", l.origin(), content));
        true
    })
    .expect("Error printing diff");
    return diff_data;
}

pub(crate) fn get_current_branch_name() -> String {
    let repo = get_repo().expect("Opening repo error get_current_branch_name");

    let head = repo.head().unwrap();
    head.shorthand().unwrap().to_string()
}

fn get_repo() -> Result<Repository, Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    let location = current_dir.as_path();
    return Ok(Repository::open(location)?);
}

pub(crate) fn get_project_struture() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let repo = get_repo().expect("Opening repo error get_project_struture");

    let rev = get_current_branch_name();
    let obj = repo.revparse_single(&rev)?;
    let tree = obj.peel_to_tree()?;

    let mut list_of_files = vec![];
    tree.walk(TreeWalkMode::PreOrder, |path, file| {
        let file = format!("{}{}", path, file.name().unwrap());
        if Path::new(&file).is_file() {
            list_of_files.push(file);
        }

        // Read file content via repo.odb() or checkout logic
        git2::TreeWalkResult::Ok
    })?;

    Ok(list_of_files)
}
