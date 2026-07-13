// Integration tests for the history persistence layer.

use std::path::PathBuf;
use tempfile::TempDir;

use terax_lib::modules::history::db::Db;

fn tmp_db() -> (TempDir, Db) {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("history.db");
    let db = Db::open(&path).expect("open db");
    (dir, db)
}

#[test]
fn data_survives_reopen() {
    let dir = TempDir::new().unwrap();
    let path: PathBuf = dir.path().join("history.db");
    {
        let db = Db::open(&path).unwrap();
        db.insert("git status", 1000, Some(0), "s1").unwrap();
        db.insert("cargo build", 2000, Some(0), "s1").unwrap();
    }
    let db2 = Db::open(&path).unwrap();
    let rows = db2.load_all().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].0, "git status");
    assert_eq!(rows[1].0, "cargo build");
}

#[test]
fn trim_after_bulk_insert() {
    let (_dir, db) = tmp_db();
    let entries: Vec<(String, i64)> = (0i64..200)
        .map(|i| (format!("cmd-{i}"), i))
        .collect();
    db.seed(&entries).unwrap();
    assert_eq!(db.load_all().unwrap().len(), 200);
    db.trim(100).unwrap();
    assert_eq!(db.load_all().unwrap().len(), 100);
}

#[test]
fn clear_then_reuse() {
    let (_dir, db) = tmp_db();
    db.insert("before", 1, None, "s").unwrap();
    db.clear().unwrap();
    db.insert("after", 2, None, "s").unwrap();
    let rows = db.load_all().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, "after");
}

#[test]
fn list_respects_offset() {
    let (_dir, db) = tmp_db();
    for i in 0i64..10 {
        db.insert(&format!("cmd{i}"), i * 100, None, "s").unwrap();
    }
    let page_a = db.list("", 3, 0).unwrap();
    let page_b = db.list("", 3, 3).unwrap();
    let ids_a: Vec<i64> = page_a.iter().map(|e| e.id).collect();
    let ids_b: Vec<i64> = page_b.iter().map(|e| e.id).collect();
    for id in &ids_b {
        assert!(!ids_a.contains(id), "pages must not overlap");
    }
}
