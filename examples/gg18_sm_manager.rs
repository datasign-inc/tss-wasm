#[cfg(not(target_arch = "wasm32"))]
use reqwest;
#[cfg(not(target_arch = "wasm32"))]
use rocket::http::Status;
#[cfg(not(target_arch = "wasm32"))]
use rocket::request::{FromRequest, Outcome};
#[cfg(not(target_arch = "wasm32"))]
use rocket::serde::json::Json;
#[cfg(not(target_arch = "wasm32"))]
use rocket::Request;
#[cfg(not(target_arch = "wasm32"))]
use rocket::{post, routes, State};
#[cfg(not(target_arch = "wasm32"))]
use rocket_cors::{AllowedOrigins, CorsOptions};
#[cfg(not(target_arch = "wasm32"))]
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::RwLock;
#[cfg(not(target_arch = "wasm32"))]
use tss_wasm::common::{Entry, Index, Key, Params, PartySignup};
#[cfg(not(target_arch = "wasm32"))]
use uuid::Uuid;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
struct ApiKey(String);

#[cfg(not(target_arch = "wasm32"))]
const SERVER_BASE: &str = "http://localhost:3000";
#[cfg(not(target_arch = "wasm32"))]
const SERVER_SIDE_SCRIPT: &str = "./../scripts/server_side_party.js";

#[cfg(not(target_arch = "wasm32"))]
#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(auth_header) = request.headers().get_one("Authorization") {
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                if check_token(token).await {
                    return Outcome::Success(ApiKey(token.to_string()));
                }
            }
        }
        Outcome::<Self, Self::Error>::Error((Status::Forbidden, ()))
    }
}

/// チェックサーバー (<SERVER_BASE>/internal/check_token) に対して
/// JSON形式で <token> を問い合わせ、レスポンスの "result" が "valid" なら true を返す。
#[cfg(not(target_arch = "wasm32"))]
async fn check_token(token: &str) -> bool {
    let client = reqwest::Client::new();
    let body = serde_json::json!({ "token": token });
    let url = format!("{}/internal/check_token", SERVER_BASE);
    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await;

    match response {
        Ok(resp) => {
            let json: serde_json::Value = resp.json().await.unwrap_or(serde_json::json!({}));
            json.get("result").map_or(false, |v| v == "valid")
        }
        Err(_) => false,
    }
}

/// チェックサーバーから task 情報を取得するためのレスポンス JSON に対応する構造体。
#[derive(Debug, Deserialize)]
struct Task {
    id: String,
    #[serde(rename = "type")]
    task_type: String,
    parameters: String,
    status: String,
    created_at: String,
    created_by: String,
}

/// 指定された taskId を元に、 <SERVER_BASE>/internal/tasks/{taskId} にアクセスし、
/// JSON をパースして Task 型として返却する関数。
#[cfg(not(target_arch = "wasm32"))]
async fn get_task(task_id: &str) -> Result<Task, reqwest::Error> {
    let url = format!("{}/internal/tasks/{}", SERVER_BASE, task_id);
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    let task = response.json::<Task>().await?;
    Ok(task)
}

#[cfg(not(target_arch = "wasm32"))]
#[post("/get", format = "json", data = "<request>")]
fn get(
    _auth: ApiKey, // Authorizationチェック済み
    db_mtx: &State<RwLock<HashMap<Key, String>>>,
    request: Json<Index>,
) -> Json<Result<Entry, ()>> {
    let index: Index = request.0;
    let hm = db_mtx.read().unwrap();
    match hm.get(&index.key) {
        Some(v) => {
            let entry = Entry {
                key: index.key,
                value: v.clone(),
            };
            Json(Ok(entry))
        }
        None => Json(Err(())),
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[post("/set", format = "json", data = "<request>")]
fn set(
    _auth: ApiKey, // Authorizationチェック済み
    db_mtx: &State<RwLock<HashMap<Key, String>>>,
    request: Json<Entry>,
) -> Json<Result<(), ()>> {
    let entry: Entry = request.0;
    let mut hm = db_mtx.write().unwrap();
    hm.insert(entry.key.clone(), entry.value);
    Json(Ok(()))
}

#[derive(Debug, Deserialize)]
struct TaskRequest {
    task_id: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[post("/signupkeygen", format = "json", data = "<request>")]
async fn signup_keygen(
    _auth: ApiKey, // Authorizationチェック済み
    db_mtx: &State<RwLock<HashMap<Key, String>>>,
    request: Json<TaskRequest>,
) -> Result<Json<PartySignup>, Status> {
    // 1. POSTされたJSONから task_id を取得
    let task_id = &request.task_id;

    // 2. 取得した task_id を用いて get_task を呼び出す
    let task = get_task(task_id).await.map_err(|_| Status::BadRequest)?;

    // 3. チェック: signup_keygenの場合、task_typeは "keygeneration" であり、statusが "created" であること
    if task.task_type != "keygeneration" || task.status != "created" {
        return Err(Status::BadRequest);
    }

    // 既存のロジック: params.json を読み込み、PartySignupを更新
    let data = fs::read_to_string("params.json")
        .expect("Unable to read params, make sure config file is present in the same folder ");
    let params: Params = serde_json::from_str(&data).unwrap();
    let parties = params.parties.parse::<u16>().unwrap();

    let key = "signup-keygen".to_string();
    let mut hm = db_mtx.write().unwrap();
    let party_signup = {
        let value = hm.get(&key).unwrap();
        let client_signup: PartySignup = serde_json::from_str(value).unwrap();
        if client_signup.number < parties {
            PartySignup {
                number: client_signup.number + 1,
                uuid: client_signup.uuid,
            }
        } else {
            PartySignup {
                number: 1,
                uuid: Uuid::new_v4().to_string(),
            }
        }
    };

    hm.insert(key, serde_json::to_string(&party_signup).unwrap());

    // 4. 外部コマンドの実行: node <SERVER_SIDE_SCRIPT> <task_id>
    let _child = tokio::process::Command::new("node")
        .arg(SERVER_SIDE_SCRIPT)
        .arg(task_id)
        .arg(_auth.0)
        .spawn()
        .map_err(|_| Status::ServiceUnavailable)?;

    Ok(Json(party_signup))
}

#[cfg(not(target_arch = "wasm32"))]
#[post("/signupsign", format = "json", data = "<request>")]
async fn signup_sign(
    _auth: ApiKey, // Authorizationチェック済み
    db_mtx: &State<RwLock<HashMap<Key, String>>>,
    request: Json<TaskRequest>,
) -> Result<Json<PartySignup>, Status> {
    // 1. POSTされたJSONから task_id を取得
    let task_id = &request.task_id;

    // 2. 取得した task_id を用いて get_task を呼び出す
    let task = get_task(task_id).await.map_err(|_| Status::BadRequest)?;

    // 3. チェック: signup_signの場合、task_typeは "signing" であり、statusが "created" であること
    if task.task_type != "signing" || task.status != "created" {
        return Err(Status::BadRequest);
    }

    // 既存のロジック: params.json を読み込み、PartySignupを更新
    let data = fs::read_to_string("params.json")
        .expect("Unable to read params, make sure config file is present in the same folder ");
    let params: Params = serde_json::from_str(&data).unwrap();
    let threshold = params.threshold.parse::<u16>().unwrap();
    let key = "signup-sign".to_string();
    let mut hm = db_mtx.write().unwrap();
    let party_signup = {
        let value = hm.get(&key).unwrap();
        let client_signup: PartySignup = serde_json::from_str(value).unwrap();
        if client_signup.number < threshold + 1 {
            PartySignup {
                number: client_signup.number + 1,
                uuid: client_signup.uuid,
            }
        } else {
            PartySignup {
                number: 1,
                uuid: Uuid::new_v4().to_string(),
            }
        }
    };

    hm.insert(key, serde_json::to_string(&party_signup).unwrap());

    // 4. 外部コマンドの実行: node <SERVER_SIDE_SCRIPT> <task_id>
    let _output = tokio::process::Command::new("node")
        .arg(SERVER_SIDE_SCRIPT)
        .arg(task_id)
        .arg(_auth.0)
        .spawn()
        .map_err(|_| Status::ServiceUnavailable)?;
    Ok(Json(party_signup))
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    let db: HashMap<Key, String> = HashMap::new();
    let db_mtx = RwLock::new(db);

    /////////////////////////////////////////////////////////////////
    //////////////////////////init signups://////////////////////////
    /////////////////////////////////////////////////////////////////

    let keygen_key = "signup-keygen".to_string();
    let sign_key = "signup-sign".to_string();

    let uuid_keygen = Uuid::new_v4().to_string();
    let uuid_sign = Uuid::new_v4().to_string();

    let party1 = 0;
    let party_signup_keygen = PartySignup {
        number: party1,
        uuid: uuid_keygen,
    };
    let party_signup_sign = PartySignup {
        number: party1,
        uuid: uuid_sign,
    };
    {
        let mut hm = db_mtx.write().unwrap();
        hm.insert(
            keygen_key,
            serde_json::to_string(&party_signup_keygen).unwrap(),
        );
        hm.insert(sign_key, serde_json::to_string(&party_signup_sign).unwrap());
    }

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            ["Get", "Post", "Patch"]
                .iter()
                .map(|s| std::str::FromStr::from_str(s).unwrap())
                .collect(),
        )
        .allow_credentials(true);

    /////////////////////////////////////////////////////////////////
    rocket::build()
        .mount("/", routes![get, set, signup_keygen, signup_sign])
        .attach(cors.to_cors().unwrap())
        .manage(db_mtx)
        .launch()
        .await
        .unwrap();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    panic!("Unimplemented")
}
