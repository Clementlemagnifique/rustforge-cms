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

pub async fn list_courses(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> ApiResult<Json<Vec<Course>>> {
    let courses = lms::list_courses(&state.db, false).await?;
    Ok(Json(courses))
}

pub async fn get_course(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Course>> {
    let course = lms::get_course(&state.db, id).await?;
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
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let course = lms::update_course(&state.db, id, input).await?;
    Ok(Json(course))
}

pub async fn delete_course(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<()>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    lms::delete_course(&state.db, id).await?;
    Ok(Json(()))
}

pub async fn list_modules(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(course_id): Path<Uuid>,
) -> ApiResult<Json<Vec<CourseModule>>> {
    let modules = lms::list_course_modules(&state.db, course_id).await?;
    Ok(Json(modules))
}

pub async fn create_module(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(course_id): Path<Uuid>,
    Json(input): Json<ModuleInput>,
) -> ApiResult<Json<CourseModule>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let m = lms::create_module(&state.db, course_id, input).await?;
    Ok(Json(m))
}

pub async fn delete_module(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((_course_id, module_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<()>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    lms::delete_module(&state.db, module_id).await?;
    Ok(Json(()))
}

pub async fn list_lessons(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(course_id): Path<Uuid>,
) -> ApiResult<Json<Vec<Lesson>>> {
    if !_auth.user.role.can_edit_others()
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
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let lesson = lms::create_lesson(&state.db, course_id, input).await?;
    Ok(Json(lesson))
}

pub async fn update_lesson(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((_course_id, lesson_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<LessonInput>,
) -> ApiResult<Json<Lesson>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let lesson = lms::update_lesson(&state.db, lesson_id, input).await?;
    Ok(Json(lesson))
}

pub async fn delete_lesson(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((_course_id, lesson_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<()>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
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
    let courses = lms::list_courses(&state.db, true).await?;
    Ok(Json(courses))
}

pub async fn public_get_course_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiResult<Json<Course>> {
    let course = lms::get_course_by_slug(&state.db, &slug).await?;
    if !course.is_published {
        return Err(ApiError::NotFound);
    }
    Ok(Json(course))
}
