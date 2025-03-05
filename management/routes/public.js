// routes/public.js
const Router = require('koa-router');
const crypto = require('crypto');
const jwt = require('jsonwebtoken');
const dbAPI = require('../db');
const { checkAuthHeader, SECRET_KEY } = require('../util');

const router = new Router();

// POST /login エンドポイント
router.post('/login', async (ctx) => {
    const { username, password } = ctx.request.body;
    if (!username || !password) {
        ctx.status = 400;
        ctx.body = { error: 'usernameとpasswordは必須です' };
        return;
    }

    // パスワードをsha256でハッシュ化
    const hashedPassword = crypto.createHash('sha256').update(password).digest('hex');

    try {
        const user = await dbAPI.getUserByUsername(username);
        if (!user || user.password !== hashedPassword) {
            ctx.status = 401;
            ctx.body = { error: '認証に失敗しました' };
            return;
        }
        // 認証成功時、JWTを発行（有効期限5分、HS256署名）
        const token = jwt.sign(
            { id: user.id, username: user.username },
            SECRET_KEY,
            { algorithm: 'HS256', expiresIn: '5m' }
        );
        ctx.body = { token };
    } catch (err) {
        ctx.status = 500;
        ctx.body = { error: 'データベースエラー' };
    }
});

// POST /tasks エンドポイント（タスク新規作成、JWTチェック付き）
router.post('/tasks', checkAuthHeader, async (ctx) => {
    const task = ctx.request.body;
    if (!task || !task.type || !task.parameters) {
        ctx.status = 400;
        ctx.body = { error: 'タスクのフォーマットが不正です。typeとparametersが必要です。' };
        return;
    }
    const nowUTC = new Date().toISOString();
    try {
        const taskId = await dbAPI.insertTask(task.type, task.parameters, "created", nowUTC, ctx.state.user.id);
        ctx.body = { message: 'タスクが登録されました', taskId };
    } catch (err) {
        ctx.status = 500;
        ctx.body = { error: 'データベースエラー' };
    }
});

module.exports = router;