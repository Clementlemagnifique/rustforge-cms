use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    state::AppState,
};
use rustforge_core::lms::{
    self, Course, CourseInput, CourseModule, Lesson, LessonInput, ModuleInput,
};

async fn manager(state: &AppState, auth: &AuthUser, id: Uuid) -> ApiResult<()> {
    let course = lms::get_course(&state.db, id).await?;
    if !auth.user.role.can_write()
        || (!auth.user.role.can_edit_others() && course.created_by != Some(auth.user.id))
    {
        return Err(ApiError::Forbidden);
    }
    Ok(())
}
async fn child_in(state: &AppState, table: &str, id: Uuid, course: Uuid) -> ApiResult<()> {
    let sql = match table {
        "lms_lessons" => "SELECT EXISTS(SELECT 1 FROM lms_lessons WHERE id=$1 AND course_id=$2)",
        "lms_modules" => "SELECT EXISTS(SELECT 1 FROM lms_modules WHERE id=$1 AND course_id=$2)",
        _ => return Err(ApiError::NotFound),
    };
    let exists = sqlx::query_scalar::<_, bool>(sql)
        .bind(id)
        .bind(course)
        .fetch_one(&state.db)
        .await
        .map_err(rustforge_core::CoreError::from)?;
    if !exists {
        return Err(ApiError::NotFound);
    }
    Ok(())
}
pub async fn list_courses(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> ApiResult<Json<Vec<Course>>> {
    let mut courses = lms::list_courses(&state.db, !_auth.user.role.can_edit_others()).await?;
    if !_auth.user.role.can_edit_others() {
        for course in &mut courses {
            if course.created_by != Some(_auth.user.id)
                && !lms::is_user_enrolled(&state.db, course.id, _auth.user.id).await?
            {
                course.content.clear();
            }
        }
    }
    Ok(Json(courses))
}

pub async fn get_course(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Course>> {
    let mut course = lms::get_course(&state.db, id).await?;
    if !_auth.user.role.can_edit_others()
        && course.created_by != Some(_auth.user.id)
        && !lms::is_user_enrolled(&state.db, id, _auth.user.id).await?
    {
        if !course.is_published {
            return Err(ApiError::NotFound);
        }
        course.content.clear();
    }
    Ok(Json(course))
}

pub async fn create_course(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(mut input): Json<CourseInput>,
) -> ApiResult<Json<Course>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    input.created_by = Some(auth.user.id);
    let course = lms::create_course(&state.db, input).await?;
    Ok(Json(course))
}

pub async fn update_course(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<CourseInput>,
) -> ApiResult<Json<Course>> {
    manager(&state, &auth, id).await?;
    let course = lms::update_course(&state.db, id, input).await?;
    Ok(Json(course))
}

pub async fn delete_course(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<()>> {
    manager(&state, &auth, id).await?;
    lms::delete_course(&state.db, id).await?;
    Ok(Json(()))
}

pub async fn list_modules(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(course_id): Path<Uuid>,
) -> ApiResult<Json<Vec<CourseModule>>> {
    if manager(&state, &_auth, course_id).await.is_err()
        && !lms::is_user_enrolled(&state.db, course_id, _auth.user.id).await?
    {
        return Err(ApiError::Forbidden);
    }
    let modules = lms::list_course_modules(&state.db, course_id).await?;
    Ok(Json(modules))
}

pub async fn create_module(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(course_id): Path<Uuid>,
    Json(input): Json<ModuleInput>,
) -> ApiResult<Json<CourseModule>> {
    manager(&state, &auth, course_id).await?;
    let m = lms::create_module(&state.db, course_id, input).await?;
    Ok(Json(m))
}

pub async fn delete_module(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((course_id, module_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<()>> {
    manager(&state, &auth, course_id).await?;
    child_in(&state, "lms_modules", module_id, course_id).await?;
    lms::delete_module(&state.db, module_id).await?;
    Ok(Json(()))
}

pub async fn list_lessons(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(course_id): Path<Uuid>,
) -> ApiResult<Json<Vec<Lesson>>> {
    if manager(&state, &_auth, course_id).await.is_err()
        && !lms::is_user_enrolled(&state.db, course_id, _auth.user.id).await?
    {
        return Err(ApiError::Forbidden);
    }
    let lessons = lms::list_course_lessons(&state.db, course_id).await?;
    Ok(Json(lessons))
}

pub async fn create_lesson(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(course_id): Path<Uuid>,
    Json(input): Json<LessonInput>,
) -> ApiResult<Json<Lesson>> {
    manager(&state, &auth, course_id).await?;
    if let Some(id) = input.module_id {
        child_in(&state, "lms_modules", id, course_id).await?;
    }
    let lesson = lms::create_lesson(&state.db, course_id, input).await?;
    Ok(Json(lesson))
}

pub async fn update_lesson(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((course_id, lesson_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<LessonInput>,
) -> ApiResult<Json<Lesson>> {
    manager(&state, &auth, course_id).await?;
    child_in(&state, "lms_lessons", lesson_id, course_id).await?;
    if let Some(id) = input.module_id {
        child_in(&state, "lms_modules", id, course_id).await?;
    }
    let lesson = lms::update_lesson(&state.db, lesson_id, input).await?;
    Ok(Json(lesson))
}

pub async fn delete_lesson(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((course_id, lesson_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<()>> {
    manager(&state, &auth, course_id).await?;
    child_in(&state, "lms_lessons", lesson_id, course_id).await?;
    lms::delete_lesson(&state.db, lesson_id).await?;
    Ok(Json(()))
}

pub async fn list_course_enrollments(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(course_id): Path<Uuid>,
) -> ApiResult<Json<Vec<lms::EnrollmentDetails>>> {
    if !auth.user.role.can_edit_others() {
        return Err(ApiError::Forbidden);
    }
    let enrollments = lms::list_course_enrollments(&state.db, course_id).await?;
    Ok(Json(enrollments))
}

pub async fn list_all_enrollments(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<Json<Vec<lms::EnrollmentDetails>>> {
    if !auth.user.role.can_edit_others() {
        return Err(ApiError::Forbidden);
    }
    let enrollments = lms::list_all_enrollments(&state.db, 100).await?;
    Ok(Json(enrollments))
}

#[derive(Debug, Deserialize)]
pub struct EnrollUserPayload {
    pub user_id: Uuid,
}

pub async fn admin_enroll_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(course_id): Path<Uuid>,
    Json(payload): Json<EnrollUserPayload>,
) -> ApiResult<Json<lms::Enrollment>> {
    if !auth.user.role.can_edit_others() {
        return Err(ApiError::Forbidden);
    }
    let enrollment = lms::enroll_user(&state.db, course_id, payload.user_id).await?;
    Ok(Json(enrollment))
}

pub async fn my_courses(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<Json<Vec<Course>>> {
    let courses = lms::get_user_enrollments(&state.db, auth.user.id).await?;
    Ok(Json(courses))
}

#[derive(Debug, Deserialize)]
pub struct ProgressUpdatePayload {
    pub progress_pct: i32,
}

pub async fn update_progress(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(course_id): Path<Uuid>,
    Json(payload): Json<ProgressUpdatePayload>,
) -> ApiResult<Json<()>> {
    lms::update_progress(&state.db, course_id, auth.user.id, payload.progress_pct).await?;
    Ok(Json(()))
}

// ---------------------------------------------------------------------------
// Endpoints publics LMS
// ---------------------------------------------------------------------------

pub async fn public_list_courses(State(state): State<AppState>) -> ApiResult<Json<Vec<Course>>> {
    let mut courses = lms::list_courses(&state.db, true).await?;
    for course in &mut courses {
        course.content.clear();
    }
    Ok(Json(courses))
}

pub async fn public_get_course_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiResult<Json<Course>> {
    let mut course = lms::get_course_by_slug(&state.db, &slug).await?;
    if !course.is_published {
        return Err(ApiError::NotFound);
    }
    course.content.clear();
    Ok(Json(course))
}

pub async fn my_enrollments(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<Json<Vec<lms::Enrollment>>> {
    let rows =
        sqlx::query_as("SELECT * FROM lms_enrollments WHERE user_id=$1 ORDER BY created_at DESC")
            .bind(auth.user.id)
            .fetch_all(&state.db)
            .await
            .map_err(rustforge_core::CoreError::from)?;
    Ok(Json(rows))
}
