#![cfg(not(target_arch = "wasm32"))]

use crate::api::{
    gg18_keygen_client_new_context, gg18_keygen_client_round1, gg18_keygen_client_round2,
    gg18_keygen_client_round3, gg18_keygen_client_round4, gg18_keygen_client_round5,
    gg18_sign_client_new_context, gg18_sign_client_round0, gg18_sign_client_round1,
    gg18_sign_client_round2, gg18_sign_client_round3, gg18_sign_client_round4,
    gg18_sign_client_round5, gg18_sign_client_round6, gg18_sign_client_round7,
    gg18_sign_client_round8, gg18_sign_client_round9,
};

// ここから JNI 用のラッパー関数を定義する
use jni::objects::{JClass, JString};
use jni::sys::{jint, jstring};
use jni::JNIEnv;
use std::ptr;
use tokio::runtime::Runtime;

macro_rules! jni_round_wrapper {
    ($jni_name:ident, $rust_fn:path) => {
        #[no_mangle]
        pub extern "system" fn $jni_name(
            mut env: JNIEnv,
            _class: JClass,
            jcontext: JString,
            jdelay: jint,
            jtoken: JString,
        ) -> jstring {
            // JString を Rust の String に変換
            let context: String = env
                .get_string(&jcontext)
                .expect("Invalid context string")
                .into();
            let delay: u32 = jdelay as u32;
            let token: String = env
                .get_string(&jtoken)
                .expect("Invalid token string")
                .into();

            let rt = Runtime::new().unwrap();
            match rt.block_on($rust_fn(context, delay, token)) {
                Ok(result_str) => env
                    .new_string(result_str)
                    .expect("Couldn't create java string")
                    .into_raw(),
                Err(e) => {
                    let _ = env.throw_new("java/lang/RuntimeException", format!("Error: {:?}", e));
                    ptr::null_mut()
                }
            }
        }
    };
}

/// JNIラッパー: com.example.myapplication2.MultiPartyECDSA.gg18KeygenClientNewContext(String, int, int, int)
#[no_mangle]
pub extern "system" fn Java_jp_datasign_bunsin_1wallet_cryptography_multiparty_1ecdsa_GG18RawInterface_gg18KeygenClientNewContext(
    mut env: JNIEnv,
    _class: JClass,
    jaddr: JString,
    jt: jint,
    jn: jint,
    jdelay: jint,
    jtoken: JString,
    jtaskid: JString
) -> jstring {
    // JStringをRustのStringに変換
    let addr: String = env
        .get_string(&jaddr)
        .expect("Invalid address string")
        .into();
    let t: usize = jt as usize;
    let n: usize = jn as usize;
    let delay: u32 = jdelay as u32;
    let token: String = env
        .get_string(&jtoken)
        .expect("Invalid token string")
        .into();
     let taskId: String = env
         .get_string(&jtaskid)
         .expect("Invalid taskId string")
         .into();

    let rt = Runtime::new().unwrap();

    // Rustの関数を呼び出す
    match rt.block_on(gg18_keygen_client_new_context(addr, t, n, delay, token, taskId)) {
        Ok(result_str) => {
            // 結果文字列をJStringに変換して返す
            env.new_string(result_str)
                .expect("Couldn't create java string")
                .into_raw()
        }
        Err(e) => {
            // エラーが発生した場合、Java例外を投げる
            let _ = env.throw_new("java/lang/RuntimeException", format!("Error: {:?}", e));
            ptr::null_mut()
        }
    }
}

/// JNIラッパー: com.example.myapplication2.MultiPartyECDSA.gg18SignClientNewContext(String, int, int, String, String)
#[no_mangle]
pub extern "system" fn Java_jp_datasign_bunsin_1wallet_cryptography_multiparty_1ecdsa_GG18RawInterface_gg18SignClientNewContext(
    mut env: JNIEnv,
    _class: JClass,
    jaddr: JString,
    jt: jint,
    jn: jint,
    jkey_store: JString,
    jmessage: JString,
    jtoken: JString,
    jtaskid: JString
) -> jstring {
    // 各JStringをRustのStringに変換
    let addr: String = env
        .get_string(&jaddr)
        .expect("Invalid address string")
        .into();
    let t: usize = jt as usize;
    let n: usize = jn as usize;
    let key_store: String = env
        .get_string(&jkey_store)
        .expect("Invalid key_store string")
        .into();
    let message: String = env
        .get_string(&jmessage)
        .expect("Invalid message string")
        .into();
    let token: String = env
        .get_string(&jtoken)
        .expect("Invalid token string")
        .into();
     let taskId: String = env
         .get_string(&jtaskid)
         .expect("Invalid taskId string")
         .into();
    let rt = Runtime::new().unwrap();

    // Rustの関数を呼び出す
    match rt.block_on(gg18_sign_client_new_context(addr, t, n, key_store, message, token, taskId)) {
        Ok(result_str) => {
            // 結果文字列をJStringに変換して返す
            env.new_string(result_str)
                .expect("Couldn't create java string")
                .into_raw()
        }
        Err(e) => {
            // エラーが発生した場合、Java例外を投げる
            let _ = env.throw_new("java/lang/RuntimeException", format!("Error: {:?}", e));
            ptr::null_mut()
        }
    }
}

// キー生成系ラッパー関数
jni_round_wrapper!(
    Java_jp_datasign_bunsin_1wallet_cryptography_multiparty_1ecdsa_GG18RawInterface_gg18KeygenClientRound1,
    gg18_keygen_client_round1
);
jni_round_wrapper!(
    Java_jp_datasign_bunsin_1wallet_cryptography_multiparty_1ecdsa_GG18RawInterface_gg18KeygenClientRound2,
    gg18_keygen_client_round2
);
jni_round_wrapper!(
    Java_jp_datasign_bunsin_1wallet_cryptography_multiparty_1ecdsa_GG18RawInterface_gg18KeygenClientRound3,
    gg18_keygen_client_round3
);
jni_round_wrapper!(
    Java_jp_datasign_bunsin_1wallet_cryptography_multiparty_1ecdsa_GG18RawInterface_gg18KeygenClientRound4,
    gg18_keygen_client_round4
);
jni_round_wrapper!(
    Java_jp_datasign_bunsin_1wallet_cryptography_multiparty_1ecdsa_GG18RawInterface_gg18KeygenClientRound5,
    gg18_keygen_client_round5
);

// 署名系ラッパー関数
jni_round_wrapper!(
    Java_jp_datasign_bunsin_1wallet_cryptography_multiparty_1ecdsa_GG18RawInterface_gg18SignClientRound0,
    gg18_sign_client_round0
);
jni_round_wrapper!(
    Java_jp_datasign_bunsin_1wallet_cryptography_multiparty_1ecdsa_GG18RawInterface_gg18SignClientRound1,
    gg18_sign_client_round1
);
jni_round_wrapper!(
    Java_jp_datasign_bunsin_1wallet_cryptography_multiparty_1ecdsa_GG18RawInterface_gg18SignClientRound2,
    gg18_sign_client_round2
);
jni_round_wrapper!(
    Java_jp_datasign_bunsin_1wallet_cryptography_multiparty_1ecdsa_GG18RawInterface_gg18SignClientRound3,
    gg18_sign_client_round3
);
jni_round_wrapper!(
    Java_jp_datasign_bunsin_1wallet_cryptography_multiparty_1ecdsa_GG18RawInterface_gg18SignClientRound4,
    gg18_sign_client_round4
);
jni_round_wrapper!(
    Java_jp_datasign_bunsin_1wallet_cryptography_multiparty_1ecdsa_GG18RawInterface_gg18SignClientRound5,
    gg18_sign_client_round5
);
jni_round_wrapper!(
    Java_jp_datasign_bunsin_1wallet_cryptography_multiparty_1ecdsa_GG18RawInterface_gg18SignClientRound6,
    gg18_sign_client_round6
);
jni_round_wrapper!(
    Java_jp_datasign_bunsin_1wallet_cryptography_multiparty_1ecdsa_GG18RawInterface_gg18SignClientRound7,
    gg18_sign_client_round7
);
jni_round_wrapper!(
    Java_jp_datasign_bunsin_1wallet_cryptography_multiparty_1ecdsa_GG18RawInterface_gg18SignClientRound8,
    gg18_sign_client_round8
);
jni_round_wrapper!(
    Java_jp_datasign_bunsin_1wallet_cryptography_multiparty_1ecdsa_GG18RawInterface_gg18SignClientRound9,
    gg18_sign_client_round9
);
