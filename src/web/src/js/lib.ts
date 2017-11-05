namespace TrollInvasion {
    let socket: WebSocket;
    let nickname: string;

    export function connect(host: string, port: number, nick: string, handler: (add: number) => void) {
        nickname = nick;

        let buf_addr = 0;
        let buf_len = 0;

        function sendLine(line: string) {
            let Module = (window as any).Module;
            if (line.length + 1 > buf_len) {
                if (buf_len != 0) {
                    Module._free(buf_addr);
                }
                buf_len = line.length + 1;
                buf_addr = Module._malloc(buf_len);
            }
            Module.writeAsciiToMemory(line, buf_addr);
            handler(buf_addr);
        }

        socket = new WebSocket("ws://" + host + ":" + port);
        socket.onopen = (e) => {
            socket.send("+" + nickname);
        };
        socket.onmessage = (e) => {
            sendLine(e.data);
        };
    }

    export function send(message: string) {
        socket.send(nickname + ":" + message);
    }
}

(window as any).TrollInvasion = TrollInvasion;