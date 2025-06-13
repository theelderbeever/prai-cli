use std::process::Command;

use lazy_static::lazy_static;

pub fn get_default_branch() -> &'static str {
    lazy_static! {
        static ref DEFAULT_BRANCH: String = Command::new("git")
            .args(["remote", "show", "origin"])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    return parse_default_branch(&o.stdout);
                }
                None
            })
            .unwrap_or(String::from("HEAD"));
    }

    &DEFAULT_BRANCH
}

fn parse_default_branch(stdout: &[u8]) -> Option<String> {
    String::from_utf8_lossy(stdout)
        .lines()
        .find(|l| l.trim().starts_with("HEAD branch: "))
        .and_then(|l| l.split_once(": "))
        .and_then(|(_, branch)| match branch.trim() {
            "" => None,
            branch => Some(String::from(branch)),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_branch() {
        let input_str = indoc! {
            r#"* remote origin
              Fetch URL: git@github.com:theelderbeever/prai-cli.git
              Push  URL: git@github.com:theelderbeever/prai-cli.git
              HEAD branch: main
              Remote branches:
                1-add-configuration-and-provider-support                                 tracked
                main                                                                     tracked
                refs/remotes/origin/5-update-dependencies                                stale (use 'git remote prune' to remove)
                refs/remotes/origin/7-dont-exclude-other-languages-from-the-default-role stale (use 'git remote prune' to remove)
              Local branches configured for 'git pull':
                1-add-configuration-and-provider-support merges with remote 1-add-configuration-and-provider-support
                main                                     merges with remote main
              Local refs configured for 'git push':
                1-add-configuration-and-provider-support pushes to 1-add-configuration-and-provider-support (up to date)
                main                                     pushes to main                                     (local out of date)
            "#
        };

        let branch = parse_default_branch(input_str.as_bytes());

        assert_eq!(Some("main"), branch.as_deref());
    }

    #[test]
    fn test_default_branch() {
        let branch = get_default_branch();

        assert_eq!("main", branch);
    }
}
