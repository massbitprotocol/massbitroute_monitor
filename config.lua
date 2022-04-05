local _config = {
    server = {
        nginx = {
            port = "80",
            port_ssl = "443",
            server_name = "massbitroute"
        }
    },
    templates = {},
    apps = {},
    supervisors = {
        ["monitor_client"] = [[
[program:monitor_client]
command=/bin/bash _SITE_ROOT_/../mkagent/agents/push.sh _SITE_ROOT_
autorestart=true
redirect_stderr=true
stdout_logfile=_SITE_ROOT_/logs/monitor_client.log
    ]]
    },
    supervisor = [[
[program:check_mk_dapi_collect]
command=/bin/bash _SITE_ROOT_/scripts/checkmk/dapi loop _collect
autorestart=true
redirect_stderr=true
stdout_logfile=_SITE_ROOT_/logs/check_mk_dapi_collect.log

[program:monitor_server]
command=/bin/bash _SITE_ROOT_/scripts/server.sh
autorestart=true
redirect_stderr=true
stdout_logfile=_SITE_ROOT_/logs/monitor_server.log
    ]]
}
return _config
