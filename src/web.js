var socket;
var nickname;

var TrollInvasion = {
    connect: function (host, port, nick, handler) {
        nickname = nick;

        var buf_addr = 0;
        var buf_len = 0;

        function sendLine(line) {
            if (line.length + 1 > buf_len) {
                if (buf_len !== 0) {
                    Module._free(buf_addr);
                }
                buf_len = line.length + 1;
                buf_addr = Module._malloc(buf_len);
            }
            Module.writeAsciiToMemory(line, buf_addr);
            handler(buf_addr);
        }

        socket = new WebSocket("ws://" + host + ":" + port);
        socket.onopen = function (e) {
            socket.send("+" + nickname);
        };
        socket.onmessage = function (e) {
            sendLine(e.data);
        };
    },
    send: function (message) {
        socket.send(message);
    }
};

window.TrollInvasion = TrollInvasion;

