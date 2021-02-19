const fs = require('fs')

fs.truncate('./log/out.log', function(err) {
    console.log(err)
})