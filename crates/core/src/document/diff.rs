//! Document comparison and diff view functionality
//!
//! This module provides tools for comparing two documents and displaying differences.

use anyhow::{format_err, Error};
use std::path::Path;

/// Represents a change in the document
#[derive(Debug, Clone)]
pub enum DiffChange {
    /// Text was added
    Added(String),
    /// Text was removed
    Removed(String),
    /// Text was unchanged
    Unchanged(String),
}

/// Result of comparing two documents
#[derive(Debug, Clone)]
pub struct DiffResult {
    /// List of changes in order
    pub changes: Vec<DiffChange>,
    /// Number of added lines
    pub added_count: usize,
    /// Number of removed lines
    pub removed_count: usize,
    /// Number of unchanged lines
    pub unchanged_count: usize,
}

/// Compare two text documents and return their differences
pub fn compare_documents(file1: &Path, file2: &Path) -> Result<DiffResult, Error> {
    let text1 = std::fs::read_to_string(file1)
        .map_err(|e| format_err!("Failed to read file {:?}: {}", file1, e))?;
    let text2 = std::fs::read_to_string(file2)
        .map_err(|e| format_err!("Failed to read file {:?}: {}", file2, e))?;

    let lines1: Vec<&str> = text1.lines().collect();
    let lines2: Vec<&str> = text2.lines().collect();

    let changes = compute_diff(&lines1, &lines2);

    let added_count = changes
        .iter()
        .filter(|c| matches!(c, DiffChange::Added(_)))
        .count();
    let removed_count = changes
        .iter()
        .filter(|c| matches!(c, DiffChange::Removed(_)))
        .count();
    let unchanged_count = changes
        .iter()
        .filter(|c| matches!(c, DiffChange::Unchanged(_)))
        .count();

    Ok(DiffResult {
        changes,
        added_count,
        removed_count,
        unchanged_count,
    })
}

/// Compute diff between two line arrays using a simple line-by-line comparison
fn compute_diff(lines1: &[&str], lines2: &[&str]) -> Vec<DiffChange> {
    let mut changes = Vec::new();
    let mut i = 0;
    let mut j = 0;

    while i < lines1.len() || j < lines2.len() {
        if i < lines1.len() && j < lines2.len() {
            if lines1[i] == lines2[j] {
                // Lines match
                changes.push(DiffChange::Unchanged(lines1[i].to_string()));
                i += 1;
                j += 1;
            } else {
                // Lines differ - try to find matches ahead
                let (found_in_2, offset_2) = find_match(lines1[i], &lines2[j..]);
                let (found_in_1, offset_1) = find_match(lines2[j], &lines1[i..]);

                if found_in_2 && (!found_in_1 || offset_2 <= offset_1) {
                    // Remove lines from first until match
                    for k in 0..offset_2 {
                        changes.push(DiffChange::Removed(lines1[i + k].to_string()));
                    }
                    i += offset_2;
                } else if found_in_1 {
                    // Add lines from second until match
                    for k in 0..offset_1 {
                        changes.push(DiffChange::Added(lines2[j + k].to_string()));
                    }
                    j += offset_1;
                } else {
                    // No match found - treat as replacement
                    changes.push(DiffChange::Removed(lines1[i].to_string()));
                    changes.push(DiffChange::Added(lines2[j].to_string()));
                    i += 1;
                    j += 1;
                }
            }
        } else if i < lines1.len() {
            // Remaining lines in first document
            changes.push(DiffChange::Removed(lines1[i].to_string()));
            i += 1;
        } else {
            // Remaining lines in second document
            changes.push(DiffChange::Added(lines2[j].to_string()));
            j += 1;
        }
    }

    changes
}

/// Find a line in a slice and return whether found and the offset
fn find_match(line: &str, lines: &[&str]) -> (bool, usize) {
    for (i, &l) in lines.iter().enumerate() {
        if line == l {
            return (true, i);
        }
    }
    (false, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_documents() {
        let lines1 = vec!["line1", "line2", "line3"];
        let lines2 = vec!["line1", "line2", "line3"];
        let changes = compute_diff(&lines1, &lines2);
        assert_eq!(changes.len(), 3);
        assert!(matches!(changes[0], DiffChange::Unchanged(_)));
        assert!(matches!(changes[1], DiffChange::Unchanged(_)));
        assert!(matches!(changes[2], DiffChange::Unchanged(_)));
    }

    #[test]
    fn test_added_lines() {
        let lines1 = vec!["line1", "line2"];
        let lines2 = vec!["line1", "line2", "line3"];
        let changes = compute_diff(&lines1, &lines2);
        assert!(changes
            .iter()
            .any(|c| matches!(c, DiffChange::Added(s) if s == "line3")));
    }

    #[test]
    fn test_removed_lines() {
        let lines1 = vec!["line1", "line2", "line3"];
        let lines2 = vec!["line1", "line2"];
        let changes = compute_diff(&lines1, &lines2);
        assert!(changes
            .iter()
            .any(|c| matches!(c, DiffChange::Removed(s) if s == "line3")));
    }
}
