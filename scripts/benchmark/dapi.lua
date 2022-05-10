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
        '{"id": "blockNumber", "jsonrpc": "2.0", "method": "eth_blockNumber", "params": []}'
    local headers = {}
    headers["Content-Type"] = "application/json"

    return wrk.format("POST", "/", headers, body)
end
