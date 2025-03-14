#![cfg(any(target_arch = "wasm32", target_arch = "aarch64"))]
#![allow(non_snake_case)]
use crate::common::{
    aes_decrypt, aes_encrypt, broadcast, check_sig, poll_for_broadcasts, poll_for_p2p, postb,
    public_key_address, sendp2p, PartySignup, AEAD, AES_KEY_BYTES_LEN, TaskRequest
};
use crate::curv::elliptic::curves::traits::{ECPoint, ECScalar};
use crate::curv::{
    arithmetic::num_bigint::BigInt,
    arithmetic::traits::Converter,
    cryptographic_primitives::{
        proofs::sigma_correct_homomorphic_elgamal_enc::HomoELGamalProof,
        proofs::sigma_dlog::DLogProof, secret_sharing::feldman_vss::VerifiableSS,
    },
    elliptic::curves::secp256_k1::{Secp256k1Point as Point, Secp256k1Scalar as Scalar},
};
use crate::errors::Result;
use crate::gg_2018::mta::*;
use crate::gg_2018::party_i::*;
use crate::paillier::EncryptionKey;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GG18KeygenClientContext {
    addr: String,
    params: Parameters,
    party_num_int: u16,
    uuid: String,
    bc1_vec: Option<Vec<KeyGenBroadcastMessage1>>,
    decom_i: Option<KeyGenDecommitMessage1>,
    party_keys: Option<Keys>,
    y_sum: Option<crate::curv::elliptic::curves::secp256_k1::Secp256k1Point>,
    vss_scheme: Option<VerifiableSS>,
    secret_shares: Option<Vec<crate::curv::elliptic::curves::secp256_k1::Secp256k1Scalar>>,
    enc_keys: Option<Vec<Vec<u8>>>,
    party_shares: Option<Vec<Scalar>>,
    point_vec: Option<Vec<Point>>,
    dlog_proof: Option<DLogProof>,
    shared_keys: Option<SharedKeys>,
    vss_scheme_vec: Option<Vec<VerifiableSS>>,
    public_key_address: Option<String>,
}

fn new_client_with_headers(token: &str) -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("Content-Type:application/json; charset=utf-8"),
    );
    headers.insert(
        "Accept",
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    );

    Ok(reqwest::Client::builder()
        .default_headers(headers)
        .build()?)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn gg18_keygen_client_new_context(
    addr: String,
    t: usize,
    n: usize,
    _delay: u32,
    token: String,
    task_id: String,
    party_type: String
) -> Result<String> {
    let client = new_client_with_headers(&token)?;
    let params = Parameters {
        threshold: t,
        share_count: n,
    };

    let (party_num_int, uuid) = match signup_keygen(&client, &addr, &task_id, &party_type).await? {
        PartySignup { number, uuid } => (number, uuid),
    };

    Ok(serde_json::to_string(&GG18KeygenClientContext {
        addr,
        params,
        party_num_int,
        uuid,
        bc1_vec: None,
        decom_i: None,
        party_keys: None,
        y_sum: None,
        vss_scheme: None,
        secret_shares: None,
        enc_keys: None,
        party_shares: None,
        point_vec: None,
        dlog_proof: None,
        shared_keys: None,
        vss_scheme_vec: None,
        public_key_address: None,
    })?)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn gg18_keygen_client_round1(context: String, delay: u32, token: String) -> Result<String> {
    let mut context = serde_json::from_str::<GG18KeygenClientContext>(&context)?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("Failed to build client");

    let party_keys = Keys::create(context.party_num_int as usize);
    let (bc_i, decom_i) = party_keys.phase1_broadcast_phase3_proof_of_correct_key();

    broadcast(
        &client,
        &context.addr,
        context.party_num_int,
        "round1",
        serde_json::to_string(&bc_i)?,
        context.uuid.clone(),
    )
    .await?;

    let round1_ans_vec = poll_for_broadcasts(
        &client,
        &context.addr,
        context.party_num_int,
        context.params.share_count as u16,
        "round1",
        context.uuid.clone(),
        delay,
    )
    .await?;

    let mut bc1_vec: Vec<_> = round1_ans_vec
        .into_iter()
        .map(|m| serde_json::from_str::<KeyGenBroadcastMessage1>(&m))
        .collect::<std::result::Result<Vec<KeyGenBroadcastMessage1>, serde_json::Error>>()?;

    bc1_vec.insert(context.party_num_int as usize - 1, bc_i);

    context.bc1_vec = Some(bc1_vec);
    context.party_keys = Some(party_keys);
    context.decom_i = Some(decom_i);

    Ok(serde_json::to_string(&context)?)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn gg18_keygen_client_round2(context: String, delay: u32, token: String) -> Result<String> {
    let mut context = serde_json::from_str::<GG18KeygenClientContext>(&context)?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("Failed to build client");

    // send ephemeral public keys and check commitments correctness
    broadcast(
        &client,
        &context.addr,
        context.party_num_int,
        "round2",
        serde_json::to_string(&context.decom_i.as_ref().unwrap())?,
        context.uuid.clone(),
    )
    .await?;

    let round2_ans_vec = poll_for_broadcasts(
        &client,
        &context.addr,
        context.party_num_int,
        context.params.share_count as u16,
        "round2",
        context.uuid.clone(),
        delay,
    )
    .await?;

    let mut j = 0;
    let mut point_vec: Vec<Point> = Vec::new();
    let mut decom_vec: Vec<KeyGenDecommitMessage1> = Vec::new();
    let mut enc_keys: Vec<Vec<u8>> = Vec::new();
    for i in 1..=context.params.share_count as u16 {
        if i == context.party_num_int {
            point_vec.push(context.decom_i.as_ref().unwrap().y_i.clone());
            decom_vec.push(context.decom_i.as_ref().unwrap().clone());
        } else {
            let decom_j: KeyGenDecommitMessage1 = serde_json::from_str(&round2_ans_vec[j])?;
            point_vec.push(decom_j.y_i.clone());
            decom_vec.push(decom_j.clone());
            let key_bn: BigInt = (decom_j.y_i.clone()
                * context.party_keys.as_ref().unwrap().u_i.clone())
            .x_coor()
            .unwrap();
            let key_bytes = BigInt::to_vec(&key_bn);
            let mut template: Vec<u8> = vec![0u8; AES_KEY_BYTES_LEN - key_bytes.len()];
            template.extend_from_slice(&key_bytes[..]);
            enc_keys.push(template);
            j += 1;
        }
    }

    let (head, tail) = point_vec.split_at(1);
    let y_sum = tail.iter().fold(head[0].clone(), |acc, x| acc + x);

    let (vss_scheme, secret_shares, _index) = context
        .party_keys
        .as_ref()
        .unwrap()
        .phase1_verify_com_phase3_verify_correct_key_phase2_distribute(
            &context.params,
            &decom_vec,
            &(context.bc1_vec.as_ref().unwrap()),
        )?;

    context.y_sum = Some(y_sum.clone());
    let pubkey_a = y_sum.get_element().serialize();
    let pubkey = secp256k1::PublicKey::parse(&pubkey_a)?;
    context.public_key_address = Some(hex::encode(public_key_address(&pubkey)));
    context.vss_scheme = Some(vss_scheme);
    context.secret_shares = Some(secret_shares);
    context.enc_keys = Some(enc_keys);
    context.point_vec = Some(point_vec);

    Ok(serde_json::to_string(&context)?)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn gg18_keygen_client_round3(context: String, delay: u32, token: String) -> Result<String> {
    let mut context = serde_json::from_str::<GG18KeygenClientContext>(&context)?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("Failed to build client");

    let mut j = 0;
    for (k, i) in (1..=context.params.share_count as u16).enumerate() {
        if i != context.party_num_int {
            // prepare encrypted ss for party i:
            let key_i = &context.enc_keys.as_ref().unwrap()[j];
            let plaintext =
                BigInt::to_vec(&context.secret_shares.as_ref().unwrap()[k].to_big_int());
            let aead_pack_i = aes_encrypt(key_i, &plaintext)?;
            sendp2p(
                &client,
                &context.addr,
                context.party_num_int,
                i,
                "round3",
                serde_json::to_string(&aead_pack_i)?,
                context.uuid.clone(),
            )
            .await?;
            j += 1;
        }
    }

    let round3_ans_vec = poll_for_p2p(
        &client,
        &context.addr,
        context.party_num_int,
        context.params.share_count as u16,
        delay,
        "round3",
        context.uuid.clone(),
    )
    .await?;

    let mut j = 0;
    let mut party_shares: Vec<Scalar> = Vec::new();
    for i in 1..=context.params.share_count as u16 {
        if i == context.party_num_int {
            party_shares.push(context.secret_shares.as_ref().unwrap()[(i - 1) as usize].clone());
        } else {
            let aead_pack: AEAD = serde_json::from_str(&round3_ans_vec[j]).unwrap();
            let key_i = &context.enc_keys.as_ref().unwrap()[j];
            let out = aes_decrypt(key_i, aead_pack)?;
            let out_bn = BigInt::from_bytes_be(&out[..]);
            let out_fe = ECScalar::from(&out_bn);
            party_shares.push(out_fe);

            j += 1;
        }
    }

    context.party_shares = Some(party_shares);

    Ok(serde_json::to_string(&context)?)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn gg18_keygen_client_round4(context: String, delay: u32, token: String) -> Result<String> {
    let mut context = serde_json::from_str::<GG18KeygenClientContext>(&context)?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("Failed to build client");
    broadcast(
        &client,
        &context.addr,
        context.party_num_int,
        "round4",
        serde_json::to_string(&context.vss_scheme.as_ref().unwrap())?,
        context.uuid.clone(),
    )
    .await?;
    let round4_ans_vec = poll_for_broadcasts(
        &client,
        &context.addr,
        context.party_num_int,
        context.params.share_count as u16,
        "round4",
        context.uuid.clone(),
        delay,
    )
    .await?;

    let mut j = 0;
    let mut vss_scheme_vec: Vec<VerifiableSS> = Vec::new();
    for i in 1..=context.params.share_count as u16 {
        if i == context.party_num_int {
            vss_scheme_vec.push(context.vss_scheme.as_ref().unwrap().clone());
        } else {
            let vss_scheme_j: VerifiableSS = serde_json::from_str(&round4_ans_vec[j]).unwrap();
            vss_scheme_vec.push(vss_scheme_j);
            j += 1;
        }
    }

    let (shared_keys, dlog_proof) = context
        .party_keys
        .as_ref()
        .unwrap()
        .phase2_verify_vss_construct_keypair_phase3_pok_dlog(
            &context.params,
            &context.point_vec.as_ref().unwrap(),
            &context.party_shares.as_ref().unwrap(),
            &vss_scheme_vec,
            &(context.party_num_int.clone() as usize), // FIXME
        )?;

    context.shared_keys = Some(shared_keys);
    context.dlog_proof = Some(dlog_proof);
    context.vss_scheme_vec = Some(vss_scheme_vec);

    Ok(serde_json::to_string(&context)?)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn gg18_keygen_client_round5(context: String, delay: u32, token: String) -> Result<String> {
    let context = serde_json::from_str::<GG18KeygenClientContext>(&context)?;
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("Failed to build client");
    broadcast(
        &client,
        &context.addr,
        context.party_num_int,
        "round5",
        serde_json::to_string(&context.dlog_proof.as_ref().unwrap())?,
        context.uuid.clone(),
    )
    .await?;
    let round5_ans_vec = poll_for_broadcasts(
        &client,
        &context.addr,
        context.party_num_int,
        context.params.share_count as u16,
        "round5",
        context.uuid.clone(),
        delay,
    )
    .await?;

    let mut j = 0;
    let mut dlog_proof_vec: Vec<DLogProof> = Vec::new();
    for i in 1..=context.params.share_count as u16 {
        if i == context.party_num_int {
            dlog_proof_vec.push(context.dlog_proof.as_ref().unwrap().clone());
        } else {
            let dlog_proof_j: DLogProof = serde_json::from_str(&round5_ans_vec[j])?;
            dlog_proof_vec.push(dlog_proof_j);
            j += 1;
        }
    }
    Keys::verify_dlog_proofs(
        &context.params,
        &dlog_proof_vec,
        &context.point_vec.as_ref().unwrap(),
    )?;

    //save key to file:
    let paillier_key_vec = (0..context.params.share_count as u16)
        .map(|i| context.bc1_vec.as_ref().unwrap()[i as usize].e.clone())
        .collect::<Vec<EncryptionKey>>();

    let keygen_json = serde_json::to_string(&(
        context.party_keys.as_ref().unwrap(),
        context.shared_keys.as_ref().unwrap(),
        context.party_num_int,
        context.vss_scheme_vec.as_ref().unwrap(),
        paillier_key_vec,
        context.y_sum.as_ref().unwrap(),
    ))?;

    Ok(keygen_json)
}

pub async fn signup_keygen(client: &Client, addr: &str, task_id: &str, party_type: &str) -> Result<PartySignup> {
    let request = TaskRequest {
        task_id: task_id.to_string(),
        party_type: party_type.to_string(),
    };
    let res_body = postb(client, addr, "signupkeygen", request).await?;
    let u: std::result::Result<PartySignup, ()> = serde_json::from_str(&res_body)?;
    Ok(u.unwrap())
}

pub async fn signup_sign(client: &Client, addr: &str, task_id: &str, party_type: &str) -> Result<PartySignup> {
    let request = TaskRequest {
        task_id: task_id.to_string(),
        party_type: party_type.to_string(),
    };
    let res_body = postb(client, addr, "signupsign", request).await?;
    let u: std::result::Result<PartySignup, ()> = serde_json::from_str(&res_body)?;
    Ok(u.unwrap())
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GG18SignClientContext {
    addr: String,
    party_keys: Keys,
    shared_keys: SharedKeys,
    party_id: u16,
    vss_scheme_vec: Vec<VerifiableSS>,
    paillier_key_vector: Vec<EncryptionKey>,
    y_sum: Point,
    threshould: u16,
    party_num_int: u16,
    uuid: String,
    sign_keys: Option<SignKeys>,
    com: Option<SignBroadcastPhase1>,
    decommit: Option<SignDecommitPhase1>,
    round1_ans_vec: Option<Vec<String>>,
    signers_vec: Option<Vec<usize>>,
    round2_ans_vec: Option<Vec<String>>,
    xi_com_vec: Option<Vec<crate::curv::elliptic::curves::secp256_k1::Secp256k1Point>>,
    beta_vec: Option<Vec<Scalar>>,
    ni_vec: Option<Vec<Scalar>>,
    bc1_vec: Option<Vec<SignBroadcastPhase1>>,
    m_b_gamma_rec_vec: Option<Vec<MessageB>>,
    delta_inv: Option<crate::curv::elliptic::curves::secp256_k1::Secp256k1Scalar>,
    sigma: Option<crate::curv::elliptic::curves::secp256_k1::Secp256k1Scalar>,
    message: Vec<u8>,
    phase5_com: Option<Phase5Com1>,
    phase_5a_decom: Option<Phase5ADecom1>,
    helgamal_proof: Option<HomoELGamalProof>,
    dlog_proof_rho: Option<DLogProof>,
    commit5a_vec: Option<Vec<Phase5Com1>>,
    local_sig: Option<LocalSignature>,
    r: Option<crate::curv::elliptic::curves::secp256_k1::Secp256k1Point>,
    phase5_com2: Option<Phase5Com2>,
    phase_5d_decom2: Option<Phase5DDecom2>,
    decommit5a_and_elgamal_and_dlog_vec: Option<Vec<(Phase5ADecom1, HomoELGamalProof, DLogProof)>>,
    decommit5a_and_elgamal_and_dlog_vec_includes_i:
        Option<Vec<(Phase5ADecom1, HomoELGamalProof, DLogProof)>>,
    s_i: Option<crate::curv::elliptic::curves::secp256_k1::Secp256k1Scalar>,
    commit5c_vec: Option<Vec<Phase5Com2>>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn gg18_sign_client_new_context(
    addr: String,
    t: usize,
    _n: usize,
    key_store: String,
    message_str: String,
    token: String,
    task_id: String,
    party_type: String
) -> Result<String> {
    let message = match hex::decode(message_str.clone()) {
        Ok(x) => x,
        Err(_e) => message_str.as_bytes().to_vec(),
    };
    // let message = &message[..];
    let client = new_client_with_headers(&token)?;

    let (party_keys, shared_keys, party_id, vss_scheme_vec, paillier_key_vector, y_sum): (
        Keys,
        SharedKeys,
        u16,
        Vec<VerifiableSS>,
        Vec<EncryptionKey>,
        Point,
    ) = serde_json::from_str(&key_store)?;

    //signup:
    let (party_num_int, uuid) = match signup_sign(&client, &addr, &task_id, &party_type).await? {
        PartySignup { number, uuid } => (number, uuid),
    };

    Ok(serde_json::to_string(&GG18SignClientContext {
        addr,
        party_keys,
        shared_keys,
        party_id,
        vss_scheme_vec,
        paillier_key_vector,
        y_sum,
        threshould: t as u16,
        party_num_int,
        uuid,
        sign_keys: None,
        com: None,
        decommit: None,
        round1_ans_vec: None,
        signers_vec: None,
        round2_ans_vec: None,
        xi_com_vec: None,
        beta_vec: None,
        ni_vec: None,
        bc1_vec: None,
        m_b_gamma_rec_vec: None,
        delta_inv: None,
        message, // TODO: The message is plain now
        sigma: None,
        phase5_com: None,
        phase_5a_decom: None,
        helgamal_proof: None,
        dlog_proof_rho: None,
        commit5a_vec: None,
        local_sig: None,
        r: None,
        phase5_com2: None,
        phase_5d_decom2: None,
        decommit5a_and_elgamal_and_dlog_vec: None,
        decommit5a_and_elgamal_and_dlog_vec_includes_i: None,
        s_i: None,
        commit5c_vec: None,
    })?)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn gg18_sign_client_round0(context: String, delay: u32, token: String) -> Result<String> {
    let mut context = serde_json::from_str::<GG18SignClientContext>(&context)?;
    let client = new_client_with_headers(&token)?;
    // round 0: collect signers IDs
    broadcast(
        &client,
        &context.addr,
        context.party_num_int,
        "round0",
        serde_json::to_string(&context.party_id)?,
        context.uuid.clone(),
    )
    .await?;
    let round0_ans_vec = poll_for_broadcasts(
        &client,
        &context.addr,
        context.party_num_int,
        context.threshould + 1,
        "round0",
        context.uuid.clone(),
        delay,
    )
    .await?;

    let mut j = 0;
    let mut signers_vec: Vec<usize> = Vec::new();
    for i in 1..=context.threshould + 1 {
        if i == context.party_num_int {
            signers_vec.push((context.party_id - 1).into());
        } else {
            let signer_j: u16 = serde_json::from_str(&round0_ans_vec[j])?;
            signers_vec.push((signer_j - 1).into());
            j += 1;
        }
    }

    let private =
        PartyPrivate::set_private(context.party_keys.clone(), context.shared_keys.clone());

    let sign_keys = SignKeys::create(
        &private,
        &context.vss_scheme_vec[usize::from(signers_vec[usize::from(context.party_num_int - 1)])],
        signers_vec[usize::from(context.party_num_int - 1)].into(),
        &signers_vec,
    );

    let xi_com_vec = Keys::get_commitments_to_xi(&context.vss_scheme_vec);

    context.sign_keys = Some(sign_keys);
    context.signers_vec = Some(signers_vec);
    context.xi_com_vec = Some(xi_com_vec);

    Ok(serde_json::to_string(&context)?)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn gg18_sign_client_round1(context: String, delay: u32, token: String) -> Result<String> {
    let mut context = serde_json::from_str::<GG18SignClientContext>(&context)?;
    let client = new_client_with_headers(&token)?;
    let (com, decommit) = context.sign_keys.as_ref().unwrap().phase1_broadcast();
    let (m_a_k, _) = MessageA::a(
        &context.sign_keys.as_ref().unwrap().k_i,
        &context.party_keys.ek,
        &[],
    );
    broadcast(
        &client,
        &context.addr,
        context.party_num_int,
        "round1",
        serde_json::to_string(&(com.clone(), m_a_k))?,
        context.uuid.clone(),
    )
    .await?;
    let round1_ans_vec = poll_for_broadcasts(
        &client,
        &context.addr,
        context.party_num_int,
        context.threshould + 1,
        "round1",
        context.uuid.clone(),
        delay,
    )
    .await?;

    context.com = Some(com);
    context.decommit = Some(decommit);
    context.round1_ans_vec = Some(round1_ans_vec);

    Ok(serde_json::to_string(&context)?)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn gg18_sign_client_round2(context: String, delay: u32, token: String) -> Result<String> {
    let mut context = serde_json::from_str::<GG18SignClientContext>(&context)?;
    let client = new_client_with_headers(&token)?;
    let mut j = 0;
    let mut bc1_vec: Vec<SignBroadcastPhase1> = Vec::new();
    let mut m_a_vec: Vec<MessageA> = Vec::new();

    for i in 1..context.threshould + 2 {
        if i == context.party_num_int {
            bc1_vec.push(context.com.as_ref().unwrap().clone());
        //   m_a_vec.push(m_a_k.clone());
        } else {
            //     if signers_vec.contains(&(i as usize)) {
            let (bc1_j, m_a_party_j): (SignBroadcastPhase1, MessageA) =
                serde_json::from_str(&context.round1_ans_vec.as_ref().unwrap()[j])?;
            bc1_vec.push(bc1_j);
            m_a_vec.push(m_a_party_j);

            j += 1;
            //       }
        }
    }
    assert_eq!(context.signers_vec.as_ref().unwrap().len(), bc1_vec.len());

    //////////////////////////////////////////////////////////////////////////////
    let mut m_b_gamma_send_vec: Vec<MessageB> = Vec::new();
    let mut beta_vec: Vec<Scalar> = Vec::new();
    let mut m_b_w_send_vec: Vec<MessageB> = Vec::new();
    let mut ni_vec: Vec<Scalar> = Vec::new();
    let mut j = 0;
    for i in 1..context.threshould + 2 {
        if i != context.party_num_int {
            let (m_b_gamma, beta_gamma, _, _) = MessageB::b(
                &context.sign_keys.as_ref().unwrap().gamma_i,
                &context.paillier_key_vector
                    [usize::from(context.signers_vec.as_ref().unwrap()[usize::from(i - 1)])],
                m_a_vec[j].clone(),
                &[],
            )
            .unwrap();
            let (m_b_w, beta_wi, _, _) = MessageB::b(
                &context.sign_keys.as_ref().unwrap().w_i,
                &context.paillier_key_vector
                    [usize::from(context.signers_vec.as_ref().unwrap()[usize::from(i - 1)])],
                m_a_vec[j].clone(),
                &[],
            )
            .unwrap();
            m_b_gamma_send_vec.push(m_b_gamma);
            m_b_w_send_vec.push(m_b_w);
            beta_vec.push(beta_gamma);
            ni_vec.push(beta_wi);
            j += 1;
        }
    }

    let mut j = 0;
    for i in 1..context.threshould + 2 {
        if i != context.party_num_int {
            sendp2p(
                &client,
                &context.addr,
                context.party_num_int,
                i,
                "round2",
                serde_json::to_string(&(m_b_gamma_send_vec[j].clone(), m_b_w_send_vec[j].clone()))?,
                context.uuid.clone(),
            )
            .await?;
            j += 1;
        }
    }

    let round2_ans_vec = poll_for_p2p(
        &client,
        &context.addr,
        context.party_num_int,
        context.threshould + 1,
        delay,
        "round2",
        context.uuid.clone(),
    )
    .await?;

    context.round2_ans_vec = Some(round2_ans_vec);
    context.beta_vec = Some(beta_vec);
    context.ni_vec = Some(ni_vec);
    context.bc1_vec = Some(bc1_vec);

    Ok(serde_json::to_string(&context)?)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn gg18_sign_client_round3(context: String, delay: u32, token: String) -> Result<String> {
    let mut context = serde_json::from_str::<GG18SignClientContext>(&context)?;
    let client = new_client_with_headers(&token)?;
    let mut m_b_gamma_rec_vec: Vec<MessageB> = Vec::new();
    let mut m_b_w_rec_vec: Vec<MessageB> = Vec::new();

    for i in 0..context.threshould {
        //  if signers_vec.contains(&(i as usize)) {
        let (m_b_gamma_i, m_b_w_i): (MessageB, MessageB) =
            serde_json::from_str(&context.round2_ans_vec.as_ref().unwrap()[i as usize])?;
        m_b_gamma_rec_vec.push(m_b_gamma_i);
        m_b_w_rec_vec.push(m_b_w_i);
        //     }
    }

    let mut alpha_vec: Vec<Scalar> = Vec::new();
    let mut miu_vec: Vec<Scalar> = Vec::new();

    let mut j = 0;
    for i in 1..context.threshould + 2 {
        if i != context.party_num_int {
            let m_b = m_b_gamma_rec_vec[j].clone();

            let alpha_ij_gamma = m_b.verify_proofs_get_alpha(
                &context.party_keys.dk,
                &context.sign_keys.as_ref().unwrap().k_i,
            )?;
            let m_b = m_b_w_rec_vec[j].clone();
            let alpha_ij_wi = m_b.verify_proofs_get_alpha(
                &context.party_keys.dk,
                &context.sign_keys.as_ref().unwrap().k_i,
            )?;
            alpha_vec.push(alpha_ij_gamma.0);
            miu_vec.push(alpha_ij_wi.0);
            let g_w_i = Keys::update_commitments_to_xi(
                &context.xi_com_vec.as_ref().unwrap()
                    [usize::from(context.signers_vec.as_ref().unwrap()[usize::from(i - 1)])],
                &context.vss_scheme_vec
                    [usize::from(context.signers_vec.as_ref().unwrap()[usize::from(i - 1)])],
                context.signers_vec.as_ref().unwrap()[usize::from(i - 1)],
                &context.signers_vec.as_ref().unwrap(),
            );
            assert_eq!(m_b.b_proof.pk, g_w_i);
            j += 1;
        }
    }
    //////////////////////////////////////////////////////////////////////////////
    let delta_i = context
        .sign_keys
        .as_ref()
        .unwrap()
        .phase2_delta_i(&alpha_vec, &context.beta_vec.as_ref().unwrap());
    let sigma = context
        .sign_keys
        .as_ref()
        .unwrap()
        .phase2_sigma_i(&miu_vec, &context.ni_vec.as_ref().unwrap());

    broadcast(
        &client,
        &context.addr,
        context.party_num_int,
        "round3",
        serde_json::to_string(&delta_i)?,
        context.uuid.clone(),
    )
    .await?;
    let round3_ans_vec = poll_for_broadcasts(
        &client,
        &context.addr,
        context.party_num_int,
        context.threshould + 1,
        "round3",
        context.uuid.clone(),
        delay,
    )
    .await?;
    let mut delta_vec: Vec<Scalar> = Vec::new();
    format_vec_from_reads(
        &round3_ans_vec,
        context.party_num_int as usize,
        delta_i,
        &mut delta_vec,
    )?;
    let delta_inv = SignKeys::phase3_reconstruct_delta(&delta_vec);

    context.m_b_gamma_rec_vec = Some(m_b_gamma_rec_vec);
    context.delta_inv = Some(delta_inv);
    context.sigma = Some(sigma);

    Ok(serde_json::to_string(&context)?)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn gg18_sign_client_round4(context: String, delay: u32, token: String) -> Result<String> {
    let mut context = serde_json::from_str::<GG18SignClientContext>(&context)?;
    let client = new_client_with_headers(&token)?;
    // decommit to gamma_i
    broadcast(
        &client,
        &context.addr,
        context.party_num_int,
        "round4",
        serde_json::to_string(&context.decommit.as_ref().unwrap())?,
        context.uuid.clone(),
    )
    .await?;
    let round4_ans_vec = poll_for_broadcasts(
        &client,
        &context.addr,
        context.party_num_int,
        context.threshould + 1,
        "round4",
        context.uuid.clone(),
        delay,
    )
    .await?;

    let mut decommit_vec: Vec<SignDecommitPhase1> = Vec::new();
    format_vec_from_reads(
        &round4_ans_vec,
        context.party_num_int as usize,
        context.decommit.clone().unwrap(),
        &mut decommit_vec,
    )?;

    let decomm_i = decommit_vec.remove(usize::from(context.party_num_int - 1));
    let _ = &context
        .bc1_vec
        .as_mut()
        .unwrap()
        .remove(usize::from(context.party_num_int - 1));
    let b_proof_vec = (0..context.m_b_gamma_rec_vec.as_ref().unwrap().len())
        .map(|i| &context.m_b_gamma_rec_vec.as_ref().unwrap()[i].b_proof)
        .collect::<Vec<&DLogProof>>();

    let R = SignKeys::phase4(
        &context.delta_inv.as_ref().unwrap(),
        &b_proof_vec,
        decommit_vec,
        &context.bc1_vec.as_ref().unwrap(),
    )?;

    // adding local g_gamma_i
    let R = R + decomm_i.g_gamma_i * context.delta_inv.as_ref().unwrap();

    // we assume the message is already hashed (by the signer).
    let message = &context.message[..];
    let message_bn = BigInt::from_bytes_be(message);
    let local_sig = LocalSignature::phase5_local_sig(
        &context.sign_keys.as_ref().unwrap().k_i,
        &message_bn,
        &R,
        &context.sigma.as_ref().unwrap(),
        &context.y_sum,
    );

    let (phase5_com, phase_5a_decom, helgamal_proof, dlog_proof_rho) =
        local_sig.phase5a_broadcast_5b_zkproof();

    context.phase5_com = Some(phase5_com);
    context.phase_5a_decom = Some(phase_5a_decom);
    context.helgamal_proof = Some(helgamal_proof);
    context.dlog_proof_rho = Some(dlog_proof_rho);
    context.local_sig = Some(local_sig);
    context.r = Some(R);

    Ok(serde_json::to_string(&context)?)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn gg18_sign_client_round5(context: String, delay: u32, token: String) -> Result<String> {
    let mut context = serde_json::from_str::<GG18SignClientContext>(&context)?;
    let client = new_client_with_headers(&token)?;
    //phase (5A)  broadcast commit
    broadcast(
        &client,
        &context.addr,
        context.party_num_int,
        "round5",
        serde_json::to_string(&context.phase5_com.as_ref().unwrap())?,
        context.uuid.clone(),
    )
    .await?;
    let round5_ans_vec = poll_for_broadcasts(
        &client,
        &context.addr,
        context.party_num_int,
        context.threshould + 1,
        "round5",
        context.uuid.clone(),
        delay,
    )
    .await?;

    let mut commit5a_vec: Vec<Phase5Com1> = Vec::new();
    format_vec_from_reads(
        &round5_ans_vec,
        context.party_num_int as usize,
        context.phase5_com.clone().unwrap(),
        &mut commit5a_vec,
    )?;

    context.commit5a_vec = Some(commit5a_vec);

    Ok(serde_json::to_string(&context)?)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn gg18_sign_client_round6(context: String, delay: u32, token: String) -> Result<String> {
    let mut context = serde_json::from_str::<GG18SignClientContext>(&context)?;
    let client = new_client_with_headers(&token)?;
    //phase (5B)  broadcast decommit and (5B) ZK proof
    broadcast(
        &client,
        &context.addr,
        context.party_num_int,
        "round6",
        serde_json::to_string(&(
            context.phase_5a_decom.clone().unwrap(),
            context.helgamal_proof.clone().unwrap(),
            context.dlog_proof_rho.clone().unwrap(),
        ))?,
        context.uuid.clone(),
    )
    .await?;
    let round6_ans_vec = poll_for_broadcasts(
        &client,
        &context.addr,
        context.party_num_int,
        context.threshould + 1,
        "round6",
        context.uuid.clone(),
        delay,
    )
    .await?;

    let mut decommit5a_and_elgamal_and_dlog_vec: Vec<(Phase5ADecom1, HomoELGamalProof, DLogProof)> =
        Vec::new();
    format_vec_from_reads(
        &round6_ans_vec,
        context.party_num_int as usize,
        (
            context.phase_5a_decom.clone().unwrap(),
            context.helgamal_proof.clone().unwrap(),
            context.dlog_proof_rho.clone().unwrap(),
        ),
        &mut decommit5a_and_elgamal_and_dlog_vec,
    )?;
    let decommit5a_and_elgamal_and_dlog_vec_includes_i =
        decommit5a_and_elgamal_and_dlog_vec.clone();
    decommit5a_and_elgamal_and_dlog_vec.remove(usize::from(context.party_num_int - 1));
    context
        .commit5a_vec
        .as_mut()
        .unwrap()
        .remove(usize::from(context.party_num_int - 1));
    let phase_5a_decomm_vec = (0..context.threshould)
        .map(|i| decommit5a_and_elgamal_and_dlog_vec[i as usize].0.clone())
        .collect::<Vec<Phase5ADecom1>>();
    let phase_5a_elgamal_vec = (0..context.threshould)
        .map(|i| decommit5a_and_elgamal_and_dlog_vec[i as usize].1.clone())
        .collect::<Vec<HomoELGamalProof>>();
    let phase_5a_dlog_vec = (0..context.threshould)
        .map(|i| decommit5a_and_elgamal_and_dlog_vec[i as usize].2.clone())
        .collect::<Vec<DLogProof>>();
    let (phase5_com2, phase_5d_decom2) = context.local_sig.clone().unwrap().phase5c(
        &phase_5a_decomm_vec,
        &context.commit5a_vec.as_ref().unwrap(),
        &phase_5a_elgamal_vec,
        &phase_5a_dlog_vec,
        &context.phase_5a_decom.as_ref().unwrap().V_i,
        &context.r.as_ref().unwrap(),
    )?;

    context.phase5_com2 = Some(phase5_com2);
    context.phase_5d_decom2 = Some(phase_5d_decom2);
    context.decommit5a_and_elgamal_and_dlog_vec_includes_i =
        Some(decommit5a_and_elgamal_and_dlog_vec_includes_i);

    Ok(serde_json::to_string(&context)?)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn gg18_sign_client_round7(context: String, delay: u32, token: String) -> Result<String> {
    let mut context = serde_json::from_str::<GG18SignClientContext>(&context)?;
    let client = new_client_with_headers(&token)?;
    //////////////////////////////////////////////////////////////////////////////
    broadcast(
        &client,
        &context.addr,
        context.party_num_int,
        "round7",
        serde_json::to_string(&context.phase5_com2.as_ref().unwrap())?,
        context.uuid.clone(),
    )
    .await?;
    let round7_ans_vec = poll_for_broadcasts(
        &client,
        &context.addr,
        context.party_num_int,
        context.threshould + 1,
        "round7",
        context.uuid.clone(),
        delay,
    )
    .await?;

    let mut commit5c_vec: Vec<Phase5Com2> = Vec::new();
    format_vec_from_reads(
        &round7_ans_vec,
        context.party_num_int as usize,
        context.phase5_com2.clone().unwrap(),
        &mut commit5c_vec,
    )?;

    context.commit5c_vec = Some(commit5c_vec);

    Ok(serde_json::to_string(&context)?)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn gg18_sign_client_round8(context: String, delay: u32, token: String) -> Result<String> {
    let mut context = serde_json::from_str::<GG18SignClientContext>(&context)?;
    let client = new_client_with_headers(&token)?;
    //phase (5B)  broadcast decommit and (5B) ZK proof
    broadcast(
        &client,
        &context.addr,
        context.party_num_int,
        "round8",
        serde_json::to_string(&context.phase_5d_decom2.as_ref().unwrap())?,
        context.uuid.clone(),
    )
    .await?;
    let round8_ans_vec = poll_for_broadcasts(
        &client,
        &context.addr,
        context.party_num_int,
        context.threshould + 1,
        "round8",
        context.uuid.clone(),
        delay,
    )
    .await?;

    let mut decommit5d_vec: Vec<Phase5DDecom2> = Vec::new();
    format_vec_from_reads(
        &round8_ans_vec,
        context.party_num_int as usize,
        context.phase_5d_decom2.clone().unwrap(),
        &mut decommit5d_vec,
    )?;

    let phase_5a_decomm_vec_includes_i = (0..=context.threshould)
        .map(|i| {
            context
                .decommit5a_and_elgamal_and_dlog_vec_includes_i
                .clone()
                .unwrap()[i as usize]
                .0
                .clone()
        })
        .collect::<Vec<Phase5ADecom1>>();
    let s_i = context.local_sig.clone().unwrap().phase5d(
        &decommit5d_vec,
        &context.commit5c_vec.as_ref().unwrap(),
        &phase_5a_decomm_vec_includes_i,
    )?;

    context.s_i = Some(s_i);

    Ok(serde_json::to_string(&context)?)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn gg18_sign_client_round9(context: String, delay: u32, token: String) -> Result<String> {
    let context = serde_json::from_str::<GG18SignClientContext>(&context)?;
    let client = new_client_with_headers(&token)?;
    //////////////////////////////////////////////////////////////////////////////
    broadcast(
        &client,
        &context.addr,
        context.party_num_int,
        "round9",
        serde_json::to_string(&context.s_i.as_ref().unwrap())?,
        context.uuid.clone(),
    )
    .await?;
    let round9_ans_vec = poll_for_broadcasts(
        &client,
        &context.addr,
        context.party_num_int,
        context.threshould + 1,
        "round9",
        context.uuid.clone(),
        delay,
    )
    .await?;

    let mut s_i_vec: Vec<Scalar> = Vec::new();
    format_vec_from_reads(
        &round9_ans_vec,
        context.party_num_int as usize,
        context.s_i.unwrap(),
        &mut s_i_vec,
    )?;

    s_i_vec.remove(usize::from(context.party_num_int - 1));
    let sig = context
        .local_sig
        .clone()
        .unwrap()
        .output_signature(&s_i_vec)?;

    let sign_json = serde_json::to_string(&vec![
        //"r",
        sig.r.to_big_int().to_hex(),
        //"s",
        sig.s.to_big_int().to_hex(),
        //"v"
        sig.recid.to_string(),
    ])?;
    // crate::console_log!("sign_json: {:?}", sign_json);

    check_sig(
        &sig.r,
        &sig.s,
        &context.local_sig.clone().unwrap().m,
        &context.y_sum.clone(),
    )?;

    Ok(sign_json)
}

fn format_vec_from_reads<'a, T: serde::Deserialize<'a> + Clone>(
    ans_vec: &'a [String],
    party_num: usize,
    value_i: T,
    new_vec: &'a mut Vec<T>,
) -> Result<()> {
    let mut j = 0;
    for i in 1..ans_vec.len() + 2 {
        if i == party_num {
            new_vec.push(value_i.clone());
        } else {
            let value_j: T = serde_json::from_str(&ans_vec[j])?;
            new_vec.push(value_j);
            j += 1;
        }
    }
    Ok(())
}
