local shell = require "shell-games"
local json = require "cjson"
local _ip = ngx.var.mbr_ip
local _proto = ngx.var.mbr_proto
local _host = ngx.var.mbr_host

local _path = "/ping"
local _opt = "-sk "
if _host then
    _opt = '-H "Host: ' .. _host .. '" '
end
local _cmd = {
    "/usr/bin/curl",
    _opt,
    _proto .. "://" .. _ip .. _path
}

local _res = shell.run(_cmd)
ngx.log(ngx.ERR, json.encode(_res))
if _res.status == 0 then
    ngx.say(json.encode({status = 0, msg = "success"}))
else
    ngx.say(json.encode({status = 1, msg = "failed"}))
end
