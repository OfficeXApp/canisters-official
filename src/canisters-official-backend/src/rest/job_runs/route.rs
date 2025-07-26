use crate::debug_log;
use crate::rest::router::{self, genroute};
use crate::rest::types::RouteHandler;

pub const JOB_RUNS_GET_PATH: &str = genroute!("/job_runs/get/{job_run_id}");
pub const JOB_RUNS_LIST_PATH: &str = genroute!("/job_runs/list");
pub const JOB_RUNS_CREATE_PATH: &str = genroute!("/job_runs/create");
pub const JOB_RUNS_UPDATE_PATH: &str = genroute!("/job_runs/update");
pub const JOB_RUNS_DELETE_PATH: &str = genroute!("/job_runs/delete");


type HandlerEntry = (&'static str, &'static str, RouteHandler);

/// Initializes and registers all API routes related to JobRuns.
pub fn init_routes() {
    let routes: &[HandlerEntry] = &[
        (
            "GET",
            JOB_RUNS_GET_PATH,
            |req, params| Box::pin(crate::rest::job_runs::handler::job_runs_handlers::get_job_run_handler(req, params)),
        ),
        (
            "POST",
            JOB_RUNS_LIST_PATH,
            |req, params| Box::pin(crate::rest::job_runs::handler::job_runs_handlers::list_job_runs_handler(req, params)),
        ),
        (
            "POST",
            JOB_RUNS_CREATE_PATH,
            |req, params| Box::pin(crate::rest::job_runs::handler::job_runs_handlers::create_job_run_handler(req, params)),
        ),
        (
            "POST",
            JOB_RUNS_UPDATE_PATH,
            |req, params| Box::pin(crate::rest::job_runs::handler::job_runs_handlers::update_job_run_handler(req, params)),
        ),
        (
            "POST",
            JOB_RUNS_DELETE_PATH,
            |req, params| Box::pin(crate::rest::job_runs::handler::job_runs_handlers::delete_job_run_handler(req, params)),
        ),
    ];

    for &(method, path, handler) in routes {
        debug_log!("Registering {} route: {}", method, path);
        router::insert_route(method, path, handler);
    }
}