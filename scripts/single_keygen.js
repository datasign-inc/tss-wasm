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

async function main(){
    let delay = Math.max(Math.random() % 500, 100)
    const result = await keygen("http://192.168.10.20:8000", 1, 3, delay)
    console.log("=== Result ===")
    console.log(JSON.stringify(result))
}

main().then(() => {
    console.log('Done')
})