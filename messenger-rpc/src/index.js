const fs = require('fs');
const login = require('facebook-chat-api');
const rpc = require('jayson');
const streamifier = require('streamifier');

let messengerApi;

const server = rpc.server({
    authenticate: (args, callback) => {
        const {
            username: email,
            password
        } = args[0];
        if (messengerApi !== undefined && messengerApi !== null) {
            callback(null, "Already logged in");
            return;
        }
        login({
            email,
            password,
        }, (err, api) => {
            if (err) {
                callback(null, "Login failed");
            } else {
                fs.writeFileSync('appstate.json', JSON.stringify(api.getAppState()));
                messengerApi = api;
                callback(null, "Login success");
            }
        });
    },
    my_fbid: (args, callback) => {
        console.log("my_fbid()");
        if (messengerApi === undefined || messengerApi === null) {
            callback(null, "Login first");
        }
        callback(null, messengerApi.getCurrentUserID());
    },
    user_info: (args, callback) => {
        console.log("user_info()");
        if (messengerApi === undefined || messengerApi === null) {
            callback(null, "Login first");
        }
        const fbid = args[0]
        messengerApi.getUserInfo([fbid], (err, obj) => {
            callback(null, obj[fbid]);
        })
    },
    message: (args, callback) => {
        console.log("message()");
        if (messengerApi === undefined || messengerApi === null) {
            callback(null, "Login first");
        }
        let [message, threadId] = args;
        messengerApi.sendMessage(message, threadId, (err, obj) => {
            callback(null, "sent message");
        })
    },
    attachment: (args, callback) => {
        console.log("attachment()");
        if (messengerApi === undefined || messengerApi === null) {
            callback(null, "Login first");
        }
        const [attachment, threadId] = args;
        const buf = Buffer.from(attachment, "ascii");
        const block = streamifier.createReadStream(buf);
        block.path = 'block';
        const msg = {
            attachment: block,
        };
        console.log(args[0], args[1]);
        messengerApi.sendMessage(msg, threadId, (err, obj) => {
            callback(null, "attachment sent");
        });
    },
    search: (args, callback) => {
        console.log("search()", args);
        if (messengerApi === undefined || messengerApi === null) {
            callback(null, "Login first");
        }
        messengerApi.searchForThread(args[0], (err, obj) => {
            console.log(obj);
            callback(null, "response");
        });
    },
    history: (args, callback) => {
        console.log("history()");
        if (messengerApi === undefined || messengerApi === null) {
            callback(null, "Login first");
        }
        let [threadId, amount, timestamp] = args;
        messengerApi.getThreadHistory(threadId, amount, timestamp, (err, obj) => {
            console.log(obj[0]);
            console.log(obj[0].attachments);
            callback(null, obj);
        });
    },
});

server.http().listen(5000, () => {
    console.log("Server listening on http://localhost:5000");
});