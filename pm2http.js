const http = require('http')

http.createServer((req, res) => {
    res.end('hi')
}).listen(5678)

setInterval(() => {
    console.log('log', new Date().toISOString());
}, 1500)

setInterval(() => {
    console.error('error', new Date().toISOString());
}, 500)