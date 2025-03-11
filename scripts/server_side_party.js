#!/usr/bin/env node

// Node.js 18以降ではfetchがグローバルに利用可能です。
// もしNode.jsのバージョンが低い場合は、node-fetch等を利用してください。
const fetch = global.fetch || require('node-fetch');

// gg18モジュール（既存の暗号ライブラリなどと仮定）
const gg18 = require('../pkg');

// 既存の keygen 実装
async function keygen(addr, t, n, delay) {
    let context = await gg18.gg18_keygen_client_new_context(addr, t, n, delay);
    console.log('keygen new context: ', context);
    context = await gg18.gg18_keygen_client_round1(context, delay);
    console.log('keygen round1: ', context);
    context = await gg18.gg18_keygen_client_round2(context, delay);
    console.log('keygen round2: ', context);
    context = await gg18.gg18_keygen_client_round3(context, delay);
    console.log('keygen round3: ', context);
    context = await gg18.gg18_keygen_client_round4(context, delay);
    console.log('keygen round4: ', context);
    let keygen_json = await gg18.gg18_keygen_client_round5(context, delay);
    console.log('keygen json: ', keygen_json);
    return keygen_json;
}

// 既存の sign 実装
async function sign(addr, t, n, message, key_store, delay) {
    console.log(`creating signature for : ${message}`);
    let context = await gg18.gg18_sign_client_new_context(addr, t, n, key_store, message);
    console.log('sign new context: ', context);
    context = await gg18.gg18_sign_client_round0(context, delay);
    console.log('sign round0: ', context);
    context = await gg18.gg18_sign_client_round1(context, delay);
    console.log('sign round1: ', context);
    context = await gg18.gg18_sign_client_round2(context, delay);
    console.log('sign round2: ', context);
    context = await gg18.gg18_sign_client_round3(context, delay);
    console.log('sign round3: ', context);
    context = await gg18.gg18_sign_client_round4(context, delay);
    console.log('sign round4: ', context);
    context = await gg18.gg18_sign_client_round5(context, delay);
    console.log('sign round5: ', context);
    context = await gg18.gg18_sign_client_round6(context, delay);
    console.log('sign round6: ', context);
    context = await gg18.gg18_sign_client_round7(context, delay);
    console.log('sign round7: ', context);
    context = await gg18.gg18_sign_client_round8(context, delay);
    console.log('sign round8: ', context);
    let sign_json = await gg18.gg18_sign_client_round9(context, delay);
    console.log('keysign json: ', sign_json);
    return sign_json;
}

async function main() {
    // 1. コマンドライン引数から taskId を取得
    const args = process.argv.slice(2);
    if (args.length !== 1) {
        console.error("Usage: node server_side_party.js <taskId>");
        process.exit(1);
    }
    const taskId = args[0];

    // 2. 管理サーバーから task 情報を取得
    const taskUrl = `http://localhost:3000/internal/tasks/${taskId}`;
    let response;
    try {
        response = await fetch(taskUrl);
    } catch (err) {
        console.error("Failed to fetch task:", err);
        process.exit(1);
    }
    if (!response.ok) {
        console.error(`Failed to fetch task: HTTP ${response.status}`);
        process.exit(1);
    }
    let task;
    try {
        task = await response.json();
    } catch (err) {
        console.error("Failed to parse task JSON:", err);
        process.exit(1);
    }

    // 3. チェック: status が "created" であり、id が taskId と一致すること
    if (task.status !== "created") {
        console.error(`Task status is not 'created': ${task.status}`);
        process.exit(1);
    }
    if (task.id !== taskId) {
        console.error(`Task id mismatch: expected ${taskId}, got ${task.id}`);
        process.exit(1);
    }

    // 4. task.parameters を JSON としてパース
    let params;
    try {
        params = JSON.parse(task.parameters);
    } catch (err) {
        console.error("Failed to parse task.parameters as JSON:", err);
        process.exit(1);
    }

    // delay の算出: Math.random() % 500 は意味がないため、0～500の乱数から100以上を保証する
    const delay = Math.max(Math.floor(Math.random() * 500), 100);

    // ハードコードしたアドレス（適当なURLを設定してください）
    const KEYGEN_ADDR = "http://hardcoded-keygen-address.example.com";
    const SIGN_ADDR = "http://hardcoded-sign-address.example.com";

    // 5. type に応じた処理
    if (task.type === "keygeneration") {
        // keygenerationの場合、paramsは "t" と "n" を含むことをチェック
        if (!("t" in params) || !("n" in params)) {
            console.error("Parameters for keygeneration must include 't' and 'n'");
            process.exit(1);
        }
        try {
            const result = await keygen(KEYGEN_ADDR, params.t, params.n, delay);
            console.log("Key generation result:", result);
        } catch (err) {
            console.error("Error during key generation:", err);
            process.exit(1);
        }
    } else if (task.type === "signing") {
        // signingの場合、paramsは "t", "n", "message" を含むことをチェック
        if (!("t" in params) || !("n" in params) || !("message" in params)) {
            console.error("Parameters for signing must include 't', 'n', and 'message'");
            process.exit(1);
        }
        // 管理サーバーから created_by を使って generated_user_key を取得
        const keyUrl = `http://localhost:3000/internal/generated_user_key/${task.created_by}`;
        let keyResponse;
        try {
            keyResponse = await fetch(keyUrl);
        } catch (err) {
            console.error("Failed to fetch generated user key:", err);
            process.exit(1);
        }
        if (!keyResponse.ok) {
            console.error(`Failed to fetch generated user key: HTTP ${keyResponse.status}`);
            process.exit(1);
        }
        let keyData;
        try {
            keyData = await keyResponse.json();
        } catch (err) {
            console.error("Failed to parse generated user key JSON:", err);
            process.exit(1);
        }
        if (!("key_data" in keyData)) {
            console.error("Generated user key JSON does not contain 'key_data'");
            process.exit(1);
        }
        try {
            const result = await sign(
                SIGN_ADDR,
                params.t,
                params.n,
                params.message,
                keyData.key_data,
                delay
            );
            console.log("Signing result:", result);
        } catch (err) {
            console.error("Error during signing:", err);
            process.exit(1);
        }
    } else {
        console.error(`Unknown task type: ${task.type}`);
        process.exit(1);
    }
}

main();