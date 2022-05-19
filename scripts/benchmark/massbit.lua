-- local inspect = require "inspect"
function init(args)
    if #args > 0 then
        token = args[1]
        host = args[2]
        provider_type = args[3]
        path = args[4]
        chain_type = args[5]
    end

    local msg = "thread addr: %s"
    print(msg:format(wrk.thread.addr))
end

function request()
    local token = wrk.thread:get("token")
    local host = wrk.thread:get("host")
    local provider_type = wrk.thread:get("provider_type")
    local chain_type = wrk.thread:get("chain_type")
    if provider_type == "node" then
        local body =""
        if chain_type == "eth" then
            body =
                '{"id": "blockNumber", "jsonrpc": "2.0", "method": "eth_getBlockByNumber", "params": ["0xde83cb", false]}'
        end
        if chain_type == "dot" then
            body =
                '{ "jsonrpc": "2.0",  "method": "chain_getBlock", "params": ["0xc0096358534ec8d21d01d34b836eed476a1c343f8724fa2153dc0725ad797a90"],"id": 1}'
        end

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
    if provider_type == "gateway" then
        local headers = {}
        headers["Content-Type"] = "application/json"
        local host = wrk.thread:get("host")
        local path = wrk.thread:get("path")

        if host then
            headers["Host"] = host
        end

        return wrk.format("GET", path, headers)
    end


end
