local cjson = require "cjson"
local mkdirp = require "mkdirp"
-- local args, err = ngx.req.get_uri_args()
local headers, err = ngx.req.get_headers()

-- local uri = ngx.var.uri
local id = ngx.var.id
local myfile = ngx.var.myfile
local site_root = ngx.var.site_root

-- ngx.say(cjson.encode({uri = uri, id = id, file = headers["x-file"]}))
-- ngx.say(cjson.encode(args))
-- ngx.say(cjson.encode(headers))
local _file = headers["x-file"]
if _file then
    local _mydir = site_root .. "/logs/debug/gateway/" .. id
    mkdirp(_mydir)
    os.rename(_file, _mydir .. "/" .. myfile)
end
