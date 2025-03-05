const Koa = require('koa');
const bodyParser = require('koa-bodyparser');
const dbAPI = require('./db');

const publicRoutes = require('./routes/public');
const internalRoutes = require('./routes/internal');

const app = new Koa();

// DBの初期化
dbAPI.initDB();

app.use(bodyParser());

// 公開ルートをマウント（例: /login, /tasks）
app.use(publicRoutes.routes());
app.use(publicRoutes.allowedMethods());

// 内部ルートをマウント（パスが /internal から始まるもの）
app.use(internalRoutes.routes());
app.use(internalRoutes.allowedMethods());

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
    console.log(`サーバーがポート${PORT}で起動しました`);
});