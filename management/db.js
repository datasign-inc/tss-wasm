const sqlite3 = require('sqlite3').verbose();
const crypto = require('crypto');


function generateUUID() {
    return crypto.randomUUID(); // Node.js v14.17.0 以降で利用可能
}

const db = new sqlite3.Database('./database.sqlite', (err) => {
    if (err) {
        console.error('データベース接続エラー:', err);
    } else {
        console.log('SQLiteデータベースに接続しました');
    }
});

// DBの初期化・スキーマ作成、サンプルユーザー登録を実行する関数
function initDB() {
    db.serialize(() => {
        db.run(`
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE,
                password TEXT
            )
        `);

        db.run(`
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,  -- UUIDに変更
                type TEXT,
                parameters TEXT,
                status TEXT,
                created_at TEXT,
                created_by INTEGER,
                FOREIGN KEY (created_by) REFERENCES users(id)
            )
        `);

        db.run(`
            CREATE TABLE IF NOT EXISTS generated_user_key (
                user_id INTEGER PRIMARY KEY,
                key_data TEXT,
                FOREIGN KEY (user_id) REFERENCES users(id)
            )
        `);

        const sampleUsername = 'test';
        const samplePassword = crypto.createHash('sha256').update('test123').digest('hex');
        db.run(
            `INSERT OR IGNORE INTO users (username, password) VALUES (?, ?)`,
            [sampleUsername, samplePassword],
            function(err) {
                if (err) {
                    console.error('テストユーザー登録エラー:', err);
                } else {
                    console.log('テストユーザーが登録済みまたは既に存在します');
                }
            }
        );
    });
}

// ユーザー名でユーザー情報を取得する関数（Promiseラップ）
function getUserByUsername(username) {
    return new Promise((resolve, reject) => {
        db.get(`SELECT * FROM users WHERE username = ?`, [username], (err, row) => {
            if (err) return reject(err);
            resolve(row);
        });
    });
}

// ユーザー名でユーザー情報を取得する関数（Promiseラップ）
function getTaskById(taskId) {
    return new Promise((resolve, reject) => {
        db.get(`SELECT * FROM tasks WHERE id = ?`, [taskId], (err, row) => {
            if (err) return reject(err);
            resolve(row);
        });
    });
}


function insertTask(taskType, parameters, status, created_at, created_by) {
    return new Promise((resolve, reject) => {
        const taskId = generateUUID(); // UUIDを生成
        const parametersStr = JSON.stringify(parameters);
        db.run(
            `INSERT INTO tasks (id, type, parameters, status, created_at, created_by) VALUES (?, ?, ?, ?, ?, ?)`,
            [taskId, taskType, parametersStr, status, created_at, created_by],
            function(err) {
                if (err) return reject(err);
                resolve(taskId); // 変更：UUIDを返す
            }
        );
    });
}

function updateTaskStatus(taskId, newStatus) {
    return new Promise((resolve, reject) => {
        db.run("BEGIN TRANSACTION", (err) => {
            if (err) return reject(err);
            db.run(
                `UPDATE tasks SET status = ? WHERE id = ?`,
                [newStatus, taskId],
                function(err) {
                    if (err) {
                        // 更新に失敗した場合、ロールバック
                        db.run("ROLLBACK", (errRollback) => {
                            return reject(err);
                        });
                    } else {
                        db.run("COMMIT", (errCommit) => {
                            if (errCommit) {
                                // コミット失敗時もロールバック
                                db.run("ROLLBACK", () => {
                                    return reject(errCommit);
                                });
                            } else {
                                // this.changes: 更新された行数
                                resolve(this.changes);
                            }
                        });
                    }
                }
            );
        });
    });
}

function upsertGeneratedUserKey(user_id, key_data) {
    return new Promise((resolve, reject) => {
        db.run(
            `INSERT INTO generated_user_key (user_id, key_data) VALUES (?, ?)
             ON CONFLICT(user_id) DO UPDATE SET key_data = excluded.key_data`,
            [user_id, key_data],
            function(err) {
                if (err) return reject(err);
                resolve(this.changes);
            }
        );
    });
}


module.exports = {
    initDB,
    getUserByUsername,
    getTaskById,
    insertTask,
    updateTaskStatus,
    upsertGeneratedUserKey,
    db
};