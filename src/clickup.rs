use crate::hierarchy::{ListRef, parse_space_lists};
use crate::task::{Task, parse_task, parse_task_list};
use crate::ticket::{Ticket, parse_ticket};
use anyhow::{Context, Result, anyhow};
use serde_json::{Value, json};
use std::{
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const API: &str = "https://api.clickup.com/api/v2";
const MAX_RETRIES: u32 = 4;
const MAX_WAIT_SECS: u64 = 60;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const READ_TIMEOUT: Duration = Duration::from_secs(30);

fn header(resp: &ureq::http::Response<ureq::Body>, name: &str) -> Option<String> {
    resp.headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
}

/// Seconds to wait before retrying a 429: the reset header, `Retry-After`, or a short backoff.
pub fn rate_limit_wait(reset: Option<&str>, retry_after: Option<&str>, attempt: u32) -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if let Some(at) = reset.and_then(|s| s.trim().parse::<u64>().ok()) {
        let wait = at.saturating_sub(now);
        if wait > 0 {
            return wait.min(MAX_WAIT_SECS);
        }
    }
    if let Some(secs) = retry_after.and_then(|s| s.trim().parse::<u64>().ok())
        && secs > 0
    {
        return secs.min(MAX_WAIT_SECS);
    }
    (1u64 << attempt.min(6)).min(MAX_WAIT_SECS)
}

/// Thin blocking client for the ClickUp v2 REST API. A 429 waits for the reset header and retries.
#[derive(Clone)]
pub struct ClickUp {
    token: String,
    team_id: Option<String>,
    agent: ureq::Agent,
}

impl ClickUp {
    pub fn new(token: String, team_id: Option<String>) -> Self {
        let agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_connect(Some(CONNECT_TIMEOUT))
            .timeout_recv_response(Some(READ_TIMEOUT))
            .timeout_recv_body(Some(READ_TIMEOUT))
            .build()
            .into();
        ClickUp {
            token,
            team_id,
            agent,
        }
    }

    fn get(&self, path: &str) -> Result<Value> {
        for attempt in 0..=MAX_RETRIES {
            let mut resp = self
                .agent
                .get(&format!("{API}{path}"))
                .header("Authorization", &self.token)
                .call()
                .with_context(|| format!("GET {path} (10s connect, 30s read)"))?;
            let status = resp.status().as_u16();
            if status == 429 && attempt < MAX_RETRIES {
                let reset = header(&resp, "x-ratelimit-reset");
                let retry = header(&resp, "retry-after");
                let wait = rate_limit_wait(reset.as_deref(), retry.as_deref(), attempt);
                thread::sleep(Duration::from_secs(wait));
                continue;
            }
            if status >= 400 {
                let body = resp.body_mut().read_to_string().unwrap_or_default();
                return Err(anyhow!("GET {path}: HTTP {status} {body}"));
            }
            return resp
                .body_mut()
                .read_json()
                .with_context(|| format!("GET {path}: bad JSON"));
        }
        unreachable!("the retry loop always returns")
    }

    fn send(&self, method: &str, path: &str, body: Value) -> Result<()> {
        let url = format!("{API}{path}");
        for attempt in 0..=MAX_RETRIES {
            let req = match method {
                "PUT" => self.agent.put(&url),
                _ => self.agent.post(&url),
            };
            let mut resp = req
                .header("Authorization", &self.token)
                .send_json(&body)
                .with_context(|| format!("{method} {path} (10s connect, 30s read)"))?;
            let status = resp.status().as_u16();
            if status == 429 && attempt < MAX_RETRIES {
                let reset = header(&resp, "x-ratelimit-reset");
                let retry = header(&resp, "retry-after");
                let wait = rate_limit_wait(reset.as_deref(), retry.as_deref(), attempt);
                thread::sleep(Duration::from_secs(wait));
                continue;
            }
            if status >= 400 {
                let body = resp.body_mut().read_to_string().unwrap_or_default();
                return Err(anyhow!("{method} {path}: HTTP {status} {body}"));
            }
            return Ok(());
        }
        unreachable!("the retry loop always returns")
    }

    pub fn team_id(&self) -> Result<String> {
        if let Some(t) = &self.team_id {
            return Ok(t.clone());
        }
        let teams = self.get("/team")?;
        teams
            .pointer("/teams/0/id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| anyhow!("this token can see no ClickUp workspace"))
    }

    pub fn my_user_id(&self) -> Result<u64> {
        self.get("/user")?
            .pointer("/user/id")
            .and_then(Value::as_u64)
            .ok_or_else(|| anyhow!("GET /user returned no user id"))
    }

    /// Every page of a task query (ClickUp pages hold 100), up to `MAX_PAGES`.
    fn all_tasks(&self, path: &str) -> Result<Vec<Task>> {
        const MAX_PAGES: u32 = 20;
        let mut tasks = Vec::new();
        for page in 0..MAX_PAGES {
            let v = self.get(&format!("{path}&page={page}"))?;
            let got = parse_task_list(&v);
            let last =
                got.is_empty() || v.get("last_page").and_then(Value::as_bool).unwrap_or(true);
            tasks.extend(got);
            if last {
                break;
            }
        }
        Ok(tasks)
    }

    /// Open tasks assigned to the token's user in the `spaces` given (all when empty), most recently updated first.
    pub fn my_tasks(&self, spaces: &[String]) -> Result<Vec<Task>> {
        let (team, me) = (self.team_id()?, self.my_user_id()?);
        let only: String = spaces.iter().map(|s| format!("&space_ids[]={s}")).collect();
        self.all_tasks(&format!(
            "/team/{team}/task?assignees[]={me}&subtasks=true&order_by=updated{only}"
        ))
    }

    /// Open tasks in one list, subtasks and tasks added from other lists (as sprints do) included.
    pub fn list_tasks(&self, list_id: &str) -> Result<Vec<Task>> {
        self.all_tasks(&format!(
            "/list/{list_id}/task?subtasks=true&include_timl=true&order_by=updated"
        ))
    }

    /// `(id, name)` of the workspace's spaces, limited to `only` when it is not empty.
    pub fn spaces(&self, only: &[String]) -> Result<Vec<(String, String)>> {
        let team = self.team_id()?;
        let v = self.get(&format!("/team/{team}/space?archived=false"))?;
        let spaces = v
            .get("spaces")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        Ok(spaces
            .iter()
            .filter_map(|s| {
                Some((
                    s.get("id")?.as_str()?.to_string(),
                    s.get("name")?.as_str()?.to_string(),
                ))
            })
            .filter(|(id, _)| only.is_empty() || only.contains(id))
            .collect())
    }

    /// Every unarchived list in a space, in folders or not.
    pub fn space_lists(&self, space_id: &str, space_name: &str) -> Result<Vec<ListRef>> {
        let folders = self.get(&format!("/space/{space_id}/folder?archived=false"))?;
        let loose = self.get(&format!("/space/{space_id}/list?archived=false"))?;
        Ok(parse_space_lists(space_name, &folders, &loose))
    }

    pub fn task(&self, id: &str) -> Result<Task> {
        parse_task(&self.get(&format!("/task/{id}"))?).ok_or_else(|| anyhow!("task {id} not found"))
    }

    /// The task with its checklists, subtasks, attachments and comments.
    pub fn ticket(&self, id: &str) -> Result<Ticket> {
        let task = self.get(&format!("/task/{id}?include_subtasks=true"))?;
        let comments = self.get(&format!("/task/{id}/comment"))?;
        parse_ticket(&task, &comments).ok_or_else(|| anyhow!("task {id} not found"))
    }

    pub fn set_status(&self, id: &str, status: &str) -> Result<()> {
        self.send("PUT", &format!("/task/{id}"), json!({ "status": status }))
    }

    pub fn comment(&self, id: &str, text: &str) -> Result<()> {
        self.send(
            "POST",
            &format!("/task/{id}/comment"),
            json!({ "comment_text": text, "notify_all": false }),
        )
    }
}
