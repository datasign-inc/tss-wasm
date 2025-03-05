
const jwt = require('jsonwebtoken');

const SECRET_KEY = 'your-secret-key';

const checkAuthHeader = async (ctx, next) => {
    const authHeader = ctx.headers['authorization'];
    if (!authHeader || !authHeader.startsWith('Bearer ')) {
        ctx.status = 401;
        ctx.body = { error: 'AuthorizationヘッダーにBearerトークンが必要です' };
        return;
    }
    const token = authHeader.slice(7); // "Bearer " を除去
    try {
        const decoded = jwt.verify(token, SECRET_KEY);
        ctx.state.user = decoded; // リクエスト内でユーザー情報を共有
        await next();
    } catch (err) {
        ctx.status = 401;
        ctx.body = { error: '無効なトークンです' };
    }
};

module.exports = { checkAuthHeader, SECRET_KEY };