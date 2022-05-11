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
    local body ='{"jsonrpc": "2.0","method": "eth_blockNumber","params": [],"id": 1}'
    local headers = {}
    headers["Content-Type"] = "application/json"
    local token = wrk.thread:get("token")

    return wrk.format("POST", "/"..token, headers, body)
end
