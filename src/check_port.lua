local shell = require "resty.shell"

local stdin = ""
local timeout = 1000 -- ms
local max_size = 4096 -- byte

-- local shell = require "shell-games"
local json = require "cjson"
local _ip = ngx.var.mbr_ip
local _proto = ngx.var.mbr_proto
local _port = ngx.var.mbr_port
local _opt = "-vz"
if _proto == "udp" then
    _opt = "-uvz"
end

local _cmd = {
    "timeout 1 /usr/bin/nc",
    _opt,
    _ip,
    _port
}

local ok, stdout, stderr, reason, status = shell.run(table.concat(_cmd, " "), stdin, timeout, max_size)

if not ok then
    ngx.say("1")
else
    ngx.say("0")
end

-- local _res = shell.run(_cmd)
-- ngx.say(_res.status)
-- if _res.status == 0 then
--     ngx.say(json.encode({status = 0, msg = "success"}))
-- else
--     ngx.say(json.encode({status = 1, msg = "failed"}))
-- end
