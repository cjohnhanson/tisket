use std::io::IsTerminal;
use std::process::ExitCode;

use camino::Utf8PathBuf;
use clap::Parser;
use colored::Colorize;

mod cli;

use cli::{Args, Command, HooksCommand, IssueCommand, ProjectCommand};
use tisket::{CreateIssueOptions, Issue, Repo, SearchResult, git};

fn main() -> ExitCode {
    let args = Args::parse();

    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}

fn colorize_status(status: &str) -> String {
    if !std::io::stdout().is_terminal() {
        return status.to_string();
    }
    let (base, suffix) = if let Some(s) = status.strip_suffix('*') {
        (s, "*")
    } else {
        (status, "")
    };
    let colored = match base {
        "discovery" => base.cyan().to_string(),
        "in_progress" => base.blue().to_string(),
        "cancelled" => base.red().dimmed().to_string(),
        "backlog" => base.dimmed().to_string(),
        "todo" => base.yellow().to_string(),
        "done" => base.green().to_string(),
        _ => base.to_string(),
    };
    format!("{colored}{suffix}")
}

/// Print aligned columns with a header row and 2-space gaps.
fn print_columns(headers: &[&str], rows: &[Vec<String>]) {
    if rows.is_empty() {
        return;
    }

    let num_cols = headers.len();
    let mut widths = vec![0usize; num_cols];
    for (i, h) in headers.iter().enumerate() {
        widths[i] = h.len();
    }
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < num_cols {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    // Print header
    let mut header_line = String::new();
    for (i, h) in headers.iter().enumerate() {
        if i > 0 {
            header_line.push_str("  ");
        }
        if i < num_cols - 1 {
            header_line.push_str(&format!("{:<width$}", h, width = widths[i]));
        } else {
            header_line.push_str(h);
        }
    }
    println!("{header_line}");

    // Print data rows
    for row in rows {
        let mut line = String::new();
        for (i, cell) in row.iter().enumerate() {
            if i > 0 {
                line.push_str("  ");
            }
            if i < num_cols - 1 {
                // Status column gets colorized but padded to the raw width
                if headers[i] == "STATUS" {
                    let padding = widths[i].saturating_sub(cell.len());
                    line.push_str(&colorize_status(cell));
                    for _ in 0..padding {
                        line.push(' ');
                    }
                } else {
                    line.push_str(&format!("{:<width$}", cell, width = widths[i]));
                }
            } else {
                line.push_str(cell);
            }
        }
        println!("{line}");
    }
}

fn print_issue_list(issues: &[Issue]) {
    let rows: Vec<Vec<String>> = issues
        .iter()
        .map(|iss| {
            let status = if iss.diverges {
                format!("{}*", iss.frontmatter.status)
            } else {
                iss.frontmatter.status.clone()
            };
            vec![iss.id.clone(), status, iss.frontmatter.title.clone()]
        })
        .collect();
    print_columns(&["ID", "STATUS", "TITLE"], &rows);
}

fn print_search_results(results: &[SearchResult]) {
    let rows: Vec<Vec<String>> = results
        .iter()
        .map(|r| {
            vec![
                r.issue.id.clone(),
                r.issue.frontmatter.status.clone(),
                r.issue.project.clone(),
                r.issue.frontmatter.title.clone(),
                r.matched_fields.join(", "),
            ]
        })
        .collect();
    print_columns(&["ID", "STATUS", "PROJECT", "TITLE", "MATCH"], &rows);
}

fn print_issue_show(iss: &Issue) {
    println!("{} ({})", iss.id, iss.project);
    println!();
    println!("  {:<10}{}", "Title:", iss.frontmatter.title);
    println!(
        "  {:<10}{}",
        "Status:",
        colorize_status(&iss.frontmatter.status)
    );
    if let Some(p) = &iss.frontmatter.priority {
        println!("  {:<10}{p}", "Priority:");
    }
    if let Some(a) = &iss.frontmatter.assignee {
        println!("  {:<10}{a}", "Assignee:");
    }
    if !iss.frontmatter.labels.is_empty() {
        println!("  {:<10}{}", "Labels:", iss.frontmatter.labels.join(", "));
    }
    if !iss.frontmatter.depends_on.is_empty() {
        println!(
            "  {:<10}{}",
            "Depends:",
            iss.frontmatter.depends_on.join(", ")
        );
    }
    if !iss.body.is_empty() {
        println!();
        println!("{}", iss.body);
    }
    if !iss.branch_statuses.is_empty() {
        println!();
        println!("  Other branches:");
        for bs in &iss.branch_statuses {
            let diffs = git::diff_fields(&iss.frontmatter, &iss.body, &iss.scratch, bs);
            if diffs.is_empty() {
                println!("    {}", bs.branch);
            } else {
                println!("    {}   {}", bs.branch, diffs.join(", "));
            }
        }
    }
}

fn run(args: Args) -> tisket::Result<()> {
    let root = if args.root.is_relative() {
        let cwd = std::env::current_dir()?;
        Utf8PathBuf::try_from(cwd)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
            .join(&args.root)
    } else {
        args.root.clone()
    };

    match args.command {
        Command::Init => Repo::init(&root),

        Command::Prime => {
            let repo = Repo::open(&root)?;
            print!("{}", repo.prime());
            Ok(())
        }

        Command::Hooks(cmd) => match cmd {
            HooksCommand::Setup(_setup_args) => {
                todo!("hooks setup not yet implemented")
            }
        },

        Command::Search(a) => {
            let repo = Repo::open(&root)?;
            let results = repo.search(&a.pattern, a.project.as_deref())?;
            print_search_results(&results);
            Ok(())
        }

        Command::Issue(cmd) => {
            let repo = Repo::open(&root)?;
            match cmd {
                IssueCommand::Create(a) => {
                    let project = a.project.as_deref().unwrap_or("default");
                    repo.create_issue(
                        &a.title,
                        project,
                        CreateIssueOptions {
                            priority: a.priority.map(|p| p.to_string()),
                            assignee: a.assignee,
                            labels: a.labels,
                            depends_on: a.depends_on,
                            status: a.status,
                        },
                    )?;
                    Ok(())
                }
                IssueCommand::List(a) => {
                    let issues =
                        repo.list_issues(a.project.as_deref(), a.status.as_deref(), a.closed)?;
                    print_issue_list(&issues);
                    Ok(())
                }
                IssueCommand::Show(a) => {
                    let iss = repo.find_issue(&a.id)?;
                    print_issue_show(&iss);
                    Ok(())
                }
                IssueCommand::Path(a) => {
                    let path = repo.issue_path(&a.id)?;
                    println!("{path}");
                    Ok(())
                }
                IssueCommand::Edit(a) => {
                    repo.edit_issue(&a.id, a.status.as_deref())?;
                    Ok(())
                }
                IssueCommand::Close(a) => {
                    repo.close_issue(&a.id, a.status.as_deref())?;
                    Ok(())
                }
                IssueCommand::Reopen(a) => {
                    repo.reopen_issue(&a.id, a.status.as_deref())?;
                    Ok(())
                }
            }
        }

        Command::Project(cmd) => {
            let repo = Repo::open(&root)?;
            match cmd {
                ProjectCommand::Create(a) => {
                    repo.create_project(&a.name)?;
                    Ok(())
                }
                ProjectCommand::List => {
                    let projects = repo.list_projects()?;
                    for p in &projects {
                        println!("{p}");
                    }
                    Ok(())
                }
            }
        }
    }
}
