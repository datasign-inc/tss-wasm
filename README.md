# ビルド・実行方法

## gg18_sm_manager

ビルド

```shell
cargo build --examples --release
```

実行

```shell
./target/release/examples/gg18_sm_manager
```

## libtss_wasm.so

ビルド
※ Android Studioを用いて、予めNDKがインストールされていることが前提
```shell
cargo ndk -t arm64-v8a build --release
ls ./target/aarch64-linux-android/release/libtss_wasm.so
```

## JS向けのpkg

```shell
npm run build_node
```

## 管理機能

```shell
cd ./management
node server.js
```

---

# 以下、オリジナルのREADME

## TSS WASM
A portable lightweight client application for threshold ECDSA (based on [GG18](https://eprint.iacr.org/2019/114.pdf)), built on&for [multi-party-ecdsa](https://github.com/ZenGo-X/multi-party-ecdsa) : 
1) Wasm/Web
2) HW friendly, like [TEE](https://github.com/0xEigenLabs/eigencc)

## Npm publish

* node: npm run build_node
* web: npm run build

### Latest release

web: @ieigen/tss-wasm@0.0.8

nodejs: @ieigen/tss-wasm-node@0.0.7, node 18.0+ is required

## Test

### Unit Test
```
npm run build
npm run test
```

### Function Test via NodeJS
```
cargo build --examples --release
./target/release/examples/gg18_sm_manager

# open another console
npm run build_node
node scripts/multi_keygen_and_sign.js
```

### Function Test via Web

```
cargo build --examples --release
./target/release/examples/gg18_sm_manager

# open another console
npm run build
export NODE_OPTIONS=--openssl-legacy-provider
npm run webpack && npm run webpack-dev-server
```

Open `http://localhost:8080/` in browser, check out the output in `console`.

## Compile SM server by Docker

```
docker build -t ieigen:tss-sm-server --build-arg "BUILDARCH=$(uname -m)" -f sm.dockerfile .
docker run -d -p 8000:8000 -v $PWD/params.json:/tss-wasm/params.json ieigen:tss-sm-server
```

## licence
GPL & Apache-2.0
