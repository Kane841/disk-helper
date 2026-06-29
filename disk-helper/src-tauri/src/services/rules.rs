use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use globset::{Glob, GlobSet, GlobSetBuilder};
use rusqlite::Connection;
use serde::Deserialize;

use crate::error::{err, AppError, ErrorCode};
use crate::models::cleanup::CleanupSuggestion;
use crate::services::index;
use crate::services::scan::engine::normalize_windows_path;

const RULES_JSON: &str = include_str!("../../rules/builtin-rules.v1.json");

#[derive(Debug, Deserialize)]
struct RulesFile {
    cleanup_rules: Vec<CleanupRuleDef>,
    danger_paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CleanupRuleDef {
    id: String,
    name: String,
    category: String,
    risk: String,
    path_globs: Vec<String>,
    min_age_days: u32,
    description: String,
    restore_hint: String,
    #[serde(default)]
    exclude_globs: Vec<String>,
}

struct CompiledRule {
    def: CleanupRuleDef,
    include: GlobSet,
    exclude: Option<GlobSet>,
}

pub struct RulesEngine {
    danger_paths: GlobSet,
    cleanup_rules: Vec<CompiledRule>,
    path_like_prefixes: Vec<String>,
}

pub struct SuggestionFilters<'a> {
    pub risk_filter: Option<&'a str>,
    pub category_filter: Option<&'a str>,
    pub path_keyword: Option<&'a str>,
    pub page: u32,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct SuggestionsCacheState {
    pub generation: String,
    pub all_items: Arc<Vec<CleanupSuggestion>>,
    pub releasable_bytes: u64,
}

pub fn invalidate_suggestions_cache(cache: &mut Option<SuggestionsCacheState>) {
    *cache = None;
}

#[derive(Debug)]
pub struct SuggestionsResult {
    pub items: Vec<CleanupSuggestion>,
    pub releasable_bytes: u64,
    pub total: u64,
}

pub struct MatchedRule {
    pub id: String,
    pub risk: String,
}

impl RulesEngine {
    pub fn load() -> Result<Self, AppError> {
        let raw: RulesFile = serde_json::from_str(RULES_JSON).map_err(|error| {
            err(
                ErrorCode::InternalError,
                format!("invalid rules json: {error}"),
            )
            .with_target("rules")
        })?;

        let username = std::env::var("USERNAME").unwrap_or_else(|_| "{user}".into());

        let mut danger_builder = GlobSetBuilder::new();
        for pattern in expand_patterns(&raw.danger_paths, &username) {
            danger_builder.add(Glob::new(&pattern).map_err(map_glob_err)?);
        }
        let danger_paths = danger_builder.build().map_err(map_glob_err)?;

        let mut cleanup_rules = Vec::new();
        let mut path_like_prefixes = Vec::new();
        for def in raw.cleanup_rules {
            for pattern in expand_patterns(&def.path_globs, &username) {
                if let Some(prefix) = glob_to_like_prefix(&pattern) {
                    path_like_prefixes.push(prefix);
                }
            }
            let include = build_glob_set(&def.path_globs, &username)?;
            let exclude = if def.exclude_globs.is_empty() {
                None
            } else {
                Some(build_glob_set(&def.exclude_globs, &username)?)
            };
            cleanup_rules.push(CompiledRule {
                def,
                include,
                exclude,
            });
        }
        for pattern in expand_patterns(&raw.danger_paths, &username) {
            if let Some(prefix) = glob_to_like_prefix(&pattern) {
                path_like_prefixes.push(prefix);
            }
        }
        path_like_prefixes.sort();
        path_like_prefixes.dedup();

        Ok(Self {
            danger_paths,
            cleanup_rules,
            path_like_prefixes,
        })
    }

    pub fn match_path_info(&self, path: &str) -> Option<MatchedRule> {
        self.match_rule(path).map(|rule| MatchedRule {
            id: rule.id.clone(),
            risk: rule.risk.clone(),
        })
    }

    pub fn match_path(&self, path: &str) -> Option<&CleanupRuleDef> {
        self.match_rule(path)
    }

    fn match_rule(&self, path: &str) -> Option<&CleanupRuleDef> {
        let normalized = normalize_path(path);

        if self.danger_paths.is_match(&normalized) {
            return self
                .cleanup_rules
                .iter()
                .find(|rule| rule.def.id == "windows_dir")
                .map(|rule| &rule.def);
        }

        for rule in &self.cleanup_rules {
            if rule.include.is_match(&normalized) {
                if let Some(exclude) = &rule.exclude {
                    if exclude.is_match(&normalized) {
                        continue;
                    }
                }
                return Some(&rule.def);
            }
        }

        None
    }
}

pub fn get_suggestions(
    conn: &Connection,
    filters: SuggestionFilters<'_>,
    cache: &mut Option<SuggestionsCacheState>,
) -> Result<SuggestionsResult, AppError> {
    if !index::index_ready(conn)? {
        return Err(
            err(
                ErrorCode::IndexNotReady,
                "索引尚未就绪，请先完成一次全盘扫描",
            )
            .with_target("rules"),
        );
    }

    let generation = index_generation(conn)?;
    let cached = get_or_build_cache(conn, cache, &generation)?;
    let filtered: Vec<CleanupSuggestion> = cached
        .all_items
        .iter()
        .filter(|item| passes_filters(item, &filters))
        .cloned()
        .collect();

    let total = filtered.len() as u64;
    let page = filters.page.max(1);
    let size = filters.size.clamp(1, 500);
    let start = ((page - 1) as usize).saturating_mul(size as usize);
    let releasable_bytes: u64 = filtered
        .iter()
        .filter(|item| item.risk == "safe")
        .map(|item| item.size_bytes)
        .sum();
    let items = filtered
        .into_iter()
        .skip(start)
        .take(size as usize)
        .collect();

    Ok(SuggestionsResult {
        items,
        releasable_bytes,
        total,
    })
}

fn index_generation(conn: &Connection) -> Result<String, AppError> {
    let last = crate::services::scan::last_completed_at(conn)?.unwrap_or_default();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM file_entry", [], |row| row.get(0))
        .map_err(map_sqlite_err)?;
    Ok(format!("{last}:{count}"))
}

fn get_or_build_cache(
    conn: &Connection,
    cache: &mut Option<SuggestionsCacheState>,
    generation: &str,
) -> Result<SuggestionsCacheState, AppError> {
    if let Some(cached) = cache {
        if cached.generation == generation {
            return Ok(cached.clone());
        }
    }

    let engine = RulesEngine::load()?;
    let all_items = collect_suggestions(conn, &engine)?;
    let releasable_bytes = all_items
        .iter()
        .filter(|item| item.risk == "safe")
        .map(|item| item.size_bytes)
        .sum();
    let next = SuggestionsCacheState {
        generation: generation.into(),
        all_items: Arc::new(all_items),
        releasable_bytes,
    };
    *cache = Some(next.clone());
    Ok(next)
}

fn collect_suggestions(
    conn: &Connection,
    engine: &RulesEngine,
) -> Result<Vec<CleanupSuggestion>, AppError> {
    let prefixes = &engine.path_like_prefixes;
    if prefixes.is_empty() {
        return Ok(Vec::new());
    }

    let conditions: Vec<String> = (1..=prefixes.len())
        .map(|index| format!("path LIKE ?{index}"))
        .collect();
    let sql = format!(
        "SELECT path, is_dir, size_bytes, folder_size, modified_at
         FROM file_entry
         WHERE {}",
        conditions.join(" OR ")
    );

    let mut stmt = conn.prepare(&sql).map_err(map_sqlite_err)?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(prefixes.iter()), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)? != 0,
                row.get::<_, i64>(2)? as u64,
                row.get::<_, i64>(3)? as u64,
                row.get::<_, Option<String>>(4)?,
            ))
        })
        .map_err(map_sqlite_err)?;

    let mut all_items = Vec::new();
    for row in rows {
        let (path, is_dir, size_bytes, folder_size, modified_at) = row.map_err(map_sqlite_err)?;
        let Some(rule) = engine.match_rule(&path) else {
            continue;
        };
        if !age_ok(rule.min_age_days, modified_at.as_deref()) {
            continue;
        }

        let effective_size = if is_dir { folder_size } else { size_bytes };
        if effective_size == 0 && !is_dir {
            continue;
        }

        all_items.push(CleanupSuggestion {
            path: path.clone(),
            is_dir,
            size_bytes: effective_size,
            risk: rule.risk.clone(),
            category: rule.category.clone(),
            rule_id: rule.id.clone(),
            description: rule.description.clone(),
            restore_hint: rule.restore_hint.clone(),
            last_modified: modified_at,
        });
    }

    all_items.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    Ok(all_items)
}

fn passes_filters(item: &CleanupSuggestion, filters: &SuggestionFilters<'_>) -> bool {
    if let Some(risk) = filters.risk_filter {
        if risk != "all" && item.risk != risk {
            return false;
        }
    }
    if let Some(category) = filters.category_filter {
        if category != "all" && item.category != category {
            return false;
        }
    }
    if let Some(keyword) = filters.path_keyword {
        let keyword = keyword.trim().to_ascii_lowercase();
        if !keyword.is_empty() {
            let haystack = item.path.to_ascii_lowercase();
            if !haystack.contains(&keyword) {
                return false;
            }
        }
    }
    true
}

fn age_ok(min_age_days: u32, modified_at: Option<&str>) -> bool {
    if min_age_days == 0 {
        return true;
    }
    let Some(raw) = modified_at else {
        return false;
    };
    let Ok(parsed) = DateTime::parse_from_rfc3339(raw) else {
        return false;
    };
    let modified = parsed.with_timezone(&Utc);
    Utc::now().signed_duration_since(modified) >= Duration::days(min_age_days as i64)
}

fn build_glob_set(patterns: &[String], username: &str) -> Result<GlobSet, AppError> {
    let mut builder = GlobSetBuilder::new();
    for pattern in expand_patterns(patterns, username) {
        builder.add(Glob::new(&pattern).map_err(map_glob_err)?);
    }
    builder.build().map_err(map_glob_err)
}

fn expand_patterns(patterns: &[String], username: &str) -> Vec<String> {
    patterns
        .iter()
        .map(|pattern| {
            pattern
                .replace("{user}", username)
                .replace('/', "\\")
        })
        .collect()
}

fn glob_to_like_prefix(pattern: &str) -> Option<String> {
    let normalized = pattern.replace('/', "\\");
    let star_idx = normalized.find('*');
    let fixed = match star_idx {
        Some(0) => return None,
        Some(index) => &normalized[..index],
        None => normalized.as_str(),
    };
    if fixed.len() < 4 {
        return None;
    }
    Some(format!("{fixed}%"))
}

fn normalize_path(path: &str) -> String {
    normalize_windows_path(path)
}

fn map_glob_err(error: globset::Error) -> AppError {
    err(
        ErrorCode::InternalError,
        format!("invalid glob pattern: {error}"),
    )
    .with_target("rules")
}

fn map_sqlite_err(error: rusqlite::Error) -> AppError {
    err(
        ErrorCode::InternalError,
        format!("database error: {error}"),
    )
    .with_target("rules")
}

pub fn rules_file_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("rules/builtin-rules.v1.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_engine_loads_builtin_file() {
        RulesEngine::load().expect("load rules");
    }

    #[test]
    fn suggestions_require_index() {
        let temp = tempfile::tempdir().expect("temp dir");
        let conn = crate::db::open(temp.path()).expect("db");
        let result = get_suggestions(
            &conn,
            SuggestionFilters {
                risk_filter: None,
                category_filter: None,
                path_keyword: None,
                page: 1,
                size: 100,
            },
            &mut None,
        );
        let err = result.expect_err("index not ready");
        assert_eq!(err.code, ErrorCode::IndexNotReady);
    }

    #[test]
    fn match_user_temp_rule() {
        let engine = RulesEngine::load().expect("load");
        let user = std::env::var("USERNAME").unwrap_or_else(|_| "TestUser".into());
        let path = format!(r"C:\Users\{user}\AppData\Local\Temp\cache.dat");
        let rule = engine.match_path_info(&path).expect("matched");
        assert_eq!(rule.id, "user_temp");
        assert_eq!(rule.risk, "safe");
    }

    #[test]
    fn windows_temp_excluded_from_windows_dir_rule() {
        let engine = RulesEngine::load().expect("load");
        let path = r"C:\Windows\Temp\foo.tmp";
        let rule = engine.match_path_info(path).expect("matched");
        assert_eq!(rule.id, "win_temp");
    }
}
