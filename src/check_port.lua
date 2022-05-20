local shell = require "shell-games"
local json = require "cjson"
local _ip = ngx.var.ip
local _proto = ngx.var.proto
local _port = ngx.var.port
local _opt = "-vz"
if _proto == "udp" then
    _opt = "-uvz"
end

local _cmd = {
    "/usr/bin/nc",
    _opt,
    _ip,
    _port
}
local _res = shell.run(_cmd)
return json.encode(_res)
