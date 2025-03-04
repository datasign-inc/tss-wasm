const gg18 = require('../pkg')


async function keygen(addr, t, n, delay) {
    let context = await gg18.gg18_keygen_client_new_context(addr, t, n, delay)
    console.log('keygen new context: ')
    context = await gg18.gg18_keygen_client_round1(context, delay)
    console.log('keygen round1:')
    context = await gg18.gg18_keygen_client_round2(context, delay)
    console.log('keygen round2: ')
    context = await gg18.gg18_keygen_client_round3(context, delay)
    console.log('keygen round3: ')
    context = await gg18.gg18_keygen_client_round4(context, delay)
    console.log('keygen round4: ')
    keygen_json = await gg18.gg18_keygen_client_round5(context, delay)
    console.log('keygen json: ', keygen_json)
    return keygen_json
}

async function sign(addr, t, n, message, key_store, delay) {
    console.log(`creating signature for : ${message}`)
    let context = await gg18.gg18_sign_client_new_context(
        addr,
        t,
        n,
        key_store,
        message
    )
    console.log('sign new context: ', context)
    context = await gg18.gg18_sign_client_round0(context, delay)
    console.log('sign round0: ')
    context = await gg18.gg18_sign_client_round1(context, delay)
    console.log('sign round1: ')
    context = await gg18.gg18_sign_client_round2(context, delay)
    console.log('sign round2: ')
    context = await gg18.gg18_sign_client_round3(context, delay)
    console.log('sign round3: ')
    context = await gg18.gg18_sign_client_round4(context, delay)
    console.log('sign round4: ')
    context = await gg18.gg18_sign_client_round5(context, delay)
    console.log('sign round5: ')
    context = await gg18.gg18_sign_client_round6(context, delay)
    console.log('sign round6: ')
    context = await gg18.gg18_sign_client_round7(context, delay)
    console.log('sign round7: ')
    context = await gg18.gg18_sign_client_round8(context, delay)
    console.log('sign round8: ')
    sign_json = await gg18.gg18_sign_client_round9(context, delay)
    console.log('keysign json: ', sign_json)
    return sign_json
}

async function main(){
    let delay = Math.max(Math.random() % 500, 100)
    const server = "http://192.168.10.20:8000"
    const result = await keygen(server, 1, 3, delay)

    console.log("=== Key generation Result ===")
    console.log(JSON.stringify(result))

    console.log("=== signature result ===")
    const signature = await sign(server, 1, 3,"7d0e4aab74d9c6a4a1fb80d57e259c7ed039c8f10e067c0658f37ce20682a353", result, delay)
    console.log(`signature value : ${signature}`)
}

main().then(() => {
    console.log('Done')
})