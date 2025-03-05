// routes/internal.js
const Router = require('koa-router');
const jwt = require('jsonwebtoken');
const dbAPI = require('../db');
const { SECRET_KEY } = require('../util');

const router = new Router();

// GET /internal/tasks/:taskId エンドポイント
router.get('/internal/tasks/:taskId', async (ctx) => {
    const taskId = ctx.params.taskId;
    if (!taskId) {
        ctx.status = 400;
        ctx.body = { error: 'specify task id' };
        return;
    }
    try {
        const task = await dbAPI.getTaskById(taskId);
        if (!task) {
            ctx.status = 404;
            ctx.body = { error: 'Task not found' };
            return;
        }
        ctx.status = 200;
        ctx.body = task; // Koaはオブジェクトを自動的にJSONレスポンスに変換します
    } catch (err) {
        ctx.status = 500;
        ctx.body = { error: 'Database error' };
    }
});

router.patch('/internal/tasks/:taskId/status', async (ctx) => {
    const { taskId } = ctx.params;
    const { status } = ctx.request.body;
    const allowedStatuses = ["created", "processing", "completed", "canceled"];

    if (!status || !allowedStatuses.includes(status)) {
        ctx.status = 400;
        ctx.body = { error: '無効なステータスです。allowed values: "created", "processing", "completed", "canceled"' };
        return;
    }

    try {
        const changes = await dbAPI.updateTaskStatus(taskId, status);
        if (changes === 0) {
            ctx.status = 404;
            ctx.body = { error: 'タスクが見つかりません' };
        } else {
            ctx.body = { message: 'タスクのステータスが更新されました' };
        }
    } catch (err) {
        ctx.status = 500;
        ctx.body = { error: 'タスクの更新中にエラーが発生しました' };
    }
});

// 指定された user_id に対して、HTTP ボディの key_data を upsert します。
router.put('/internal/generated_user_key/:user_id', async (ctx) => {
    const user_id = ctx.params.user_id;
    const { key_data } = ctx.request.body;
    if (!user_id || !key_data) {
        ctx.status = 400;
        ctx.body = { error: 'user_id と key_data は必須です' };
        return;
    }
    try {
        await dbAPI.upsertGeneratedUserKey(user_id, key_data);
        ctx.body = { message: 'generated_user_key がアップサートされました' };
    } catch (err) {
        ctx.status = 500;
        ctx.body = { error: 'データベースエラー' };
    }
});

// POST /internal/check_token エンドポイント
router.post('/internal/check_token', async (ctx) => {
    const body = ctx.request.body;
    const token = body.token;
    if (!token) {
        ctx.status = 400;
        ctx.body = { error: 'specify token' };
        return;
    }
    try {
        jwt.verify(token, SECRET_KEY);
        ctx.body = { result: "valid" };
    } catch (err) {
        ctx.body = { result: "invalid" };
    }
});

module.exports = router;