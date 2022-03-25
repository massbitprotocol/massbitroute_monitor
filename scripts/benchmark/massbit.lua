function init(args)
    if #args > 0 then
        token = args[1]
    end

    local msg = "thread addr: %s"
    print(msg:format(wrk.thread.addr))
end

function request()
    local body =
        '{"id": "blockNumber", "jsonrpc": "2.0", "method": "eth_getBlockByNumber", "params": ["latest", false]}'
    local headers = {}
    headers["Content-Type"] = "application/json"
    local token = wrk.thread:get("token")
    if token then
        headers["X-Api-Key"] = token
    end

    return wrk.format("POST", "/", headers, body)
end
