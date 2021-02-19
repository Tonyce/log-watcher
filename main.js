const MyEventEmitter = require('./lib')
const fs = require('fs')
const { open } = require('fs/promises');

let filehandle;
let watcher;
const ac = new AbortController();
const { signal } = ac;

async function watchFile() {
    filehandle = await open('./log/err.log', "r")
    await filehandle.read({})
    watcher = fs.watch('./log/err.log', async () => {
        const data = await filehandle.read({})
        console.log("data", data.bytesRead, data.buffer.toString())
    })
}

(async () => {
    await watchFile()
})()

const emitter = new MyEventEmitter({
    logFilePath: ""
});
emitter.on('tick', async ({ count }) => {
    console.log(count);
    watcher.close()
    filehandle.close()
    watcher = null
    filehandle = null
    
    await watchFile()
});
