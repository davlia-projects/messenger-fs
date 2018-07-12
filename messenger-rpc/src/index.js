const login = require('facebook-chat-api');
const rpc = require('jayson');

let messengerApi;

const server = rpc.server({
    authenticate: (args, callback) => {
        console.log("login()");
        login({
            email: args[0],
            password: args[1]
        }, (err, api) => {
            if (err) {
                callback("Login failed");
            } else {
                messengerApi = api;
                callback("Login success");
            }
        });
    },
    user_info: (args, callback) => {
        console.log("user_info()");
        if (messengerApi === undefined || messengerApi === null) {
            callback("Login first");
        }
        callback(0);
    },
    message: (args, callback) => {
        console.log("message()");
        if (messengerApi === undefined || messengerApi === null) {
            callback("Login first");
        }
        callback(0);
    },
    attachment: (args, callback) => {
        console.log("attachment()");
        if (messengerApi === undefined || messengerApi === null) {
            callback("Login first");
        }
        callback(0);
    },
    search: (args, callback) => {
        console.log("search()");
        if (messengerApi === undefined || messengerApi === null) {
            callback("Login first");
        }
        callback(0);
    },
    history: (args, callback) => {
        console.log("history()");
        if (messengerApi === undefined || messengerApi === null) {
            callback("Login first");
        }
        callback(0);
    },

});

server.http().listen(5000, () => {
    console.log("Server listening on http://localhost:5000");
});