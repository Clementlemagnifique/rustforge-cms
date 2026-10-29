//! # Module LMS (Learning Management System) Natif
//!
//! Inspiré de Sensei LMS mais entièrement codé en Rust avec PostgreSQL :
//! - Cours, modules, leçons vidéo / texte, prévisualisation gratuite
//! - Inscription des apprenants et suivi de progression
//! - Liaison bidirectionnelle native avec le module E-Commerce (EDD)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{CoreError, CoreResult, slugify};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Course {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub content: String,
    pub featured_media_id: Option<Uuid>,
    pub price_cents: i32,
    pub is_published: bool,
    pub created_by: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseInput {
    pub title: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub featured_media_id: Option<Uuid>,
    pub price_cents: Option<i32>,
    pub is_published: Option<bool>,
    pub product_id: Option<Uuid>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CourseModule {
    pub id: Uuid,
    pub course_id: Uuid,
    pub title: String,
    pub description: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInput {
    pub title: String,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Lesson {
    pub id: Uuid,
    pub course_id: Uuid,
    pub module_id: Option<Uuid>,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub video_url: Option<String>,
    pub duration_min: i32,
    pub is_preview: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonInput {
    pub module_id: Option<Uuid>,
    pub title: String,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub video_url: Option<String>,
    pub duration_min: Option<i32>,
    pub is_preview: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Enrollment {
    pub id: Uuid,
    pub course_id: Uuid,
    pub user_id: Uuid,
    pub progress_pct: i32,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Cours
// ---------------------------------------------------------------------------

pub async fn list_courses(db: &PgPool, published_only: bool) -> CoreResult<Vec<Course>> {
    let sql = if published_only {
        "SELECT * FROM lms_courses WHERE is_published = true ORDER BY created_at DESC"
    } else {
        "SELECT * FROM lms_courses ORDER BY created_at DESC"
    };
    Ok(sqlx::query_as::<_, Course>(sql).fetch_all(db).await?)
}

pub async fn get_course(db: &PgPool, id: Uuid) -> CoreResult<Course> {
    sqlx::query_as::<_, Course>("SELECT * FROM lms_courses WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or(CoreError::NotFound)
}

pub async fn get_course_by_slug(db: &PgPool, slug: &str) -> CoreResult<Course> {
    sqlx::query_as::<_, Course>("SELECT * FROM lms_courses WHERE slug = $1")
        .bind(slug)
        .fetch_optional(db)
        .await?
        .ok_or(CoreError::NotFound)
}

pub async fn create_course(db: &PgPool, input: CourseInput) -> CoreResult<Course> {
    let slug = input
        .slug
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| slugify(&input.title));
    let desc = input.description.unwrap_or_default();
    let content = input.content.unwrap_or_default();
    let price = input.price_cents.unwrap_or(0);
    let is_pub = input.is_published.unwrap_or(false);

    let course = sqlx::query_as::<_, Course>(
        r#"
        INSERT INTO lms_courses (title, slug, description, content, featured_media_id, price_cents, is_published, product_id, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
        "#
    )
    .bind(input.title)
    .bind(slug)
    .bind(desc)
    .bind(content)
    .bind(input.featured_media_id)
    .bind(price)
    .bind(is_pub)
    .bind(input.product_id)
    .bind(input.created_by)
    .fetch_one(db)
    .await?;

    Ok(course)
}

pub async fn update_course(db: &PgPool, id: Uuid, input: CourseInput) -> CoreResult<Course> {
    let current = get_course(db, id).await?;
    let slug = input
        .slug
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(current.slug);
    let desc = input.description.unwrap_or(current.description);
    let content = input.content.unwrap_or(current.content);
    let price = input.price_cents.unwrap_or(current.price_cents);
    let is_pub = input.is_published.unwrap_or(current.is_published);
    let feat = input.featured_media_id.or(current.featured_media_id);
    let prod = input.product_id.or(current.product_id);

    let updated = sqlx::query_as::<_, Course>(
        r#"
        UPDATE lms_courses
        SET title = $2, slug = $3, description = $4, content = $5,
            featured_media_id = $6, price_cents = $7, is_published = $8,
            product_id = $9, updated_at = now()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(input.title)
    .bind(slug)
    .bind(desc)
    .bind(content)
    .bind(feat)
    .bind(price)
    .bind(is_pub)
    .bind(prod)
    .fetch_one(db)
    .await?;

    Ok(updated)
}

pub async fn delete_course(db: &PgPool, id: Uuid) -> CoreResult<()> {
    sqlx::query("DELETE FROM lms_courses WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Modules & Leçons
// ---------------------------------------------------------------------------

pub async fn list_course_modules(db: &PgPool, course_id: Uuid) -> CoreResult<Vec<CourseModule>> {
    Ok(sqlx::query_as::<_, CourseModule>(
        "SELECT * FROM lms_modules WHERE course_id = $1 ORDER BY sort_order ASC, created_at ASC",
    )
    .bind(course_id)
    .fetch_all(db)
    .await?)
}

pub async fn create_module(
    db: &PgPool,
    course_id: Uuid,
    input: ModuleInput,
) -> CoreResult<CourseModule> {
    let desc = input.description.unwrap_or_default();
    let sort = input.sort_order.unwrap_or(0);

    let m = sqlx::query_as::<_, CourseModule>(
        r#"
        INSERT INTO lms_modules (course_id, title, description, sort_order)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(course_id)
    .bind(input.title)
    .bind(desc)
    .bind(sort)
    .fetch_one(db)
    .await?;

    Ok(m)
}

pub async fn delete_module(db: &PgPool, id: Uuid) -> CoreResult<()> {
    sqlx::query("DELETE FROM lms_modules WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn list_course_lessons(db: &PgPool, course_id: Uuid) -> CoreResult<Vec<Lesson>> {
    Ok(sqlx::query_as::<_, Lesson>(
        "SELECT * FROM lms_lessons WHERE course_id = $1 ORDER BY sort_order ASC, created_at ASC",
    )
    .bind(course_id)
    .fetch_all(db)
    .await?)
}

pub async fn create_lesson(db: &PgPool, course_id: Uuid, input: LessonInput) -> CoreResult<Lesson> {
    let slug = input
        .slug
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| slugify(&input.title));
    let content = input.content.unwrap_or_default();
    let dur = input.duration_min.unwrap_or(10);
    let prev = input.is_preview.unwrap_or(false);
    let sort = input.sort_order.unwrap_or(0);

    let lesson = sqlx::query_as::<_, Lesson>(
        r#"
        INSERT INTO lms_lessons (course_id, module_id, title, slug, content, video_url, duration_min, is_preview, sort_order)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
        "#
    )
    .bind(course_id)
    .bind(input.module_id)
    .bind(input.title)
    .bind(slug)
    .bind(content)
    .bind(input.video_url)
    .bind(dur)
    .bind(prev)
    .bind(sort)
    .fetch_one(db)
    .await?;

    Ok(lesson)
}

pub async fn update_lesson(
    db: &PgPool,
    id: Uuid,
    input: LessonInput,
) -> CoreResult<Lesson> {
    let current = sqlx::query_as::<_, Lesson>("SELECT * FROM lms_lessons WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or(CoreError::NotFound)?;

    let slug = input
        .slug
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(current.slug);
    let content = input.content.unwrap_or(current.content);
    let dur = input.duration_min.unwrap_or(current.duration_min);
    let prev = input.is_preview.unwrap_or(current.is_preview);
    let sort = input.sort_order.unwrap_or(current.sort_order);
    let mod_id = input.module_id.or(current.module_id);

    let updated = sqlx::query_as::<_, Lesson>(
        r#"
        UPDATE lms_lessons
        SET title = $2, slug = $3, content = $4, video_url = $5,
            duration_min = $6, is_preview = $7, sort_order = $8,
            module_id = $9, updated_at = now()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(input.title)
    .bind(slug)
    .bind(content)
    .bind(input.video_url.or(current.video_url))
    .bind(dur)
    .bind(prev)
    .bind(sort)
    .bind(mod_id)
    .fetch_one(db)
    .await?;

    Ok(updated)
}

pub async fn delete_lesson(db: &PgPool, id: Uuid) -> CoreResult<()> {
    sqlx::query("DELETE FROM lms_lessons WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Inscriptions & Progression des apprenants
// ---------------------------------------------------------------------------

pub async fn enroll_user(db: &PgPool, course_id: Uuid, user_id: Uuid) -> CoreResult<Enrollment> {
    let enrollment = sqlx::query_as::<_, Enrollment>(
        r#"
        INSERT INTO lms_enrollments (course_id, user_id, progress_pct)
        VALUES ($1, $2, 0)
        ON CONFLICT (course_id, user_id) DO UPDATE SET progress_pct = lms_enrollments.progress_pct
        RETURNING *
        "#,
    )
    .bind(course_id)
    .bind(user_id)
    .fetch_one(db)
    .await?;

    Ok(enrollment)
}

pub async fn is_user_enrolled(db: &PgPool, course_id: Uuid, user_id: Uuid) -> CoreResult<bool> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM lms_enrollments WHERE course_id = $1 AND user_id = $2)",
    )
    .bind(course_id)
    .bind(user_id)
    .fetch_one(db)
    .await?;

    Ok(exists)
}

pub async fn get_user_enrollments(db: &PgPool, user_id: Uuid) -> CoreResult<Vec<Course>> {
    Ok(sqlx::query_as::<_, Course>(
        r#"
        SELECT c.* FROM lms_courses c
        JOIN lms_enrollments e ON e.course_id = c.id
        WHERE e.user_id = $1
        ORDER BY e.created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await?)
}

pub async fn update_progress(
    db: &PgPool,
    course_id: Uuid,
    user_id: Uuid,
    progress_pct: i32,
) -> CoreResult<()> {
    let progress = progress_pct.clamp(0, 100);
    let completed = if progress >= 100 {
        Some(Utc::now())
    } else {
        None
    };

    sqlx::query(
        r#"
        UPDATE lms_enrollments
        SET progress_pct = $3, completed_at = COALESCE($4, completed_at)
        WHERE course_id = $1 AND user_id = $2
        "#,
    )
    .bind(course_id)
    .bind(user_id)
    .bind(progress)
    .bind(completed)
    .execute(db)
    .await?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EnrollmentDetails {
    pub id: Uuid,
    pub course_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub progress_pct: i32,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub async fn list_course_enrollments(
    db: &PgPool,
    course_id: Uuid,
) -> CoreResult<Vec<EnrollmentDetails>> {
    Ok(sqlx::query_as::<_, EnrollmentDetails>(
        r#"
        SELECT e.id, e.course_id, e.user_id, u.username, u.email, u.display_name,
               e.progress_pct, e.completed_at, e.created_at
        FROM lms_enrollments e
        JOIN users u ON u.id = e.user_id
        WHERE e.course_id = $1
        ORDER BY e.created_at DESC
        "#,
    )
    .bind(course_id)
    .fetch_all(db)
    .await?)
}

pub async fn list_all_enrollments(
    db: &PgPool,
    limit: i64,
) -> CoreResult<Vec<EnrollmentDetails>> {
    Ok(sqlx::query_as::<_, EnrollmentDetails>(
        r#"
        SELECT e.id, e.course_id, e.user_id, u.username, u.email, u.display_name,
               e.progress_pct, e.completed_at, e.created_at
        FROM lms_enrollments e
        JOIN users u ON u.id = e.user_id
        ORDER BY e.created_at DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(db)
    .await?)
}
