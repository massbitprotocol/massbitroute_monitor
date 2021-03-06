-- local inspect = require "inspect"
function init(args)
    if #args > 0 then
        token = args[1]
        host = args[2]
    end

    local msg = "thread addr: %s"
    print(msg:format(wrk.thread.addr))
end

function request()
    local body =
        '{"id": "blockNumber", "jsonrpc": "2.0", "method": "eth_getBlockByNumber", "params": ["0xde83cb", false]}'
    local headers = {}
    headers["Content-Type"] = "application/json"
    local token = wrk.thread:get("token")
    local host = wrk.thread:get("host")
    if token then
        headers["X-Api-Key"] = token
    end
    if host then
        headers["Host"] = host
    end

    return wrk.format("POST", "/", headers, body)
end
