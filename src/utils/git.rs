use git2::TreeWalkMode;
use git2::{DiffFormat, DiffOptions, Repository};
use std::env;
use std::path::Path;

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

pub(crate) fn get_staged_files() -> Vec<String> {
    let repo = get_repo().expect("Opening repo error get_git_diff");

    let mut path_list = vec![];
    let mut diff_opts = DiffOptions::new();
    let old_tree = repo.head().unwrap().peel_to_tree().unwrap();

    let staged_diff = repo
        .diff_tree_to_index(
            Some(&old_tree),
            Some(&repo.index().unwrap()),
            Some(&mut diff_opts),
        )
        .unwrap();

    for diff in staged_diff.deltas().into_iter() {
        path_list.push(
            diff.new_file()
                .path()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        );
    }
    path_list
}

pub(crate) fn get_file_diff(path: String)  -> Result<String, Box<dyn std::error::Error>>{
    let repo = get_repo().expect("Opening repo error get_git_diff");

    let mut diff_opts = DiffOptions::new();
    diff_opts
        .patience(true)
        .minimal(true)
        .include_ignored(false)
        .include_untracked(false)
        .ignore_whitespace_eol(false)
        .pathspec(path.clone());
    let old_tree = repo.head()?.peel_to_tree()?;

    let mut diff_data: Vec<String> = vec![];
    repo.diff_tree_to_index(
        Some(&old_tree),
        Some(&repo.index()?),
        Some(&mut diff_opts),
    )?
    .print(DiffFormat::Patch, |_d, _h, l| {
        let content = str::from_utf8(l.content())
            .expect("Content is not utf-8")
            .to_string();
        diff_data.push(format!("{}:{}", l.origin(), content));
        true
    })?;

    Ok(diff_data.join(""))
    
}
