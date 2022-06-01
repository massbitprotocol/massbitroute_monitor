local cjson = require "cjson"
-- local args, err = ngx.req.get_uri_args()
local headers, err = ngx.req.get_headers()

-- local uri = ngx.var.uri
local id = ngx.var.id
local site_root = ngx.var.site_root

-- ngx.say(cjson.encode({uri = uri, id = id, file = headers["x-file"]}))
-- ngx.say(cjson.encode(args))
-- ngx.say(cjson.encode(headers))
local _file = headers["x-file"]
if _file then
    os.rename(_file, site_root .. "/logs/debug/node/" .. id)
end
