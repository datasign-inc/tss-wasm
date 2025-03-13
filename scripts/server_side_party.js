#!/usr/bin/env node

const fetch = global.fetch || require('node-fetch');

const gg18 = require('../pkg');

const GG18_KEYGEN_ADDR = "http://localhost:8000";
const GG18_SIGN_ADDR = "http://localhost:8000";
const MGT_SERVER_ADDR = "http://localhost:3000";

// MGT_SERVER_ADDRとパス部分を結合するヘルパー関数
function buildManagementServerUrl(path) {
    return `${MGT_SERVER_ADDR}${path}`;
}

/**
 * 既存の keygen 実装
 */
async function keygen(addr, t, n, delay, token, taskId) {

    // todo: send `token` to GG18 server

    let context = await gg18.gg18_keygen_client_new_context(addr, t, n, delay, token, taskId, "server_side");
    console.log('keygen new context: ', context);
    context = await gg18.gg18_keygen_client_round1(context, delay, token);
    console.log('keygen round1: ', context);
    context = await gg18.gg18_keygen_client_round2(context, delay, token);
    console.log('keygen round2: ', context);
    context = await gg18.gg18_keygen_client_round3(context, delay, token);
    console.log('keygen round3: ', context);
    context = await gg18.gg18_keygen_client_round4(context, delay, token);
    console.log('keygen round4: ', context);
    let keygen_json = await gg18.gg18_keygen_client_round5(context, delay, token);
    console.log('keygen json: ', keygen_json);
    return keygen_json;
}

/**
 * 既存の sign 実装
 */
async function sign(addr, t, n, message, key_store, delay, token, task_id) {

    // todo: send `token` to GG18 server

    console.log(`creating signature for : ${message}`);
    let context = await gg18.gg18_sign_client_new_context(addr, t, n, key_store, message, token, task_id, "server_side");
    console.log('sign new context: ', context);
    context = await gg18.gg18_sign_client_round0(context, delay, token);
    console.log('sign round0: ', context);
    context = await gg18.gg18_sign_client_round1(context, delay, token);
    console.log('sign round1: ', context);
    context = await gg18.gg18_sign_client_round2(context, delay, token);
    console.log('sign round2: ', context);
    context = await gg18.gg18_sign_client_round3(context, delay, token);
    console.log('sign round3: ', context);
    context = await gg18.gg18_sign_client_round4(context, delay, token);
    console.log('sign round4: ', context);
    context = await gg18.gg18_sign_client_round5(context, delay, token);
    console.log('sign round5: ', context);
    context = await gg18.gg18_sign_client_round6(context, delay, token);
    console.log('sign round6: ', context);
    context = await gg18.gg18_sign_client_round7(context, delay, token);
    console.log('sign round7: ', context);
    context = await gg18.gg18_sign_client_round8(context, delay, token);
    console.log('sign round8: ', context);
    let sign_json = await gg18.gg18_sign_client_round9(context, delay, token);
    console.log('keysign json: ', sign_json);
    return sign_json;
}

/**
 * keygeneration タイプのタスク処理
 * 1. パラメータのチェック
 * 2. keygen の実行
 * 3. keygen の結果を管理サーバーへPUTで永続化（PUT先: /internal/generated_user_key/{user_id}）
 */
async function processKeyGeneration(task, params, delay, token) {
    if (!("t" in params) || !("n" in params)) {
        throw new Error("Parameters for keygeneration must include 't' and 'n'");
    }
    const result = await keygen(GG18_KEYGEN_ADDR, params.t, params.n, delay, token, task.id);
    console.log("Key generation result:", result);

    // PUTで結果を永続化
    const putUrl = buildManagementServerUrl(`/internal/generated_user_key/${task.created_by}`);
    const putResponse = await fetch(putUrl, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ key_data: result })
    });
    if (!putResponse.ok) {
        throw new Error(`Failed to persist key generation result: HTTP ${putResponse.status}`);
    }
}

/**
 * signing タイプのタスク処理
 * 1. パラメータのチェック
 * 2. 管理サーバーから generated_user_key を取得
 * 3. sign の実行
 */
async function processSigning(task, params, delay, token) {
    if (!("t" in params) || !("n" in params) || !("message" in params)) {
        throw new Error("Parameters for signing must include 't', 'n', and 'message'");
    }
    const keyUrl = buildManagementServerUrl(`/internal/generated_user_key/${task.created_by}`);
    let keyResponse;
    try {
        keyResponse = await fetch(keyUrl);
    } catch (err) {
        throw new Error("Failed to fetch generated user key: " + err);
    }
    if (!keyResponse.ok) {
        throw new Error(`Failed to fetch generated user key: HTTP ${keyResponse.status}`);
    }
    let keyData;
    try {
        keyData = await keyResponse.json();
    } catch (err) {
        throw new Error("Failed to parse generated user key JSON: " + err);
    }
    if (!("key_data" in keyData)) {
        throw new Error("Generated user key JSON does not contain 'key_data'");
    }
    const result = await sign(GG18_SIGN_ADDR, params.t, params.n, params.message, keyData.key_data, delay, token, task.id);
    console.log("Signing result:", result);
}

/**
 * タスクのステータスをPATCHで更新する補助関数
 * PATCH先: /internal/tasks/{taskId}/status
 */
async function patchTaskStatus(taskId, status) {
    const patchUrl = buildManagementServerUrl(`/internal/tasks/${taskId}/status`);
    const response = await fetch(patchUrl, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ status })
    });
    if (!response.ok) {
        console.error(`Failed to patch task status to ${status}: HTTP ${response.status}`);
    } else {
        console.log(`Task status updated to ${status}`);
    }
}

async function getTask(taskId) {
    // 管理サーバーから task 情報を取得
    const taskUrl = buildManagementServerUrl(`/internal/tasks/${taskId}`);
    let response;
    try {
        response = await fetch(taskUrl);
    } catch (err) {
        console.error("Failed to fetch task:", err);
        await patchTaskStatus(taskId, "failed");
        process.exit(1);
    }
    if (!response.ok) {
        console.error(`Failed to fetch task: HTTP ${response.status}`);
        await patchTaskStatus(taskId, "failed");
        process.exit(1);
    }
    let task;
    try {
        task = await response.json();
        return task;
    } catch (err) {
        console.error("Failed to parse task JSON:", err);
        await patchTaskStatus(taskId, "failed");
        process.exit(1);
    }
}

/**
 * main処理
 * 1. taskIdの取得、タスク情報のフェッチ・検証
 * 2. task.parametersのパースとdelay算出
 * 3. task.typeに応じた処理の呼び出し（エラー発生時はキャッチ）
 * 4. 正常／異常に応じたタスクステータスのPATCH更新
 */
async function main() {
    const args = process.argv.slice(2);
    if (args.length !== 2) {
        console.error("Usage: node server_side_party.js <taskId> <token>");
        process.exit(1);
    }
    const taskId = args[0];
    const token = args[1];
    const task = await getTask(taskId);

    if (!task) {
        console.error(`unable to get task: ${taskId}`);
        process.exit(1);
    }

    // チェック: status が "created" であり、id が taskId と一致すること
    if (task.status !== "created") {
        console.error(`Task status is not 'created': ${task.status}`);
        await patchTaskStatus(taskId, "failed");
        process.exit(1);
    }
    if (task.id !== taskId) {
        console.error(`Task id mismatch: expected ${taskId}, got ${task.id}`);
        await patchTaskStatus(taskId, "failed");
        process.exit(1);
    }

    // task.parameters を JSON としてパース
    let params;
    try {
        params = JSON.parse(task.parameters);
    } catch (err) {
        console.error("Failed to parse task.parameters as JSON:", err);
        await patchTaskStatus(taskId, "failed");
        process.exit(1);
    }

    // delay の算出: 0～500の乱数から100以上を保証
    const delay = Math.max(Math.floor(Math.random() * 500), 100);

    try {
        await patchTaskStatus(taskId, "processing");
        if (task.type === "keygeneration") {
            await processKeyGeneration(task, params, delay, token);
        } else if (task.type === "signing") {
            await processSigning(task, params, delay, token);
        } else {
            throw new Error(`Unknown task type: ${task.type}`);
        }
        // 正常終了時はタスクステータスを completed に更新
        await patchTaskStatus(taskId, "completed");
    } catch (err) {
        console.error("Error processing task:", err);
        // エラー発生時はタスクステータスを failed に更新
        await patchTaskStatus(taskId, "failed");
        process.exit(1);
    }
}

main();